//! Tag management for file classification.
//! These are the stored (enriched) tags; FileTag is the plain value for parse/save.
//!
//! TagList is generic over the store type S (like Directory). Store is used to load stored tags and to add tags.

use crate::db::StoredTagStore;
use crate::{FileTag, FileTagList, StoredTag};

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

    /// Set the checked state of this tag (crate-only).
    pub(crate) fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// Toggle the checked state of this tag.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    /// Convert this tag to a FileTag (crate-only; used when saving to file).
    pub(crate) fn to_file_tag(&self) -> FileTag {
        FileTag::new(self.tag().to_string())
    }
}

/// A collection of available tags. Generic over the store type S (load and add stored tags).
#[derive(Clone, Debug)]
pub struct TagList<S> {
    store: S,
    tags: Vec<Tag>,
}

impl<S: StoredTagStore + Clone> TagList<S> {
    /// Create a new TagList from the store (stored tags) and optional file tag list for checked state.
    pub fn new(store: S, file_tag_list: Option<&FileTagList>) -> Self {
        let stored_tags = store.get_stored_tags().unwrap_or_default();
        let tags: Vec<Tag> = stored_tags
            .iter()
            .map(|st| {
                let mut tag = Tag::new(st.value());
                tag.set_checked(file_tag_list.map_or(false, |list| list.has_tag(st.value())));
                tag
            })
            .collect();
        if let Some(list) = file_tag_list {
            let values: Vec<&str> = list.file_tags().iter().map(|ft| ft.value()).collect();
            log::info!("TagList::new file_tag_list: {:?}", values);
        } else {
            log::info!("TagList::new file_tag_list: None");
        }
        Self { store, tags }
    }

    /// Add a stored tag (persists to store and appends to the list). Unused for now; for future UI.
    #[allow(dead_code)]
    pub fn add_tag(&mut self, value: impl Into<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let value = value.into();
        let index = self.tags.len() as i64;
        let st = StoredTag::new(index, &value);
        self.store.add_stored_tag(st)?;
        self.tags.push(Tag::new(value));
        Ok(())
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

impl Default for TagList<crate::db::AppDatabase> {
    fn default() -> Self {
        Self::new(crate::db::AppDatabase::new(), None)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::fake_app_storage::FakeAppStorage;
    use crate::StoredTag;

    use super::*;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("Action");
        assert_eq!(tag.tag(), "Action");
    }

    #[test]
    fn test_tag_list_reflects_stored_tags() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(0, "A"))
            .add_stored_tag(StoredTag::new(1, "B"))
            .add_stored_tag(StoredTag::new(2, "C"));
        let list = TagList::new(store, None);
        assert_eq!(list.tags().len(), 3);
        assert_eq!(list.tags()[0].tag(), "A");
        assert_eq!(list.tags()[1].tag(), "B");
        assert_eq!(list.tags()[2].tag(), "C");
    }

    #[test]
    fn test_tag_list_first_and_last() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(0, "First"))
            .add_stored_tag(StoredTag::new(1, "Last"));
        let list = TagList::new(store, None);
        assert_eq!(list.tags().first().unwrap().tag(), "First");
        assert_eq!(list.tags().last().unwrap().tag(), "Last");
    }
}
