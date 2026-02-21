//! Tag management for file classification.
//! TagList is generic over the store type S (like Directory). Store is used to load stored tags and to add tags.
//!
//! Storage: [tags_by_id] (hash) for all tag data; [tags_ordered] (order + tag id only). Display = ordered; filtered = display filtered by name.
//!
//! Initialization order:
//! 1. Stored [all] → hash (checked = exists in snapshot, order = from db).
//! 2. Snapshot [where value not in stored] → hash (checked = false, order = 0).
//! 3. Hash [stored and not in snapshot] → ordered (order = from db).
//! 4. Snapshot [reversed, deduped] → ordered (insert before first) so snapshot tags are at the beginning in snapshot order.
//! 5. display_tag_ids = all tags in order (never reordered). 6. selected_tag_ids = same ids as display, order changeable in file name panel. 7. Build filtered_display_tag_ids. File name panel shows only checked (selected) tags in [selected_tag_ids] order.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::db::StoredTagStore;
use crate::ordered::OrderedCollection;
use super::{tag::{Tag, TagId}, FileSnapshot, StoredTag};

/// Number of tag colors in the UI palette (must match the UI crate).
const TAG_PALETTE_LEN: u8 = 16;

fn random_color_index() -> u8 {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (n as u64 % u64::from(TAG_PALETTE_LEN)) as u8
}

/// A collection of available tags. Generic over the store type S (load and add stored tags).
#[derive(Clone, Debug)]
pub struct TagList<S> {
    store: S,
    /// All tags by id (the only storage for tag data).
    tags_by_id: HashMap<TagId, Tag>,
    /// Tag ids in display order (tag panel with checkboxes). Order set once at init; never changed. Saving uses order from this.
    display_tag_ids: OrderedCollection<TagId, ()>,
    /// Tag ids that pass the filter (search bar). UI-only for tag panel.
    filtered_display_tag_ids: Vec<TagId>,
    /// All tag ids in order for this file; order changed only via file name panel drag. Duplicates [display_tag_ids] set; file name panel shows only the checked (selected) ones in this order.
    selected_tag_ids: OrderedCollection<TagId, ()>,
    /// Case-insensitive filter: only tags whose text contains this string are shown.
    filter_query: String,
    /// Name without extension (from snapshot at construction).
    name_without_extension: String,
    /// File extension (from snapshot at construction).
    extension: String,
    /// Initial file name (from snapshot at construction).
    initial_file_name: String,
}

impl<S> TagList<S> {
    fn tag_matches_filter(&self, t: &Tag) -> bool {
        let q = self.filter_query.trim().to_lowercase();
        q.is_empty() || t.tag().to_lowercase().contains(&q)
    }

    fn rebuild_filtered_display_tag_ids(&mut self) {
        self.filtered_display_tag_ids = self
            .display_tag_ids
            .iter()
            .map(|(id, _, _)| *id)
            .filter_map(|id| self.tags_by_id.get(&id))
            .filter(|t| self.tag_matches_filter(t))
            .map(|t| t.id())
            .collect();
    }

    /// Checked (selected) tag ids in [selected_tag_ids] order (file name panel order).
    fn checked_ids_in_selected_order(&self) -> Vec<TagId> {
        self.selected_tag_ids
            .iter()
            .map(|(id, _, _)| *id)
            .filter(|id| self.tags_by_id.get(id).map_or(false, |t| t.is_checked()))
            .collect()
    }
}

