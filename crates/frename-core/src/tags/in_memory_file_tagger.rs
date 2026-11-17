//! In-memory FileTagger: no disk access, except the read-only existence checks `save` makes for
//! its overwrite guard (issue #84), which mirror `ProductionFileTagger`'s own. Used in debug/test
//! mode.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::FileTaggerBackend;
use super::FolderInfo;
use crate::markers::Marker;
use crate::metadata::{MarkersError, Rotation, RotationError};

#[derive(Default)]
pub struct InMemoryFileTagger {
    storage: Mutex<HashMap<PathBuf, FileSnapshot>>,
    /// Where each file "renamed" in memory still is on disk, by its in-memory path.
    disk_paths: Mutex<HashMap<PathBuf, PathBuf>>,
    /// Markers "written" to each file, by where it is on disk.
    markers: Mutex<HashMap<PathBuf, Vec<Marker>>>,
    /// Rotations "written" to each file, by where it is on disk.
    rotations: Mutex<HashMap<PathBuf, Rotation>>,
}

impl FileTaggerBackend for InMemoryFileTagger {
    fn parse(&self, path: &Path, _folder_info: &FolderInfo) -> FileSnapshot {
        let stored = self.storage.lock().expect("lock").get(path).cloned();
        if let Some(snap) = stored {
            return snap;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        FileSnapshot::parse(name)
    }

    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        let new_file_name = snapshot.file_name();
        let new_path = path
            .parent()
            .map(|p| p.join(&new_file_name))
            .unwrap_or_else(|| PathBuf::from(&new_file_name));
        // Mirrors ProductionFileTagger's overwrite guard (issue #84): a real file (or sidecar)
        // already at the target name must refuse the same way here, or this backend (meant to be
        // "always correct for tests and debug builds", see the module doc) would let a test's
        // directory of real files be silently clobbered where production would not.
        if super::file_tagger::target_name_taken(path, &new_path) {
            return path.to_path_buf();
        }
        self.storage
            .lock()
            .expect("lock")
            .insert(new_path.clone(), snapshot.clone());
        let on_disk = self.disk_path(path);
        self.disk_paths
            .lock()
            .expect("lock")
            .insert(new_path.clone(), on_disk);
        new_path
    }

    fn load_markers(&self, path: &Path) -> Option<Vec<Marker>> {
        let on_disk = self.disk_path(path);
        let saved = self.markers.lock().expect("lock").get(&on_disk).cloned();
        saved.or_else(|| crate::metadata::load_markers(&on_disk))
    }

    /// Merges like the file does (`markers_xmp::apply_markers`): a marker of `markers` replaces
    /// the one with its GUID or is added after the others; a marker already there whose GUID
    /// frename knew (`known`) and that is no longer wanted goes; one it never knew (made
    /// elsewhere, e.g. in Premiere, since frename last read the file) stays (#518). A marker
    /// without a GUID is not written, as in the file.
    fn save_markers(
        &self,
        path: &Path,
        markers: &[Marker],
        known: &HashSet<String>,
    ) -> Result<(), MarkersError> {
        let guid = |marker: &Marker| marker.guid.as_deref().map(crate::metadata::normalize_guid);
        let known: HashSet<String> = known
            .iter()
            .map(|g| crate::metadata::normalize_guid(g))
            .collect();
        let existing = self.load_markers(path).unwrap_or_default();
        let mut merged: Vec<Marker> = Vec::with_capacity(existing.len() + markers.len());
        for marker in existing {
            let Some(id) = guid(&marker) else {
                merged.push(marker);
                continue;
            };
            match markers.iter().find(|m| guid(m).as_ref() == Some(&id)) {
                Some(wanted) => merged.push(wanted.clone()),
                None if known.contains(&id) => {}
                None => merged.push(marker),
            }
        }
        for wanted in markers {
            let Some(id) = guid(wanted) else { continue };
            if !merged.iter().any(|m| guid(m).as_ref() == Some(&id)) {
                merged.push(wanted.clone());
            }
        }
        // The file keeps them in time order when read back.
        crate::markers::sort_markers(&mut merged);
        self.markers
            .lock()
            .expect("lock")
            .insert(self.disk_path(path), merged);
        Ok(())
    }

