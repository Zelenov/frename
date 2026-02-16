//! List of file tags and file name built from them (tags joined by dots + optional stem).

use crate::FileTag;

/// Holds the tags on a file and builds the display file name from them (with a stem passed in).
#[derive(Debug, Clone)]
pub struct FileTagList {
    file_tags: Vec<FileTag>,
}

impl FileTagList {
    /// Create an empty tag list.
    pub fn new() -> Self {
        Self {
            file_tags: Vec::new(),
        }
    }

    /// Set the tags (e.g. from parser).
    pub fn set_file_tags(&mut self, tags: &[FileTag]) {
        self.file_tags = tags.to_vec();
    }

    /// Tags on this list (for display and for saving).
    pub fn file_tags(&self) -> &[FileTag] {
        &self.file_tags
    }

    /// Whether this list contains a tag with the given value.
    pub fn has_tag(&self, value: &str) -> bool {
        self.file_tags.iter().any(|ft| ft.value() == value)
    }

    /// Add a tag with the given value (no-op if already present).
    pub fn add_tag(&mut self, value: &str) {
        if !self.has_tag(value) {
            self.file_tags.push(FileTag::new(value));
        }
    }

    /// Remove the tag with the given value (no-op if not present).
    pub fn remove_tag(&mut self, value: &str) {
        if let Some(pos) = self.file_tags.iter().position(|ft| ft.value() == value) {
            self.file_tags.remove(pos);
        }
    }

    /// Build the file name from tags (joined by dots) and the given stem (e.g. initial filename).
    pub fn file_name(&self, initial_filename: &str) -> String {
        let tags_part: String = self
            .file_tags
            .iter()
            .map(|ft| ft.value())
            .collect::<Vec<&str>>()
            .join(".");
        if tags_part.is_empty() {
            initial_filename.to_string()
        } else if initial_filename.is_empty() {
            tags_part
        } else {
            format!("{}.{}", tags_part, initial_filename)
        }
    }

    /// Tags as a vec (for saving when switching file).
    pub fn to_vec(&self) -> Vec<FileTag> {
        self.file_tags.clone()
    }
}

impl Default for FileTagList {
    fn default() -> Self {
        Self::new()
    }
}
