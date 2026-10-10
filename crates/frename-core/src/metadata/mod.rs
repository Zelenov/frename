//! Where a file's comment and in/out points are kept, and loading and saving them.
//!
//! Each has two homes. The comment lives in the file's XMP (see [`xmp`]) or in a
//! `.comment.txt` next to it ([`crate::comment`]); the in/out points live in the file's XMP
//! as an Adobe clip marker or in the comment as one line (see [`in_out_line`]), wherever the
//! comment is. The user picks each with [`CommentStorage`] and [`InOutStorage`]. A file that
//! cannot hold XMP keeps both in the other home, so switching storage never loses anything.
//! File names never hold in/out points.

mod bmff;
pub(crate) mod cache;
mod conversion;
mod in_out_line;
mod markers_xmp;
pub(crate) mod rotation;
mod xmp;

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::RwLock;

use crate::markers::Marker;
use crate::tags::FileSnapshot;

pub(crate) use conversion::{clear_moved_xmp, Inspection};
pub use conversion::{MetadataMove, MoveOutcome};
pub use in_out_line::format_in_out_range;
#[cfg(test)]
pub(crate) use in_out_line::split_in_out_line;
pub(crate) use in_out_line::{format_in_out_line, parse_in_out_line};
pub use rotation::{Rotation, RotationError};
pub use xmp::Segment;

/// Where comments are saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommentStorage {
    /// Inside the video file, as XMP `dc:description` (Premiere's Description column).
    #[default]
    InVideo,
    /// In a `{filename}.comment.txt` file next to the media file.
    TextFile,
}

impl CommentStorage {
    /// Stable name for persisting the setting.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InVideo => "xmp",
            Self::TextFile => "text_file",
        }
    }

    /// Parse a persisted name; unknown names fall back to the default.
    pub fn from_name(name: &str) -> Self {
        match name {
            "text_file" => Self::TextFile,
            _ => Self::InVideo,
        }
    }
}

/// Where in/out points are saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InOutStorage {
    /// Inside the video file, as an XMP clip marker, which Premiere Pro turns into a subclip.
    #[default]
    InVideo,
    /// In the comment, as one line (`In/Out: 00:01:05.250 – 00:02:10.000`), wherever the
    /// comment is kept.
    Comment,
}

impl InOutStorage {
    /// Stable name for persisting the setting.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InVideo => "xmp",
            Self::Comment => "comment",
        }
    }

    /// Parse a persisted name; unknown names fall back to the default. `file_name`, the
    /// storage older versions had, is one of them: file names no longer hold in/out points.
    pub fn from_name(name: &str) -> Self {
        match name {
            "comment" => Self::Comment,
            _ => Self::InVideo,
        }
    }
}

/// Where clip markers (points and ranges) are saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkerStorage {
    /// Inside the video file, as XMP clip markers Premiere Pro shows on the clip.
    #[default]
    InVideo,
    /// In the comment, one line per marker (`0:41–0:47 — Lion`); the AI's markers are the
    /// lines of the comment's AI block.
    Comment,
}

impl MarkerStorage {
    /// Stable name for persisting the setting.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InVideo => "xmp",
            Self::Comment => "comment",
        }
    }

    /// Parse a persisted name; unknown names fall back to the default.
    pub fn from_name(name: &str) -> Self {
        match name {
            "comment" => Self::Comment,
            _ => Self::InVideo,
        }
    }
}

/// Both storage choices, as the file tagger applies them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MetadataStorage {
    pub comment: CommentStorage,
    pub in_out: InOutStorage,
}

// The active storage, process-wide like the installed file tagger: parsing and saving happen
// deep inside the tagger, far from the settings that choose it.
static COMMENT_TEXT_FILE: AtomicU8 = AtomicU8::new(0);
static IN_OUT_IN_COMMENT: AtomicU8 = AtomicU8::new(0);
static MARKERS_IN_COMMENT: AtomicU8 = AtomicU8::new(0);

/// Choose where markers are saved from now on. The open file's markers are written to the new
/// storage when it is next saved; other files keep theirs where they are until moved.
pub fn set_marker_storage(storage: MarkerStorage) {
    MARKERS_IN_COMMENT.store(
        u8::from(storage == MarkerStorage::Comment),
        Ordering::Relaxed,
    );
}

std::thread_local! {
    /// Storage for the current thread only, which tests use so that they do not disturb the
    /// others running at the same time (the statics above are shared by the whole process).
    static THIS_THREAD: std::cell::Cell<Option<(MetadataStorage, MarkerStorage)>> =
        const { std::cell::Cell::new(None) };
}

/// Use `metadata` and `markers` as the storage on the current thread only, instead of the
/// process-wide choice, until the returned guard is dropped. For tests of code that reads the
/// storage; the app never calls it.
#[doc(hidden)]
#[must_use = "the storage is the process-wide one again when the guard is dropped"]
pub fn use_storage_on_this_thread(
    metadata: MetadataStorage,
    markers: MarkerStorage,
) -> StorageGuard {
    let previous = THIS_THREAD.with(|cell| cell.replace(Some((metadata, markers))));
    StorageGuard {
        previous,
        _not_send: std::marker::PhantomData,
    }
}

