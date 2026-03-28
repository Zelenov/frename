//! FileTaggerBackend trait: the interface for parsing and saving file snapshots.

use std::path::{Path, PathBuf};

use super::FileSnapshot;

/// The interface that both InMemoryFileTagger and ProductionFileTagger implement.
pub trait FileTaggerBackend: Send + Sync {
    /// Parse the file at `path` and return its snapshot (tags, name, extension, comment).
    fn parse(&self, path: &Path) -> FileSnapshot;

    /// Save `snapshot` for the file at `path`.
    /// Returns the new path (which may differ from `path` if the file was renamed).
    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf;

    /// Returns true if `path` is a sidecar file (should be hidden from the file list).
    /// Default impl covers `.snap.*.jpg` screenshot sidecars and comment files.
    fn is_sidecar_file(&self, path: &Path) -> bool {
        if crate::comment::is_comment_file(path) {
            return true;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if let Some(idx) = name.find(".snap.") {
            let rest = &name[idx + 6..];
            if let Some(time_str) = rest.strip_suffix(".jpg") {
                return crate::Screenshot::parse_time(time_str).is_some();
            }
        }
        false
    }

    /// Save a screenshot image for the given file and position.
    /// `image_data` is raw JPEG bytes (e.g. from GStreamer). Pass `&[]` to create an empty placeholder.
    fn save_screenshot(&self, _file_path: &Path, _position_ms: u64, _image_data: &[u8]) {}

    /// Load the raw image bytes for a screenshot by position.
    #[allow(dead_code)]
    fn load_screenshot_image(&self, _file_path: &Path, _position_ms: u64) -> Option<Vec<u8>> { None }
}
