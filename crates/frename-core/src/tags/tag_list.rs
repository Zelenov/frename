//! Tag management for file classification.
//! TagList is generic over the store type S (like Directory). Store is used to load stored tags and to add tags.
//!
//! Storage: [tags_by_id] (hash) for all tag data; [tags_ordered] (order + tag id only). Display = ordered; filtered = display filtered by name.
//!
//! Initialization order (locked = initial file open; unlocked = paste):
//! 1. Stored [all] → hash (checked = exists in snapshot, order = from db, starred = from db).
//! 2. Snapshot [where value not in stored] → hash (checked = true, order = 0).
//! 3. locked=true:  stored-NOT-in-snapshot → display (snapshot tags added in step 5).
//!    locked=false: ALL stored → display (full DB view; snapshot-order correction applied only to selected).
//! 4. Dedupe snapshot (first occurrence wins), then reverse.
//! 5. Snapshot items NOT yet in display → insert at beginning (reversed iteration → snapshot order at front).
//!    locked=true:  all snapshot items inserted (stored-in-snapshot excluded from step 3 so not in display yet).
//!    locked=false: only not-stored items inserted (stored-in-snapshot already in display at their DB positions).
//! 6. selected_tag_ids = display_tag_ids.clone().
//! 7. For each consecutive snapshot pair: if cur is not after prev in display, move cur after prev in selected.
//!    Corrects snapshot ordering in selected for locked=false (stored items may be at non-snapshot positions in display).
//! 8. locked=true: copy selected → display (final display matches snapshot order). Build filtered_display_tag_ids.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{OrderedThing, db::StoredTagStore};
use crate::ordered::OrderedCollection;
use super::{tag::{Tag, TagId}, FileSnapshot, StoredTag};

/// Number of tag colors in the UI palette (must match the UI crate).
const TAG_PALETTE_LEN: u8 = 16;


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
    /// True when display_tag_ids was rebalanced (insert_before caused order respread). Cleared after persisting all tag orders in save_tag.
    display_tags_rebalanced: bool,
    /// Counter for assigning sequential tag colors (cycles through palette). Avoids timer-resolution bias.
    color_counter: u8,
    /// Name without extension (from snapshot at construction).
    name_without_extension: String,
    /// File extension (from snapshot at construction).
    extension: String,
    /// Initial file name (from snapshot at construction).
    initial_file_name: String,
    /// Segment start in seconds, if set.
    segment_start: Option<f32>,
    /// Segment end in seconds, if set.
    segment_end: Option<f32>,
}

/// Returns the match rank for a non-empty, pre-lowercased query against a tag name.
/// None    = no match (tag hidden)
/// Some(0) = full match
/// Some(1) = prefix match
/// Some(2) = contains match
fn match_rank(query_lower: &str, tag_name: &str) -> Option<u8> {
    let name_lower = tag_name.to_lowercase();
    if name_lower == query_lower            { return Some(0); }
    if name_lower.starts_with(query_lower)  { return Some(1); }
    if name_lower.contains(query_lower)     { return Some(2); }
    None
}

impl<S> TagList<S> {
    /// Returns the next color index from the sequential counter (cycles through 0..TAG_PALETTE_LEN).
    fn next_color_index(&mut self) -> u8 {
        let index = self.color_counter % TAG_PALETTE_LEN;
        self.color_counter = self.color_counter.wrapping_add(1);
        index
    }

    fn tag_matches_filter(&self, t: &Tag) -> bool {
        let q = self.filter_query.trim().to_lowercase();
        q.is_empty() || match_rank(&q, t.tag()).is_some()
    }