impl<S: StoredTagStore + Clone> TagList<S> {
    /// Create a new TagList from the store and the initial file snapshot.
    /// Init: (1) Stored → hash; (2) Snapshot [value not in stored] → hash; (3)–(4) build display_tag_ids (all tags, order never changed); (5) selected_tag_ids = display_tag_ids.clone(); (6) build filtered_display_tag_ids.
    pub fn new(store: S, file_snapshot: FileSnapshot) -> Self {
        let name_without_extension = file_snapshot.name_without_extension().to_string();
        let extension = file_snapshot.extension().to_string();
        let initial_file_name = file_snapshot.initial_file_name().to_string();
        let stored_tags = store.get_stored_tags().unwrap_or_default();
        let color_mapping = store.get_tag_color_mapping().unwrap_or_default();
        let snapshot_tags = file_snapshot.tags();
        let stored_values: HashSet<String> = stored_tags.iter().map(|st| st.value().to_string()).collect();

        // 1. Stored [all] → hash (checked = exists in snapshot, order = from db)
        let mut tags_by_id = HashMap::new();
        let mut value_to_id: HashMap<String, TagId> = HashMap::new();
        for st in stored_tags.iter() {
            let id = TagId(st.id());
            let color_index = color_mapping.color_index_for(st.value());
            let order = st.sort_order();
            let checked = file_snapshot.has_tag(st.value());
            let tag = Tag::with_id_order_checked(id, st.value(), color_index, true, order, checked);
            tags_by_id.insert(id, tag);
            value_to_id.insert(st.value().to_string(), id);
        }

        // 2. Snapshot [where value not in stored] → hash (checked = false, order = 0)
        for name in snapshot_tags.iter().filter(|name| !stored_values.contains(name.as_str())) {
            let id = TagId::new();
            let tag = Tag::with_id_and_order(id, name.as_str(), 0, false, 0);
            tags_by_id.insert(id, tag);
            value_to_id.insert(name.clone(), id);
        }

        // 3. Hash [stored and not in snapshot] → display (order = from db)
        let mut display_tag_ids: OrderedCollection<TagId, ()> = OrderedCollection::new();
        for st in stored_tags.iter().filter(|st| !file_snapshot.has_tag(st.value())) {
            let id = TagId(st.id());
            let order = st.sort_order();
            display_tag_ids.insert(id, (), order);
        }

        // 4. Snapshot [reversed, deduped] → display (insert before first). Dedupe: first occurrence in snapshot order; then process reversed so display = snapshot order.
        let snapshot_deduped: Vec<String> = {
            let mut seen = HashSet::new();
            snapshot_tags
                .iter()
                .filter(|s| seen.insert((*s).clone()))
                .cloned()
                .collect()
        };
        let mut first_id_opt: Option<TagId> =
            display_tag_ids.iter().next().map(|(id, _, _)| *id);
        for name in snapshot_deduped.iter().rev() {
            if let Some(&id) = value_to_id.get(name) {
                match &first_id_opt {
                    Some(before_id) => display_tag_ids.insert_before(id, (), Some(before_id)),
                    None => display_tag_ids.insert(id, (), 1),
                }
                first_id_opt = Some(id);
            }
        }

        // 5. selected_tag_ids = same ids as display, order changeable in file name panel; file name panel shows only checked in this order
        let selected_tag_ids = display_tag_ids.clone();

        let filter_query = String::new();
        let mut list = Self {
            store,
            tags_by_id,
            display_tag_ids,
            filtered_display_tag_ids: Vec::new(),
            selected_tag_ids,
            filter_query,
            name_without_extension,
            extension,
            initial_file_name,
        };
        list.rebuild_filtered_display_tag_ids();
        log::info!(
            "TagList::new snapshot: name={} ext={} initial={:?}",
            list.name_without_extension,
            list.extension,
            list.initial_file_name
        );
        list
    }

    /// Set the filter query. Empty string shows all tags. Matching is case-insensitive and "contains".
    pub fn set_filter(&mut self, query: impl Into<String>) {
        self.filter_query = query.into();
        self.rebuild_filtered_display_tag_ids();
    }

    /// Current filter query (for binding the search bar).
    pub fn filter_query(&self) -> &str {
        self.filter_query.as_str()
    }

    /// Tag ids that pass the current text filter (case-insensitive contains on tag name), in display order.
    pub fn filtered_display_tag_ids(&self) -> &[TagId] {
        &self.filtered_display_tag_ids
    }

