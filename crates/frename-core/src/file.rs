//! File structure: path/metadata and tag-based rename state.
//! File holds a FileSnapshot (tags + name + extension). From File's perspective we only update tags in it.

use crate::{FileKind, FileSnapshot, FileTagger};
use std::path::Path;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// Types and data
// ---------------------------------------------------------------------------

/// A file being processed: path and metadata plus a tag list (tags, name, extension).
#[derive(Debug, Clone)]
pub struct File {
    /// Full path to the file (immutable).
    file_path: Box<Path>,
    /// File creation time (used for sorting).
    created_at: SystemTime,
    /// Tags, name without extension, and extension. File name is built from this snapshot.
    file_snapshot: FileSnapshot,
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl File {
    fn new_from_path_and_time(file_path: Box<Path>, created_at: SystemTime) -> Self {
        let file_snapshot = FileTagger::parse(&file_path);
        Self {
            file_path,
            created_at,
            file_snapshot,
        }
    }

    /// Create a file from a path only: loads metadata (created_at) and tags via FileTagger::parse.
    pub async fn open(path: impl AsRef<Path> + Send) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let metadata = tokio::fs::metadata(path).await?;
        let created_at = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);
        let file_path = path.to_path_buf().into_boxed_path();
        Ok(Self::new_from_path_and_time(file_path, created_at))
    }

    /// Create a file from path and creation time. For tests and programmatic use; tags are loaded via FileTagger::parse.
    pub fn from_path(file_path: impl AsRef<Path>, created_at: SystemTime) -> Self {
        let file_path = file_path.as_ref().to_path_buf().into_boxed_path();
        Self::new_from_path_and_time(file_path, created_at)
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Get the file path.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the original file name stem (without extension), from the parsed snapshot.
    pub fn initial_filename(&self) -> &str {
        self.file_snapshot.name_without_extension()
    }

    /// Get the creation time.
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    // -----------------------------------------------------------------------
    // Mediators (tag list and mutation)
    // -----------------------------------------------------------------------

    /// Set the file's snapshot (e.g. after save-and-reparse).
    pub fn set_file_snapshot(&mut self, snapshot: &FileSnapshot) {
        self.file_snapshot = snapshot.clone();
    }

    /// The file's snapshot (tags, name, extension; display and for building TagList from stored names).
    pub fn snapshot(&self) -> &FileSnapshot {
        &self.file_snapshot
    }

    /// Media type of this file based on its extension.
    pub fn kind(&self) -> FileKind {
        let ext = self.file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        FileKind::from_extension(ext)
    }

    /// Mutable reference to the file's snapshot (crate-only; used by tests).
    #[allow(dead_code)]
    pub(crate) fn snapshot_mut(&mut self) -> &mut FileSnapshot {
        &mut self.file_snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_by_default() {
        let f = File::from_path("", SystemTime::UNIX_EPOCH);
        assert_eq!(f.snapshot().file_name(), "");
    }

    #[test]
    fn test_single_tag() {
        let mut f = File::from_path("", SystemTime::UNIX_EPOCH);
        f.snapshot_mut().set_tags(["Action"]);
        assert_eq!(f.snapshot().file_name(), "Action");
    }

    #[test]
    fn test_multiple_tags_joined_by_dots() {
        let mut f = File::from_path("", SystemTime::UNIX_EPOCH);
        f.snapshot_mut().set_tags(["Action", "Adventure"]);
        assert_eq!(f.snapshot().file_name(), "Action.Adventure");
    }

    #[test]
    fn test_add_and_remove_tag() {
        let mut f = File::from_path("", SystemTime::UNIX_EPOCH);
        f.snapshot_mut().set_tags(["Action"]);
        assert_eq!(f.snapshot().file_name(), "Action");
        f.snapshot_mut().set_tags(["Action", "Adventure"]);
        assert_eq!(f.snapshot().file_name(), "Action.Adventure");
        f.snapshot_mut().set_tags(["Adventure"]);
        assert_eq!(f.snapshot().file_name(), "Adventure");
    }

    #[test]
    fn test_has_tag() {
        let mut f = File::from_path("", SystemTime::UNIX_EPOCH);
        assert!(!f.snapshot().has_tag("Action"));
        f.snapshot_mut().set_tags(["Action"]);
        assert!(f.snapshot().has_tag("Action"));
    }

    #[test]
    fn test_from_path() {
        let now = SystemTime::now();
        let f = File::from_path("/some/path/file.mp4", now);
        assert_eq!(f.file_path(), Path::new("/some/path/file.mp4"));
        assert_eq!(f.initial_filename(), "file");
        assert_eq!(f.created_at(), now);
    }

    #[test]
    fn test_from_path_no_extension() {
        let now = SystemTime::now();
        let f = File::from_path("/some/path/readme", now);
        assert_eq!(f.initial_filename(), "readme");
    }

    #[test]
    fn test_file_name_starts_with_initial_then_tags_prepended() {
        let now = SystemTime::now();
        let mut f = File::from_path("/path/my_video.mp4", now);
        assert_eq!(f.snapshot().file_name(), "my_video.mp4");
        f.snapshot_mut().set_tags(["Action"]);
        assert_eq!(f.snapshot().file_name(), "Action.my_video.mp4");
        f.snapshot_mut().set_tags(["Action", "Adventure"]);
        assert_eq!(f.snapshot().file_name(), "Action.Adventure.my_video.mp4");
        f.snapshot_mut().set_tags(["Adventure"]);
        assert_eq!(f.snapshot().file_name(), "Adventure.my_video.mp4");
    }
}
