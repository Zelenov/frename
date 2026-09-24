//! FileTagger facade: routes parse/save to the installed backend.
//!
//! Call `install_file_tagger(backend)` once at startup (in main.rs) before any
//! file operations. If no backend is installed the default is InMemoryFileTagger,
//! which is always correct for tests and debug builds.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::FileTaggerBackend;
use super::folder_info::FolderInfo;
use super::in_memory_file_tagger::InMemoryFileTagger;
use crate::metadata::{FileConversion, MetadataStorage};

static BACKEND: OnceLock<Box<dyn FileTaggerBackend>> = OnceLock::new();

/// Install the file tagger backend. Must be called before any file operations.
/// Subsequent calls are ignored (OnceLock semantics).
pub fn install_file_tagger(backend: Box<dyn FileTaggerBackend>) {
    let _ = BACKEND.set(backend);
}

fn backend() -> &'static dyn FileTaggerBackend {
    BACKEND.get_or_init(|| Box::new(InMemoryFileTagger::default())).as_ref()
}

/// Thin facade over the installed `FileTaggerBackend`.
pub struct FileTagger;

impl FileTagger {
    pub fn parse(path: &Path, folder_info: &FolderInfo) -> FileSnapshot {
        backend().parse(path, folder_info)
    }

    /// Returns the new path (may differ from `path` after a disk rename).
    pub fn save(snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        backend().save(snapshot, path)
    }

    /// Returns true if `path` is a sidecar file that should be hidden from the file list.
    pub fn is_sidecar_file(path: &Path) -> bool {
        backend().is_sidecar_file(path)
    }

    /// What converting the file's comment and in/out points to `storage` would move.
    pub fn metadata_conversion(path: &Path, storage: MetadataStorage) -> FileConversion {
        backend().metadata_conversion(path, storage)
    }

    /// Move the file's comment and in/out points into `storage`; returns the path afterwards.
    pub fn convert_metadata(path: &Path, storage: MetadataStorage) -> PathBuf {
        backend().convert_metadata(path, storage)
    }

    /// Save a screenshot image for the given file and position.
    pub fn save_screenshot(file_path: &Path, position_ms: u64, image_data: &[u8]) {
        backend().save_screenshot(file_path, position_ms, image_data);
    }

    /// Load the raw image bytes for a screenshot by position.
    #[allow(dead_code)]
    pub fn load_screenshot_image(file_path: &Path, position_ms: u64) -> Option<Vec<u8>> {
        backend().load_screenshot_image(file_path, position_ms)
    }
}

/// Extension trait: save this snapshot then re-parse from the new path.
/// Returns `(new_path, new_snapshot)`.
pub trait SaveAndReparse {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, FileSnapshot);
}

impl SaveAndReparse for FileSnapshot {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, FileSnapshot) {
        let new_path = FileTagger::save(self, path);
        let folder_info = FolderInfo::default();
        let snapshot = FileTagger::parse(&new_path, &folder_info);
        (new_path, snapshot)
    }
}
