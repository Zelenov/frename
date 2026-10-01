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
        Some(Self {
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
        })
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

    /// How much the entry holds, for the message: `5 tags, 3 markers, comment`.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.tags.is_empty() {
            parts.push(count(self.tags.len(), "tag"));
        }
        let markers = self.markers.as_ref().map_or(0, Vec::len);
        if markers > 0 {
            parts.push(count(markers, "marker"));
        }
        if !self.comment.trim().is_empty() {
            parts.push("comment".to_string());
        }
        if self.segment_start.is_some() || self.segment_end.is_some() {
            parts.push("in/out points".to_string());
        }
        if parts.is_empty() {
            parts.push("edits".to_string());
        }
        parts.join(", ")
    }
}

fn count(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
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

/// What [`restore_all`] did with one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Restored {
    /// The edits were applied to the clip, now at `path`: `summary` says what they were.
    Applied { path: PathBuf, summary: String },
    /// The edits were not applied and are kept at `kept`: `why` says why.
    Kept {
        clip: PathBuf,
        kept: PathBuf,
        why: String,
    },
    /// A file in the journal could not be read; it is set aside at `kept`.
    Ignored { kept: PathBuf },
}

/// Apply every entry left from an earlier run, once, before any clip is open (so nothing
/// locks the clips). Entries that cannot be applied safely are moved to `recovery/kept/`.
pub fn restore_all() -> Vec<Restored> {
    restore_all_in(&journal_dir())
}

fn restore_all_in(dir: &Path) -> Vec<Restored> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = read
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut reports = Vec::new();
    for file in files {
        let entry = std::fs::read(&file)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Entry>(&bytes).ok())
            .filter(|entry| entry.version == VERSION);
        let Some(entry) = entry else {
            reports.push(Restored::Ignored {
                kept: set_aside(dir, &file),
            });
            continue;
        };
        reports.push(restore_one(dir, &file, &entry));
    }
    reports
}

fn restore_one(dir: &Path, file: &Path, entry: &Entry) -> Restored {
    let keep = |why: &str| Restored::Kept {
        clip: entry.path.clone(),
        kept: set_aside(dir, file),
        why: why.to_string(),
    };
    if !entry.path.exists() {
        return keep("the clip is gone or was renamed");
    }
    if !entry.clip_unchanged() {
        return keep("the clip changed after the edits were made");
    }
    match apply(entry) {
        Ok(path) => {
            let _ = std::fs::remove_file(file);
            Restored::Applied {
                path,
                summary: entry.summary(),
            }
        }
        Err(why) => keep(&why),
    }
}

/// Save the entry's state to its clip, the way leaving the clip would: markers first (while the
/// file still has its name), then the name, tags and comment. Returns the clip's path after.
fn apply(entry: &Entry) -> Result<PathBuf, String> {
    let mut snapshot = entry.snapshot();
    let path = &entry.path;
    if let Some(markers) = snapshot.markers().map(<[Marker]>::to_vec) {
        match crate::marker_storage() {
            crate::MarkerStorage::Comment => {
                let comment = crate::markers_into_comment(snapshot.comment(), &markers);
                snapshot.set_comment(comment);
            }
            crate::MarkerStorage::InVideo => {
                let known = FileTagger::load_markers(path)
                    .unwrap_or_default()
                    .iter()
                    .chain(markers.iter())
                    .filter_map(|m| m.guid.clone())
                    .collect();
                FileTagger::save_markers(path, &markers, &known)
                    .map_err(|e| format!("the markers could not be written: {e}"))?;
            }
        }
    }
    let wanted = snapshot.clone();
    let (new_path, saved) = snapshot.save_and_reparse(path);
    if wanted.tags() != saved.tags()
        || wanted.name_without_extension() != saved.name_without_extension()
    {
        return Err("a file with that name already exists, or the clip is in use".to_string());
    }
    Ok(new_path)
}

/// Move a journal file out of the way into `recovery/kept/` (never deleting it).
fn set_aside(dir: &Path, file: &Path) -> PathBuf {
    let kept_dir = dir.join("kept");
    let name = file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "entry.json".to_string());
    let target = kept_dir.join(&name);
    let moved = std::fs::create_dir_all(&kept_dir).and_then(|()| std::fs::rename(file, &target));
    match moved {
        Ok(()) => target,
        Err(e) => {
            log::warn!("recovery: could not set {file:?} aside: {e}");
            file.to_path_buf()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("frename-recovery-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
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
    fn an_entry_survives_the_journal_and_says_what_it_holds() {
        let dir = temp_dir("roundtrip");
        let path = clip(&dir);
        let entry = entry_for(&path);
        write_in(&dir.join("recovery"), &entry).expect("write");
        let file = entry_file(&dir.join("recovery"), &path);
        let back: Entry =
            serde_json::from_slice(&std::fs::read(file).expect("read")).expect("json");
        assert_eq!(back, entry);
        assert_eq!(entry.summary(), "2 tags, 1 marker, comment");
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
                assert!(why.contains("changed"), "{why}");
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
                assert_eq!(summary, "2 tags, 1 marker, comment");
            }
            other => panic!("{other:?}"),
        }
        assert!(
            !entry_file(&journal, &path).exists(),
            "a restored entry is gone"
        );
    }
}
