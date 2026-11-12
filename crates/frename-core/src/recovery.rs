//! The recovery journal: what the editor did to the open clip and has not saved yet, kept on
//! disk so a crash, a kill or a power cut does not lose it.
//!
//! Most edits of the open clip reach the file only when the clip is left (Windows locks a
//! playing video, and the rename waits for it to unload). The app writes the pending state of
//! the open clip here, a second after it changes, and deletes the entry once the edits are
//! really applied. Entries still there at the next start mean an abnormal exit: [`restore_all`]
//! applies them through the normal save path when the clip is still the one they were made on,
//! and keeps them aside (never overwriting) when it is not.
//!
//! One JSON file per clip, in `recovery/` of [`crate::app_data_dir`]. Writes are atomic (a
//! temporary file, flushed, then renamed over the entry), so the journal itself survives a
//! power cut; a truncated or unknown file is set aside with a note, never a crash.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::markers::{Marker, MarkerColor};
use crate::tags::{FileSnapshot, FileTagger, SaveAndReparse};

/// The entry layout; a file of another version is set aside, not read.
const VERSION: u32 = 1;

/// A marker as the journal stores it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct StoredMarker {
    guid: Option<String>,
    start_ms: u64,
    duration_ms: u64,
    name: String,
    comment: String,
    color: Option<u32>,
}

impl StoredMarker {
    fn new(marker: &Marker) -> Self {
        Self {
            guid: marker.guid.clone(),
            start_ms: marker.start_ms,
            duration_ms: marker.duration_ms,
            name: marker.name.clone(),
            comment: marker.comment.clone(),
            color: marker.color.value(),
        }
    }

    /// Whether `marker` is the one stored.
    fn is(&self, marker: &Marker) -> bool {
        self.guid == marker.guid
            && self.start_ms == marker.start_ms
            && self.duration_ms == marker.duration_ms
            && self.name == marker.name
            && self.comment == marker.comment
            && self.color == marker.color.value()
    }

    fn marker(&self) -> Marker {
        Marker {
            guid: self.guid.clone(),
            start_ms: self.start_ms,
            duration_ms: self.duration_ms,
            name: self.name.clone(),
            comment: self.comment.clone(),
            color: MarkerColor::from_value(self.color),
        }
    }
}

/// The unsaved state of one clip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    version: u32,
    /// Where the clip was on disk when this was written.
    pub path: PathBuf,
    /// The clip's size and modification time then: the edits are applied only to this clip.
    size: u64,
    modified_ms: u64,
    tags: Vec<String>,
    name: String,
    extension: String,
    comment: String,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    markers: Option<Vec<StoredMarker>>,
}

impl Entry {
    /// The entry for the clip at `path` with the state `snapshot` holds, or `None` when the
    /// clip cannot be read (it is gone).
    pub fn new(path: &Path, snapshot: &FileSnapshot) -> Option<Self> {
        let (size, modified_ms) = fingerprint(path)?;
        Some(Self::from_parts(path, snapshot, size, modified_ms))
    }

    fn from_parts(path: &Path, snapshot: &FileSnapshot, size: u64, modified_ms: u64) -> Self {
        Self {
            version: VERSION,
            path: path.to_path_buf(),
            size,
            modified_ms,
            tags: snapshot.tags().to_vec(),
            name: snapshot.name_without_extension().to_string(),
            extension: snapshot.extension().to_string(),
            comment: snapshot.comment().to_string(),
            segment_start: snapshot.segment_start(),
            segment_end: snapshot.segment_end(),
            markers: snapshot
                .markers()
                .map(|markers| markers.iter().map(StoredMarker::new).collect()),
        }
    }

    /// Whether `snapshot` holds the edits the entry holds. Unlike [`Entry::new`] it does not
    /// look at the file, so the clip's path, size and time are not compared (the entry's `path`
    /// is public for that): the interface asks on every frame. It builds no entry, so it
    /// copies nothing.
    pub fn same_state_as(&self, snapshot: &FileSnapshot) -> bool {
        self.tags == snapshot.tags()
            && self.name == snapshot.name_without_extension()
            && self.extension == snapshot.extension()
            && self.comment == snapshot.comment()
            && self.segment_start == snapshot.segment_start()
            && self.segment_end == snapshot.segment_end()
            && match (&self.markers, snapshot.markers()) {
                (None, None) => true,
                (Some(stored), Some(markers)) => {
                    stored.len() == markers.len()
                        && stored.iter().zip(markers).all(|(s, m)| s.is(m))
                }
                _ => false,
            }
    }

    /// The state the entry holds, as a snapshot of the clip it was made on.
    fn snapshot(&self) -> FileSnapshot {
        let initial = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut snapshot = FileSnapshot::new(
            self.tags.clone(),
            self.name.clone(),
            self.extension.clone(),
            initial,
        );
        snapshot.set_comment(self.comment.clone());
        snapshot.set_segment_start(self.segment_start);
        snapshot.set_segment_end(self.segment_end);
        snapshot.set_markers(
            self.markers
                .as_ref()
                .map(|markers| markers.iter().map(StoredMarker::marker).collect()),
        );
        snapshot
    }

