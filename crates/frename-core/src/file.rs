//! File structure: path/metadata and tag-based rename state.
//! File holds a FileTagList (tags + name building). From File's perspective we only update tags in it.

use crate::FileTagList;
use std::path::Path;
use std::time::SystemTime;

/// A file being processed: path and metadata plus a tag list (tags + file name built from them).
#[derive(Debug, Clone)]
pub struct File {
    /// Full path to the file (immutable).
    file_path: Box<Path>,
    /// Original file name stem (without extension), parsed from the path.
    initial_filename: String,
    /// File creation time (used for sorting).
    created_at: SystemTime,
    /// Tags on this file. File name is built from this list and initial_filename.
    file_tag_list: FileTagList,
}

impl File {
    /// Create a file entry from a path, creation time, and tags (e.g. when scanning a directory).
    pub fn from_path(
        file_path: impl AsRef<Path>,
        created_at: SystemTime,
        tags: &[crate::FileTag],
    ) -> Self {
        let file_path = file_path.as_ref().to_path_buf().into_boxed_path();
        let initial_filename = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let mut file_tag_list = FileTagList::new();
        file_tag_list.set_file_tags(tags);
        Self {
            file_path,
            initial_filename,
            created_at,
            file_tag_list,
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

    /// Set the tags on this file (e.g. from parser).
    pub fn set_file_tags(&mut self, tags: &[crate::FileTag]) {
        self.file_tag_list.set_file_tags(tags);
    }

    /// The file's tag list (tags, display, and for building TagList from stored names).
    pub fn tag_list(&self) -> &FileTagList {
        &self.file_tag_list
    }

    /// Mutable reference to the file's tag list.
    pub fn tag_list_mut(&mut self) -> &mut FileTagList {
        &mut self.file_tag_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileTag;

    #[test]
    fn test_empty_by_default() {
        let f = File::from_path("", SystemTime::UNIX_EPOCH, &[]);
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "");
    }

    #[test]
    fn test_single_tag() {
        let mut f = File::from_path("", SystemTime::UNIX_EPOCH, &[]);
        f.set_file_tags(&[FileTag::new("Action")]);
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "Action");
    }

    #[test]
    fn test_multiple_tags_joined_by_dots() {
        let mut f = File::from_path("", SystemTime::UNIX_EPOCH, &[]);
        f.set_file_tags(&[FileTag::new("Action"), FileTag::new("Adventure")]);
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "Action.Adventure");
    }

    #[test]
    fn test_add_and_remove_tag() {
        let mut f = File::from_path("", SystemTime::UNIX_EPOCH, &[]);
        f.tag_list_mut().add_tag("Action");
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "Action");
        f.tag_list_mut().add_tag("Adventure");
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "Action.Adventure");
        f.tag_list_mut().remove_tag("Action");
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "Adventure");
    }

    #[test]
    fn test_has_tag() {
        let mut f = File::from_path("", SystemTime::UNIX_EPOCH, &[]);
        assert!(!f.tag_list().has_tag("Action"));
        f.tag_list_mut().add_tag("Action");
        assert!(f.tag_list().has_tag("Action"));
    }

    #[test]
    fn test_from_path() {
        let now = SystemTime::now();
        let f = File::from_path("/some/path/file.mp4", now, &[]);
        assert_eq!(f.file_path(), Path::new("/some/path/file.mp4"));
        assert_eq!(f.initial_filename(), "file");
        assert_eq!(f.created_at(), now);
    }

    #[test]
    fn test_from_path_no_extension() {
        let now = SystemTime::now();
        let f = File::from_path("/some/path/readme", now, &[]);
        assert_eq!(f.initial_filename(), "readme");
    }

    #[test]
    fn test_file_name_starts_with_initial_then_tags_prepended() {
        let now = SystemTime::now();
        let mut f = File::from_path("/path/my_video.mp4", now, &[]);
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "my_video");
        f.set_file_tags(&[FileTag::new("Action")]);
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "Action.my_video");
        f.set_file_tags(&[FileTag::new("Action"), FileTag::new("Adventure")]);
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "Action.Adventure.my_video");
        f.tag_list_mut().remove_tag("Action");
        assert_eq!(f.tag_list().file_name(f.initial_filename()), "Adventure.my_video");
    }
}
