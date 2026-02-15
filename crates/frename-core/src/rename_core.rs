//! Core rename logic - builds a file name from checked tags.

use crate::TagList;

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
}