    /// Whether `other` holds the same edits (the clip's size and time left out).
    pub fn same_state(&self, other: &Entry) -> bool {
        self.tags == other.tags
            && self.name == other.name
            && self.extension == other.extension
            && self.comment == other.comment
            && self.segment_start == other.segment_start
            && self.segment_end == other.segment_end
            && self.markers == other.markers
    }

    /// Whether the clip is still the one the entry was made on: same size and modification
    /// time. A clip changed since (by Premiere, by a rotation) must not be overwritten.
    fn clip_unchanged(&self) -> bool {
        fingerprint(&self.path) == Some((self.size, self.modified_ms))
    }

    /// How much the entry holds, for the message.
    pub fn summary(&self) -> Summary {
        Summary {
            tags: self.tags.len(),
            markers: self.markers.as_ref().map_or(0, Vec::len),
            comment: !self.comment.trim().is_empty(),
            in_out: self.segment_start.is_some() || self.segment_end.is_some(),
        }
    }

    /// The entry as text a person can read and retype from, for the copy kept beside it.
    pub fn readable(&self) -> String {
        let mut text = format!(
            "frename could not apply these unsaved edits to the clip:\n{}\n\nTags: {}\nName: {}.{}\n",
            self.path.display(),
            self.tags.join(", "),
            self.name,
            self.extension
        );
        if self.segment_start.is_some() || self.segment_end.is_some() {
            text.push_str(&format!(
                "In/out: {} - {} (seconds)\n",
                self.segment_start
                    .map_or("-".to_string(), |s| s.to_string()),
                self.segment_end.map_or("-".to_string(), |s| s.to_string())
            ));
        }
        if let Some(markers) = &self.markers {
            text.push_str("\nMarkers:\n");
            for marker in markers.iter().map(StoredMarker::marker) {
                text.push_str(&crate::format_marker_line(&marker));
                text.push('\n');
            }
        }
        if !self.comment.trim().is_empty() {
            text.push_str("\nComment:\n");
            text.push_str(&self.comment);
            text.push('\n');
        }
        text
    }
}

/// What an entry holds, as counts; the app words it in the editor's language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub tags: usize,
    pub markers: usize,
    pub comment: bool,
    pub in_out: bool,
}

/// The size and modification time (ms) of the file at `path`.
fn fingerprint(path: &Path) -> Option<(u64, u64)> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    Some((metadata.len(), modified.as_millis() as u64))
}

std::thread_local! {
    /// A journal folder for the current thread only, which tests use so that they neither share
    /// nor disturb the real one.
    static THIS_THREAD_DIR: std::cell::RefCell<Option<PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

/// Keep the journal in `dir` on the current thread only. For tests; the app never calls it.
#[doc(hidden)]
pub fn use_dir_on_this_thread(dir: PathBuf) {
    THIS_THREAD_DIR.with(|cell| *cell.borrow_mut() = Some(dir));
}

/// The journal folder.
pub fn journal_dir() -> PathBuf {
    THIS_THREAD_DIR
        .with(|cell| cell.borrow().clone())
        .unwrap_or_else(|| crate::app_data_dir().join("recovery"))
}

/// Where a clip's entry is kept in `dir`: one file per clip path.
fn entry_file(dir: &Path, clip: &Path) -> PathBuf {
    // FNV-1a of the path: stable across runs and Rust versions, unlike `DefaultHasher`.
    let hash = clip
        .to_string_lossy()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        });
    let name: String = clip
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .take(40)
        .collect();
    dir.join(format!("{hash:016x}-{name}.json"))
}

/// Write `entry` into the journal, replacing the clip's earlier one, atomically.
pub fn write(entry: &Entry) -> std::io::Result<()> {
    write_in(&journal_dir(), entry)
}

fn write_in(dir: &Path, entry: &Entry) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let target = entry_file(dir, &entry.path);
    let temporary = target.with_extension("json.tmp");
    let json = serde_json::to_vec(entry).map_err(std::io::Error::other)?;
    let mut file = std::fs::File::create(&temporary)?;
    file.write_all(&json)?;
    // On disk before it replaces the old entry: a power cut leaves one or the other whole.
    file.sync_all()?;
    drop(file);
    std::fs::rename(temporary, target)
}

/// Delete the clip's entry: its edits are applied or undone.
pub fn remove(clip: &Path) {
    remove_in(&journal_dir(), clip);
}

fn remove_in(dir: &Path, clip: &Path) {
    let file = entry_file(dir, clip);
    if let Err(e) = std::fs::remove_file(&file) {
        if e.kind() != std::io::ErrorKind::NotFound {
            log::warn!("recovery: could not remove {file:?}: {e}");
        }
    }
}

/// Proof that this is the only frename using the journal: only then are the entries in it
/// leftovers of a crash and not another running instance's live edits. Held for the life of
/// the process (the lock goes with it, even when it is killed).
pub struct InstanceLock {
    _file: std::fs::File,
}

