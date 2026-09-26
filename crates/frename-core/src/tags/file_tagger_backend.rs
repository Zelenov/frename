//! FileTaggerBackend trait: the interface for parsing and saving file snapshots.

use std::path::{Path, PathBuf};

use super::file_snapshot::FileSnapshot;
use super::folder_info::FolderInfo;
use super::production_file_tagger::is_screenshot_sidecar;
use crate::metadata::MetadataMove;

/// The interface that both InMemoryFileTagger and ProductionFileTagger implement.
pub trait FileTaggerBackend: Send + Sync {
    /// Parse the file at `path` and return its snapshot (tags, name, extension, comment).
    fn parse(&self, path: &Path, folder_info: &FolderInfo) -> FileSnapshot;

    /// Save `snapshot` for the file at `path`.
    /// Returns the new path (which may differ from `path` if the file was renamed).
    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf;

    /// Returns true if `path` is a sidecar file (should be hidden from the file list):
    /// a comment, a screenshot, or the folder's tag file.
    ///
    /// Which names are sidecars follows from the tag format, not from the backend, so every
    /// backend shares this. A debug build saves nothing to disk, but it still lists a real
    /// folder, and those files are no more part of it there than in a release build.
    fn is_sidecar_file(&self, path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        crate::comment::is_comment_file(path)
            || is_screenshot_sidecar(name)
            || crate::FolderTagStore::is_tag_file(path)
    }

    /// Load the comments a folder scan deferred (see [`FileSnapshot::comment_loading`]) for
    /// each `(path, snapshot)`, returning the snapshots in the same order. Backends that keep
    /// no comments inside videos have nothing to load.
    fn load_comments(&self, items: &[(PathBuf, FileSnapshot)]) -> Vec<FileSnapshot> {
        items
            .iter()
            .map(|(_, snapshot)| {
                let mut resolved = snapshot.clone();
                resolved.set_comment_loading(false);
                resolved
            })
            .collect()
    }

    /// Whether the file keeps anything `what` would move outside its destination.
    /// Backends that never touch the disk have nothing to move.
    fn metadata_move_needed(&self, _path: &Path, _what: MetadataMove) -> bool {
        false
    }

    /// Move the file's comment or in/out points as `what` says, reading both of their homes.
    /// Returns the file's path afterwards, which changes when in/out points move in or out
    /// of the name.
    fn move_metadata(&self, path: &Path, _what: MetadataMove) -> PathBuf {
        path.to_path_buf()
    }

    /// Read the file's comment and in/out points again and replace the folder file list's line
    /// for it. Returns whether the line was missing or stale. Backends that never touch the
    /// disk keep no file list.
    fn reload_metadata(&self, _path: &Path) -> bool {
        false
    }

    /// Where the file at `path` is on disk. A backend that renames files only in memory
    /// keeps the files where they were; reading one (e.g. to play it) must use this path.
    fn disk_path(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }

    /// Save a screenshot image for the given file and position.
    /// `image_data` is raw JPEG bytes (e.g. from GStreamer). Pass `&[]` to create an empty placeholder.
    fn save_screenshot(&self, _file_path: &Path, _position_ms: u64, _image_data: &[u8]) {}

    /// Load the raw image bytes for a screenshot by position.
    #[allow(dead_code)]
    fn load_screenshot_image(&self, _file_path: &Path, _position_ms: u64) -> Option<Vec<u8>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::FileTaggerBackend;
    use crate::InMemoryFileTagger;

    /// Every backend hides sidecars, the in-memory one used by debug builds included:
    /// they back the folder list rather than appearing in it.
    #[test]
    fn default_backend_hides_sidecar_files() {
        let backend = InMemoryFileTagger::default();
        assert!(backend.is_sidecar_file(Path::new("shoots/.frename")));
        assert!(backend.is_sidecar_file(Path::new("shoots/clip.mp4.comment.txt")));
        assert!(backend.is_sidecar_file(Path::new("shoots/clip.mp4.snap.00-00-10-936.jpg")));
        assert!(!backend.is_sidecar_file(Path::new("shoots/clip.mp4")));
    }
}