    /// Index of the tag in the checked-tags order (file name panel order). Returns `None` if the tag is not checked.
    pub fn checked_index_of(&self, tag_id: TagId) -> Option<usize> {
        self.checked_ids_in_selected_order()
            .iter()
            .position(|&id| id == tag_id)
    }

    /// Tag ID at the given index in checked order (same as [Self::file_snapshot](Self::file_snapshot)().tags()).
    /// Returns `None` if index is out of range.
    pub fn checked_tag_id_at(&self, index: usize) -> Option<TagId> {
        self.checked_ids_in_selected_order().get(index).copied()
    }

    /// Look up a tag by id.
    pub fn get_tag(&self, id: TagId) -> Option<&Tag> {
        self.tags_by_id.get(&id)
    }

    /// Toggle the tag with the given id. No-op if id not found. [selected_tag_ids] is unchanged (has all tags).
    pub fn toggle_by_id(&mut self, id: TagId) {
        if let Some(tag) = self.tags_by_id.get_mut(&id) {
            tag.toggle();
        }
    }


    /// Save a tag to the store: find by id, then upsert in the DB (insert or update name/color/order).
    /// For a tag not yet stored: set stored=true, assign random color, save to DB, update the in-list tag.
    pub fn save_tag(
        &mut self,
        id: TagId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (text, stored_already, color_index) = self
            .tags_by_id
            .get(&id)
            .map(|t| {
                (
                    t.tag().to_string(),
                    t.is_stored(),
                    t.color_index(),
                )
            })
            .ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "tag not found",
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
        let order = self.display_tag_ids.get_order(&id).unwrap_or(0);

        let color_index = match stored_already {
            true =>  color_index,
            false => random_color_index(),
        };

        let st = StoredTag::with_sort_order(id.0, &text, order);
        self.store.save_tag(st, color_index)?;
        Ok(())
    }

    /// Reorders tags (file name panel): place `moved_id` at `index` in the checked (selected) list. Index is in the checked-only view; order is changed only in [selected_tag_ids].
    pub fn reorder_tag_to_index(&mut self, moved_id: TagId, index: usize) {
        let checked = self.checked_ids_in_selected_order();
        if !checked.contains(&moved_id) {
            return;
        }
        if checked.iter().position(|&id| id == moved_id) == Some(index) {
            return;
        }
        self.selected_tag_ids.remove(&moved_id);
        let checked_after: Vec<TagId> = checked.into_iter().filter(|id| *id != moved_id).collect();
        let before_id = match index >= checked_after.len() {
            false => Some(&checked_after[index]),
            true => None,
        };
        self.selected_tag_ids.insert_before(moved_id, (), before_id);
    }

