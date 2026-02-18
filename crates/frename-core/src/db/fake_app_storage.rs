//! Single in-memory fake for app storage in tests. Implements AppStateStore and StoredTagStore; no database.

use crate::FolderAndFile;
use crate::StoredTag;

use super::traits::{AppStateStore, StoredTagStore};

/// In-memory app storage for tests. Implements AppStateStore and StoredTagStore.
/// Use add_stored_tag / with_last_session etc. to set up data for each test.
#[derive(Clone, Debug, Default)]
pub(crate) struct FakeAppStorage {
    last_session: Option<FolderAndFile>,
    stored_tags: Vec<StoredTag>,
}

#[allow(dead_code)] // Test helpers used from #[cfg(test)] and integration tests
impl FakeAppStorage {
    /// New empty fake (no session, no stored tags).
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Add one stored tag (builder-style).
    pub(crate) fn add_stored_tag(mut self, tag: StoredTag) -> Self {
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

    fn add_stored_tag(&mut self, tag: StoredTag) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stored_tags.push(tag);
        Ok(())
    }
}
