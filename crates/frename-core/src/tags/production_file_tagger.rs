//! Production FileTagger: renames files on disk.

use std::path::{Path, PathBuf};

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::FileTaggerBackend;

pub struct ProductionFileTagger;

impl FileTaggerBackend for ProductionFileTagger {
    fn parse(&self, path: &Path) -> FileSnapshot {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        FileSnapshot::parse(name)
    }

    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        let new_file_name = snapshot.file_name();
        let new_path = path
            .parent()
            .map(|p| p.join(&new_file_name))
            .unwrap_or_else(|| PathBuf::from(&new_file_name));

        if new_path != path {
            if let Err(e) = std::fs::rename(path, &new_path) {
                log::error!("ProductionFileTagger: rename {:?} → {:?} failed: {}", path, new_path, e);
                return path.to_path_buf();
            }
            log::info!("Renamed on disk: {:?} → {:?}", path, new_path);
        }
        new_path
    }
}
