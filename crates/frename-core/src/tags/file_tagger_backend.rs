//! FileTaggerBackend trait: the interface for parsing and saving file snapshots.

use std::path::{Path, PathBuf};

use super::FileSnapshot;

/// The interface that both InMemoryFileTagger and ProductionFileTagger implement.
pub trait FileTaggerBackend: Send + Sync {
    /// Parse the file at `path` and return its snapshot (tags, name, extension).
    fn parse(&self, path: &Path) -> FileSnapshot;

    /// Save `snapshot` for the file at `path`.
    /// Returns the new path (which may differ from `path` if the file was renamed).
    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf;
}
