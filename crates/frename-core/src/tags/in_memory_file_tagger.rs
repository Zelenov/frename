//! In-memory FileTagger: no disk access. Used in debug/test mode.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::FileTaggerBackend;
use super::FolderInfo;

#[derive(Default)]
pub struct InMemoryFileTagger {
    storage: Mutex<HashMap<PathBuf, FileSnapshot>>,
}

impl FileTaggerBackend for InMemoryFileTagger {
    fn parse(&self, path: &Path, _folder_info: &FolderInfo) -> FileSnapshot {
        let stored = self.storage.lock().expect("lock").get(path).cloned();
        if let Some(snap) = stored { return snap; }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        FileSnapshot::parse(name)
    }

    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        let new_file_name = snapshot.file_name();
        let new_path = path
            .parent()
            .map(|p| p.join(&new_file_name))
            .unwrap_or_else(|| PathBuf::from(&new_file_name));
        self.storage.lock().expect("lock").insert(new_path.clone(), snapshot.clone());
        new_path
    }
}
