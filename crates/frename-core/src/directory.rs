//! Directory scanning and file list management.
//!
//! Also provides last-session persistence (DB-backed) so the UI does not depend on the database type.

use crate::db::{AppDatabase, AppStateStore};
use crate::{File, FileTag, FolderAndFile};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types and data
// ---------------------------------------------------------------------------

/// A scanned directory containing a sorted list of files and an optional selection.
#[derive(Debug, Clone)]
pub struct Directory {
    /// Directory path that was scanned (immutable).
    path: Box<Path>,
    /// Files in the directory, sorted by creation date (oldest first).
    files: Vec<File>,
    /// Index of the currently selected file.
    selected_index: Option<usize>,
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl Directory {
    /// Create an empty directory (no files scanned).
    pub fn empty(directory: impl AsRef<Path>) -> Self {
        Self {
            path: directory.as_ref().to_path_buf().into_boxed_path(),
            files: Vec::new(),
            selected_index: None,
        }
    }

    /// Create a directory with the given path and files. For testing and programmatic use only (not used by production UI).
    pub fn with_files(path: impl AsRef<Path>, files: Vec<File>) -> Self {
        Self {
            path: path.as_ref().to_path_buf().into_boxed_path(),
            files,
            selected_index: None,
        }
    }

    /// Open a directory asynchronously: scan all files and sort by creation date.
    /// Persists (folder, no file) as current session.
    pub async fn open(directory: &Path) -> Result<Self, std::io::Error> {
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
        write_session(directory, None);
        log::info!("Directory scan complete: {} files found", files.len());
        Ok(Self {
            path: directory.to_path_buf().into_boxed_path(),
            files,
            selected_index: None,
        })
    }

    // -----------------------------------------------------------------------
    // Accessors (read-only)
    // -----------------------------------------------------------------------

    /// Get the directory path (used internally for session persistence).
    fn path(&self) -> &Path {
        &self.path
    }

    /// Get the list of files.
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

    /// Update the tags of the file at the given path. Returns true if the file was found and updated.
    pub fn update_file(&mut self, path: &Path, new_tags: &[FileTag]) -> bool {
        let found = self
            .find_by_path(path)
            .and_then(|index| self.file_at_mut(index))
            .map(|file| {
                file.set_file_tags(new_tags);
            })
            .is_some();
        if !found {
            log::warn!("Directory::update_file path not found: {}", path.display());
        } else {
            log::info!("Updated file: {} ({} tag(s))", path.display(), new_tags.len());
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
        write_session(
            self.path(),
            self.selected_file().map(|f| f.file_path().to_path_buf()),
        );
    }
}

// ---------------------------------------------------------------------------
// Last-session persistence (default DB; UI does not touch the database)
// ---------------------------------------------------------------------------

fn write_session(folder: &Path, file: Option<PathBuf>) {
    save_last_session(&FolderAndFile::new(folder, file));
}

fn save_last_session(value: &FolderAndFile) {
    let db = AppDatabase::new();
    db.initialize();
    db.set_last_folder_and_file(value);
}

/// Ensures the app database exists and migrations are run. Call once at startup before any iced work.
pub fn ensure_db_initialized() {
    AppDatabase::new().initialize();
}

/// Returns the last opened folder and file, if any. Uses the default app database.
pub fn open_last_directory() -> Option<FolderAndFile> {
    let db = AppDatabase::new();
    db.initialize();
    db.get_last_session()
}
