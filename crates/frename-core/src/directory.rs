//! Directory scanning and file list management.
//!
//! Files are stored in two structures:
//!   1. `files_by_id: HashMap<FileId, File>` — O(1) lookup and mutation by stable ID.
//!   2. `order: Vec<FileId>`                 — creation-date order for indexed access.
//!
//! Selection is stored as `selected_id: Option<FileId>`, stable across renames.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::db::AppStateStore;
use crate::{File, FileId, FileSnapshot, FolderAndFile};

/// A scanned directory. Generic over the store type S; store is used only for session persistence.
#[derive(Clone, Debug)]
pub struct Directory<S> {
    path: Box<Path>,
    files_by_id: HashMap<FileId, File>,
    /// Creation-date order; stable across renames.
    order: Vec<FileId>,
    /// Identity of the currently selected file (stable across renames).
    selected_id: Option<FileId>,
    store: S,
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl<S: AppStateStore + Clone> Directory<S> {
    /// Create a directory from an already-sorted file list. For testing and programmatic use.
    pub fn with_files(path: impl AsRef<Path>, files: Vec<File>, store: S) -> Self {
        let order: Vec<FileId> = files.iter().map(|f| f.id()).collect();
        let files_by_id: HashMap<FileId, File> = files.into_iter().map(|f| (f.id(), f)).collect();
        Self {
            path: path.as_ref().to_path_buf().into_boxed_path(),
            files_by_id,
            order,
            selected_id: None,
            store,
        }
    }

    /// Open a directory asynchronously: scan all files and sort by creation date.
    pub async fn open(directory: &Path, store: S) -> Result<Self, std::io::Error> {
        log::info!("Scanning directory: {}", directory.display());
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(directory)
            .await
            .map_err(|e| { log::error!("Failed to scan directory: {e}"); e })?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| { log::error!("Failed to scan directory: {e}"); e })?
        {
            let path = entry.path();
            if !path.is_file() { continue; }
            if crate::FileTagger::is_sidecar_file(&path) { continue; }
            files.push(
                File::open(&path)
                    .await
                    .map_err(|e| { log::error!("Failed to scan directory: {e}"); e })?,
            );
        }
        files.sort_by(|a, b| a.created_at().cmp(&b.created_at()));
        store.set_last_folder_and_file(&FolderAndFile::new(directory, None::<PathBuf>));
        log::info!("Directory scan complete: {} files found", files.len());
        Ok(Self::with_files(directory, files, store))
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    pub fn is_empty(&self) -> bool { self.order.is_empty() }

    /// Files in creation-date order (for rendering the list).
    pub fn files_in_order(&self) -> impl Iterator<Item = &File> {
        self.order.iter().filter_map(|id| self.files_by_id.get(id))
    }

    /// Look up a file by its stable ID. O(1).
    pub fn file_by_id(&self, id: FileId) -> Option<&File> {
        self.files_by_id.get(&id)
    }

    /// Index of the selected file in creation-date order.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_id.and_then(|id| self.order.iter().position(|oid| *oid == id))
    }

    pub fn selected_file(&self) -> Option<&File> {
        self.selected_id.and_then(|id| self.files_by_id.get(&id))
    }

    pub fn has_previous_next(&self) -> (bool, bool) {
        let idx = self.selected_index().unwrap_or(0);
        (idx > 0, idx + 1 < self.order.len())
    }

    // -----------------------------------------------------------------------
    // Selection
    // -----------------------------------------------------------------------

    /// Open a file by path. Returns the file if it is in this directory and was opened.
    pub fn open_path(&mut self, path: &Path) -> Option<File> {
        let id = self.files_by_id.values().find(|f| f.file_path() == path)?.id();
        if self.selected_id == Some(id) { return None; }
        self.select_by_id(id)
    }

    /// Select by stable ID. O(1) existence check. Returns the selected file if found.
    pub(crate) fn select_by_id(&mut self, id: FileId) -> Option<File> {
        if !self.files_by_id.contains_key(&id) { return None; }
        self.selected_id = Some(id);
        self.persist_session();
        self.files_by_id.get(&id).cloned()
    }

    /// Select by index in creation-date order. O(1).
    pub fn select_index(&mut self, index: usize) -> Option<File> {
        let id = *self.order.get(index)?;
        self.select_by_id(id)
    }

    pub fn select_previous(&mut self) -> Option<File> {
        let current = self.selected_index().unwrap_or(0);
        if current == 0 { return None; }
        self.select_index(current - 1)
    }

    pub fn select_next(&mut self) -> Option<File> {
        let current = self.selected_index().unwrap_or(0);
        if current + 1 >= self.order.len() { return None; }
        self.select_index(current + 1)
    }

    // -----------------------------------------------------------------------
    // File mutation (rename / update)
    // -----------------------------------------------------------------------

    /// Update both the path and snapshot of the file identified by `id`. O(1).
    /// Returns true if the file was found.
    pub fn rename_file(&mut self, id: FileId, new_path: &Path, snapshot: &FileSnapshot) -> bool {
        match self.files_by_id.get_mut(&id) {
            Some(file) => {
                let old_path = file.file_path().to_path_buf();
                file.set_file_path(new_path);
                file.set_file_snapshot(snapshot);
                if old_path != new_path {
                    log::info!(
                        "Directory: renamed {} → {} ({} tag(s))",
                        old_path.display(), new_path.display(), snapshot.tags().len()
                    );
                } else {
                    log::info!(
                        "Directory: updated {} ({} tag(s))",
                        new_path.display(), snapshot.tags().len()
                    );
                }
                true
            }
            None => {
                log::warn!("Directory::rename_file: id not found");
                false
            }
        }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn persist_session(&self) {
        self.store.set_last_folder_and_file(&FolderAndFile::new(
            self.path.as_ref(),
            self.selected_file().map(|f| f.file_path().to_path_buf()),
        ));
    }
}
