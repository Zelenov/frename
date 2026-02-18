//! Tag management for file classification.
//! These are the stored (enriched) tags; FileTag is the plain value for parse/save.
//!
//! TagList is generic over the store type S (like Directory). Store is used to load stored tags and to add tags.

use crate::db::StoredTagStore;
use crate::{FileTag, FileTagList, StoredTag};

/// Number of tag colors in the UI palette (must match the UI crate).
const TAG_PALETTE_LEN: u8 = 16;

fn random_color_index() -> u8 {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (n as u64 % u64::from(TAG_PALETTE_LEN)) as u8
}

/// Stable unique id for a tag (from stored tag index). Used for widget identity and messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagId(pub i64);

impl TagId {
    /// For Iced widget identity: stable string per id.
    pub fn widget_id(&self) -> String {
        format!("tag-{}", self.0)
    }
}

/// A single tag that can be applied to a file (stored tag with checked state).
#[derive(Debug, Clone)]
pub struct Tag {
    /// Stable id (from stored tag index).
    id: TagId,
    /// The tag text.
    tag: String,
    /// Whether this tag is currently checked.
    checked: bool,
    /// Index into the app's tag color palette (0-based).
    color_index: u8,
}

impl Tag {
    /// Create a new tag with the given id, text, and color index (used when building from store).
    pub fn with_id(id: TagId, tag: impl Into<String>, color_index: u8) -> Self {
        Self {
            id,
            tag: tag.into(),
            checked: false,
            color_index,
        }
    }

    /// Get the tag id.
    pub fn id(&self) -> TagId {
        self.id
    }

    /// Get the tag text.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Whether this tag is checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Color palette index for this tag (for UI styling).
    pub fn color_index(&self) -> u8 {
        self.color_index
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
/// Holds the full tag list and an optional filter query; use [TagList::filtered_tag_ids] for display.
#[derive(Clone, Debug)]
pub struct TagList<S> {
    store: S,
    tags: Vec<Tag>,
    /// Case-insensitive filter: only tags whose text contains this string are shown.
    filter_query: String,
}

impl<S: StoredTagStore + Clone> TagList<S> {
    /// Create a new TagList from the store (stored tags) and optional file tag list for checked state.
    pub fn new(store: S, file_tag_list: Option<&FileTagList>) -> Self {
        let stored_tags = store.get_stored_tags().unwrap_or_default();
        let tags: Vec<Tag> = stored_tags
            .iter()
            .map(|st| {
                let id = TagId(st.index());
                let mut tag = Tag::with_id(id, st.value(), st.color_index());
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
        Self {
            store,
            tags,
            filter_query: String::new(),
        }
    }

    /// Set the filter query. Empty string shows all tags. Matching is case-insensitive and "contains".
    pub fn set_filter(&mut self, query: impl Into<String>) {
        self.filter_query = query.into();
    }

    /// Current filter query (for binding the search bar).
    pub fn filter_query(&self) -> &str {
        self.filter_query.as_str()
    }

    /// Tag ids that pass the current filter (case-insensitive contains), in display order.
    pub fn filtered_tag_ids(&self) -> Vec<TagId> {
        let q = self.filter_query.trim().to_lowercase();
        if q.is_empty() {
            return self.tags.iter().map(|t| t.id()).collect();
        }
        self.tags
            .iter()
            .filter(|t| t.tag().to_lowercase().contains(&q))
            .map(|t| t.id())
            .collect()
    }

    /// Look up a tag by id.
    pub fn get_tag(&self, id: TagId) -> Option<&Tag> {
        self.tags.iter().find(|t| t.id() == id)
    }

    /// Toggle the tag with the given id. No-op if id not found.
    pub fn toggle_by_id(&mut self, id: TagId) {
        if let Some(tag) = self.tags.iter_mut().find(|t| t.id() == id) {
            tag.toggle();
        }
    }

    /// Add a stored tag (persists to store and appends to the list).
    /// Color index is chosen at random from the palette range (0..16).
    #[allow(dead_code)]
    pub fn add_tag(
        &mut self,
        value: impl Into<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let value = value.into();
        let index = self.tags.len() as i64;
        let color_index = random_color_index();
        let st = StoredTag::new(index, &value, color_index);
        self.store.add_stored_tag(st)?;
        self.tags.push(Tag::with_id(TagId(index), value, color_index));
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
        let tag = Tag::with_id(TagId(0), "Action", 0);
        assert_eq!(tag.id(), TagId(0));
        assert_eq!(tag.tag(), "Action");
        assert_eq!(tag.color_index(), 0);
    }

    #[test]
    fn test_tag_list_reflects_stored_tags() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(0, "A", 0))
            .add_stored_tag(StoredTag::new(1, "B", 1))
            .add_stored_tag(StoredTag::new(2, "C", 2));
        let list = TagList::new(store, None);
        assert_eq!(list.tags().len(), 3);
        assert_eq!(list.tags()[0].tag(), "A");
        assert_eq!(list.tags()[1].tag(), "B");
        assert_eq!(list.tags()[2].tag(), "C");
    }

    #[test]
    fn test_tag_list_first_and_last() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(0, "First", 0))
            .add_stored_tag(StoredTag::new(1, "Last", 0));
        let list = TagList::new(store, None);
        assert_eq!(list.tags().first().unwrap().tag(), "First");
        assert_eq!(list.tags().last().unwrap().tag(), "Last");
    }

    #[test]
    fn test_filtered_tag_ids_case_insensitive_contains() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(0, "Action", 0))
            .add_stored_tag(StoredTag::new(1, "Comedy", 0))
            .add_stored_tag(StoredTag::new(2, "Sci-Fi", 0))
            .add_stored_tag(StoredTag::new(3, "Documentary", 0));
        let mut list = TagList::new(store, None);
        assert_eq!(
            list.filtered_tag_ids(),
            [TagId(0), TagId(1), TagId(2), TagId(3)]
        );
        list.set_filter("com");
        assert_eq!(list.filtered_tag_ids(), [TagId(1)]);
        list.set_filter("COM");
        assert_eq!(list.filtered_tag_ids(), [TagId(1)]);
        list.set_filter("i");
        assert_eq!(list.filtered_tag_ids(), [TagId(0), TagId(2)]);
        list.set_filter("  ");
        assert_eq!(
            list.filtered_tag_ids(),
            [TagId(0), TagId(1), TagId(2), TagId(3)]
        );
    }
}