/// Ends [`use_storage_on_this_thread`] when dropped, bringing back what it replaced. It belongs
/// to the thread that made it.
#[doc(hidden)]
pub struct StorageGuard {
    previous: Option<(MetadataStorage, MarkerStorage)>,
    _not_send: std::marker::PhantomData<*const ()>,
}

impl Drop for StorageGuard {
    fn drop(&mut self) {
        THIS_THREAD.with(|cell| cell.set(self.previous));
    }
}

/// The storage chosen by [`set_marker_storage`].
pub fn marker_storage() -> MarkerStorage {
    if let Some((_, markers)) = THIS_THREAD.with(|cell| cell.get()) {
        return markers;
    }
    if MARKERS_IN_COMMENT.load(Ordering::Relaxed) == 1 {
        MarkerStorage::Comment
    } else {
        MarkerStorage::InVideo
    }
}

/// Markers kept in the comment while the comment is a `.comment.txt` file: bring that file's
/// marker lines in line with `markers` and leave the rest of its text as it is, so a marker edit
/// is on disk at once and does not wait for the clip to be left. `Ok(false)` when markers or
/// comments are stored elsewhere (nothing is done) and `Ok(true)` when the file is in line,
/// written or already so; `Err` when it could not be written. A comment kept in the video
/// cannot be written while the video plays: it is saved when the clip is left.
pub fn write_marker_lines(path: &Path, markers: &[crate::Marker]) -> std::io::Result<bool> {
    write_marker_lines_with(
        path,
        markers,
        marker_storage() == MarkerStorage::Comment
            && metadata_storage().comment == CommentStorage::TextFile,
    )
}

fn write_marker_lines_with(
    path: &Path,
    markers: &[crate::Marker],
    applies: bool,
) -> std::io::Result<bool> {
    if !applies {
        return Ok(false);
    }
    let current = crate::comment::load_comment(path);
    let (text, _) = crate::markers_from_comment(&current);
    let updated = crate::markers_into_comment(&text, markers);
    if updated.trim() != current.trim() {
        crate::comment::try_save_comment(path, &updated)?;
    }
    Ok(true)
}

/// Choose where comments are saved from now on. Comments already loaded keep their text and
/// are written to the new storage when their file is next saved.
pub fn set_comment_storage(storage: CommentStorage) {
    COMMENT_TEXT_FILE.store(
        u8::from(storage == CommentStorage::TextFile),
        Ordering::Relaxed,
    );
}

/// Choose where in/out points are saved from now on. Loaded files keep their points and
/// move them to the new storage when they are next saved.
pub fn set_in_out_storage(storage: InOutStorage) {
    IN_OUT_IN_COMMENT.store(
        u8::from(storage == InOutStorage::Comment),
        Ordering::Relaxed,
    );
}

/// The tag frename checks on a video when it gets a comment while comments are stored in XMP,
/// e.g. `Food.Commented.IMG_0424.MOV`; see [`active_commented_tag`]. The comment itself is not
/// visible in the file name or in Explorer; the tag is, and it is searchable and filterable
/// like any other tag.
pub const DEFAULT_COMMENTED_TAG: &str = "Commented";

static COMMENTED_TAG: RwLock<Cow<'static, str>> = RwLock::new(Cow::Borrowed(DEFAULT_COMMENTED_TAG));

/// Choose the tag for commented videos; see [`DEFAULT_COMMENTED_TAG`]. Characters a tag
/// cannot hold are dropped; an empty tag turns the feature off. Videos keep the old tag
/// until they are saved, and the old one is not removed.
pub fn set_commented_tag(tag: &str) {
    let tag = clean_commented_tag(tag).map_or(Cow::Borrowed(""), Cow::Owned);
    if let Ok(mut current) = COMMENTED_TAG.write() {
        *current = tag;
    }
}

/// The tag chosen by [`set_commented_tag`]; `None` when it is off.
pub fn commented_tag() -> Option<String> {
    let tag = COMMENTED_TAG.read().ok()?;
    (!tag.is_empty()).then(|| tag.to_string())
}

/// The tag as it can appear in a file name: without dots, which separate tags, and without
/// characters Windows forbids in names. `None` when nothing is left.
pub fn clean_commented_tag(tag: &str) -> Option<String> {
    let cleaned: String = tag
        .chars()
        .filter(|c| {
            !matches!(
                c,
                '.' | '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
            ) && !c.is_control()
        })
        .collect();
    let cleaned = cleaned.trim();
    (!cleaned.is_empty()).then(|| cleaned.to_string())
}

/// The commented tag to follow comments with now: [`commented_tag`] while comments are stored
/// in XMP, `None` otherwise (a text file shows next to the video already) or when it is off.
///
/// The tag is checked when a file's comment goes from empty to non-empty and unchecked when
/// it is cleared, like a click in the tag panel; it is never enforced on save, so the user can
/// still remove or move it.
pub fn active_commented_tag() -> Option<String> {
    (metadata_storage().comment == CommentStorage::InVideo)
        .then(commented_tag)
        .flatten()
}

