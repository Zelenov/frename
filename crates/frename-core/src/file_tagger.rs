//! File tagger: parse file name to extract tags, and save tags back to the file.
//! Dummy implementation: in-memory map (path -> tags). Save updates it; parse returns what was saved.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::FileTag;

/// In-memory storage for the dummy tagger: path -> tags.
fn storage() -> &'static Mutex<HashMap<PathBuf, Vec<FileTag>>> {
    static STORAGE: OnceLock<Mutex<HashMap<PathBuf, Vec<FileTag>>>> = OnceLock::new();
    STORAGE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Parses a file (by path) and returns the list of tags found in it.
/// Save takes the current file tags and the file path and persists them.
pub struct FileTagger;

impl FileTagger {
    /// Parse the file at the given path and return the list of tags (file tag values) found.
    /// Returns owned `Vec`; dummy returns the tags previously saved for this path, or empty if never saved.
    pub fn parse(path: &Path) -> Vec<FileTag> {
        let tags = storage()
            .lock()
            .expect("dummy tagger storage lock")
            .get(&path.to_path_buf())
            .cloned()
            .unwrap_or_default();
        let values: Vec<&str> = tags.iter().map(FileTag::value).collect();
        log::info!("FileTagger parse {} -> {:?}", path.display(), values);
        tags
    }

    /// Save the given file tags to the file at the given path. Synchronous.
    /// Takes `&[FileTag]` (borrowed slice); returns the current path (unchanged for now).
    /// Dummy: stores in the in-memory map; parse will return this for the same path.
    pub fn save(file_tags: &[FileTag], path: &Path) -> String {
        let values: Vec<&str> = file_tags.iter().map(FileTag::value).collect();
        log::info!("FileTagger save {} <- {:?}", path.display(), values);
        storage()
            .lock()
            .expect("dummy tagger storage lock")
            .insert(path.to_path_buf(), file_tags.to_vec());
        path.display().to_string()
    }
}

/// Extension trait: save these file tags to the path then re-parse. Returns path and tags as stored (e.g. for UI sync).
pub trait SaveAndReparse {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, Vec<FileTag>);
}

impl SaveAndReparse for [FileTag] {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, Vec<FileTag>) {
        FileTagger::save(self, path);
        let path_buf = path.to_path_buf();
        let new_tags = FileTagger::parse(&path_buf);
        (path_buf, new_tags)
    }
}
