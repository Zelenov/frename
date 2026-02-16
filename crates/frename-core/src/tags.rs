//! Tag management for file classification.
//! These are the stored (enriched) tags; FileTag is the plain value for parse/save.

use crate::{tag_storage::TagStorage, FileTag, FileTagList};

/// A single tag that can be applied to a file (stored tag with checked state).
#[derive(Debug, Clone)]
pub struct Tag {
    /// The tag text.
    tag: String,
    /// Whether this tag is currently checked.
    checked: bool,
}

impl Tag {
    /// Create a new tag with the given text.
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            checked: false,
        }
    }

    /// Get the tag text.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Whether this tag is checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Set the checked state of this tag.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// Toggle the checked state of this tag.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    /// Convert this tag to a FileTag (the value only). Used when saving to file.
    pub fn to_file_tag(&self) -> FileTag {
        FileTag::new(self.tag().to_string())
    }
}

/// A collection of available tags.
#[derive(Clone, Debug)]
pub struct TagList {
    tags: Vec<Tag>,
}

impl TagList {
    /// Create a new TagList from stored tag names; checked state from file_tag_list (no duplicate).
    pub fn new(stored_tag_names: &[&str], file_tag_list: Option<&FileTagList>) -> Self {
        let tags: Vec<Tag> = stored_tag_names
            .iter()
            .map(|name| {
                let mut tag = Tag::new(*name);
                tag.set_checked(file_tag_list.map_or(false, |list| list.has_tag(name)));
                tag
            })
            .collect();
        if let Some(list) = file_tag_list {
            let values: Vec<&str> = list.file_tags().iter().map(|ft| ft.value()).collect();
            log::info!("TagList::new file_tag_list: {:?}", values);
        } else {
            log::info!("TagList::new file_tag_list: None");
        }
        Self { tags }
    }

    /// Get the list of all available tags.
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Get a mutable reference to the list of all available tags.
    pub fn tags_mut(&mut self) -> &mut [Tag] {
        &mut self.tags
    }

    // TODO: initial filename refactoring
    /// Checked tags as a FileTagList (for save and sync).
    pub fn checked_file_tags(&self) -> FileTagList {
        let tags: Vec<FileTag> = self
            .tags
            .iter()
            .filter(|t| t.is_checked())
            .map(|t| t.to_file_tag())
            .collect();
        let mut list = FileTagList::new();
        list.set_file_tags(&tags);
        list
    }
}

impl Default for TagList {
    fn default() -> Self {
        Self::new(TagStorage::names(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("Action");
        assert_eq!(tag.tag(), "Action");
    }

    #[test]
    fn test_tag_list_has_100_tags() {
        let list = TagList::new(TagStorage::names(), None);
        assert_eq!(list.tags().len(), 100);
    }

    #[test]
    fn test_tag_list_first_and_last() {
        let list = TagList::new(TagStorage::names(), None);
        assert_eq!(list.tags().first().unwrap().tag(), "Action");
        assert_eq!(list.tags().last().unwrap().tag(), "Yoga");
    }
}
