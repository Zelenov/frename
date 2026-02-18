//! Directory scanning and file list management.
//!
//! Directory is generic over the store type S. Store is passed only to the constructor; used internally for persistence. No Arc.

use crate::db::AppStateStore;
use crate::{File, FileSnapshot, FolderAndFile};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types and data
// ---------------------------------------------------------------------------

/// A scanned directory containing a sorted list of files and an optional selection.
/// Generic over the store type S; store is set only in the constructor.
#[derive(Clone, Debug)]
pub struct Directory<S> {
    path: Box<Path>,
    files: Vec<File>,
    selected_index: Option<usize>,
    store: S,
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl<S: AppStateStore + Clone> Directory<S> {
    /// Create a directory with the given path and files. For testing and programmatic use only (not used by production UI). Store is kept for persistence.
    pub fn with_files(path: impl AsRef<Path>, files: Vec<File>, store: S) -> Self {
        Self {
            path: path.as_ref().to_path_buf().into_boxed_path(),
            files,
            selected_index: None,
            store,
        }
    }

    /// Open a directory asynchronously: scan all files and sort by creation date. Store is kept for later persistence.
    pub async fn open(directory: &Path, store: S) -> Result<Self, std::io::Error> {
        log::info!("Scanning directory: {}", directory.display());
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(directory)
            .await
            .map_err(|e| {
                log::error!("Failed to scan directory: {e}");
                e
            })?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| {
                log::error!("Failed to scan directory: {e}");
                e
            })?
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            files.push(
                File::open(&path)
                    .await
                    .map_err(|e| {
                        log::error!("Failed to scan directory: {e}");
                        e
                    })?,
            );
        }
        files.sort_by(|a, b| a.created_at().cmp(&b.created_at()));
        store.set_last_folder_and_file(&FolderAndFile::new(directory, None::<PathBuf>));
        log::info!("Directory scan complete: {} files found", files.len());
        Ok(Self {
            path: directory.to_path_buf().into_boxed_path(),
            files,
            selected_index: None,
            store,
        })
    }

    /// Directory path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// List of files.
    pub fn files(&self) -> &[File] {
        &self.files
    }

    /// Get the selected file index.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Get the currently selected file.
    pub fn selected_file(&self) -> Option<&File> {
        self.selected_index.and_then(|i| self.files.get(i))
    }

    /// Mutable reference to the file at the given index (for updating tags in update_file).
    fn file_at_mut(&mut self, index: usize) -> Option<&mut File> {
        self.files.get_mut(index)
    }

    /// Whether the directory has no files.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns (has_previous, has_next) for the current selection.
    pub fn has_previous_next(&self) -> (bool, bool) {
        let idx = self.selected_index.unwrap_or(0);
        let len = self.files.len();
        (idx > 0, idx + 1 < len)
    }

    // -----------------------------------------------------------------------
    // Selection and mutation
    // -----------------------------------------------------------------------

    /// Set selection by path. `None` = clear selection. Returns the selected file if any.
    pub fn set_selection(&mut self, target: Option<&Path>) -> Option<File> {
        match target {
            None => {
                self.clear_selection();
                None
            }
            Some(path) => {
                let index = match self.find_by_path(path) {
                    Some(i) => i,
                    None => {
                        log::warn!(
                            "Target file not found in scanned directory: {}",
                            path.display()
                        );
                        self.clear_selection();
                        return None;
                    }
                };
                self.select_index(index)
            }
        }
    }

    /// Open a file by path. Returns the file if it is in this directory and was opened, `None` if path is not in this directory or already the selected file.
    pub fn open_path(&mut self, path: &Path) -> Option<File> {
        if self.find_by_path(path).is_none() {
            return None;
        }
        if self.selected_file().map(|f| f.file_path()) == Some(path) {
            return None;
        }
        self.set_selection(Some(path))
    }

    /// Select by index. Returns the selected file if valid, `None` otherwise.
    pub fn select_index(&mut self, index: usize) -> Option<File> {
        if index >= self.files.len() {
            return None;
        }
        self.selected_index = Some(index);
        self.persist_session();
        self.selected_file().cloned()
    }

    /// Select previous file. Returns the file if selection changed, `None` at first file.
    pub fn select_previous(&mut self) -> Option<File> {
        let current = self.selected_index.unwrap_or(0);
        if current == 0 {
            return None;
        }
        self.select_index(current - 1)
    }

    /// Select next file. Returns the file if selection changed, `None` at last file.
    pub fn select_next(&mut self) -> Option<File> {
        let current = self.selected_index.unwrap_or(0);
        if current + 1 >= self.files.len() {
            return None;
        }
        self.select_index(current + 1)
    }

    /// Update the snapshot of the file at the given path. Returns true if the file was found and updated.
    pub fn update_file(&mut self, path: &Path, snapshot: &FileSnapshot) -> bool {
        let found = self
            .find_by_path(path)
            .and_then(|index| self.file_at_mut(index))
            .map(|file| {
                file.set_file_snapshot(snapshot);
            })
            .is_some();
        if !found {
            log::warn!("Directory::update_file path not found: {}", path.display());
        } else {
            log::info!(
                "Updated file: {} ({} tag(s))",
                path.display(),
                snapshot.tags().len()
            );
        }
        found
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn find_by_path(&self, path: &Path) -> Option<usize> {
        self.files.iter().position(|f| f.file_path() == path)
    }

    fn clear_selection(&mut self) {
        self.selected_index = None;
        self.persist_session();
    }

    fn persist_session(&self) {
        self.store.set_last_folder_and_file(&FolderAndFile::new(
            self.path(),
            self.selected_file().map(|f| f.file_path().to_path_buf()),
        ));
    }
}
