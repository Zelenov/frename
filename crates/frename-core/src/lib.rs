//! Core logic for frename - file renaming utility.

mod rename_core;
mod tags;

use std::path::{Path, PathBuf};

pub use rename_core::RenameCore;
pub use tags::{Tag, TagList};

/// Holds the context for a file being processed.
pub struct FileInfo {
    file_path: PathBuf,
}

impl FileInfo {
    /// Create a new FileInfo from a file path.
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }

    /// Get the file path.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}
