//! Where a file's comment and in/out points are kept, and loading and saving them.
//!
//! Each has two homes. The comment lives in the file's XMP (see [`xmp`]) or in a
//! `.comment.txt` next to it ([`crate::comment`]); the in/out points live in the file's XMP
//! as an Adobe clip marker or in the file name (`in_HH_MM_SS` / `out_HH_MM_SS`). The user picks
//! each with [`CommentStorage`] and [`InOutStorage`]. A file that cannot hold XMP keeps both
//! in the other home, so switching storage never loses anything.

mod bmff;
pub(crate) mod cache;
mod conversion;
mod xmp;

use std::borrow::Cow;
use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::RwLock;

use crate::tags::FileSnapshot;

pub(crate) use conversion::{clear_moved_xmp, Inspection};
pub use conversion::{MetadataMove, MoveOutcome};
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
    /// In the file name as `in_HH_MM_SS` / `out_HH_MM_SS`.
    #[default]
    FileName,
    /// Inside the video file, as an XMP clip marker, which Premiere Pro turns into a subclip.
    /// The file name then carries no in/out.
    InVideo,
}

impl InOutStorage {
    /// Stable name for persisting the setting.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FileName => "file_name",
            Self::InVideo => "xmp",
        }
    }

    /// Parse a persisted name; unknown names fall back to the default.
    pub fn from_name(name: &str) -> Self {
        match name {
            "xmp" => Self::InVideo,
            _ => Self::FileName,
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
static IN_OUT_XMP: AtomicU8 = AtomicU8::new(0);

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
    IN_OUT_XMP.store(
        u8::from(storage == InOutStorage::InVideo),
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
    MetadataStorage {
        comment: if COMMENT_TEXT_FILE.load(Ordering::Relaxed) == 1 {
            CommentStorage::TextFile
        } else {
            CommentStorage::InVideo
        },
        in_out: if IN_OUT_XMP.load(Ordering::Relaxed) == 1 {
            InOutStorage::InVideo
        } else {
            InOutStorage::FileName
        },
    }
}

/// Fill in the comment, and the in/out points when the file name has none, of a snapshot
/// parsed from the file name.
///
/// `has_text_file` says whether the folder listing shows a `.comment.txt` for this file.
/// A text file wins over XMP: in XMP storage one only exists as a legacy comment or as the
/// fallback for a failed XMP write, and either way it holds the newest text. In/out points
/// in the file name win over XMP the same way. The file is opened only when XMP is needed.
pub(crate) fn load(
    path: &Path,
    has_text_file: bool,
    snapshot: &mut FileSnapshot,
    storage: MetadataStorage,
    source: XmpSource<'_>,
) {
    let name_has_in_out = snapshot.segment_start().is_some() || snapshot.segment_end().is_some();
    let comment_from_xmp = storage.comment == CommentStorage::InVideo && !has_text_file;
    let in_out_from_xmp = storage.in_out == InOutStorage::InVideo && !name_has_in_out;
    if has_text_file {
        snapshot.set_comment(crate::comment::load_comment(path));
    }
    if !comment_from_xmp && !in_out_from_xmp {
        return;
    }
    let fields = match source {
        XmpSource::Read => xmp::read(path),
        XmpSource::Cached(cached) => xmp::XmpFields {
            comment: cached.comment.trim().to_string(),
            segment: Segment {
                start: cached.start,
                end: cached.end,
            },
        },
        XmpSource::Deferred => {
            snapshot.set_comment_loading(true);
            return;
        }
    };
    if comment_from_xmp {
        snapshot.set_comment(fields.comment);
    }
    if in_out_from_xmp {
        snapshot.set_segment_start(fields.segment.start);
        snapshot.set_segment_end(fields.segment.end);
    }
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
/// write belongs in the other home: the caller keeps in/out in the file name and the comment
/// in the text file (see [`save_comment_text_file`]).
pub(crate) fn save_to_xmp(
    path: &Path,
    snapshot: &FileSnapshot,
    storage: MetadataStorage,
) -> SavedToXmp {
    // Unread XMP is unknown, not empty: writing it would wipe the file's real comment.
    if snapshot.comment_loading() {
        return SavedToXmp::default();
    }
    let comment = (storage.comment == CommentStorage::InVideo).then(|| snapshot.comment().trim());
    let segment = (storage.in_out == InOutStorage::InVideo).then(|| Segment {
        start: snapshot.segment_start(),
        end: snapshot.segment_end(),
    });
    if comment.is_none() && segment.is_none() {
        return SavedToXmp::default();
    }
    match xmp::write(path, comment, segment) {
        Ok(in_out) => SavedToXmp {
            comment: comment.is_some(),
            in_out,
        },
        Err(xmp::XmpWriteError::Unsupported) => SavedToXmp::default(),
        Err(e) => {
            log::warn!(
                "metadata: XMP write to {:?} failed, keeping file name and text file: {}",
                path,
                e
            );
            SavedToXmp::default()
        }
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
        in_out: InOutStorage::FileName,
    };

    /// A fresh copy of a tiny 0.2 s QuickTime clip with no XMP, in its own temp folder.
    fn copy_of_clip(name: &str) -> PathBuf {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        let dir =
            std::env::temp_dir().join(format!("frename-metadata-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
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
        for storage in [InOutStorage::InVideo, InOutStorage::FileName] {
            assert_eq!(InOutStorage::from_name(storage.as_str()), storage);
        }
        assert_eq!(
            CommentStorage::from_name("garbage"),
            CommentStorage::InVideo
        );
        assert_eq!(InOutStorage::from_name("garbage"), InOutStorage::FileName);
    }

    #[test]
    fn setters_change_the_active_storage() {
        // The only test touching the process-wide storage; every other one passes it explicitly.
        set_comment_storage(CommentStorage::TextFile);
        set_in_out_storage(InOutStorage::InVideo);
        assert_eq!(
            metadata_storage(),
            MetadataStorage {
                comment: CommentStorage::TextFile,
                in_out: InOutStorage::InVideo
            }
        );
        set_comment_storage(CommentStorage::InVideo);
        set_in_out_storage(InOutStorage::FileName);
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
    fn text_file_and_file_name_storage_leave_the_media_file_alone() {
        let file = copy_of_clip("plain");
        let bytes_before = std::fs::read(&file).expect("read");
        let saved = save_to_xmp(&file, &snapshot("note", Some(0.05), Some(0.15)), PLAIN);
        assert_eq!(saved, SavedToXmp::default());
        assert_eq!(std::fs::read(&file).expect("read"), bytes_before);

        save_comment_text_file(&file, "note", saved.comment);
        assert_eq!(loaded(&file, true, PLAIN).comment(), "note");
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
                in_out: InOutStorage::FileName,
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
    fn a_text_file_and_in_out_in_the_name_win_over_xmp() {
        let file = copy_of_clip("precedence");
        save_to_xmp(&file, &snapshot("old", Some(0.05), Some(0.15)), XMP_BOTH);
        crate::comment::save_comment(&file, "newer");

        let mut from_name = FileSnapshot::parse("clip.in_00_00_07.mov");
        load(&file, true, &mut from_name, XMP_BOTH, XmpSource::Read);
        assert_eq!(from_name.comment(), "newer");
        assert_eq!(
            (from_name.segment_start(), from_name.segment_end()),
            (Some(7.0), None)
        );
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
    fn an_in_point_past_the_clip_end_is_not_claimed_as_saved() {
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
    }

    #[test]
    fn a_file_that_cannot_hold_xmp_saves_nothing_to_xmp() {
        let dir = copy_of_clip("unsupported");
        let file = dir.with_file_name("notes.zip");
        std::fs::write(&file, b"not really a zip").expect("write");
        let saved = save_to_xmp(&file, &snapshot("fallback", Some(1.0), None), XMP_BOTH);
        assert_eq!(saved, SavedToXmp::default());
    }

    #[test]
    fn commented_tag_names_are_cleaned_for_file_names() {
        assert_eq!(
            clean_commented_tag(" Com.ment:ed "),
            Some("Commented".to_string())
        );
        assert_eq!(clean_commented_tag(" . "), None);
    }
}