impl InstanceLock {
    /// Take the lock, or `None` when another instance holds it.
    pub fn acquire() -> Option<Self> {
        Self::acquire_in(&journal_dir())
    }

    // `File::try_lock` is stable since Rust 1.89; the workspace's `rust-version` says 1.75 but
    // `rust-toolchain.toml` pins a newer one, which is what builds and CI use.
    #[allow(clippy::incompatible_msrv)]
    fn acquire_in(dir: &Path) -> Option<Self> {
        std::fs::create_dir_all(dir).ok()?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.join("instance.lock"))
            .ok()?;
        match file.try_lock() {
            Ok(()) => Some(Self { _file: file }),
            // Another frename holds it.
            Err(std::fs::TryLockError::WouldBlock) => None,
            Err(std::fs::TryLockError::Error(e)) => {
                log::warn!("recovery: the journal lock could not be taken: {e}");
                None
            }
        }
    }
}

/// Why an entry was not applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeptWhy {
    /// The clip is gone or was renamed.
    ClipGone,
    /// The clip changed after the edits were made (Premiere, a rotation, another program).
    ClipChanged,
    /// The save did not take: a file with that name exists, or the clip is read-only or in use.
    NotWritten,
}

/// What [`restore_all`] did with one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Restored {
    /// The edits were applied to the clip, now at `path`: `summary` says what they were.
    Applied { path: PathBuf, summary: Summary },
    /// The edits were not applied and are kept at `kept` (with a readable `.txt` beside it).
    Kept {
        clip: PathBuf,
        kept: PathBuf,
        why: KeptWhy,
        summary: Summary,
    },
    /// A file in the journal could not be read; it is set aside at `kept`.
    Ignored { kept: PathBuf },
}

/// Apply every entry left from an earlier run, once, before any clip is open (so nothing
/// locks the clips). Entries that cannot be applied safely are moved to `recovery/kept/`.
/// The lock proves no other frename is running and owns live entries.
pub fn restore_all(_alone: &InstanceLock) -> Vec<Restored> {
    restore_all_in(&journal_dir())
}

fn restore_all_in(dir: &Path) -> Vec<Restored> {
    // First the renames a crash cut in two, so an entry made on the old name finds its clip.
    let renamed = finish_renames_in(dir);
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let all: Vec<PathBuf> = read.filter_map(Result::ok).map(|e| e.path()).collect();
    // A write the crash cut short leaves its temporary file: it holds nothing whole.
    for temporary in all
        .iter()
        .filter(|p| p.to_string_lossy().ends_with(".json.tmp"))
    {
        let _ = std::fs::remove_file(temporary);
    }
    let mut files: Vec<PathBuf> = all
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut reports = Vec::new();
    for file in files {
        let entry = std::fs::read(&file)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Entry>(&bytes).ok())
            .filter(|entry| entry.version == VERSION);
        let Some(mut entry) = entry else {
            reports.push(Restored::Ignored {
                kept: set_aside(dir, &file, None),
            });
            continue;
        };
        if let Some((_, to)) = renamed.iter().find(|(from, _)| *from == entry.path) {
            entry.path = to.clone();
        }
        reports.push(restore_one(dir, &file, &entry));
    }
    reports
}

fn restore_one(dir: &Path, file: &Path, entry: &Entry) -> Restored {
    let keep = |why: KeptWhy| Restored::Kept {
        clip: entry.path.clone(),
        kept: set_aside(dir, file, Some(entry)),
        why,
        summary: entry.summary(),
    };
    if !entry.path.exists() {
        return keep(KeptWhy::ClipGone);
    }
    if !entry.clip_unchanged() {
        return keep(KeptWhy::ClipChanged);
    }
    match apply(entry) {
        Some(path) => {
            let _ = std::fs::remove_file(file);
            Restored::Applied {
                path,
                summary: entry.summary(),
            }
        }
        None => keep(KeptWhy::NotWritten),
    }
}

/// Whether two segment points are the same time: the file keeps them in milliseconds.
fn same_point(wanted: Option<f32>, saved: Option<f32>) -> bool {
    match (wanted, saved) {
        (None, None) => true,
        (Some(a), Some(b)) => (a - b).abs() < 0.002,
        _ => false,
    }
}

/// Whether a save did what the snapshot `wanted` asked: the tags and name, the comment, and the
/// in/out points read back from the file. A save that failed silently (a read-only clip, a name
/// taken) differs in at least one of them. A snapshot whose comment was not read stays out.
pub fn saved_what_was_wanted(wanted: &FileSnapshot, saved: &FileSnapshot) -> bool {
    let text = |s: &str| s.replace("\r\n", "\n").trim().to_string();
    wanted.tags() == saved.tags()
        && wanted.name_without_extension() == saved.name_without_extension()
        && wanted.extension().eq_ignore_ascii_case(saved.extension())
        && (wanted.comment_loading()
            || saved.comment_loading()
            || text(wanted.comment()) == text(saved.comment()))
        && same_point(wanted.segment_start(), saved.segment_start())
        && same_point(wanted.segment_end(), saved.segment_end())
}

