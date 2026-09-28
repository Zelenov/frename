//! In-memory FileTagger: no disk access. Used in debug/test mode.

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

    fn save_markers(
        &self,
        path: &Path,
        markers: &[Marker],
        _known: &HashSet<String>,
    ) -> Result<(), MarkersError> {
        self.markers
            .lock()
            .expect("lock")
            .insert(self.disk_path(path), markers.to_vec());
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

    /// A debug build turns videos in memory only: the file keeps its bytes, and the turn
    /// follows the file through in-memory renames.
    #[test]
    fn rotations_stay_in_memory() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wide.mp4");
        let dir =
            std::env::temp_dir().join(format!("frename-memory-rotation-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
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
