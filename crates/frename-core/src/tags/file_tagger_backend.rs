//! FileTaggerBackend trait: the interface for parsing and saving file snapshots.

use std::path::{Path, PathBuf};

use super::file_snapshot::FileSnapshot;
use super::folder_info::FolderInfo;

/// The interface that both InMemoryFileTagger and ProductionFileTagger implement.
pub trait FileTaggerBackend: Send + Sync {
    /// Parse the file at `path` and return its snapshot (tags, name, extension, comment).
    fn parse(&self, path: &Path, folder_info: &FolderInfo) -> FileSnapshot;

    /// Save `snapshot` for the file at `path`.
    /// Returns the new path (which may differ from `path` if the file was renamed).
    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf;

    /// Returns true if `path` is a sidecar file (should be hidden from the file list).
    fn is_sidecar_file(&self, _path: &Path) -> bool { false }

    /// Save a screenshot image for the given file and position.
    /// `image_data` is raw JPEG bytes (e.g. from GStreamer). Pass `&[]` to create an empty placeholder.
    fn save_screenshot(&self, _file_path: &Path, _position_ms: u64, _image_data: &[u8]) {}

    /// Load the raw image bytes for a screenshot by position.
    #[allow(dead_code)]
    fn load_screenshot_image(&self, _file_path: &Path, _position_ms: u64) -> Option<Vec<u8>> { None }
}
