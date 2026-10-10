//! Single in-memory fake for app storage in tests. Implements AppStateStore and StoredTagStore; no database.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use uuid::Uuid;

use crate::recent_folders::{self, RecentFolder};
use crate::FolderAndFile;
use crate::StoredTag;
use crate::TagColorMapping;

use super::traits::{AppStateStore, StoredTagStore};

/// In-memory app storage for tests. Implements AppStateStore and StoredTagStore.
/// Use add_stored_tag / with_last_session etc. to set up data for each test.
/// Stored tags are in an IndexMap (id -> tag) for add-or-update and insertion order.
#[derive(Clone, Debug, Default)]
pub(crate) struct FakeAppStorage {
    last_session: Option<FolderAndFile>,
    /// The recent folders, newest first (shared by clones, like a database).
    recent_folders: Arc<Mutex<Vec<RecentFolder>>>,
    /// Tag id -> tag; insertion order preserved for get_stored_tags.
    stored_tags: IndexMap<Uuid, StoredTag>,
    /// Tag name -> color index (mirrors tag_color_mapping table).
    tag_colors: HashMap<String, u8>,
}

#[allow(dead_code)] // Test helpers used from #[cfg(test)] and integration tests
impl FakeAppStorage {
    /// New empty fake (no session, no stored tags).
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Add one stored tag with color (builder-style).
    pub(crate) fn add_stored_tag(mut self, tag: StoredTag, color_index: u8) -> Self {
        self.tag_colors.insert(tag.value().to_string(), color_index);
        self.stored_tags.insert(tag.id(), tag);
        self
    }

    /// Set last session (for tests).
    pub(crate) fn with_last_session(mut self, session: FolderAndFile) -> Self {
        self.last_session = Some(session);
        self
    }

    /// Mutate last session (for future tests).
    pub(crate) fn set_last_session(&mut self, session: Option<FolderAndFile>) {
        self.last_session = session;
    }
}

impl AppStateStore for FakeAppStorage {
    fn get_last_session(&self) -> Option<FolderAndFile> {
        self.last_session.clone()
    }

    fn set_last_folder_and_file(&self, value: &FolderAndFile) {
        self.record_recent_folder(value, crate::recent_folders::now_ms());
    }

    fn get_recent_folders(&self) -> Vec<RecentFolder> {
        self.recent_folders.lock().map_or_else(
            |poisoned| poisoned.into_inner().clone(),
            |list| list.clone(),
        )
    }

    fn record_recent_folder(&self, value: &FolderAndFile, opened_at_ms: i64) {
        let mut list = self
            .recent_folders
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        recent_folders::record(&mut list, value, opened_at_ms);
    }

    fn forget_recent_folder(&self, folder: &Path) {
        let mut list = self
            .recent_folders
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        recent_folders::forget(&mut list, folder);
    }

    fn clear_recent_folders(&self) {
        self.recent_folders
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }
}

impl StoredTagStore for FakeAppStorage {
    fn get_stored_tags(&self) -> Result<Vec<StoredTag>, Box<dyn std::error::Error + Send + Sync>> {
        let mut tags: Vec<StoredTag> = self.stored_tags.values().cloned().collect();
        tags.sort_by_key(StoredTag::sort_order);
        Ok(tags)
    }

    fn get_tag_color_mapping(
        &self,
    ) -> Result<TagColorMapping, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TagColorMapping::from_entries(self.tag_colors.clone()))
    }

    fn save_tag(
        &mut self,
        tag: StoredTag,
        color_index: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tag_colors.insert(tag.value().to_string(), color_index);
        self.stored_tags.insert(tag.id(), tag);
        Ok(())
    }

    fn remove_stored_tag_by_id(
        &mut self,
        tag_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tag) = self.stored_tags.get(&tag_id) {
            self.tag_colors.remove(tag.value());
        }
        self.stored_tags.shift_remove(&tag_id);
        Ok(())
    }

    fn update_tag_orders(
        &mut self,
        tag_orders: &[(Uuid, i64)],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (id, order) in tag_orders {
            if let Some(tag) = self.stored_tags.get(id) {
                let updated =
                    StoredTag::with_all(*id, tag.value().to_string(), *order, tag.starred());
                self.stored_tags.insert(*id, updated);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fake_keeps_recent_folders_like_the_database() {
        let store = FakeAppStorage::new();
        store.set_last_folder_and_file(&FolderAndFile::new("/a", Some("/a/x.mp4")));
        store.set_last_folder_and_file(&FolderAndFile::new("/b", None::<&str>));
        store.set_last_folder_and_file(&FolderAndFile::new("/a", None::<&str>));
        let recent = store.get_recent_folders();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].folder, Path::new("/a"));
        assert_eq!(recent[0].last_file.as_deref(), Some(Path::new("/a/x.mp4")));
        store.forget_recent_folder(Path::new("/a"));
        assert_eq!(store.get_recent_folders().len(), 1);
        store.clear_recent_folders();
        assert!(store.get_recent_folders().is_empty());
    }
}