/// Save the entry's state to its clip, the way leaving the clip would (keep the marker handling
/// in step with `FolderWorkspace::apply_file_updated`, the live save): markers first (while the
/// file still has its name), then the name, tags and comment. `Some(path)` of the clip after,
/// when what was wanted is really on disk; `None` when the save did not take.
fn apply(entry: &Entry) -> Option<PathBuf> {
    let mut snapshot = entry.snapshot();
    let path = &entry.path;
    // Markers the video holds and the entry still has as they were are not touched.
    let held = FileTagger::load_markers(path).unwrap_or_default();
    let mut clear_from_video = Vec::new();
    let mut markers_in_comment = Vec::new();
    if let Some(markers) = snapshot.markers().map(<[Marker]>::to_vec) {
        if markers != held {
            match crate::marker_storage() {
                crate::MarkerStorage::Comment => {
                    let comment = crate::markers_into_comment(snapshot.comment(), &markers);
                    snapshot.set_comment(comment);
                    clear_from_video = held.iter().filter_map(|m| m.guid.clone()).collect();
                    markers_in_comment = markers;
                }
                crate::MarkerStorage::InVideo => {
                    let known = held
                        .iter()
                        .chain(markers.iter())
                        .filter_map(|m| m.guid.clone())
                        .collect();
                    match FileTagger::save_markers(path, &markers, &known) {
                        Ok(()) => {}
                        // The clip holds none (its format, a damaged file): the other edits
                        // still go in, as when leaving the clip.
                        Err(
                            crate::MarkersError::CannotHoldMarkers | crate::MarkersError::Damaged,
                        ) => {}
                        Err(e) => {
                            log::warn!("recovery: markers of {path:?} not written: {e}");
                            return None;
                        }
                    }
                }
            }
        }
    }
    let wanted = snapshot.clone();
    let (new_path, saved) = snapshot.save_and_reparse(path);
    if !saved_what_was_wanted(&wanted, &saved) {
        return None;
    }
    // The markers are in the comment now: the video's own go, so the two never disagree. Only
    // when the comment holds them as they are: markers the line format would merge or cut (close
    // unnamed ones, one with a comment next to an AI block) stay in the video, where they are
    // kept whole, as when leaving the clip.
    if !clear_from_video.is_empty()
        && crate::comment_holds_markers(saved.comment(), &markers_in_comment)
    {
        let known = clear_from_video.into_iter().collect();
        if let Err(e) = FileTagger::save_markers(&new_path, &[], &known) {
            log::warn!("recovery: markers of {new_path:?} not taken out of the video: {e}");
        }
    }
    Some(new_path)
}

/// Move a journal file out of the way into `recovery/kept/` (never deleting or overwriting
/// another), with a readable copy beside it when it is an entry.
fn set_aside(dir: &Path, file: &Path, entry: Option<&Entry>) -> PathBuf {
    let kept_dir = dir.join("kept");
    let name = file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "entry.json".to_string());
    let stem = name.trim_end_matches(".json").to_string();
    let _ = std::fs::create_dir_all(&kept_dir);
    // A clip kept twice keeps both.
    let mut target = kept_dir.join(&name);
    for n in 1.. {
        if !target.exists() {
            break;
        }
        target = kept_dir.join(format!("{stem}-{n}.json"));
    }
    match std::fs::rename(file, &target) {
        Ok(()) => {
            if let Some(entry) = entry {
                let _ = std::fs::write(target.with_extension("txt"), entry.readable());
            }
            target
        }
        Err(e) => {
            log::warn!("recovery: could not set {file:?} aside: {e}");
            file.to_path_buf()
        }
    }
}

// ---------------------------------------------------------------------------
// Clip renames that a crash cut in two (#377)
// ---------------------------------------------------------------------------

/// A clip rename in progress: written before the video is renamed, removed once its sidecars
/// have followed it. One left at the next start means the run stopped between the two.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RenameIntent {
    version: u32,
    from: PathBuf,
    to: PathBuf,
}

/// Where rename intents are kept: a folder of the journal, so only the one frename that holds
/// the [`InstanceLock`] finishes them.
fn renames_dir(journal: &Path) -> PathBuf {
    journal.join("renames")
}

/// Rename the clip `from` to `to`, then its comment, subtitle, transcript and screenshot files.
/// An intent is on disk first, so a crash, a kill or a power cut between the two steps is
/// finished at the next start ([`restore_all`]) instead of leaving the sidecars under the old
/// name for good. The caller has checked that `to` and its sidecars are free. An error means the
/// video was not renamed, and nothing else was.
pub(crate) fn rename_clip(from: &Path, to: &Path) -> std::io::Result<()> {
    rename_clip_in(&journal_dir(), from, to)
}

