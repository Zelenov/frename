//! Messages for the folder feature

use std::path::PathBuf;

use frename_core::Directory;

/// Messages handled by the folder feature
#[derive(Debug, Clone)]
pub enum Message {
    /// Scan a directory and auto-select the target file afterwards
    ScanFolder {
        directory: PathBuf,
        target_file: PathBuf,
    },
    /// Directory scan completed
    FolderLoaded {
        directory: Directory,
        target_file: PathBuf,
    },
    /// Select a file by its index in the list
    SelectFile(usize),
    /// Select the previous file in the list (from folder controls)
    PreviousFile,
    /// Select the next file in the list (from folder controls)
    NextFile,
}