/// The storage chosen by [`set_comment_storage`] and [`set_in_out_storage`].
pub fn metadata_storage() -> MetadataStorage {
    if let Some((metadata, _)) = THIS_THREAD.with(|cell| cell.get()) {
        return metadata;
    }
    MetadataStorage {
        comment: if COMMENT_TEXT_FILE.load(Ordering::Relaxed) == 1 {
            CommentStorage::TextFile
        } else {
            CommentStorage::InVideo
        },
        in_out: if IN_OUT_IN_COMMENT.load(Ordering::Relaxed) == 1 {
            InOutStorage::Comment
        } else {
            InOutStorage::InVideo
        },
    }
}

/// Fill in the comment, the in/out points and the marker count of a snapshot parsed from the
/// file name.
///
/// `has_text_file` says whether the folder listing shows a `.comment.txt` for this file.
/// A text file wins over XMP: in XMP storage one only exists as a legacy comment or as the
/// fallback for a failed XMP write, and either way it holds the newest text. A text file that
/// holds only an in/out line wins for the in/out points, not for the comment. An in/out line
/// in the comment wins over the XMP marker the same way: it is where in/out points go when the
/// marker cannot hold them. The line is taken out of the comment into the snapshot's in/out
/// points whatever the storage. The XMP is read whatever the storage, for the marker count; a
/// folder scan takes it from the file list or defers it.
pub(crate) fn load(
    path: &Path,
    has_text_file: bool,
    snapshot: &mut FileSnapshot,
    storage: MetadataStorage,
    source: XmpSource<'_>,
) {
    let mut comment_has_in_out = false;
    let mut text_has_comment = false;
    if has_text_file {
        snapshot.set_comment(crate::comment::load_comment(path));
        comment_has_in_out = take_in_out_line(snapshot);
        text_has_comment = !snapshot.comment().trim().is_empty();
    }
    // A text file holding nothing but the in/out line (written while comments were kept in
    // text files) has no comment to win with: the one in the video stays the comment.
    let comment_from_xmp = storage.comment == CommentStorage::InVideo && !text_has_comment;
    // Read whatever the storage: the marker count always comes from the video.
    let fields = match source {
        XmpSource::Read => xmp::read(path),
        XmpSource::Cached(cached) => xmp::XmpFields {
            comment: cached.comment.trim().to_string(),
            segment: Segment {
                start: cached.start,
                end: cached.end,
            },
            markers: cached.markers.unwrap_or_default(),
        },
        XmpSource::Deferred => {
            snapshot.set_comment_loading(true);
            return;
        }
    };
    snapshot.set_marker_count(fields.markers);
    if comment_from_xmp {
        let (comment, segment) = in_out_line::split_in_out_line(&fields.comment);
        snapshot.set_comment(comment);
        if !comment_has_in_out && !segment.is_empty() {
            snapshot.set_segment(segment);
            comment_has_in_out = true;
        }
    }
    if storage.in_out == InOutStorage::InVideo && !comment_has_in_out {
        snapshot.set_segment(fields.segment);
    }
}

/// Move the in/out line of the snapshot's comment into its in/out points. Returns whether
/// the comment had one.
fn take_in_out_line(snapshot: &mut FileSnapshot) -> bool {
    let (comment, segment) = in_out_line::split_in_out_line(snapshot.comment());
    if segment.is_empty() {
        return false;
    }
    snapshot.set_comment(comment);
    snapshot.set_segment(segment);
    true
}

/// The comment in the file's XMP, as it is stored (with an in/out line, if any).
#[cfg(test)]
pub(crate) fn xmp_comment(path: &Path) -> String {
    xmp::read(path).comment
}

/// The comment as it is stored: the snapshot's comment, with the in/out line as the last line
/// of the editor's part (see [`in_out_line`]) unless the points are kept in the XMP marker
/// (`in_out_in_xmp`).
pub(crate) fn stored_comment(snapshot: &FileSnapshot, in_out_in_xmp: bool) -> String {
    if in_out_in_xmp {
        snapshot.comment().to_string()
    } else {
        in_out_line::with_in_out_line(snapshot.comment(), snapshot.segment())
    }
}

/// Why a file's clip markers were not saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkersError {
    /// The format cannot hold XMP (the toolkit has no handler that writes into it).
    CannotHoldMarkers,
    /// Named MOV/MP4, but the content is not a movie: a download or copy that did not finish
    /// leaves such a file, often all zeros. It does not play either.
    Damaged,
    /// The write failed: the file is read-only, open in another app (Premiere holds clips it
    /// imported), or the disk is full. The text says what the system reported.
    WriteFailed(String),
}

impl std::fmt::Display for MarkersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CannotHoldMarkers => write!(f, "this format cannot hold markers"),
            Self::Damaged => write!(
                f,
                "the file is damaged: its content is not a video (a download or copy that did                  not finish?)"
            ),
            Self::WriteFailed(reason) => write!(f, "{reason}"),
        }
    }
}

/// The clip markers kept in the file's XMP. `None` when the file cannot hold them.
pub(crate) fn load_markers(path: &Path) -> Option<Vec<Marker>> {
    let mut markers = xmp::read_markers(path)?;
    crate::markers::sort_markers(&mut markers);
    Some(markers)
}

/// Why the file at `path` holds no markers when its XMP cannot be opened: damaged, or a
/// format without XMP.
pub(crate) fn cannot_hold_markers(path: &Path) -> MarkersError {
    if bmff::is_damaged(path) {
        MarkersError::Damaged
    } else {
        MarkersError::CannotHoldMarkers
    }
}

