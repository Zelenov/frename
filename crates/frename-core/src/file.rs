//! File structure: path/metadata and tag-based rename state.

use crate::TagList;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A file being processed: path and metadata plus tag-based rename state.
#[derive(Debug, Clone)]
pub struct File {
    /// Full path to the file (immutable).
    file_path: Box<Path>,
    /// Original file name stem (without extension), parsed from the path.
    initial_filename: String,
    /// File creation time (used for sorting).
    created_at: SystemTime,
    /// Available tags with checked state.
    tag_list: TagList,
    /// Current file name built from checked tags (joined by dots).
    file_name: String,
    /// True when file_name has been changed (e.g. by toggling tags) and not yet applied to disk.
    dirty: bool,
}

impl File {
    /// Create a file entry from a path and creation time (e.g. when scanning a directory).
    pub fn from_path(file_path: impl AsRef<Path>, created_at: SystemTime) -> Self {
        let file_path = file_path.as_ref().to_path_buf().into_boxed_path();
        let initial_filename = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let file_name = initial_filename.clone();
        Self {
            file_path,
            initial_filename,
            created_at,
            tag_list: TagList::new(),
            file_name,
            dirty: false,
        }
    }

    /// Create a new empty File (default tag list, no path).
    pub fn new() -> Self {
        Self {
            file_path: PathBuf::new().into_boxed_path(),
            initial_filename: String::new(),
            created_at: SystemTime::UNIX_EPOCH,
            tag_list: TagList::new(),
            file_name: String::new(),
            dirty: false,
        }
    }

    /// Get the file path.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the original file name stem (without extension).
    pub fn initial_filename(&self) -> &str {
        &self.initial_filename
    }

    /// Get the creation time.
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Toggle a tag by index and rebuild the file name.
    pub fn toggle_tag(&mut self, index: usize) {
        if let Some(tag) = self.tag_list.tags_mut().get_mut(index) {
            tag.toggle();
        }
        self.rebuild_file_name();
        self.dirty = true;
    }

    /// Set the initial file name (e.g. when no path is set).
    pub fn set_file_name(&mut self, name: impl Into<String>) {
        self.initial_filename = name.into();
        self.rebuild_file_name();
    }

    /// Get the current file name: tags (if any) joined by dots, then the initial filename.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Get a reference to the tag list.
    pub fn tags(&self) -> &[crate::Tag] {
        self.tag_list.tags()
    }

    /// Whether the file name has been changed and not yet applied to disk.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the file as applied (clear dirty). Called after a successful apply/rename.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn rebuild_file_name(&mut self) {
        let tags_part: String = self
            .tag_list
            .tags()
            .iter()
            .filter(|tag| tag.is_checked())
            .map(|tag| tag.tag())
            .collect::<Vec<&str>>()
            .join(".");
        self.file_name = if tags_part.is_empty() {
            self.initial_filename.clone()
        } else if self.initial_filename.is_empty() {
            tags_part
        } else {
            format!("{}.{}", tags_part, self.initial_filename)
        };
    }
}

impl Default for File {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_by_default() {
        let f = File::new();
        assert_eq!(f.file_name(), "");
    }

    #[test]
    fn test_single_tag_checked() {
        let mut f = File::new();
        f.toggle_tag(0);
        assert_eq!(f.file_name(), "Action");
    }

    #[test]
    fn test_multiple_tags_joined_by_dots() {
        let mut f = File::new();
        f.toggle_tag(0);
        f.toggle_tag(1);
        assert_eq!(f.file_name(), "Action.Adventure");
    }

    #[test]
    fn test_uncheck_removes_from_name() {
        let mut f = File::new();
        f.toggle_tag(0);
        f.toggle_tag(1);
        assert_eq!(f.file_name(), "Action.Adventure");
        f.toggle_tag(0);
        assert_eq!(f.file_name(), "Adventure");
    }

    #[test]
    fn test_toggle_out_of_bounds_is_safe() {
        let mut f = File::new();
        f.toggle_tag(9999);
        assert_eq!(f.file_name(), "");
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
        assert_eq!(f.file_name(), "my_video");
        f.toggle_tag(0);
        assert_eq!(f.file_name(), "Action.my_video");
        f.toggle_tag(1);
        assert_eq!(f.file_name(), "Action.Adventure.my_video");
        f.toggle_tag(0);
        assert_eq!(f.file_name(), "Adventure.my_video");
    }
}
