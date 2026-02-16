//! Snapshot of a file's path and tags (to be persisted by the caller). No I/O.

use std::path::PathBuf;

use crate::FileTag;

/// Path and tags for a file. Folder workspace persists this when switching to another file.
#[derive(Debug, Clone)]
pub struct FileTagSnapshot {
    pub path: PathBuf,
    pub tags: Vec<FileTag>,
}