    fn video_rotation(&self, path: &Path) -> Result<Rotation, RotationError> {
        let on_disk = self.disk_path(path);
        let saved = self.rotations.lock().expect("lock").get(&on_disk).copied();
        // A file that cannot turn says so even after a turn was kept for it (it cannot have one).
        let from_file = crate::metadata::rotation::read(&on_disk)?;
        Ok(saved.unwrap_or(from_file))
    }

    fn rotate_video(&self, path: &Path, quarter_turns: i32) -> Result<Rotation, RotationError> {
        let turned = self.video_rotation(path)?.turned(quarter_turns);
        self.rotations
            .lock()
            .expect("lock")
            .insert(self.disk_path(path), turned);
        Ok(turned)
    }

    fn disk_path(&self, path: &Path) -> PathBuf {
        self.disk_paths
            .lock()
            .expect("lock")
            .get(path)
            .cloned()
            .unwrap_or_else(|| path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Renames stay in memory, so a renamed file is still found on disk under its first name,
    /// however many times it was renamed.
    #[test]
    fn renamed_files_are_still_found_on_disk() {
        let tagger = InMemoryFileTagger::default();
        let original = Path::new("shoots/clip.mp4");
        let mut snapshot = FileSnapshot::parse("clip.mp4");
        snapshot.set_tags(["Goat"]);
        let renamed = tagger.save(&snapshot, original);
        assert_eq!(renamed, Path::new("shoots/Goat.clip.mp4"));
        snapshot.set_tags(["Goat", "Commented"]);
        let renamed = tagger.save(&snapshot, &renamed);
        assert_eq!(tagger.disk_path(&renamed), original);
        assert_eq!(tagger.disk_path(original), original);
    }

    /// #518: like the file, a save keeps a marker frename never knew (made elsewhere since it
    /// last read the clip), drops one it knew and no longer wants, and updates one by its GUID.
    #[test]
    fn saved_markers_keep_the_ones_frename_never_knew() {
        let tagger = InMemoryFileTagger::default();
        let path = Path::new("shoots/no-such-clip.mp4");
        let named = |start_ms: u64, name: &str| {
            let mut marker = Marker::new(start_ms);
            marker.name = name.to_string();
            marker
        };
        let mut kept = named(1_000, "Kept");
        let gone = named(2_000, "Gone");
        let outside = named(3_000, "Outside");
        tagger
            .save_markers(path, &[kept.clone(), gone.clone()], &HashSet::new())
            .expect("first");
        tagger
            .save_markers(path, &[outside], &HashSet::new())
            .expect("outside");
        kept.name = "Kept, renamed".to_string();
        let known: HashSet<String> = [&kept, &gone]
            .iter()
            .filter_map(|m| m.guid.as_ref().map(|g| g.to_ascii_uppercase()))
            .collect();
        tagger.save_markers(path, &[kept], &known).expect("again");
        let names: Vec<String> = tagger
            .load_markers(path)
            .expect("markers")
            .into_iter()
            .map(|m| m.name)
            .collect();
        assert_eq!(names, ["Kept, renamed", "Outside"]);
    }

    /// A debug build turns videos in memory only: the file keeps its bytes, and the turn
    /// follows the file through in-memory renames.
    #[test]
    fn rotations_stay_in_memory() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wide.mp4");
        let dir = crate::test_support::fresh_dir("memory-rotation");
        let original = dir.join("clip.mp4");
        std::fs::copy(fixture, &original).expect("copy");
        let bytes = std::fs::read(&original).expect("read");

        let tagger = InMemoryFileTagger::default();
        assert_eq!(tagger.video_rotation(&original), Ok(Rotation::UPRIGHT));
        assert_eq!(
            tagger.rotate_video(&original, 1).map(Rotation::degrees),
            Ok(90)
        );
        let mut snapshot = FileSnapshot::parse("clip.mp4");
        snapshot.set_tags(["Goat"]);
        let renamed = tagger.save(&snapshot, &original);
        assert_eq!(
            tagger.rotate_video(&renamed, 1).map(Rotation::degrees),
            Ok(180)
        );
        assert_eq!(
            tagger.video_rotation(&original).map(Rotation::degrees),
            Ok(180)
        );
        assert_eq!(std::fs::read(&original).expect("read"), bytes);

        let text = dir.join("notes.mkv");
        std::fs::write(&text, b"no").expect("write");
        assert_eq!(
            tagger.rotate_video(&text, 1),
            Err(RotationError::CannotRotate)
        );
    }
}
