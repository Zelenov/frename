//! Tag management for file classification.
//! TagList is generic over the store type S (like Directory). Store is used to load stored tags and to add tags.
//! Tag entity has ID (stored index or GUID for snapshot-only); color is resolved from TagColorMapping by tag name.

use std::collections::HashSet;

use uuid::Uuid;

use crate::db::StoredTagStore;
use super::{FileSnapshot, StoredTag};

/// Number of tag colors in the UI palette (must match the UI crate).
const TAG_PALETTE_LEN: u8 = 16;

fn random_color_index() -> u8 {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (n as u64 % u64::from(TAG_PALETTE_LEN)) as u8
}

/// Stable unique id for a tag (UUID).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagId(pub Uuid);

impl TagId {
    /// For Iced widget identity: stable string per id.
    pub fn widget_id(&self) -> String {
        format!("tag-{}", self.0)
    }

    /// Create a new TagId (random UUID). Used for snapshot-only tags.
    pub fn new_snapshot() -> Self {
        TagId(Uuid::new_v4())
    }
}

/// A single tag that can be applied to a file (stored tag with checked state).
#[derive(Debug, Clone)]
pub struct Tag {
    /// Stable id (from stored tag index, or from high range for snapshot-only tags).
    id: TagId,
    /// The tag text.
    tag: String,
    /// Whether this tag is currently checked.
    checked: bool,
    /// Index into the app's tag color palette (0-based).
    color_index: u8,
    /// Whether this tag comes from the stored tag store (false for tags only present in the file snapshot).
    stored: bool,
}

impl Tag {
    /// Create a new tag with the given id, text, color index, and stored flag (used when building from store or from file snapshot).
    pub fn with_id(id: TagId, tag: impl Into<String>, color_index: u8, stored: bool) -> Self {
        Self {
            id,
            tag: tag.into(),
            checked: false,
            color_index,
            stored,
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

    /// Whether this tag comes from the stored tag store (false for tags only present in the file snapshot).
    pub fn is_stored(&self) -> bool {
        self.stored
    }

    /// Set the checked state of this tag (crate-only).
    pub(crate) fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// Toggle the checked state of this tag.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }
}

/// A collection of available tags. Generic over the store type S (load and add stored tags).
/// Holds the full tag list, snapshot inners (name, extension, initial file name) copied at construction, and filter query.
#[derive(Clone, Debug)]
pub struct TagList<S> {
    store: S,
    tags: Vec<Tag>,
    /// Case-insensitive filter: only tags whose text contains this string are shown.
    filter_query: String,
    /// Name without extension (from snapshot at construction; used to build snapshot from checked tags).
    name_without_extension: String,
    /// File extension (from snapshot at construction).
    extension: String,
    /// Initial file name (from snapshot at construction).
    initial_file_name: String,
}

impl<S: StoredTagStore + Clone> TagList<S> {
    /// Create a new TagList from the store and the initial file snapshot. Snapshot inners (name, extension, initial file name) are copied. Tags that appear in the file snapshot but not in the store are added first (stored=false, color_index=0, ids from i64::MAX-1 down). Then stored tags follow; checked state is set from the snapshot's tags. Colors for stored tags are resolved from the store's tag color mapping by tag name.
    pub fn new(store: S, file_snapshot: FileSnapshot) -> Self {
        let name_without_extension = file_snapshot.name_without_extension().to_string();
        let extension = file_snapshot.extension().to_string();
        let initial_file_name = file_snapshot.initial_file_name().to_string();
        let stored_tags = store.get_stored_tags().unwrap_or_default();
        let color_mapping = store.get_tag_color_mapping().unwrap_or_default();
        let stored_values: HashSet<String> = stored_tags.iter().map(|st| st.value().to_string()).collect();
        // If the store does not contain the snapshot tag, it goes to snapshot-only (first); otherwise it appears in the stored tags list.
        let snapshot_only: Vec<String> = file_snapshot
            .tags()
            .iter()
            .filter(|t| !stored_values.contains(*t))
            .cloned()
            .collect();
        let mut tags: Vec<Tag> = snapshot_only
            .iter()
            .map(|text| {
                let id = TagId::new_snapshot();
                let mut tag = Tag::with_id(id, text.as_str(), 0, false); // stored = false
                tag.set_checked(true);
                tag
            })
            .collect();
        let stored_tags_vec: Vec<Tag> = stored_tags
            .iter()
            .map(|st| {
                let id = TagId(st.id());
                let color_index = color_mapping.color_index_for(st.value());
                let mut tag = Tag::with_id(id, st.value(), color_index, true); // stored = true
                tag.set_checked(file_snapshot.has_tag(st.value()));
                tag
            })
            .collect();
        tags.extend(stored_tags_vec);
        log::info!(
            "TagList::new snapshot: name={} ext={} initial={:?}",
            name_without_extension,
            extension,
            initial_file_name
        );
        Self {
            store,
            tags,
            filter_query: String::new(),
            name_without_extension,
            extension,
            initial_file_name,
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

    /// Index of the tag in the checked-tags order (file name order). Returns `None` if the tag is not checked.
    pub fn checked_index_of(&self, tag_id: TagId) -> Option<usize> {
        self.tags
            .iter()
            .filter(|t| t.is_checked())
            .position(|t| t.id() == tag_id)
    }

    /// Tag ID at the given index in checked order (same as [Self::file_snapshot](Self::file_snapshot)().tags()).
    /// Returns `None` if index is out of range.
    pub fn checked_tag_id_at(&self, index: usize) -> Option<TagId> {
        self.tags
            .iter()
            .filter(|t| t.is_checked())
            .nth(index)
            .map(|t| t.id())
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

    /// Add a stored tag (persists to store and appends to the list). Color is stored in tag_color_mapping by name.
    /// Color index is chosen at random from the palette range (0..16).
    #[allow(dead_code)]
    pub fn add_tag(
        &mut self,
        value: impl Into<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let value = value.into();
        let id = Uuid::new_v4();
        let color_index = random_color_index();
        let st = StoredTag::new(id, &value);
        self.store.add_stored_tag(st, color_index)?;
        self.tags.push(Tag::with_id(TagId(id), value.clone(), color_index, true));
        Ok(())
    }

    /// Save a tag to the store: find by id, then upsert in the DB (insert or update name/color).
    /// For a tag not yet stored: set stored=true, assign random color, save to DB, update the in-list tag.
    /// For an already stored tag: only update the DB with current name and color; in-list tag unchanged.
    pub fn save_tag(
        &mut self,
        id: TagId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pos = self
            .tags
            .iter()
            .position(|t| t.id() == id)
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "tag not found",
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
        let text = self.tags[pos].tag().to_string();
        let checked = self.tags[pos].is_checked();
        let stored_already = self.tags[pos].is_stored();

        if stored_already {
            let color_index = self.tags[pos].color_index();
            let st = StoredTag::new(id.0, &text);
            self.store.save_tag(st, color_index)?;
        } else {
            let color_index = random_color_index();
            let st = StoredTag::new(id.0, &text);
            self.store.save_tag(st, color_index)?;
            let mut tag = Tag::with_id(id, text, color_index, true);
            tag.set_checked(checked);
            self.tags[pos] = tag;
        }
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

    /// Build a FileSnapshot from the currently checked tags and the snapshot inners stored at construction (name, extension, initial file name).
    pub fn file_snapshot(&self) -> FileSnapshot {
        let tags: Vec<String> = self
            .tags
            .iter()
            .filter(|t| t.is_checked())
            .map(|t| t.tag().to_string())
            .collect();
        FileSnapshot::new(
            tags,
            self.name_without_extension.as_str(),
            self.extension.as_str(),
            self.initial_file_name.as_str(),
        )
    }

    /// Remove a stored tag by tag id from the store and from the in-memory list. No-op for snapshot-only tags (use save to add them first).
    pub fn remove_stored_tag_by_id(
        &mut self,
        id: TagId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.tags.iter().any(|t| t.id() == id && t.is_stored()) {
            self.store.remove_stored_tag_by_id(id.0)?;
        }
        self.tags.retain(|t| t.id() != id);
        Ok(())
    }
}

impl Default for TagList<crate::db::AppDatabase> {
    fn default() -> Self {
        Self::new(crate::db::AppDatabase::new(), FileSnapshot::default())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::db::fake_app_storage::FakeAppStorage;
    use super::StoredTag;

    use super::*;

    #[test]
    fn test_tag_creation() {
        let id = Uuid::new_v4();
        let tag = Tag::with_id(TagId(id), "Action", 0, true);
        assert_eq!(tag.id().0, id);
        assert_eq!(tag.tag(), "Action");
        assert_eq!(tag.color_index(), 0);
    }

    #[test]
    fn test_tag_list_reflects_stored_tags() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "A"), 0)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "B"), 1)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "C"), 2);
        let list = TagList::new(store, FileSnapshot::default());
        assert_eq!(list.tags().len(), 3);
        assert_eq!(list.tags()[0].tag(), "A");
        assert_eq!(list.tags()[1].tag(), "B");
        assert_eq!(list.tags()[2].tag(), "C");
    }

