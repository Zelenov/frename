//! In-memory FileTagger: no disk access. Used in debug/test mode.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::FileTaggerBackend;
use super::FolderInfo;
use crate::markers::Marker;
use crate::metadata::MarkersError;

#[derive(Default)]
pub struct InMemoryFileTagger {
    storage: Mutex<HashMap<PathBuf, FileSnapshot>>,
    /// Where each file "renamed" in memory still is on disk, by its in-memory path.
    disk_paths: Mutex<HashMap<PathBuf, PathBuf>>,
    /// Markers "written" to each file, by where it is on disk.
    markers: Mutex<HashMap<PathBuf, Vec<Marker>>>,
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
}
