//! File structure: path/metadata and tag-based rename state.
//! File holds a FileSnapshot (tags + name + extension). From File's perspective we only update tags in it.

use crate::subtitles::subtitle_path;
use crate::{FileKind, FileSnapshot, FileTagger, FolderInfo};
use std::path::Path;
use std::time::SystemTime;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// FileId
// ---------------------------------------------------------------------------

/// Stable identifier for a file within a session.
/// Assigned once when the file is first scanned; survives renames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(Uuid);

impl FileId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FileId {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// File
// ---------------------------------------------------------------------------

/// A file being processed: path and metadata plus a tag list (tags, name, extension).
#[derive(Debug, Clone)]
pub struct File {
    /// Stable session identity — survives disk renames.
    id: FileId,
    /// Full path to the file (updated after a rename).
    file_path: Box<Path>,
    /// File modification time (used for sorting).
    modified_at: SystemTime,
    /// Tags, name without extension, extension, and comment. File name is built from this snapshot.
    file_snapshot: FileSnapshot,
    /// A `.srt` with the same stem sits next to the file. Checked once when the file is
    /// listed; renames carry the `.srt` along, so it stays true.
    has_subtitles: bool,
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl File {
    fn new_from_parts(
        file_path: Box<Path>,
        modified_at: SystemTime,
        file_snapshot: FileSnapshot,
        has_subtitles: bool,
    ) -> Self {
        Self {
            id: FileId::new(),
            file_path,
            modified_at,
            file_snapshot,
            has_subtitles,
        }
    }

    fn new_from_path_and_time(file_path: Box<Path>, modified_at: SystemTime) -> Self {
        let folder_info = FolderInfo::default();
        let file_snapshot = FileTagger::parse(&file_path, &folder_info);
        // No folder listing here, so ask the disk; this path opens one file, not a folder.
        let has_subtitles = subtitle_path(&file_path).is_file();
        Self::new_from_parts(file_path, modified_at, file_snapshot, has_subtitles)
    }

    /// Create a file from a path only: loads metadata (modified_at) and tags via FileTagger::parse.
    pub async fn open(path: impl AsRef<Path> + Send) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let metadata = tokio::fs::metadata(path).await?;
        let modified_at = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let file_path = path.to_path_buf().into_boxed_path();
        Ok(Self::new_from_path_and_time(file_path, modified_at))
    }

    /// Create a file from path, modification time and folder info (used by the directory scanner).
    ///
    /// Synchronous by design: the scanner already runs on a blocking thread and already holds the
    /// modification time the OS returned with the directory entry, so there is nothing to await.
    pub fn from_path_with_folder_info(
        path: impl AsRef<Path>,
        modified_at: SystemTime,
        folder_info: &FolderInfo,
    ) -> Self {
        let file_path = path.as_ref().to_path_buf().into_boxed_path();
        let file_snapshot = FileTagger::parse(&file_path, folder_info);
        let has_subtitles = subtitle_path(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| folder_info.contains(name));
        Self::new_from_parts(file_path, modified_at, file_snapshot, has_subtitles)
    }

    /// Create a file from path and creation time. For tests and programmatic use.
    pub fn from_path(file_path: impl AsRef<Path>, modified_at: SystemTime) -> Self {
        let file_path = file_path.as_ref().to_path_buf().into_boxed_path();
        Self::new_from_path_and_time(file_path, modified_at)
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Stable session identifier — survives renames.
    pub fn id(&self) -> FileId {
        self.id
    }

    /// Get the file path.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the original file name stem (without extension), from the parsed snapshot.
    pub fn initial_filename(&self) -> &str {
        self.file_snapshot.name_without_extension()
    }

    /// Get the modification time.
    pub fn modified_at(&self) -> SystemTime {
        self.modified_at
    }

    // -----------------------------------------------------------------------
    // Mediators (tag list and mutation)
    // -----------------------------------------------------------------------

    /// Set the file's snapshot (e.g. after save-and-reparse).
    pub fn set_file_snapshot(&mut self, snapshot: &FileSnapshot) {
        self.file_snapshot = snapshot.clone();
    }

    /// Update the file's path (e.g. after a disk rename).
    pub(crate) fn set_file_path(&mut self, new_path: &Path) {
        self.file_path = new_path.to_path_buf().into_boxed_path();
    }

    /// The file's snapshot (tags, name, extension; display and for building TagList from stored names).
    pub fn snapshot(&self) -> &FileSnapshot {
        &self.file_snapshot
    }

    /// Comment text for this file. Empty = no comment.
    pub fn comment(&self) -> &str {
        self.file_snapshot.comment()
    }

    /// Whether a `.srt` subtitle file sits next to this file.
    pub fn has_subtitles(&self) -> bool {
        self.has_subtitles
    }

    /// Media type of this file based on its extension.
    pub fn kind(&self) -> FileKind {
        let ext = self
            .file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
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
        assert_eq!(f.modified_at(), now);
    }

    #[test]
    fn file_id_is_stable_across_clone() {
        let f = File::from_path("/some/path/file.mp4", SystemTime::UNIX_EPOCH);
        let cloned = f.clone();
        assert_eq!(f.id(), cloned.id());
    }

    #[test]
    fn two_files_have_different_ids() {
        let f1 = File::from_path("/some/path/file.mp4", SystemTime::UNIX_EPOCH);
        let f2 = File::from_path("/some/path/file.mp4", SystemTime::UNIX_EPOCH);
        assert_ne!(f1.id(), f2.id());
    }
}
