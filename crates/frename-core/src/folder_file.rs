//! A folder path and an optional file path within that folder. Used for last session, scan target, etc.

use std::path::{Path, PathBuf};

/// A folder path and an optional file in that folder. Session = folder (required) + file (optional).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderAndFile {
    pub folder: PathBuf,
    pub file: Option<PathBuf>,
}

impl FolderAndFile {
    pub fn new(folder: impl Into<PathBuf>, file: Option<impl Into<PathBuf>>) -> Self {
        Self {
            folder: folder.into(),
            file: file.map(Into::into),
        }
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }
}