fn rename_clip_in(journal: &Path, from: &Path, to: &Path) -> std::io::Result<()> {
    let intent = write_intent(journal, from, to);
    if let Err(e) = std::fs::rename(from, to) {
        remove_intent(intent.as_deref());
        return Err(e);
    }
    crate::tags::rename_sidecars(from, to);
    remove_intent(intent.as_deref());
    Ok(())
}

/// Write the intent of renaming `from` to `to`, flushed to disk. `None` when it cannot be
/// written: the rename still goes on, as it did before intents existed.
fn write_intent(journal: &Path, from: &Path, to: &Path) -> Option<PathBuf> {
    let intent = RenameIntent {
        version: VERSION,
        from: from.to_path_buf(),
        to: to.to_path_buf(),
    };
    let dir = renames_dir(journal);
    let file = entry_file(&dir, from);
    let written = (|| -> std::io::Result<()> {
        std::fs::create_dir_all(&dir)?;
        let temporary = file.with_extension("json.tmp");
        let json = serde_json::to_vec(&intent).map_err(std::io::Error::other)?;
        let mut out = std::fs::File::create(&temporary)?;
        out.write_all(&json)?;
        // On disk before the video moves: the next start must find it after a power cut.
        out.sync_all()?;
        drop(out);
        std::fs::rename(temporary, &file)
    })();
    match written {
        Ok(()) => Some(file),
        Err(e) => {
            log::warn!("recovery: the rename of {from:?} is not journaled: {e}");
            None
        }
    }
}

fn remove_intent(file: Option<&Path>) {
    if let Some(file) = file {
        if let Err(e) = std::fs::remove_file(file) {
            log::warn!("recovery: could not remove {file:?}: {e}");
        }
    }
}

/// Finish the clip renames an earlier run started and did not end: the video is under its new
/// name and the old one is gone, so the sidecars still under the old name are moved after it.
/// A rename that never happened (the video still has its old name) is simply forgotten.
/// Returns the renames that had happened, `(from, to)`.
fn finish_renames_in(journal: &Path) -> Vec<(PathBuf, PathBuf)> {
    let dir = renames_dir(journal);
    let Ok(read) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = read.filter_map(Result::ok).map(|e| e.path()).collect();
    files.sort();
    let mut finished = Vec::new();
    for file in files {
        if !file.extension().is_some_and(|e| e == "json") {
            // A temporary file the crash cut short: the rename had not started.
            let _ = std::fs::remove_file(&file);
            continue;
        }
        let intent = std::fs::read(&file)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<RenameIntent>(&bytes).ok())
            .filter(|intent| intent.version == VERSION);
        if let Some(RenameIntent { from, to, .. }) = intent {
            // Both names present: the rename did not happen (or only changed the case of the
            // name on a file system where the sidecars are found in any case); nothing to move.
            if to.exists() && !from.exists() {
                finish_sidecars(&from, &to);
                log::info!("recovery: finished the rename {from:?} → {to:?}");
                finished.push((from, to));
            }
        } else {
            log::warn!("recovery: an unreadable rename intent {file:?} is dropped");
        }
        let _ = std::fs::remove_file(&file);
    }
    finished
}

