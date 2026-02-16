//! State for file workspace: the file currently being edited and stored tags with checked state.
//!
//! We do not change the open file's file_tag_list on toggle; we only change the workspace tag_list.
//! File workspace does not save; it provides a snapshot (path + tags) that folder workspace persists when switching file.

use frename_core::{File, FileTagSnapshot, TagList, TagStorage};

/// File workspace: current file and stored tags with checked state (source of truth for UI).
#[derive(Default)]
pub struct FileWorkspace {
    loading: bool,
    /// Current file when ready. None while loading or when nothing set.
    file: Option<File>,
    /// Stored tags with checked state (synced from file on load; toggles update only this, not the file).
    tag_list: TagList,
}

impl FileWorkspace {
    /// Set the file to work on. When changing file, syncs workspace tags to the current file first, then loads the new one.
    pub fn set_file(&mut self, file: Option<File>) {
        match file {
            None => {
                self.loading = false;
                self.file = None;
                self.tag_list = TagList::new(TagStorage::names(), None);
            }
            Some(f) => {
                let already_loaded = self
                    .file
                    .as_ref()
                    .map_or(false, |current| current.file_path() == f.file_path());
                if already_loaded {
                    return;
                }
                self.file = Some(f);
                self.tag_list = TagList::new(TagStorage::names(), self.file().map(|file| file.tag_list()));
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
    pub fn tag_list(&self) -> &TagList {
        &self.tag_list
    }

    /// Toggle stored tag at index. Only updates workspace tag_list; file is synced after save.
    pub fn toggle_tag(&mut self, index: usize) {
        if let Some(tag) = self.tag_list.tags_mut().get_mut(index) {
            tag.toggle();
        }
    }

    /// Checked tags from workspace as FileTagList (for save). Use this, not file's tags.
    pub fn checked_file_tags(&self) -> frename_core::FileTagList {
        self.tag_list.checked_file_tags()
    }

    /// Snapshot of the current file's path and workspace tags (to be saved by folder workspace when switching file). Does not persist anything.
    pub fn get_snapshot(&self) -> Option<FileTagSnapshot> {
        let file = self.file()?;
        let path = file.file_path().to_path_buf();
        let tags = self.checked_file_tags();
        let tags_vec = tags.file_tags().to_vec();
        Some(FileTagSnapshot { path, tags: tags_vec })
    }
}