/// Length of the clip in milliseconds, when the file's header tells it.
pub(crate) fn clip_length_ms(path: &Path) -> Option<u64> {
    xmp::clip_length_ms(path)
}

/// Write `markers` into the file's XMP. Markers are always stored in the video, whatever the
/// comment and in/out storage. `known` holds every GUID frename read from or wrote to the file:
/// a file marker with one of them that is missing from `markers` was deleted by the user,
/// while other markers not in `markers` (added by Premiere meanwhile) are kept.
pub(crate) fn save_markers(
    path: &Path,
    markers: &[Marker],
    known: &HashSet<String>,
) -> Result<(), MarkersError> {
    xmp::write_markers(path, markers, known).map_err(|e| match e {
        xmp::XmpWriteError::Unsupported => cannot_hold_markers(path),
        other => {
            log::warn!("metadata: markers not saved into {path:?}: {other}");
            MarkersError::WriteFailed(other.to_string())
        }
    })?;
    // The write keeps the file's time, so a same-size file would still match its old line.
    cache::reload_line(path);
    Ok(())
}

/// Where [`load`] gets a file's XMP from.
pub(crate) enum XmpSource<'a> {
    /// Open the file and read it.
    Read,
    /// The tag file's line for it, which still matches the file.
    Cached(&'a crate::tags::CachedFile),
    /// Not now: mark the snapshot [`FileSnapshot::comment_loading`].
    Deferred,
}

/// What [`save_to_xmp`] put into the file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SavedToXmp {
    pub comment: bool,
    pub in_out: bool,
}

/// Write the fields whose storage is XMP into the file at `path`. Whatever this could not
/// write belongs in the other home: the caller keeps the comment in the text file (see
/// [`save_comment_text_file`]), and in/out points the marker could not hold go into the
/// comment as its in/out line (see [`stored_comment`]). When the comment itself goes into the
/// XMP, it carries that line already.
pub(crate) fn save_to_xmp(
    path: &Path,
    snapshot: &FileSnapshot,
    storage: MetadataStorage,
) -> SavedToXmp {
    // Unread XMP is unknown, not empty: writing it would wipe the file's real comment.
    if snapshot.comment_loading() {
        return SavedToXmp::default();
    }
    let comment_to_xmp = storage.comment == CommentStorage::InVideo;
    let in_out_to_xmp = storage.in_out == InOutStorage::InVideo;
    if !comment_to_xmp && !in_out_to_xmp {
        return SavedToXmp::default();
    }
    let comment = comment_to_xmp.then(|| stored_comment(snapshot, in_out_to_xmp));
    let segment = in_out_to_xmp.then(|| snapshot.segment());
    let segment_stored = match xmp::write(path, comment.as_deref().map(str::trim), segment) {
        Ok(stored) => stored,
        Err(xmp::XmpWriteError::Unsupported) => return SavedToXmp::default(),
        Err(e) => {
            log::warn!(
                "metadata: XMP write to {:?} failed, keeping the text file: {}",
                path,
                e
            );
            return SavedToXmp::default();
        }
    };
    let in_out = in_out_to_xmp && segment_stored;
    if comment_to_xmp && in_out_to_xmp && !segment_stored {
        // The marker cannot hold these points (an in past the clip end): the comment in the
        // video gets them as its in/out line.
        let with_line = stored_comment(snapshot, false);
        if let Err(e) = xmp::write(path, Some(with_line.trim()), None) {
            log::warn!(
                "metadata: XMP write to {:?} failed, keeping the text file: {}",
                path,
                e
            );
            return SavedToXmp::default();
        }
    }
    SavedToXmp {
        comment: comment_to_xmp,
        in_out,
    }
}

