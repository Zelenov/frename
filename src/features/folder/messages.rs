//! Messages for the folder feature

use std::path::PathBuf;

use frename_core::FileInfo;

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
        files: Vec<FileInfo>,
        target_file: PathBuf,
    },
    /// Select a file by its index in the list
    SelectFile(usize),
}
