//! File structures, directory scanning, and rename logic for frename.

use crate::TagList;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// FileInfo - metadata for a file in the folder listing
// ---------------------------------------------------------------------------

/// Holds the context for a file being processed.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Full path to the file.
    file_path: PathBuf,
    /// File creation time (used for sorting).
    created_at: SystemTime,
}

impl FileInfo {
    /// Create a new FileInfo from a file path and creation time.
    pub fn new(file_path: impl Into<PathBuf>, created_at: SystemTime) -> Self {
        Self {
            file_path: file_path.into(),
            created_at,
        }
    }

    /// Get the file path.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the creation time.
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }
}

// ---------------------------------------------------------------------------
// Directory scanning
// ---------------------------------------------------------------------------

/// Scan a directory and return all files sorted by creation date (oldest first).
pub fn scan_directory(directory: &Path) -> Result<Vec<FileInfo>, std::io::Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let metadata = entry.metadata()?;
            let created = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);
            files.push(FileInfo::new(path, created));
        }
    }
    files.sort_by(|a, b| a.created_at().cmp(&b.created_at()));
    Ok(files)
}

// ---------------------------------------------------------------------------
// RenameCore - tag-based file name builder
// ---------------------------------------------------------------------------

/// Core rename engine that owns the tag list and maintains
/// a file name built from the currently checked tags.
pub struct RenameCore {
    /// Available tags with checked state
    tag_list: TagList,
    /// The current file name built from checked tags (joined by dots)
    file_name: String,
}

impl RenameCore {
    /// Create a new RenameCore with the default tag list.
    pub fn new() -> Self {
        Self {
            tag_list: TagList::new(),
            file_name: String::new(),
        }
    }

    /// Toggle a tag by index and rebuild the file name.
    pub fn toggle_tag(&mut self, index: usize) {
        if let Some(tag) = self.tag_list.tags_mut().get_mut(index) {
            tag.toggle();
        }
        self.rebuild_file_name();
    }

    /// Set the initial file name (e.g. from a dropped file).
    pub fn set_file_name(&mut self, name: impl Into<String>) {
        self.file_name = name.into();
    }

    /// Get the current file name (concatenation of checked tags separated by dots).
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Get a reference to the tag list.
    pub fn tags(&self) -> &[crate::Tag] {
        self.tag_list.tags()
    }

    /// Rebuild the file name from all currently checked tags.
    fn rebuild_file_name(&mut self) {
        self.file_name = self
            .tag_list
            .tags()
            .iter()
            .filter(|tag| tag.is_checked())
            .map(|tag| tag.tag())
            .collect::<Vec<&str>>()
            .join(".");
    }
}

impl Default for RenameCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- RenameCore tests --

    #[test]
    fn test_empty_by_default() {
        let core = RenameCore::new();
        assert_eq!(core.file_name(), "");
    }

    #[test]
    fn test_single_tag_checked() {
        let mut core = RenameCore::new();
        // First tag is "Action"
        core.toggle_tag(0);
        assert_eq!(core.file_name(), "Action");
    }

    #[test]
    fn test_multiple_tags_joined_by_dots() {
        let mut core = RenameCore::new();
        // Toggle first two tags: "Action" and "Adventure"
        core.toggle_tag(0);
        core.toggle_tag(1);
        assert_eq!(core.file_name(), "Action.Adventure");
    }

    #[test]
    fn test_uncheck_removes_from_name() {
        let mut core = RenameCore::new();
        core.toggle_tag(0);
        core.toggle_tag(1);
        assert_eq!(core.file_name(), "Action.Adventure");

        // Uncheck first tag
        core.toggle_tag(0);
        assert_eq!(core.file_name(), "Adventure");
    }

    #[test]
    fn test_toggle_out_of_bounds_is_safe() {
        let mut core = RenameCore::new();
        core.toggle_tag(9999);
        assert_eq!(core.file_name(), "");
    }

    // -- FileInfo tests --

    #[test]
    fn test_file_info_creation() {
        let now = SystemTime::now();
        let info = FileInfo::new("/some/path/file.mp4", now);
        assert_eq!(info.file_path(), Path::new("/some/path/file.mp4"));
        assert_eq!(info.created_at(), now);
    }
}
