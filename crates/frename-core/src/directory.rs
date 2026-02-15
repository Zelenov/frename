//! Directory scanning and file list management.

use crate::File;
use std::path::Path;
use std::time::SystemTime;

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

    /// Open a directory asynchronously: scan all files and sort by creation date.
    pub async fn open(directory: &Path) -> Result<Self, std::io::Error> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(directory).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let metadata = entry.metadata().await?;
                let created = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);
                files.push(File::from_path(path, created));
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

    /// Get a mutable reference to the currently selected file.
    pub fn selected_file_mut(&mut self) -> Option<&mut File> {
        let idx = self.selected_index?;
        self.files.get_mut(idx)
    }

    /// Get a mutable reference to the file at the given index.
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

    /// Number of files in the directory.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the directory has no files.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}