/// Bring the `.comment.txt` of the file at `path` in line with a save: removed when the
/// comment went into XMP (which moves legacy comments into the file), written otherwise.
pub(crate) fn save_comment_text_file(path: &Path, comment: &str, saved_to_xmp: bool) {
    // A pending snapshot's comment came from the text file if there is one, and is unknown
    // otherwise; either way there is nothing to change.
    if saved_to_xmp {
        crate::comment::remove_comment_file(path);
    } else {
        crate::comment::save_comment(path, comment);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comment::comment_path;
    use std::path::PathBuf;

    const XMP_BOTH: MetadataStorage = MetadataStorage {
        comment: CommentStorage::InVideo,
        in_out: InOutStorage::InVideo,
    };
    const PLAIN: MetadataStorage = MetadataStorage {
        comment: CommentStorage::TextFile,
        in_out: InOutStorage::Comment,
    };

    /// A fresh copy of a tiny 0.2 s QuickTime clip with no XMP, in its own temp folder.
    fn copy_of_clip(name: &str) -> PathBuf {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        let dir = crate::test_support::fresh_dir(&format!("metadata-{name}"));
        let file = dir.join("clip.mov");
        std::fs::copy(fixture, &file).expect("copy fixture");
        file
    }

    fn snapshot(comment: &str, start: Option<f32>, end: Option<f32>) -> FileSnapshot {
        let mut snapshot = FileSnapshot::parse("clip.mov");
        snapshot.set_comment(comment.to_string());
        snapshot.set_segment_start(start);
        snapshot.set_segment_end(end);
        snapshot
    }

    fn loaded(path: &Path, has_text_file: bool, storage: MetadataStorage) -> FileSnapshot {
        let mut snapshot = FileSnapshot::parse("clip.mov");
        load(path, has_text_file, &mut snapshot, storage, XmpSource::Read);
        snapshot
    }

    #[test]
    fn storage_names_round_trip() {
        for storage in [CommentStorage::InVideo, CommentStorage::TextFile] {
            assert_eq!(CommentStorage::from_name(storage.as_str()), storage);
        }
        for storage in [InOutStorage::InVideo, InOutStorage::Comment] {
            assert_eq!(InOutStorage::from_name(storage.as_str()), storage);
        }
        assert_eq!(
            CommentStorage::from_name("garbage"),
            CommentStorage::InVideo
        );
        assert_eq!(InOutStorage::from_name("garbage"), InOutStorage::InVideo);
        assert_eq!(
            InOutStorage::from_name("file_name"),
            InOutStorage::InVideo,
            "the file-name storage of older versions"
        );
    }

    #[test]
    fn setters_change_the_active_storage() {
        // The only test touching the process-wide storage; every other one passes it explicitly.
        set_comment_storage(CommentStorage::TextFile);
        set_in_out_storage(InOutStorage::Comment);
        assert_eq!(
            metadata_storage(),
            MetadataStorage {
                comment: CommentStorage::TextFile,
                in_out: InOutStorage::Comment
            }
        );
        set_comment_storage(CommentStorage::InVideo);
        set_in_out_storage(InOutStorage::InVideo);
        assert_eq!(metadata_storage(), MetadataStorage::default());
    }

    #[test]
    fn xmp_storage_round_trips_comment_and_in_out_and_keeps_the_modified_time() {
        let file = copy_of_clip("round-trip");
        let modified_before = std::fs::metadata(&file)
            .and_then(|m| m.modified())
            .expect("mtime");
        let saved = save_to_xmp(
            &file,
            &snapshot("Козёл ест траву 🐐", Some(0.05), Some(0.15)),
            XMP_BOTH,
        );
        assert_eq!(
            saved,
            SavedToXmp {
                comment: true,
                in_out: true
            }
        );

        let back = loaded(&file, false, XMP_BOTH);
        assert_eq!(back.comment(), "Козёл ест траву 🐐");
        assert_eq!(
            (back.segment_start(), back.segment_end()),
            (Some(0.05), Some(0.15))
        );
        let modified_after = std::fs::metadata(&file)
            .and_then(|m| m.modified())
            .expect("mtime");
        assert_eq!(modified_after, modified_before);
    }

    #[test]
    fn open_ended_in_out_reads_back_open_ended() {
        let file = copy_of_clip("open-ended");
        save_to_xmp(&file, &snapshot("", Some(0.05), None), XMP_BOTH);
        let back = loaded(&file, false, XMP_BOTH);
        assert_eq!(
            (back.segment_start(), back.segment_end()),
            (Some(0.05), None)
        );

        save_to_xmp(&file, &snapshot("", None, Some(0.1)), XMP_BOTH);
        let back = loaded(&file, false, XMP_BOTH);
        assert_eq!(
            (back.segment_start(), back.segment_end()),
            (None, Some(0.1))
        );
    }

    #[test]
    fn text_file_and_comment_storage_leave_the_media_file_alone() {
        let file = copy_of_clip("plain");
        let bytes_before = std::fs::read(&file).expect("read");
        let note = snapshot("note", Some(0.05), Some(0.15));
        let saved = save_to_xmp(&file, &note, PLAIN);
        assert_eq!(saved, SavedToXmp::default());
        assert_eq!(std::fs::read(&file).expect("read"), bytes_before);

        save_comment_text_file(&file, &stored_comment(&note, saved.in_out), saved.comment);
        assert_eq!(
            crate::comment::load_comment(&file),
            "note\nIn/Out: 00:00:00.050 – 00:00:00.150"
        );
        let back = loaded(&file, true, PLAIN);
        assert_eq!(back.comment(), "note");
        assert_eq!(
            (back.segment_start(), back.segment_end()),
            (Some(0.05), Some(0.15))
        );
    }

    #[test]
    fn comment_storage_puts_the_in_out_line_into_the_xmp_comment() {
        let file = copy_of_clip("line-in-xmp");
        let in_comment = MetadataStorage {
            comment: CommentStorage::InVideo,
            in_out: InOutStorage::Comment,
        };
        let saved = save_to_xmp(&file, &snapshot("note", Some(0.05), None), in_comment);
        assert_eq!(
            saved,
            SavedToXmp {
                comment: true,
                in_out: false
            }
        );
        assert_eq!(xmp::read(&file).comment, "note\nIn/Out: 00:00:00.050 – end");
        assert!(xmp::read(&file).segment.is_empty(), "no marker");
        let back = loaded(&file, false, in_comment);
        assert_eq!(back.comment(), "note");
        assert_eq!(
            (back.segment_start(), back.segment_end()),
            (Some(0.05), None)
        );

        // Clearing the points drops the line.
        save_to_xmp(&file, &snapshot("note", None, None), in_comment);
        assert_eq!(xmp::read(&file).comment, "note");
    }

    #[test]
    fn each_storage_reads_only_its_own_home() {
        let file = copy_of_clip("own-home");
        save_to_xmp(&file, &snapshot("in xmp", Some(0.05), Some(0.15)), XMP_BOTH);

        let plain = loaded(&file, false, PLAIN);
        assert_eq!(plain.comment(), "");
        assert_eq!((plain.segment_start(), plain.segment_end()), (None, None));

        let comment_only = loaded(
            &file,
            false,
            MetadataStorage {
                comment: CommentStorage::InVideo,
                in_out: InOutStorage::Comment,
            },
        );
        assert_eq!(comment_only.comment(), "in xmp");
        assert_eq!(comment_only.segment_start(), None);
    }

    #[test]
    fn in_out_only_storage_leaves_the_xmp_comment_alone() {
        let file = copy_of_clip("in-out-only");
        save_to_xmp(&file, &snapshot("keep me", None, None), XMP_BOTH);
        let in_out_only = MetadataStorage {
            comment: CommentStorage::TextFile,
            in_out: InOutStorage::InVideo,
        };
        let saved = save_to_xmp(
            &file,
            &snapshot("text comment", Some(0.05), Some(0.15)),
            in_out_only,
        );
        assert_eq!(
            saved,
            SavedToXmp {
                comment: false,
                in_out: true
            }
        );
        assert_eq!(loaded(&file, false, XMP_BOTH).comment(), "keep me");
    }

    #[test]
    fn a_text_file_and_its_in_out_line_win_over_xmp() {
        let file = copy_of_clip("precedence");
        save_to_xmp(&file, &snapshot("old", Some(0.05), Some(0.15)), XMP_BOTH);
        crate::comment::save_comment(&file, "In/Out: 00:00:00.100 – end\nnewer");

        let back = loaded(&file, true, XMP_BOTH);
        assert_eq!(back.comment(), "newer");
        assert_eq!(
            (back.segment_start(), back.segment_end()),
            (Some(0.1), None)
        );

        // Without a line, the marker in the video counts.
        crate::comment::save_comment(&file, "newer");
        let back = loaded(&file, true, XMP_BOTH);
        assert_eq!(
            (back.segment_start(), back.segment_end()),
            (Some(0.05), Some(0.15))
        );
    }

    #[test]
    fn in_out_names_of_older_versions_are_not_read() {
        let file = copy_of_clip("old-name");
        let mut from_name = FileSnapshot::parse("clip.in_00_00_07.mov");
        load(&file, false, &mut from_name, XMP_BOTH, XmpSource::Read);
        assert_eq!(
            (from_name.segment_start(), from_name.segment_end()),
            (None, None)
        );
        assert_eq!(from_name.name_without_extension(), "clip.in_00_00_07");
    }

    #[test]
    fn saving_the_same_values_leaves_the_file_untouched() {
        let file = copy_of_clip("unchanged");
        save_to_xmp(&file, &snapshot("goat", Some(0.05), None), XMP_BOTH);
        let bytes_before = std::fs::read(&file).expect("read");
        save_to_xmp(&file, &snapshot("  goat  ", Some(0.05), None), XMP_BOTH);
        assert_eq!(std::fs::read(&file).expect("read"), bytes_before);
    }

    #[test]
    fn saving_moves_a_legacy_text_comment_into_xmp() {
        let file = copy_of_clip("migrate");
        crate::comment::save_comment(&file, "old comment");
        let legacy = loaded(&file, true, XMP_BOTH);
        assert_eq!(legacy.comment(), "old comment");

        let saved = save_to_xmp(&file, &legacy, XMP_BOTH);
        save_comment_text_file(&file, legacy.comment(), saved.comment);
        assert!(!comment_path(&file).exists());
        assert_eq!(loaded(&file, false, XMP_BOTH).comment(), "old comment");
    }

    #[test]
    fn an_in_point_past_the_clip_end_goes_into_the_comment() {
        // The fixture is 0.2 s long: an in at 1 s with no out makes an empty marker range.
        let file = copy_of_clip("past-end");
        let saved = save_to_xmp(&file, &snapshot("note", Some(1.0), None), XMP_BOTH);
        assert_eq!(
            saved,
            SavedToXmp {
                comment: true,
                in_out: false
            }
        );
        assert_eq!(xmp::read(&file).comment, "note\nIn/Out: 00:00:01.000 – end");
        let back = loaded(&file, false, XMP_BOTH);
        assert_eq!(back.comment(), "note");
        assert_eq!(
            (back.segment_start(), back.segment_end()),
            (Some(1.0), None)
        );
    }

    #[test]
    fn a_file_that_cannot_hold_xmp_saves_nothing_to_xmp() {
        let dir = copy_of_clip("unsupported");
        let file = dir.with_file_name("notes.zip");
        std::fs::write(&file, b"not really a zip").expect("write");
        let saved = save_to_xmp(&file, &snapshot("fallback", Some(1.0), None), XMP_BOTH);
        assert_eq!(saved, SavedToXmp::default());
    }

    fn marker(start_ms: u64, name: &str) -> Marker {
        let mut marker = Marker::new(start_ms);
        marker.name = name.to_string();
        marker
    }

    #[test]
    fn markers_round_trip_through_the_video_with_every_field_and_color() {
        let file = copy_of_clip("markers-round-trip");
        let modified_before = std::fs::metadata(&file)
            .and_then(|m| m.modified())
            .expect("mtime");
        assert_eq!(load_markers(&file), Some(Vec::new()));
        let mut markers: Vec<Marker> = crate::MarkerColor::ALL
            .into_iter()
            .enumerate()
            .map(|(i, color)| {
                let mut m = marker(i as u64 * 10, &format!("Козёл {i} 🐐"));
                m.color = color;
                m
            })
            .collect();
        markers[0].comment = "line one\nстрока два 🎬".to_string();
        markers[1].duration_ms = 150;
        save_markers(&file, &markers, &HashSet::new()).expect("save");
        assert_eq!(load_markers(&file), Some(markers));
        let modified_after = std::fs::metadata(&file)
            .and_then(|m| m.modified())
            .expect("mtime");
        assert_eq!(modified_after, modified_before);
    }

    #[test]
    fn saving_the_same_markers_leaves_the_file_untouched() {
        let file = copy_of_clip("markers-unchanged");
        let markers = vec![marker(50, "a")];
        save_markers(&file, &markers, &HashSet::new()).expect("save");
        let bytes_before = std::fs::read(&file).expect("read");
        save_markers(&file, &markers, &HashSet::new()).expect("save");
        assert_eq!(std::fs::read(&file).expect("read"), bytes_before);
    }

    #[test]
    fn markers_and_in_out_and_comment_live_side_by_side() {
        let file = copy_of_clip("markers-and-in-out");
        save_to_xmp(&file, &snapshot("note", Some(0.05), Some(0.15)), XMP_BOTH);
        let markers = vec![marker(20, "moment")];
        save_markers(&file, &markers, &HashSet::new()).expect("save");
        save_to_xmp(&file, &snapshot("note 2", Some(0.1), Some(0.15)), XMP_BOTH);
        let back = loaded(&file, false, XMP_BOTH);
        assert_eq!(back.comment(), "note 2");
        assert_eq!(
            (back.segment_start(), back.segment_end()),
            (Some(0.1), Some(0.15))
        );
        assert_eq!(load_markers(&file), Some(markers));
    }

    #[test]
    fn a_marker_added_elsewhere_survives_and_a_deleted_one_goes() {
        let file = copy_of_clip("markers-elsewhere");
        let mine = marker(10, "mine");
        let gone = marker(20, "gone");
        save_markers(&file, &[mine.clone(), gone.clone()], &HashSet::new()).expect("save");
        let known: HashSet<String> = [&mine, &gone]
            .iter()
            .filter_map(|m| m.guid.clone())
            .collect();
        // Premiere adds a marker while frename has the file open.
        let premiere = marker(30, "premiere");
        let mut in_file = load_markers(&file).expect("markers");
        in_file.push(premiere.clone());
        save_markers(&file, &in_file, &HashSet::new()).expect("premiere");

        save_markers(&file, std::slice::from_ref(&mine), &known).expect("save");
        assert_eq!(load_markers(&file), Some(vec![mine, premiere]));
    }

    #[test]
    fn a_file_that_cannot_hold_xmp_has_no_markers() {
        let dir = copy_of_clip("markers-unsupported");
        let file = dir.with_file_name("notes.zip");
        std::fs::write(&file, b"not really a zip").expect("write");
        assert_eq!(load_markers(&file), None);
        assert_eq!(
            save_markers(&file, &[marker(1, "x")], &HashSet::new()),
            Err(MarkersError::CannotHoldMarkers)
        );
    }

    #[test]
    fn comment_in_out_and_markers_survive_a_rotation() {
        for fixture in ["wide.mp4", "wide.mov"] {
            let source = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(fixture);
            let dir = crate::test_support::fresh_dir(&format!(
                "metadata-rotation-{}",
                fixture.replace('.', "-")
            ));
            let file = dir.join(fixture);
            std::fs::copy(source, &file).expect("copy fixture");

            let mut snap = FileSnapshot::parse(fixture);
            snap.set_comment("Козёл 🐐".to_string());
            snap.set_segment_start(Some(0.05));
            snap.set_segment_end(Some(0.15));
            save_to_xmp(&file, &snap, XMP_BOTH);
            let markers = vec![marker(40, "moment")];
            save_markers(&file, &markers, &HashSet::new()).expect("markers");

            rotation::rotate(&file, 1).expect("rotate");
            assert_eq!(rotation::read(&file).map(Rotation::degrees), Ok(90));

            let mut back = FileSnapshot::parse(fixture);
            load(&file, false, &mut back, XMP_BOTH, XmpSource::Read);
            assert_eq!(back.comment(), "Козёл 🐐", "{fixture}");
            assert_eq!(
                (back.segment_start(), back.segment_end()),
                (Some(0.05), Some(0.15)),
                "{fixture}"
            );
            assert_eq!(load_markers(&file), Some(markers), "{fixture}");
            // The movie is still one the XMP toolkit opens and knows the length of.
            assert_eq!(clip_length_ms(&file), Some(200), "{fixture}");
        }
    }

    #[test]
    fn commented_tag_names_are_cleaned_for_file_names() {
        assert_eq!(
            clean_commented_tag(" Com.ment:ed "),
            Some("Commented".to_string())
        );
        assert_eq!(clean_commented_tag(" . "), None);
    }

    #[test]
    fn marker_lines_are_written_into_the_comment_file_and_the_rest_is_kept() {
        let file = temp_clip_path("marker-lines");
        crate::comment::save_comment(&file, "In/Out: 00:00:01.000 – 00:00:09.000\nA note");
        let mut marker = crate::Marker::new(5_000);
        marker.name = "Lion".to_string();
        assert!(write_marker_lines_with(&file, &[marker.clone()], true).unwrap());
        let text = crate::comment::load_comment(&file);
        assert!(
            text.contains("A note") && text.contains("In/Out:"),
            "{text}"
        );
        assert!(text.lines().any(|l| l.contains("Lion")), "{text}");
        // Idempotent, and a delete takes the line out again.
        assert!(write_marker_lines_with(&file, &[marker], true).unwrap());
        assert_eq!(crate::comment::load_comment(&file), text);
        assert!(write_marker_lines_with(&file, &[], true).unwrap());
        let after = crate::comment::load_comment(&file);
        assert!(
            !after.contains("Lion") && after.contains("A note"),
            "{after}"
        );
    }

    #[test]
    fn marker_lines_are_left_alone_when_markers_are_stored_in_the_video() {
        let file = temp_clip_path("marker-lines-off");
        crate::comment::save_comment(&file, "A note");
        assert!(!write_marker_lines_with(&file, &[crate::Marker::new(5_000)], false).unwrap());
        assert_eq!(crate::comment::load_comment(&file), "A note");
    }

    fn temp_clip_path(name: &str) -> PathBuf {
        let dir = crate::test_support::fresh_dir(&format!("meta-{name}"));
        dir.join("clip.mp4")
    }

    /// Issue #212: Windows reuses process ids, so a folder an earlier run left under this name
    /// must not leak its comment into this run.
    #[test]
    fn a_folder_left_by_an_earlier_run_does_not_leak_into_this_one() {
        let stale = std::env::temp_dir().join(format!("frename-meta-stale-{}", std::process::id()));
        std::fs::create_dir_all(&stale).expect("stale folder");
        std::fs::write(stale.join("clip.mp4.comment.txt"), "0:05").expect("stale comment");

        let file = temp_clip_path("stale");
        assert_eq!(file, stale.join("clip.mp4"));
        assert_eq!(crate::comment::load_comment(&file), "");
    }

    #[test]
    fn the_storage_decides_whether_marker_lines_are_written() {
        let file = temp_clip_path("marker-lines-gate");
        let marker = [crate::Marker::new(5_000)];
        let text_file = MetadataStorage {
            comment: CommentStorage::TextFile,
            in_out: InOutStorage::InVideo,
        };
        {
            let _storage = use_storage_on_this_thread(text_file, MarkerStorage::InVideo);
            assert!(
                !write_marker_lines(&file, &marker).unwrap(),
                "markers in the video"
            );
        }
        {
            let in_video = MetadataStorage {
                comment: CommentStorage::InVideo,
                ..text_file
            };
            let _storage = use_storage_on_this_thread(in_video, MarkerStorage::Comment);
            assert!(
                !write_marker_lines(&file, &marker).unwrap(),
                "comment in the video"
            );
        }
        assert_eq!(crate::comment::load_comment(&file), "");
        let _storage = use_storage_on_this_thread(text_file, MarkerStorage::Comment);
        assert!(write_marker_lines(&file, &marker).unwrap());
        assert!(crate::comment::load_comment(&file).contains("0:05"));
    }

    #[test]
    fn a_comment_file_that_cannot_be_written_is_an_error() {
        let file = temp_clip_path("marker-lines-unwritable");
        // A folder where the comment file should be: it cannot be written.
        std::fs::create_dir_all(crate::comment::comment_path(&file)).expect("folder");
        assert!(write_marker_lines_with(&file, &[crate::Marker::new(5_000)], true).is_err());
    }

    /// Issue #145: merging two markers on one moment must reach the file. A marker left out of
    /// the list leaves the file only when its GUID is among the `known` ones (the real XMP path;
    /// the in-memory test backend of the file tagger ignores `known`).
    #[test]
    fn merged_away_markers_leave_the_video_when_their_guids_are_known() {
        let file = copy_of_clip("merged-away");
        let both = [
            marker(63_558, "Субтитр: нет субтитра"),
            marker(63_558, "нет субтитра"),
        ];
        save_markers(&file, &both, &HashSet::new()).expect("save");
        let loaded = load_markers(&file).expect("markers");
        assert_eq!(loaded.len(), 2);
        let (merged, away) = crate::merge_duplicate_markers(&loaded);
        assert_eq!(away, 1);
        let known: HashSet<String> = loaded.iter().filter_map(|m| m.guid.clone()).collect();
        save_markers(&file, &merged, &known).expect("save merged");
        let after = load_markers(&file).expect("markers");
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].name, "Субтитр: нет субтитра");
    }
}