    /// Build a FileSnapshot from checked (selected) tags in [selected_tag_ids] order (file name panel order).
    pub fn file_snapshot(&self) -> FileSnapshot {
        let tags: Vec<String> = self
            .checked_ids_in_selected_order()
            .iter()
            .filter_map(|id| self.tags_by_id.get(id))
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
        if self.tags_by_id.get(&id).map_or(false, |t| t.is_stored()) {
            self.store.remove_stored_tag_by_id(id.0)?;
        }
        self.tags_by_id.remove(&id);
        self.display_tag_ids.remove(&id);
        self.selected_tag_ids.remove(&id);
        self.rebuild_filtered_display_tag_ids();
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
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "A", 1), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "B", 2), 1)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "C", 3), 2);
        let list = TagList::new(store, FileSnapshot::default());
        assert_eq!(list.filtered_display_tag_ids().len(), 3);
        assert_eq!(list.get_tag(list.filtered_display_tag_ids()[0]).unwrap().tag(), "A");
        assert_eq!(list.get_tag(list.filtered_display_tag_ids()[1]).unwrap().tag(), "B");
        assert_eq!(list.get_tag(list.filtered_display_tag_ids()[2]).unwrap().tag(), "C");
    }

    #[test]
    fn test_tag_list_first_and_last() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "First", 1), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "Last", 2), 0);
        let list = TagList::new(store, FileSnapshot::default());
        let ids = list.filtered_display_tag_ids();
        assert_eq!(list.get_tag(ids[0]).unwrap().tag(), "First");
        assert_eq!(list.get_tag(ids[ids.len() - 1]).unwrap().tag(), "Last");
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
        assert!(list.filtered_display_tag_ids().len() >= 2);
        let first_id = list.filtered_display_tag_ids()[0];
        let first = list.get_tag(first_id).unwrap();
        assert_eq!(first.tag(), "OnlyInSnapshot");
        assert!(!first.is_checked(), "snapshot-only tags init with checked=false per spec");
        assert!(!first.is_stored());
        assert_eq!(first.color_index(), 0);
        assert!(!first.is_stored());
        let stored_tag_id = list
            .filtered_display_tag_ids()
            .iter()
            .find(|id| list.get_tag(**id).map(|t| t.tag() == "StoredA").unwrap_or(false))
            .copied()
            .unwrap();
        let stored_tag = list.get_tag(stored_tag_id).unwrap();
        assert!(stored_tag.is_stored());
        assert!(stored_tag.is_checked());
    }

    #[test]
    fn test_filtered_display_tag_ids_case_insensitive_contains() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "Action"), 0)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "Comedy"), 0)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "Sci-Fi"), 0)
            .add_stored_tag(StoredTag::new(Uuid::new_v4(), "Documentary"), 0);
        let mut list = TagList::new(store, FileSnapshot::default());
        assert_eq!(
            list.filtered_display_tag_ids().len(),
            4
        );
        list.set_filter("com");
        assert_eq!(list.filtered_display_tag_ids().len(), 1);
        assert_eq!(list.filtered_display_tag_ids().first().and_then(|id| list.get_tag(*id)).map(|t| t.tag()), Some("Comedy"));
        list.set_filter("COM");
        assert_eq!(list.filtered_display_tag_ids().len(), 1);
        list.set_filter("i");
        assert_eq!(list.filtered_display_tag_ids().len(), 2);
        list.set_filter("  ");
        assert_eq!(list.filtered_display_tag_ids().len(), 4);
    }

    #[test]
    fn reorder_checked_to_index_only_affects_file_name_order() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "A", 1), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "B", 2), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "C", 3), 0);
        let snapshot = FileSnapshot::new(vec!["A".into(), "B".into(), "C".into()], "n", "ext", "init");
        let mut list = TagList::new(store, snapshot);
        assert_eq!(list.file_snapshot().tags(), ["A", "B", "C"]);
        let a_id = list.checked_tag_id_at(0).unwrap();
        let c_id = list.checked_tag_id_at(2).unwrap();
        list.reorder_tag_to_index(c_id, 0);
        assert_eq!(list.file_snapshot().tags(), ["C", "A", "B"]);
        list.reorder_tag_to_index(a_id, 2);
        assert_eq!(list.file_snapshot().tags(), ["C", "B", "A"]);
        assert_eq!(list.filtered_display_tag_ids().len(), 3);
    }

    // --- Init order and invariants (stored → hash; snapshot not in stored → hash; stored not in snapshot → ordered; snapshot reversed deduped → ordered; display = ordered; filter empty ⇒ display = filtered) ---

    #[test]
    fn init_hash_has_all_stored_and_snapshot_no_duplicates() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "StoredA", 1), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "StoredB", 2), 0);
        let snapshot = FileSnapshot::new(
            vec!["StoredA".into(), "OnlySnapshot".into()],
            "n",
            "ext",
            "init",
        );
        let list = TagList::new(store, snapshot);
        let ids = list.filtered_display_tag_ids();
        let names: Vec<String> = ids
            .iter()
            .filter_map(|id| list.get_tag(*id))
            .map(|t| t.tag().to_string())
            .collect();
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), ids.len(), "hash has no duplicate values");
        assert!(names.contains(&"StoredA".to_string()));
        assert!(names.contains(&"StoredB".to_string()));
        assert!(names.contains(&"OnlySnapshot".to_string()));
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn init_snapshot_duplicates_removed_from_ordered() {
        let store = FakeAppStorage::new();
        let snapshot = FileSnapshot::new(
            vec!["A".into(), "B".into(), "A".into(), "C".into()],
            "n",
            "ext",
            "init",
        );
        let list = TagList::new(store, snapshot);
        let ids = list.filtered_display_tag_ids();
        let names: Vec<String> = ids
            .iter()
            .filter_map(|id| list.get_tag(*id))
            .map(|t| t.tag().to_string())
            .collect();
        assert_eq!(names, ["A", "B", "C"], "deduped snapshot order");
    }

    #[test]
    fn init_snapshot_tags_at_beginning_in_snapshot_order() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "Stored1", 10), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "Stored2", 20), 0);
        let snapshot = FileSnapshot::new(
            vec!["SnapX".into(), "SnapY".into()],
            "n",
            "ext",
            "init",
        );
        let list = TagList::new(store, snapshot);
        let ids = list.filtered_display_tag_ids();
        let names: Vec<String> = ids
            .iter()
            .filter_map(|id| list.get_tag(*id))
            .map(|t| t.tag().to_string())
            .collect();
        assert_eq!(names[0], "SnapX");
        assert_eq!(names[1], "SnapY");
        assert!(names[2] == "Stored1" || names[2] == "Stored2");
        assert!(names[3] == "Stored1" || names[3] == "Stored2");
    }

    #[test]
    fn init_stored_empty_ordered_equals_snapshot_order() {
        let store = FakeAppStorage::new();
        let snapshot = FileSnapshot::new(
            vec!["P".into(), "Q".into(), "R".into()],
            "n",
            "ext",
            "init",
        );
        let list = TagList::new(store, snapshot);
        let ids = list.filtered_display_tag_ids();
        let names: Vec<String> = ids
            .iter()
            .filter_map(|id| list.get_tag(*id))
            .map(|t| t.tag().to_string())
            .collect();
        assert_eq!(names, ["P", "Q", "R"]);
    }

    #[test]
    fn init_snapshot_empty_ordered_equals_stored_order() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "A", 1), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "B", 2), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "C", 3), 0);
        let list = TagList::new(store, FileSnapshot::default());
        let ids = list.filtered_display_tag_ids();
        assert_eq!(list.get_tag(ids[0]).unwrap().tag(), "A");
        assert_eq!(list.get_tag(ids[1]).unwrap().tag(), "B");
        assert_eq!(list.get_tag(ids[2]).unwrap().tag(), "C");
    }

    #[test]
    fn init_stored_not_in_snapshot_after_snapshot_tags() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "S", 1), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "T", 2), 0);
        let snapshot = FileSnapshot::new(vec!["S".into()], "n", "ext", "init");
        let list = TagList::new(store, snapshot);
        let ids = list.filtered_display_tag_ids();
        let names: Vec<String> = ids
            .iter()
            .filter_map(|id| list.get_tag(*id))
            .map(|t| t.tag().to_string())
            .collect();
        assert_eq!(names[0], "S");
        assert_eq!(names[1], "T");
    }

    #[test]
    fn init_filter_empty_display_order_equals_filtered_order() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "A", 1), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "B", 2), 0)
            .add_stored_tag(StoredTag::with_sort_order(Uuid::new_v4(), "C", 3), 0);
        let mut list = TagList::new(store, FileSnapshot::default());
        list.set_filter("");
        let filtered = list.filtered_display_tag_ids();
        assert_eq!(filtered.len(), 3);
        let names: Vec<String> = filtered
            .iter()
            .filter_map(|id| list.get_tag(*id))
            .map(|t| t.tag().to_string())
            .collect();
        assert_eq!(names, ["A", "B", "C"], "filtered order = display order when filter empty");
    }
}
