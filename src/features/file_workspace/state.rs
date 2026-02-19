//! State for file workspace: the file currently being edited and stored tags with checked state.
//!
//! We do not change the open file's snapshot on toggle; we only change the workspace tag list.
//! File workspace does not save; it provides a snapshot (path + tags) that folder workspace persists when switching file.
//!
//! Generic over the store type S (like Directory and TagList). Store is passed to the constructor; used to build the tag list.

use std::path::PathBuf;

use frename_core::{AppDatabase, File, FileSnapshot, StoredTagStore, TagColorMapping, TagId, TagList};

/// File workspace: current file and stored tags with checked state (source of truth for UI).
/// Generic over the store type S; store is set only in the constructor.
#[derive(Clone, Debug)]
pub struct FileWorkspace<S> {
    loading: bool,
    /// Current file when ready. None while loading or when nothing set.
    file: Option<File>,
    /// Store for tag names (used to build tag list on file change).
    store: S,
    /// Stored tags with checked state (synced from file on load; toggles update only this, not the file).
    tag_list: TagList<S>,
}

impl<S: StoredTagStore + Clone> FileWorkspace<S> {
    /// Create a file workspace with the given store. Tag list is built from the store; no file selected.
    pub fn new(store: S) -> Self {
        Self {
            loading: false,
            file: None,
            store: store.clone(),
            tag_list: TagList::new(store, FileSnapshot::default()),
        }
    }

    /// Set the file to work on. When changing file, syncs workspace tags to the current file first, then loads the new one.
    pub fn set_file(&mut self, file: Option<File>) {
        match file {
            None => {
                self.loading = false;
                self.file = None;
                self.tag_list = TagList::new(self.store.clone(), FileSnapshot::default());
            }
            Some(f) => {
                let already_loaded = self
                    .file
                    .as_ref()
                    .map_or(false, |current| current.file_path() == f.file_path());
                if already_loaded {
                    return;
                }
                let snapshot = f.snapshot().clone();
                self.file = Some(f);
                self.tag_list = TagList::new(self.store.clone(), snapshot);
            }
        }
    }

    /// The current file when ready. Returns None if nothing set or still loading.
    pub fn file(&self) -> Option<&File> {
        if self.loading {
            return None;
        }
        self.file.as_ref()
    }

    /// Stored tags with checked state (use this for UI; checked is the workspace source of truth).
    pub fn tag_list(&self) -> &TagList<S> {
        &self.tag_list
    }

    /// Tag name -> color index mapping (for rendering file name chips in lists).
    pub fn tag_color_mapping(&self) -> TagColorMapping {
        self.store.get_tag_color_mapping().unwrap_or_default()
    }

    /// Set the tag list filter query (case-insensitive contains). Used by the search bar.
    pub fn set_tag_filter(&mut self, query: String) {
        self.tag_list.set_filter(query);
    }

    /// Toggle stored tag by id. Only updates workspace tag_list; file is synced after save.
    pub fn toggle_tag_by_id(&mut self, id: TagId) {
        self.tag_list.toggle_by_id(id);
    }

    /// Remove a stored tag by id from the store and rebuild the tag list. Delegates to TagList.
    pub fn remove_stored_tag_by_id(
        &mut self,
        id: TagId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tag_list.remove_stored_tag_by_id(id)
    }

    /// Save a snapshot-only tag to the store (add to DB). Delegates to TagList.
    pub fn save_tag(
        &mut self,
        id: TagId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tag_list.save_tag(id)
    }

    /// Snapshot of the current file's path and workspace snapshot (to be saved by folder workspace when switching file). Does not persist anything.
    pub fn get_snapshot(&self) -> Option<(PathBuf, FileSnapshot)> {
        let file = self.file()?;
        let path = file.file_path().to_path_buf();
        let snapshot = self.tag_list.file_snapshot();
        Some((path, snapshot))
    }
}

impl Default for FileWorkspace<AppDatabase> {
    fn default() -> Self {
        Self::new(AppDatabase::new())
    }
}
