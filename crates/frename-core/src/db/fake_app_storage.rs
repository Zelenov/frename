//! Single in-memory fake for app storage in tests. Implements AppStateStore and StoredTagStore; no database.

use std::collections::HashMap;

use crate::FolderAndFile;
use crate::StoredTag;
use crate::TagColorMapping;

use super::traits::{AppStateStore, StoredTagStore};

/// In-memory app storage for tests. Implements AppStateStore and StoredTagStore.
/// Use add_stored_tag / with_last_session etc. to set up data for each test.
#[derive(Clone, Debug, Default)]
pub(crate) struct FakeAppStorage {
    last_session: Option<FolderAndFile>,
    stored_tags: Vec<StoredTag>,
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
        self.stored_tags.push(tag);
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
        let _ = value;
    }
}

impl StoredTagStore for FakeAppStorage {
    fn get_stored_tags(&self) -> Result<Vec<StoredTag>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.stored_tags.clone())
    }

    fn get_tag_color_mapping(&self) -> Result<TagColorMapping, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TagColorMapping::from_entries(self.tag_colors.clone().into_iter()))
    }

    fn add_stored_tag(
        &mut self,
        tag: StoredTag,
        color_index: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tag_colors.insert(tag.value().to_string(), color_index);
        self.stored_tags.push(tag);
        Ok(())
    }

    fn remove_stored_tag_by_id(&mut self, tag_id: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tag) = self.stored_tags.iter().find(|t| t.index() == tag_id) {
            self.tag_colors.remove(tag.value());
        }
        self.stored_tags.retain(|t| t.index() != tag_id);
        Ok(())
    }
}
