//! File tagger: parse file name to extract tags, and save tags back to the file.
//! Parse is async (used when loading directory); save is synchronous.

use std::path::Path;

use crate::FileTag;

/// Parses a file (by path) and returns the list of tags found in it.
/// Save takes the current file tags and the file path and persists them.
pub struct FileTagger;

impl FileTagger {
    /// Parse the file at the given path and return the list of tags (file tag values) found.
    /// For debug: returns an empty list.
    pub async fn parse(_path: &Path) -> Vec<FileTag> {
        Vec::new()
    }

    /// Save the given file tags to the file at the given path. Synchronous.
    /// Stub: no-op; real implementation would write to disk.
    pub fn save(_file_tags: &[FileTag], _path: &Path) {
    }
}
