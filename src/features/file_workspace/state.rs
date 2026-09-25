//! State for file workspace: the file currently being edited and stored tags with checked state.
//!
//! We do not change the open file's snapshot on toggle; we only change the workspace tag list.
//! File workspace does not save; it provides a snapshot (path + tags) that folder workspace persists when switching file.
//!
//! Generic over the store type S (like Directory and TagList). Store is passed to the constructor; used to build the tag list.

use frename_core::{
    File, FileId, FileSnapshot, FolderTagStore, StoredTagStore, TagColorMapping, TagId, TagList,
};
use iced::widget::text_editor;

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
    /// Backing state for the multiline comment editor.
    pub comment_content: text_editor::Content,
}

impl<S: StoredTagStore + Clone> FileWorkspace<S> {
    /// Create a file workspace with the given store. Tag list is built from the store; no file selected.
    pub fn new(store: S) -> Self {
        Self {
            loading: false,
            file: None,
            store: store.clone(),
            tag_list: TagList::new(store, FileSnapshot::default()),
            comment_content: text_editor::Content::new(),
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
                    .is_some_and(|current| current.file_path() == f.file_path());
                if already_loaded {
                    return;
                }
                let snapshot = f.snapshot().clone();
                self.comment_content = text_editor::Content::with_text(snapshot.comment());
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

    /// Mutable access to the tag list (used by undo/redo to apply ReorderTagCommand).
    pub fn tag_list_mut(&mut self) -> &mut TagList<S> {
        &mut self.tag_list
    }

    /// Tag name -> color index mapping (for rendering file name chips in lists).
    pub fn tag_color_mapping(&self) -> TagColorMapping {
        self.store.get_tag_color_mapping().unwrap_or_default()
    }

    /// Comment text for the current file.
    pub fn comment(&self) -> &str {
        self.tag_list.comment()
    }

    /// Update the comment (does not write to disk). Positions the cursor at the end.
    pub fn set_comment(&mut self, comment: String) {
        self.comment_content = text_editor::Content::with_text(&comment);
        self.comment_content
            .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
        self.store_comment(comment);
    }

    /// Apply a text_editor action to the comment content and sync the string to tag_list.
    pub fn apply_comment_action(&mut self, action: text_editor::Action) {
        self.comment_content.perform(action);
        let text = self.comment_content.text();
        // text() appends a trailing newline; strip it for storage.
        let trimmed = text.trim_end_matches('\n').to_string();
        self.store_comment(trimmed);
    }

    /// Put the comment in the tag list. When it goes from empty to non-empty the commented tag
    /// is checked, and when it is cleared the tag is unchecked; any other edit leaves the tag to
    /// the user. See [`frename_core::active_commented_tag`].
    fn store_comment(&mut self, comment: String) {
        let was_empty = self.tag_list.comment().trim().is_empty();
        let is_empty = comment.trim().is_empty();
        self.tag_list.set_comment(comment);
        if was_empty == is_empty {
            return;
        }
        if let Some(tag) = frename_core::active_commented_tag() {
            self.tag_list.set_checked_by_name(&tag, !is_empty);
        }
    }

    /// Screenshot markers for the current file.
    pub fn screenshots(&self) -> &[frename_core::Screenshot] {
        self.tag_list.screenshots()
    }

    /// Add a screenshot marker (deduplicates, keeps sorted).
    pub fn add_screenshot(&mut self, screenshot: frename_core::Screenshot) {
        self.tag_list.add_screenshot(screenshot);
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
    pub fn save_tag(&mut self, id: TagId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tag_list.save_tag(id)
    }

    /// Returns true if any tag in the list has the given name (case-insensitive exact match). Used to
    /// decide whether to show the "create" button in the search bar.
    pub fn has_tag_with_name(&self, name: &str) -> bool {
        self.tag_list.has_tag_with_name(name)
    }

    /// Create a new unsaved tag, insert at the front of both collections, then immediately save to
    /// the store (assigns a random color). Returns the new tag's id.
    pub fn create_and_save_new_tag(
        &mut self,
        name: String,
    ) -> Result<TagId, Box<dyn std::error::Error + Send + Sync>> {
        let id = self.tag_list.create_new_tag(name).ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "tag with this name already exists",
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;
        self.tag_list.save_tag(id)?;
        self.set_tag_filter(String::new());
        Ok(id)
    }

    /// Reorders tags: place `dragged_id` at `drop_index` (file name panel drag). Only reorder entry point.
    pub fn reorder_tag_to_index(&mut self, dragged_id: TagId, drop_index: usize) {
        self.tag_list.reorder_tag_to_index(dragged_id, drop_index);
    }

    /// Star a stored tag (pin to section 2). Delegates to TagList.
    pub fn star_tag(&mut self, id: TagId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tag_list.star_tag(id)
    }

    /// Unstar a stored tag (move to section 3). Delegates to TagList.
    pub fn unstar_tag(
        &mut self,
        id: TagId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tag_list.unstar_tag(id)
    }

    /// Sync Up: copy file name panel order → display/DB. Returns error if persist fails.
    pub fn sync_selected_to_display(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tag_list.sync_selected_to_display()
    }

    /// Sync Down: copy display/DB order → file name panel. No DB write.
    pub fn sync_display_to_selected(&mut self) {
        self.tag_list.sync_display_to_selected();
    }

    /// Rebuild the tag list from scratch using the given snapshot (but keeping the same store).
    /// Used by paste: constructs a new TagList with the pasted tags as the snapshot.
    /// Uses unlocked mode: display keeps DB order (new unstored tags prepended), selected reflects
    /// the pasted snapshot order.
    pub fn reinitialize_tags_from_snapshot(&mut self, snapshot: FileSnapshot) {
        self.tag_list = TagList::new(self.store.clone(), snapshot);
    }

    /// Segment start in seconds for the current file, if set.
    pub fn segment_start_secs(&self) -> Option<f32> {
        self.tag_list.segment_start_secs()
    }

    /// Segment end in seconds for the current file, if set.
    pub fn segment_end_secs(&self) -> Option<f32> {
        self.tag_list.segment_end_secs()
    }

    /// Set the segment start marker to the given position in seconds.
    pub fn set_segment_start_secs(&mut self, secs: Option<f32>) {
        self.tag_list.set_segment_start_secs(secs);
    }

    /// Set the segment end marker to the given position in seconds.
    pub fn set_segment_end_secs(&mut self, secs: Option<f32>) {
        self.tag_list.set_segment_end_secs(secs);
    }

    /// Snapshot of the current file's stable id and workspace tag state.
    /// Used by folder workspace to save when switching file. Does not persist anything.
    pub fn get_snapshot(&self) -> Option<(FileId, FileSnapshot)> {
        let file = self.file()?;
        let id = file.id();
        let snapshot = self.tag_list.file_snapshot();
        Some((id, snapshot))
    }
}

impl Default for FileWorkspace<FolderTagStore> {
    /// Workspace with no folder open yet: the tag store holds nothing until a folder is loaded.
    fn default() -> Self {
        Self::new(FolderTagStore::empty())
    }
}