    #[test]
    fn test_tag_list_first_and_last() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "First"), 0)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "Last"), 0);
        let list = TagList::new(store, FileSnapshot::default());
        assert_eq!(list.tags().first().unwrap().tag(), "First");
        assert_eq!(list.tags().last().unwrap().tag(), "Last");
    }

    #[test]
    fn test_tag_list_snapshot_only_tags_first_and_stored_flag() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "StoredA"), 0)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "StoredB"), 0);
        let snapshot = FileSnapshot::new(
            vec!["OnlyInSnapshot".to_string(), "StoredA".to_string()],
            "name",
            "ext",
            "initial",
        );
        let list = TagList::new(store, snapshot);
        let tags = list.tags();
        assert!(tags.len() >= 2);
        let first = &tags[0];
        assert_eq!(first.tag(), "OnlyInSnapshot");
        assert!(first.is_checked());
        assert!(!first.is_stored());
        assert_eq!(first.color_index(), 0);
        assert!(!first.is_stored());
        let stored_tag = tags.iter().find(|t| t.tag() == "StoredA").unwrap();
        assert!(stored_tag.is_stored());
        assert!(stored_tag.is_checked());
    }

    #[test]
    fn test_filtered_tag_ids_case_insensitive_contains() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "Action"), 0)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "Comedy"), 0)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "Sci-Fi"), 0)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "Documentary"), 0);
        let mut list = TagList::new(store, FileSnapshot::default());
        assert_eq!(
            list.filtered_tag_ids().len(),
            4
        );
        list.set_filter("com");
        assert_eq!(list.filtered_tag_ids().len(), 1);
        assert_eq!(list.filtered_tag_ids().first().and_then(|id| list.get_tag(*id)).map(|t| t.tag()), Some("Comedy"));
        list.set_filter("COM");
        assert_eq!(list.filtered_tag_ids().len(), 1);
        list.set_filter("i");
        assert_eq!(list.filtered_tag_ids().len(), 2);
        list.set_filter("  ");
        assert_eq!(list.filtered_tag_ids().len(), 4);
    }
}
