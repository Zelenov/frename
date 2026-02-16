//! Directory scanning and file list management.

use crate::{File, FileTag};
use std::path::Path;

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

impl Directory {
    /// Create an empty directory (no files scanned).
    pub fn empty(directory: impl AsRef<Path>) -> Self {
        Self {
            path: directory.as_ref().to_path_buf().into_boxed_path(),
            files: Vec::new(),
            selected_index: None,
        }
    }

    /// Create a directory with the given path and files. For testing and programmatic use.
    pub fn with_files(path: impl AsRef<Path>, files: Vec<File>) -> Self {
        Self {
            path: path.as_ref().to_path_buf().into_boxed_path(),
            files,
            selected_index: None,
        }
    }

    /// Open a directory asynchronously: scan all files and sort by creation date.
    pub async fn open(directory: &Path) -> Result<Self, std::io::Error> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(directory).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                files.push(File::open(&path).await?);
            }
        }
        files.sort_by(|a, b| a.created_at().cmp(&b.created_at()));
        Ok(Self {
            path: directory.to_path_buf().into_boxed_path(),
            files,
            selected_index: None,
        })
    }

    /// Get the directory path.
    pub fn path(&self) -> &Path {
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

    /// Mutable reference to the file at the given index.
    pub fn file_at_mut(&mut self, index: usize) -> Option<&mut File> {
        self.files.get_mut(index)
    }

    /// Select a file by index. Returns true if the index was valid.
    pub fn select(&mut self, index: usize) -> bool {
        if index < self.files.len() {
            self.selected_index = Some(index);
            true
        } else {
            false
        }
    }

    /// Find a file by path and return its index.
    pub fn find_by_path(&self, path: &Path) -> Option<usize> {
        self.files.iter().position(|f| f.file_path() == path)
    }

    /// Select a file by path. Returns true if the path was found and selected.
    pub fn select_file(&mut self, path: &Path) -> bool {
        self.find_by_path(path).map_or(false, |index| self.select(index))
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
        }
        found
    }

    /// Number of files in the directory.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the directory has no files.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Clone the file at the given path if it exists in this directory.
    pub fn file_for_path(&self, path: &Path) -> Option<File> {
        self.find_by_path(path)
            .and_then(|index| self.files.get(index))
            .cloned()
    }

    /// Select the file at the given path and return it (cloned). Returns `None` if path not found.
    pub fn select_file_by_path(&mut self, path: &Path) -> Option<File> {
        let index = self.find_by_path(path)?;
        self.select(index);
        self.selected_file().cloned()
    }

    /// Select the previous file (by index). Returns true if selection changed.
    pub fn select_previous(&mut self) -> bool {
        let current = self.selected_index.unwrap_or(0);
        if current == 0 {
            false
        } else {
            self.select(current - 1)
        }
    }

    /// Select the next file (by index). Returns true if selection changed.
    pub fn select_next(&mut self) -> bool {
        let current = self.selected_index.unwrap_or(0);
        if current + 1 >= self.files.len() {
            false
        } else {
            self.select(current + 1)
        }
    }

    /// Returns (has_previous, has_next) for the current selection.
    pub fn has_previous_next(&self) -> (bool, bool) {
        let idx = self.selected_index.unwrap_or(0);
        let len = self.files.len();
        (idx > 0, idx + 1 < len)
    }
}