    fn rebuild_filtered_display_tag_ids(&mut self) {
        // Filter (preserving display order), then stable-sort by (section, rank):
        //   section: 0 = unstored, 1 = starred stored, 2 = unstarred stored
        //   rank:    0 = full match, 1 = prefix match, 2 = contains match
        // Starred tags always float to the top of their section.
        // sort_by_key is stable so ties keep the original display order.
        let mut filtered: Vec<TagId> = self
            .display_tag_ids
            .iter()
            .map(|(id, _, _)| *id)
            .filter_map(|id| self.tags_by_id.get(&id))
            .filter(|t| self.tag_matches_filter(t))
            .map(|t| t.id())
            .collect();
        let q = self.filter_query.trim().to_lowercase();
        filtered.sort_by_key(|id| {
            let tag = self.tags_by_id.get(id);
            let section: u8 = match tag {
                Some(t) if !t.is_stored() => 0,
                Some(t) if t.is_starred() => 1,
                _ => 2,
            };
            let rank: u8 = if q.is_empty() {
                0
            } else {
                tag.and_then(|t| match_rank(&q, t.tag())).unwrap_or(2)
            };
            (section, rank)
        });
        self.filtered_display_tag_ids = filtered;
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
    /// Create a new TagList from the store and the initial file snapshot (locked = true).
    /// Use this for normal file opens. For paste operations use [`TagList::new_unlocked`].
    pub fn new(store: S, file_snapshot: FileSnapshot) -> Self {
        Self::new_with_lock(store, file_snapshot, true)
    }

    /// Create a new TagList with locked = false (paste / reinitialize from pasted snapshot).
    /// Display keeps the full DB order (new unstored tags prepended); selected is corrected to
    /// snapshot order so the file-name panel reflects the pasted tag sequence.
    pub fn new_unlocked(store: S, file_snapshot: FileSnapshot) -> Self {
        Self::new_with_lock(store, file_snapshot, false)
    }

    /// Core constructor. `locked` controls how display_tag_ids and selected_tag_ids relate to the
    /// snapshot order — see the module-level doc comment for the full step-by-step description.
    fn new_with_lock(store: S, file_snapshot: FileSnapshot, locked: bool) -> Self {
        let name_without_extension = file_snapshot.name_without_extension().to_string();
        let extension = file_snapshot.extension().to_string();
        let initial_file_name = file_snapshot.initial_file_name().to_string();
        let segment_start = file_snapshot.segment_start();
        let segment_end = file_snapshot.segment_end();
        let stored_tags = store.get_stored_tags().unwrap_or_default();
        let color_mapping = store.get_tag_color_mapping().unwrap_or_default();
        let snapshot_tags = file_snapshot.tags();
        let stored_values: HashSet<String> =
            stored_tags.iter().map(|st| st.value().to_string()).collect();

        // 1. Stored [all] → hash (checked = exists in snapshot, order = from db, starred = from db)
        let mut tags_by_id = HashMap::new();
        let mut value_to_id: HashMap<String, TagId> = HashMap::new();
        for st in stored_tags.iter() {
            let id = TagId(st.id());
            let color_index = color_mapping.color_index_for(st.value());
            let order = st.sort_order();
            let checked = file_snapshot.has_tag(st.value());
            let mut tag = Tag::with_id_order_checked(id, st.value(), color_index, true, order, checked);
            tag.set_starred(st.starred());
            tags_by_id.insert(id, tag);
            value_to_id.insert(st.value().to_string(), id);
        }

        // 2. Snapshot [where value not in stored] → hash (checked = true, order = 0)
        for name in snapshot_tags.iter().filter(|name| !stored_values.contains(name.as_str())) {
            let id = TagId::new();
            let tag = Tag::with_id_order_checked(id, name.as_str(), 0, false, 0, true);
            tags_by_id.insert(id, tag);
            value_to_id.insert(name.clone(), id);
        }

        // 3. Seed display_tag_ids from stored tags.
        //    locked=true:  stored-NOT-in-snapshot only (snapshot tags will be prepended in step 5).
        //    locked=false: ALL stored in DB order (paste keeps the grid order; only new tags prepend).
        let mut display_tag_ids: OrderedCollection<TagId, ()> = OrderedCollection::new();
        for st in stored_tags.iter() {
            if locked && file_snapshot.has_tag(st.value()) {
                continue; // locked=true: skip; snapshot tags are positioned by step 5 + step 8
            }
            let id = TagId(st.id());
            let order = st.sort_order();
            display_tag_ids.insert(id, (), order);
        }

        // 4. Dedupe snapshot (first occurrence wins) and reverse in one pass for the insert loop.
        //    The reversed vec is also used for the pair loop via windows(2).rev() with swapped indices.
        let snapshot_deduped_reversed: Vec<String> = {
            let mut seen = HashSet::new();
            let mut v: Vec<String> = snapshot_tags
                .iter()
                .filter(|s| seen.insert((*s).clone()))
                .cloned()
                .collect();
            v.reverse();
            v
        };

        // 5. Snapshot items NOT yet in display → insert at beginning (reversed → snapshot order at front).
        //    locked=true:  all snapshot items qualify (stored-in-snapshot excluded from step 3).
        //    locked=false: only not-stored items qualify (stored-in-snapshot already in display).
        let mut display_tags_rebalanced = false;
        for id in snapshot_deduped_reversed.iter().filter_map(|n| value_to_id.get(n)).copied() {
            if display_tag_ids.get(&id).is_none() {
                display_tags_rebalanced |= display_tag_ids.insert_after(id, (), None);
            }
        }

        // 6. selected_tag_ids starts as a copy of display order.
        let mut selected_tag_ids = display_tag_ids.clone();

        // 7. For each consecutive snapshot pair (original order): if cur is not after prev in
        //    display_tag_ids, move cur after prev in selected_tag_ids. Iterating windows(2).rev()
        //    on the reversed vec with swapped indices yields the original-order pairs.
        //    This corrects the file-name-panel order for locked=false, where stored-in-snapshot
        //    items sit at their DB positions in display (which may differ from snapshot order).
        //    For locked=true this loop is a no-op because step 5 already placed all snapshot items
        //    in snapshot order in display.
        for window in snapshot_deduped_reversed.windows(2).rev() {
            let prev_name = &window[1]; // original order: prev (appears earlier in snapshot)
            let cur_name = &window[0];  // original order: cur  (appears later  in snapshot)
            if let (Some(&prev_id), Some(&cur_id)) =
                (value_to_id.get(prev_name), value_to_id.get(cur_name))
            {
                let cur_is_after_prev = match (
                    display_tag_ids.get_order(&prev_id),
                    display_tag_ids.get_order(&cur_id),
                ) {
                    (Some(p), Some(c)) => c > p,
                    _ => false,
                };
                if !cur_is_after_prev {
                    selected_tag_ids.insert_after(cur_id, (), Some(&prev_id));
                }
            }
        }

        // 8. locked=true: sync selected order → display (no-op for locked=true since step 5
        //    already placed all snapshot items in order, making selected == display after step 6).
        if locked {
            display_tag_ids.sync_order_from(&selected_tag_ids);
        }

        let filter_query = String::new();
        // Start color counter at number of stored tags so sequential tags get distinct colors.
        let color_counter = stored_tags.len() as u8;
        let mut list = Self {
            store,
            tags_by_id,
            display_tag_ids,
            filtered_display_tag_ids: Vec::new(),
            selected_tag_ids,
            filter_query,
            display_tags_rebalanced,
            color_counter,
            name_without_extension,
            extension,
            initial_file_name,
            segment_start,
            segment_end,
        };
        list.rebuild_filtered_display_tag_ids();
        log::info!(
            "TagList::new_with_lock locked={} snapshot: name={} ext={} initial={:?}",
            locked,
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

    /// Tag id whose tag name has the greatest length among filtered display tags. Used to size the tag grid by the longest chip.
    pub fn longest_display_tag_id(&self) -> Option<TagId> {
        self.filtered_display_tag_ids
            .iter()
            .filter_map(|id| self.tags_by_id.get(id))
            .max_by_key(|t| t.tag().len())
            .map(|t| t.id())
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
        let was_checked = self.tags_by_id.get(&id).map_or(false, |t| t.is_checked());
        if let Some(tag) = self.tags_by_id.get_mut(&id) {
            tag.toggle();
        }
        // When a tag becomes checked, reposition it in selected_tag_ids so starred tags
        // always appear before non-starred tags in the file name chips panel.
        if !was_checked {
            let is_starred = self.tags_by_id.get(&id).map_or(false, |t| t.is_starred());
            if is_starred {
                // Move starred tag before the first checked non-starred tag.
                let first_non_starred_checked = self.selected_tag_ids
                    .iter()
                    .find_map(|(tid, _, _)| {
                        if *tid == id { return None; }
                        self.tags_by_id.get(tid)
                            .filter(|t| t.is_checked() && !t.is_starred())
                            .map(|t| t.id())
                    });
                if let Some(anchor) = first_non_starred_checked {
                    self.selected_tag_ids.insert_before(id, (), Some(&anchor));
                }
            } else {
                // Move non-starred tag after the last checked starred tag.
                let last_starred_checked = self.selected_tag_ids
                    .iter()
                    .filter_map(|(tid, _, _)| {
                        if *tid == id { return None; }
                        self.tags_by_id.get(tid)
                            .filter(|t| t.is_checked() && t.is_starred())
                            .map(|t| t.id())
                    })
                    .last();
                if let Some(anchor) = last_starred_checked {
                    self.selected_tag_ids.insert_after(id, (), Some(&anchor));
                }
            }
        }
    }


    /// Save a tag to the store: find by id, then upsert in the DB (insert or update name/color/order).
    /// For a tag not yet stored: set stored=true, assign random color, save to DB, update the in-list tag.
    pub fn save_tag(
        &mut self,
        id: TagId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (text, stored_already, color_index, starred) = self
            .tags_by_id
            .get(&id)
            .map(|t| {
                (
                    t.tag().to_string(),
                    t.is_stored(),
                    t.color_index(),
                    t.is_starred(),
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
            false => self.next_color_index(),
        };

        let st = StoredTag::with_all(id.0, &text, order, starred);
        self.store.save_tag(st, color_index)?;
        if let Some(tag) = self.tags_by_id.get_mut(&id) {
            tag.set_stored(true);
            tag.set_color_index(color_index);
        }
        if self.display_tags_rebalanced {
            let tag_orders: Vec<(Uuid, i64)> = self
                .display_tag_ids
                .iter()
                .filter_map(|(tid, _, ord)| {
                    self.tags_by_id.get(tid).filter(|t| t.is_stored()).map(|_| (tid.0, ord))
                })
                .collect();
            self.store.update_tag_orders(&tag_orders)?;
            self.display_tags_rebalanced = false;
        }
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
        if index == 0 {
            // Place before everything (None anchor = absolute first in collection).
            self.selected_tag_ids.insert_before(moved_id, (), None);
        } else if index < checked_after.len() {
            self.selected_tag_ids.insert_before(moved_id, (), Some(&checked_after[index]));
        } else if let Some(last) = checked_after.last() {
            // Append after the last checked tag (not insert_before(None) which goes to the front).
            self.selected_tag_ids.insert_after(moved_id, (), Some(last));
        } else {
            self.selected_tag_ids.insert_before(moved_id, (), None);
        }
    }

    /// Build a FileSnapshot from checked (selected) tags in [selected_tag_ids] order (file name panel order).
    pub fn file_snapshot(&self) -> FileSnapshot {
        let tags: Vec<String> = self
            .checked_ids_in_selected_order()
            .iter()
            .filter_map(|id| self.tags_by_id.get(id))
            .map(|t| t.tag().to_string())
            .collect();
        let mut snap = FileSnapshot::new(
            tags,
            self.name_without_extension.as_str(),
            self.extension.as_str(),
            self.initial_file_name.as_str(),
        );
        snap.set_segment_start(self.segment_start);  // Option<f32>
        snap.set_segment_end(self.segment_end);      // Option<f32>
        snap
    }

    /// Segment start in seconds, if set.
    pub fn segment_start_secs(&self) -> Option<f32> {
        self.segment_start
    }

    /// Segment end in seconds, if set.
    pub fn segment_end_secs(&self) -> Option<f32> {
        self.segment_end
    }

    /// Set the segment start marker (in seconds). Pass `None` to clear.
    pub fn set_segment_start_secs(&mut self, secs: Option<f32>) {
        self.segment_start = secs;
    }

    /// Set the segment end marker (in seconds). Pass `None` to clear.
    pub fn set_segment_end_secs(&mut self, secs: Option<f32>) {
        self.segment_end = secs;
    }

    /// Returns true if any tag in the list has the given name (case-insensitive exact match).
    pub fn has_tag_with_name(&self, name: &str) -> bool {
        self.tags_by_id
            .values()
            .any(|t| t.tag().eq_ignore_ascii_case(name))
    }

    /// Create a new unsaved tag with the given name, insert it at the front of both display and
    /// selected collections, mark it checked=true. Returns the new tag's id.
    /// Returns None if a tag with this name already exists (case-insensitive).
    pub fn create_new_tag(&mut self, name: impl Into<String>) -> Option<TagId> {
        let name = name.into();
        if self.has_tag_with_name(&name) {
            return None;
        }
        let id = TagId::new();
        let tag = Tag::with_id_order_checked(id, &name, 0, false, 0, true);
        self.tags_by_id.insert(id, tag);
        self.display_tags_rebalanced |= self.display_tag_ids.insert_before(id, (), Option::None);
        self.selected_tag_ids.insert_before(id, (), Option::None);

        let order = self.display_tag_ids.get_order(&id).unwrap_or(0);
        let tag = self.tags_by_id.get_mut(&id).unwrap();
        tag.set_order(order);
        self.rebuild_filtered_display_tag_ids();
        Some(id)
    }

    /// Reinitialize the tag list from a snapshot, preserving the current store.
    /// Used by PasteTagsCommand to undo/redo paste operations.
    pub fn reinitialize_from_snapshot(&mut self, snapshot: FileSnapshot) {
        *self = TagList::new_unlocked(self.store.clone(), snapshot);
    }

    /// Capture all data needed to undo a delete for the tag with the given id.
    /// Returns (name, color_index, was_stored, was_starred, was_checked, sort_order).
    /// Returns None if the tag is not found.
    pub fn capture_delete_data(&self, id: TagId) -> Option<(String, u8, bool, bool, bool, i64)> {
        let tag = self.tags_by_id.get(&id)?;
        let sort_order = self.display_tag_ids.get_order(&id).unwrap_or(0);
        Some((
            tag.tag().to_string(),
            tag.color_index(),
            tag.is_stored(),
            tag.is_starred(),
            tag.is_checked(),
            sort_order,
        ))
    }

    /// Restore a tag that was previously deleted. Inserts back into the store (if stored) and
    /// both ordered collections using the original sort_order.
    pub fn restore_deleted_tag(
        &mut self,
        tag_id: TagId,
        tag_name: &str,
        color_index: u8,
        was_stored: bool,
        was_starred: bool,
        was_checked: bool,
        sort_order: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if was_stored {
            let st = StoredTag::with_all(tag_id.0, tag_name, sort_order, was_starred);
            self.store.save_tag(st, color_index)?;
        }
        let mut tag = Tag::with_id_order_checked(
            tag_id, tag_name, color_index, was_stored, sort_order, was_checked,
        );
        tag.set_starred(was_starred);
        self.tags_by_id.insert(tag_id, tag);
        self.display_tag_ids.insert(tag_id, (), sort_order);
        self.selected_tag_ids.insert(tag_id, (), sort_order);
        self.rebuild_filtered_display_tag_ids();
        Ok(())
    }

    /// Create a new unsaved tag with a specific id (for redo of CreateTagCommand so the UUID
    /// is stable across redo). Returns false if a tag with that name already exists.
    pub fn create_tag_with_id(&mut self, id: TagId, name: impl Into<String>) -> bool {
        let name = name.into();
        if self.has_tag_with_name(&name) {
            return false;
        }
        let tag = Tag::with_id_order_checked(id, &name, 0, false, 0, true);
        self.tags_by_id.insert(id, tag);
        self.display_tags_rebalanced |= self.display_tag_ids.insert_before(id, (), None);
        self.selected_tag_ids.insert_before(id, (), None);
        let order = self.display_tag_ids.get_order(&id).unwrap_or(0);
        if let Some(t) = self.tags_by_id.get_mut(&id) {
            t.set_order(order);
        }
        self.rebuild_filtered_display_tag_ids();
        true
    }

    /// Save a tag to the store with a forced color index (used by CreateTagCommand::redo to
    /// preserve the original color across undo/redo cycles).
    pub fn save_tag_with_color(
        &mut self,
        id: TagId,
        color_index: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (text, starred) = self
            .tags_by_id
            .get(&id)
            .map(|t| (t.tag().to_string(), t.is_starred()))
            .ok_or_else(|| {
                Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "tag not found"))
                    as Box<dyn std::error::Error + Send + Sync>
            })?;
        let order = self.display_tag_ids.get_order(&id).unwrap_or(0);
        let st = StoredTag::with_all(id.0, &text, order, starred);
        self.store.save_tag(st, color_index)?;
        if let Some(t) = self.tags_by_id.get_mut(&id) {
            t.set_stored(true);
            t.set_color_index(color_index);
        }
        if self.display_tags_rebalanced {
            let tag_orders: Vec<(uuid::Uuid, i64)> = self
                .display_tag_ids
                .iter()
                .filter_map(|(tid, _, ord)| {
                    self.tags_by_id.get(tid).filter(|t| t.is_stored()).map(|_| (tid.0, ord))
                })
                .collect();
            self.store.update_tag_orders(&tag_orders)?;
            self.display_tags_rebalanced = false;
        }
        Ok(())
    }

    /// Remove a stored tag from the DB and mark it as unsaved in memory (for SaveTagCommand undo).
    pub fn unsave_tag(
        &mut self,
        id: TagId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.tags_by_id.get(&id).map_or(false, |t| t.is_stored()) {
            self.store.remove_stored_tag_by_id(id.0)?;
        }
        if let Some(t) = self.tags_by_id.get_mut(&id) {
            t.set_stored(false);
        }
        Ok(())
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

    // --- Sync helpers ---

    /// Sync Up: apply the order from `selected_tag_ids` (file name panel) onto
    /// `display_tag_ids` (grid/DB), persist to the store, then rebuild the grid cache.
    pub fn sync_selected_to_display(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.display_tag_ids
            .sync_order_from(&self.selected_tag_ids);
        self.persist_display_order()?;
        self.rebuild_filtered_display_tag_ids();
        Ok(())
    }

    /// Sync Down: apply the order from `display_tag_ids` (grid/DB) onto
    /// `selected_tag_ids` (file name panel). No DB write needed.
    pub fn sync_display_to_selected(&mut self) {
        self.selected_tag_ids
            .sync_order_from(&self.display_tag_ids);
    }

    /// Returns `true` if the checked tags have the same relative order in both
    /// `display_tag_ids` (grid/DB) and `selected_tag_ids` (file name panel).
    pub fn is_selected_order_same_as_display_order(&self) -> bool {
        let in_display: Vec<TagId> = self.display_tag_ids
            .iter()
            .map(|(id, _, _)| *id)
            .filter(|id| self.tags_by_id.get(id).map_or(false, |t| t.is_checked()))
            .collect();
        let in_selected = self.checked_ids_in_selected_order();
        in_display == in_selected
    }

    // --- Star / unstar ---

    /// Flush the current display order for all stored tags to the database.
    fn persist_display_order(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tag_orders: Vec<(Uuid, i64)> = self
            .display_tag_ids
            .iter()
            .filter_map(|(tid, _, ord)| {
                self.tags_by_id.get(tid).filter(|t| t.is_stored()).map(|_| (tid.0, ord))
            })
            .collect();
        self.store.update_tag_orders(&tag_orders)
    }

    /// Star a stored tag: set `starred = true` and persist the flag.
    /// Does not change the tag's position in either ordered collection —
    /// the grid sorts starred tags to the top visually via `rebuild_filtered_display_tag_ids`.
    pub fn star_tag(&mut self, id: TagId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (is_stored, already_starred) = self
            .tags_by_id
            .get(&id)
            .map(|t| (t.is_stored(), t.is_starred()))
            .ok_or_else(|| {
                Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "tag not found"))
                    as Box<dyn std::error::Error + Send + Sync>
            })?;
        if !is_stored {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cannot star an unsaved tag",
            )));
        }
        if already_starred {
            return Ok(());
        }
        if let Some(tag) = self.tags_by_id.get_mut(&id) {
            tag.set_starred(true);
        }
        self.save_tag(id)?;
        self.rebuild_filtered_display_tag_ids();
        Ok(())
    }

    /// Unstar a stored tag: set `starred = false` and persist the flag.
    /// Does not change the tag's position in either ordered collection.
    pub fn unstar_tag(&mut self, id: TagId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (is_stored, already_unstarred) = self
            .tags_by_id
            .get(&id)
            .map(|t| (t.is_stored(), !t.is_starred()))
            .ok_or_else(|| {
                Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "tag not found"))
                    as Box<dyn std::error::Error + Send + Sync>
            })?;
        if !is_stored {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cannot unstar an unsaved tag",
            )));
        }
        if already_unstarred {
            return Ok(());
        }
        if let Some(tag) = self.tags_by_id.get_mut(&id) {
            tag.set_starred(false);
        }
        self.save_tag(id)?;
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
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "A", 1, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "B", 2, false), 1)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "C", 3, false), 2);
        let list = TagList::new(store, FileSnapshot::default());
        assert_eq!(list.filtered_display_tag_ids().len(), 3);
        assert_eq!(list.get_tag(list.filtered_display_tag_ids()[0]).unwrap().tag(), "A");
        assert_eq!(list.get_tag(list.filtered_display_tag_ids()[1]).unwrap().tag(), "B");
        assert_eq!(list.get_tag(list.filtered_display_tag_ids()[2]).unwrap().tag(), "C");
    }

    #[test]
    fn test_tag_list_first_and_last() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "First", 1, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "Last", 2, false), 0);
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
        assert!(first.is_checked(), "snapshot-only tags appear in the file name so they init with checked=true");
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
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "A", 1, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "B", 2, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "C", 3, false), 0);
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
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "StoredA", 1, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "StoredB", 2, false), 0);
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
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "Stored1", 10, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "Stored2", 20, false), 0);
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
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "A", 1, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "B", 2, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "C", 3, false), 0);
        let list = TagList::new(store, FileSnapshot::default());
        let ids = list.filtered_display_tag_ids();
        assert_eq!(list.get_tag(ids[0]).unwrap().tag(), "A");
        assert_eq!(list.get_tag(ids[1]).unwrap().tag(), "B");
        assert_eq!(list.get_tag(ids[2]).unwrap().tag(), "C");
    }

    #[test]
    fn init_stored_not_in_snapshot_after_snapshot_tags() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "S", 1, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "T", 2, false), 0);
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

    // --- Search ranking tests ---

    fn make_stored_list(names: &[&str]) -> TagList<FakeAppStorage> {
        let mut store = FakeAppStorage::new();
        for (i, name) in names.iter().enumerate() {
            store = store.add_stored_tag(StoredTag::with_all(Uuid::new_v4(), *name, (i + 1) as i64, false), 0);
        }
        TagList::new(store, FileSnapshot::default())
    }

    fn filtered_names(list: &TagList<FakeAppStorage>) -> Vec<String> {
        list.filtered_display_tag_ids()
            .iter()
            .filter_map(|id| list.get_tag(*id))
            .map(|t| t.tag().to_string())
            .collect()
    }

    #[test]
    fn rank_full_match_first() {
        let mut list = make_stored_list(&["Hello", "aHello", "Hel"]);
        list.set_filter("Hel");
        assert_eq!(filtered_names(&list), ["Hel", "Hello", "aHello"]);
    }

    #[test]
    fn rank_prefix_before_contains() {
        let mut list = make_stored_list(&["Hello", "aHello", "Hel"]);
        list.set_filter("H");
        assert_eq!(filtered_names(&list), ["Hello", "Hel", "aHello"]);
    }

    #[test]
    fn rank_contains_only() {
        let mut list = make_stored_list(&["Hello", "aHello", "Hel"]);
        list.set_filter("l");
        assert_eq!(filtered_names(&list), ["Hello", "aHello", "Hel"]);
    }

    #[test]
    fn rank_case_insensitive() {
        let mut list = make_stored_list(&["Hello", "aHello", "Hel"]);
        list.set_filter("hel");
        assert_eq!(filtered_names(&list), ["Hel", "Hello", "aHello"]);
    }

    #[test]
    fn rank_empty_query_preserves_original_order() {
        let mut list = make_stored_list(&["Hello", "aHello", "Hel"]);
        list.set_filter("");
        assert_eq!(filtered_names(&list), ["Hello", "aHello", "Hel"]);
    }

    #[test]
    fn init_filter_empty_display_order_equals_filtered_order() {
        let store = FakeAppStorage::new()
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "A", 1, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "B", 2, false), 0)
            .add_stored_tag(StoredTag::with_all(Uuid::new_v4(), "C", 3, false), 0);
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