/// Move the sidecars still under `from`'s names to `to`'s, never replacing a file: the comment
/// rename would replace one, so a comment already under the new name keeps the old one in place.
fn finish_sidecars(from: &Path, to: &Path) {
    let (old_comment, new_comment) = (
        crate::comment::comment_path(from),
        crate::comment::comment_path(to),
    );
    if old_comment.exists() && new_comment.exists() {
        log::warn!("recovery: {old_comment:?} stays: {new_comment:?} already exists");
        crate::subtitles::rename_subtitle_file(from, to);
        crate::tags::rename_screenshots(from, to);
        return;
    }
    crate::tags::rename_sidecars(from, to);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        crate::test_support::fresh_dir(&format!("recovery-{name}"))
    }

    fn clip(dir: &Path) -> PathBuf {
        let path = dir.join("clip.mp4");
        std::fs::write(&path, b"video").expect("clip");
        path
    }

    fn entry_for(path: &Path) -> Entry {
        let mut snapshot = FileSnapshot::parse("clip.mp4");
        snapshot.set_tags(["pick", "wide"]);
        snapshot.set_comment("A note".to_string());
        let mut marker = Marker::new(5_000);
        marker.name = "Lion".to_string();
        marker.color = MarkerColor::Red;
        snapshot.set_markers(Some(vec![marker]));
        Entry::new(path, &snapshot).expect("entry")
    }

    #[test]
    fn an_entry_compares_with_a_snapshot_without_touching_the_file() {
        let dir = temp_dir("same-state-as");
        let path = clip(&dir);
        let mut snapshot = FileSnapshot::parse("clip.mp4");
        snapshot.set_tags(["pick", "wide"]);
        snapshot.set_comment("A note".to_string());
        let mut marker = Marker::new(5_000);
        marker.name = "Lion".to_string();
        snapshot.set_markers(Some(vec![marker]));
        let entry = Entry::new(&path, &snapshot).expect("entry");
        // The clip is gone: a comparison that looked at it would fail.
        std::fs::remove_file(&path).expect("remove");
        assert!(entry.same_state_as(&snapshot));

        let mut other = snapshot.clone();
        other.set_tags(["pick"]);
        assert!(!entry.same_state_as(&other), "tags");
        let mut other = snapshot.clone();
        other.set_markers(None);
        assert!(!entry.same_state_as(&other), "markers");
        let mut other = snapshot.clone();
        let mut renamed = other.markers().expect("markers").to_vec();
        renamed[0].name = "Tiger".to_string();
        other.set_markers(Some(renamed));
        assert!(!entry.same_state_as(&other), "a marker's name");
    }

    #[test]
    fn an_entry_survives_the_journal_and_says_what_it_holds() {
        let dir = temp_dir("roundtrip");
        let path = clip(&dir);
        let entry = entry_for(&path);
        write_in(&dir.join("recovery"), &entry).expect("write");
        let file = entry_file(&dir.join("recovery"), &path);
        let back: Entry =
            serde_json::from_slice(&std::fs::read(file).expect("read")).expect("json");
        assert_eq!(back, entry);
        assert_eq!(
            entry.summary(),
            Summary {
                tags: 2,
                markers: 1,
                comment: true,
                in_out: false
            }
        );
        let snapshot = back.snapshot();
        assert_eq!(snapshot.tags(), ["pick", "wide"]);
        assert_eq!(snapshot.markers().map(<[Marker]>::len), Some(1));
        assert_eq!(
            snapshot.markers().expect("markers")[0].color,
            MarkerColor::Red
        );
    }

    #[test]
    fn a_second_write_replaces_the_first_and_leaves_no_temporary_file() {
        let dir = temp_dir("replace");
        let path = clip(&dir);
        let journal = dir.join("recovery");
        let mut entry = entry_for(&path);
        write_in(&journal, &entry).expect("write");
        entry.comment = "Newer".to_string();
        write_in(&journal, &entry).expect("write again");
        let files: Vec<_> = std::fs::read_dir(&journal)
            .expect("dir")
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(files.len(), 1, "{files:?}");
        assert!(files[0].ends_with(".json"));
        let back: Entry =
            serde_json::from_slice(&std::fs::read(entry_file(&journal, &path)).expect("read"))
                .expect("json");
        assert_eq!(back.comment, "Newer");
        remove_in(&journal, &path);
        assert!(!entry_file(&journal, &path).exists());
    }

    #[test]
    fn a_truncated_or_foreign_file_is_set_aside_not_a_crash() {
        let dir = temp_dir("truncated");
        let path = clip(&dir);
        let journal = dir.join("recovery");
        write_in(&journal, &entry_for(&path)).expect("write");
        let file = entry_file(&journal, &path);
        let bytes = std::fs::read(&file).expect("read");
        std::fs::write(&file, &bytes[..bytes.len() / 2]).expect("truncate");
        std::fs::write(journal.join("other.json"), b"{\"version\": 99}").expect("foreign");
        let reports = restore_all_in(&journal);
        assert_eq!(reports.len(), 2);
        assert!(reports
            .iter()
            .all(|r| matches!(r, Restored::Ignored { .. })));
        assert_eq!(
            std::fs::read_dir(journal.join("kept"))
                .expect("kept")
                .count(),
            2
        );
        assert!(restore_all_in(&journal).is_empty(), "nothing left to read");
    }

    #[test]
    fn a_clip_changed_since_is_not_overwritten_and_the_entry_is_kept() {
        let dir = temp_dir("changed");
        let path = clip(&dir);
        let journal = dir.join("recovery");
        write_in(&journal, &entry_for(&path)).expect("write");
        std::fs::write(&path, b"another video entirely").expect("change the clip");
        let reports = restore_all_in(&journal);
        assert_eq!(reports.len(), 1);
        match &reports[0] {
            Restored::Kept { kept, why, .. } => {
                assert!(kept.exists(), "the entry is kept");
                assert!(kept.with_extension("txt").exists(), "with a readable copy");
                assert_eq!(*why, KeptWhy::ClipChanged);
                let text = std::fs::read_to_string(kept.with_extension("txt")).expect("text");
                assert!(
                    text.contains("pick, wide") && text.contains("A note"),
                    "{text}"
                );
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(
            std::fs::read(&path).expect("clip"),
            b"another video entirely"
        );
    }

    #[test]
    fn a_clip_that_is_gone_keeps_its_entry() {
        let dir = temp_dir("gone");
        let path = clip(&dir);
        let journal = dir.join("recovery");
        write_in(&journal, &entry_for(&path)).expect("write");
        std::fs::remove_file(&path).expect("remove");
        assert!(matches!(
            restore_all_in(&journal).as_slice(),
            [Restored::Kept { .. }]
        ));
    }

    #[test]
    fn an_unchanged_clip_gets_its_edits_and_the_entry_goes() {
        let dir = temp_dir("applied");
        let path = clip(&dir);
        let journal = dir.join("recovery");
        write_in(&journal, &entry_for(&path)).expect("write");
        let reports = restore_all_in(&journal);
        match reports.as_slice() {
            [Restored::Applied { summary, .. }] => {
                assert_eq!(summary.tags, 2);
                assert_eq!(summary.markers, 1);
            }
            other => panic!("{other:?}"),
        }
        assert!(
            !entry_file(&journal, &path).exists(),
            "a restored entry is gone"
        );
    }

    /// #377: a run that stopped after the video was renamed and before its sidecars followed
    /// has them moved at the next start, and forgets the intent.
    #[test]
    fn a_rename_cut_in_two_by_a_crash_is_finished_at_the_next_start() {
        let dir = temp_dir("rename-cut");
        let from = clip(&dir);
        let to = dir.join("pick.clip.mp4");
        let journal = dir.join("recovery");
        std::fs::write(crate::comment::comment_path(&from), "A note").expect("comment");
        std::fs::write(dir.join("clip.srt"), "1").expect("srt");
        // The crash: the intent is written and the video renamed, nothing more.
        assert!(write_intent(&journal, &from, &to).is_some());
        std::fs::rename(&from, &to).expect("rename");

        assert!(restore_all_in(&journal).is_empty());

        assert_eq!(
            std::fs::read_to_string(crate::comment::comment_path(&to)).expect("comment moved"),
            "A note"
        );
        assert!(dir.join("pick.clip.srt").exists(), "the subtitles follow");
        assert!(!crate::comment::comment_path(&from).exists());
        assert!(!dir.join("clip.srt").exists());
        assert_eq!(
            std::fs::read_dir(renames_dir(&journal))
                .expect("dir")
                .count(),
            0,
            "the intent is gone"
        );
    }

    /// The journal entry made on the old name is applied to the clip under its new one, not
    /// kept aside as "clip gone".
    #[test]
    fn an_entry_made_before_a_cut_rename_finds_its_clip() {
        let dir = temp_dir("rename-cut-entry");
        let from = clip(&dir);
        let to = dir.join("renamed.mp4");
        let journal = dir.join("recovery");
        write_in(&journal, &entry_for(&from)).expect("write");
        assert!(write_intent(&journal, &from, &to).is_some());
        std::fs::rename(&from, &to).expect("rename");

        match restore_all_in(&journal).as_slice() {
            // Before #377 it was kept aside as `ClipGone`.
            [Restored::Applied { path, .. }] => {
                assert_eq!(path, &dir.join("pick.wide.clip.mp4"));
            }
            other => panic!("{other:?}"),
        }
    }

    /// An intent whose rename never happened (the video still has its old name) moves nothing.
    #[test]
    fn an_intent_without_its_rename_moves_nothing() {
        let dir = temp_dir("rename-not-started");
        let from = clip(&dir);
        let to = dir.join("pick.clip.mp4");
        let journal = dir.join("recovery");
        std::fs::write(crate::comment::comment_path(&from), "A note").expect("comment");
        assert!(write_intent(&journal, &from, &to).is_some());
        std::fs::write(renames_dir(&journal).join("y.json.tmp"), b"half").expect("temporary");

        assert!(restore_all_in(&journal).is_empty());

        assert!(from.exists() && !to.exists());
        assert!(crate::comment::comment_path(&from).exists());
        assert!(!crate::comment::comment_path(&to).exists());
        assert_eq!(
            std::fs::read_dir(renames_dir(&journal))
                .expect("dir")
                .count(),
            0
        );
    }

    /// A comment already under the new name is never replaced by the one left behind.
    #[test]
    fn finishing_a_rename_never_replaces_a_comment() {
        let dir = temp_dir("rename-cut-comment-taken");
        let from = clip(&dir);
        let to = dir.join("pick.clip.mp4");
        let journal = dir.join("recovery");
        std::fs::write(crate::comment::comment_path(&from), "old").expect("comment");
        assert!(write_intent(&journal, &from, &to).is_some());
        std::fs::rename(&from, &to).expect("rename");
        std::fs::write(crate::comment::comment_path(&to), "new").expect("comment");
        std::fs::write(dir.join("clip.srt"), "1").expect("srt");

        restore_all_in(&journal);

        assert_eq!(
            std::fs::read_to_string(crate::comment::comment_path(&to)).expect("comment"),
            "new"
        );
        assert!(crate::comment::comment_path(&from).exists());
        assert!(
            dir.join("pick.clip.srt").exists(),
            "the subtitles still follow"
        );
    }

    /// A whole rename moves the video and its sidecars and leaves no intent behind; a failed
    /// one changes nothing and leaves none either.
    #[test]
    fn a_whole_rename_leaves_no_intent() {
        let dir = temp_dir("rename-whole");
        let from = clip(&dir);
        let to = dir.join("pick.clip.mp4");
        let journal = dir.join("recovery");
        std::fs::write(crate::comment::comment_path(&from), "A note").expect("comment");

        rename_clip_in(&journal, &from, &to).expect("rename");
        assert!(to.exists() && crate::comment::comment_path(&to).exists());
        assert!(!from.exists() && !crate::comment::comment_path(&from).exists());

        assert!(rename_clip_in(&journal, &dir.join("missing.mp4"), &dir.join("x.mp4")).is_err());
        assert_eq!(
            std::fs::read_dir(renames_dir(&journal))
                .expect("dir")
                .count(),
            0
        );
    }

    #[test]
    fn only_one_instance_holds_the_journal() {
        let dir = temp_dir("lock");
        let first = InstanceLock::acquire_in(&dir).expect("the first instance");
        assert!(
            InstanceLock::acquire_in(&dir).is_none(),
            "a second one must not restore"
        );
        drop(first);
        assert!(InstanceLock::acquire_in(&dir).is_some(), "free again");
    }

    #[test]
    fn a_clip_kept_twice_keeps_both_and_stale_temporary_files_go() {
        let dir = temp_dir("kept-twice");
        let path = clip(&dir);
        let journal = dir.join("recovery");
        for _ in 0..2 {
            write_in(&journal, &entry_for(&path)).expect("write");
            std::fs::write(&path, format!("changed {}", rand_suffix())).expect("change");
            std::fs::write(journal.join("x.json.tmp"), b"half").expect("temporary");
            assert!(matches!(
                restore_all_in(&journal).as_slice(),
                [Restored::Kept { .. }]
            ));
            // The next crash is on the changed clip as it is now.
            std::fs::write(&path, b"video").expect("back");
        }
        let kept: Vec<_> = std::fs::read_dir(journal.join("kept"))
            .expect("kept")
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .collect();
        assert_eq!(kept.len(), 2, "the first one is not overwritten");
        assert!(!journal.join("x.json.tmp").exists());
    }

    /// Where the child of the kill test finds its clip and journal folder (set by the parent).
    const KILL_DIR: &str = "FRENAME_RECOVERY_KILL_TEST_DIR";

    /// The child of `a_process_killed_while_editing_loses_nothing_the_journal_held`: it journals
    /// two states of a clip, starts a third write and dies with `abort` in the middle of it, as
    /// a kill or a crash would. Without the variable (a normal test run) it does nothing.
    #[test]
    fn kill_test_child() {
        let Some(dir) = std::env::var_os(KILL_DIR).map(PathBuf::from) else {
            return;
        };
        let path = dir.join("clip.mp4");
        let journal = dir.join("recovery");
        write_in(&journal, &entry_for(&path)).expect("first write");
        let mut snapshot = FileSnapshot::parse("clip.mp4");
        snapshot.set_tags(["pick", "wide", "late"]);
        write_in(&journal, &Entry::new(&path, &snapshot).expect("entry")).expect("second write");
        // The third write is cut short: only its temporary file exists.
        std::fs::write(
            entry_file(&journal, &path).with_extension("json.tmp"),
            b"{\"ver",
        )
        .expect("temporary");
        std::process::abort();
    }

    #[test]
    fn a_process_killed_while_editing_loses_nothing_the_journal_held() {
        let dir = temp_dir("killed");
        let path = clip(&dir);
        let journal = dir.join("recovery");
        let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "recovery::tests::kill_test_child",
                "--test-threads=1",
            ])
            .env(KILL_DIR, &dir)
            .status()
            .expect("the child starts");
        assert!(
            !status.success(),
            "the child died instead of ending: {status:?}"
        );
        assert!(
            entry_file(&journal, &path).exists(),
            "the entry written before the kill is on disk"
        );
        match restore_all_in(&journal).as_slice() {
            [Restored::Applied { summary, .. }] => {
                assert_eq!(summary.tags, 3, "the last whole entry, not the first");
            }
            other => panic!("{other:?}"),
        }
        assert!(
            std::fs::read_dir(&journal).expect("journal").all(|e| e
                .expect("entry")
                .path()
                .extension()
                .is_none_or(|x| x != "tmp")),
            "the cut-short write left nothing behind"
        );
    }

    fn rand_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    }

    #[test]
    fn a_save_that_did_not_take_is_told_from_one_that_did() {
        let mut wanted = FileSnapshot::parse("pick.clip.mp4");
        wanted.set_comment("A note".to_string());
        wanted.set_segment_start(Some(1.5));
        assert!(saved_what_was_wanted(&wanted, &wanted.clone()));
        let mut no_comment = wanted.clone();
        no_comment.set_comment(String::new());
        assert!(
            !saved_what_was_wanted(&wanted, &no_comment),
            "the comment did not get written"
        );
        let mut no_point = wanted.clone();
        no_point.set_segment_start(None);
        assert!(
            !saved_what_was_wanted(&wanted, &no_point),
            "the in point did not get written"
        );
        let mut milliseconds = wanted.clone();
        milliseconds.set_segment_start(Some(1.5004));
        assert!(saved_what_was_wanted(&wanted, &milliseconds));
        let other_name = FileSnapshot::parse("clip.mp4");
        assert!(
            !saved_what_was_wanted(&wanted, &other_name),
            "the tag did not get written"
        );
    }
}
