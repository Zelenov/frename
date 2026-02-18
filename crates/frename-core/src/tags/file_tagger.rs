//! File tagger: parse file name to extract tags, name, extension; save tags back.
//! Parse: split file name by dots, trim parts; last = extension, second-to-last = name, rest = tags.
//! Dummy: in-memory map (path -> list). Save updates it; parse returns stored if present else parsed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::FileSnapshot;

/// In-memory storage for the dummy tagger: path -> list.
fn storage() -> &'static Mutex<HashMap<PathBuf, FileSnapshot>> {
    static STORAGE: OnceLock<Mutex<HashMap<PathBuf, FileSnapshot>>> = OnceLock::new();
    STORAGE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Parses the file name at the given path: split by dots, trim; last = extension, second-to-last = name, rest = tags.
fn parse_file_name(path: &Path) -> FileSnapshot {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let parts: Vec<String> = name
        .split('.')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let (tags, name_without_extension, extension) = match parts.len() {
        0 => (Vec::new(), String::new(), String::new()),
        1 => (Vec::new(), parts[0].clone(), String::new()),
        2 => (Vec::new(), parts[0].clone(), parts[1].clone()),
        n => (
            parts[..n - 2].to_vec(),
            parts[n - 2].clone(),
            parts[n - 1].clone(),
        ),
    };
    FileSnapshot::new(tags, name_without_extension, extension, name)
}

/// Parses a file (by path) and returns the parsed tag list.
/// Dummy: returns the list previously saved for this path, or parsed from the path's file name.
pub struct FileTagger;

impl FileTagger {
    /// Parse the file at the given path and return the file tag list (tags, name, extension).
    pub fn parse(path: &Path) -> FileSnapshot {
        let path_buf = path.to_path_buf();
        let stored = storage()
            .lock()
            .expect("dummy tagger storage lock")
            .get(&path_buf)
            .cloned();
        match stored {
            Some(list) => list,
            None => parse_file_name(path),
        }
    }

    /// Save the given file tag list for the file at the given path. Synchronous.
    /// Dummy: stores in the in-memory map; parse will return this for the same path.
    pub fn save(snapshot: &FileSnapshot, path: &Path) -> String {
        storage()
            .lock()
            .expect("dummy tagger storage lock")
            .insert(path.to_path_buf(), snapshot.clone());
        path.display().to_string()
    }
}

/// Extension trait: save this file tag list to the path then re-parse. Returns path and list (e.g. for UI sync).
pub trait SaveAndReparse {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, FileSnapshot);
}

impl SaveAndReparse for FileSnapshot {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, FileSnapshot) {
        FileTagger::save(self, path);
        let path_buf = path.to_path_buf();
        let snapshot = FileTagger::parse(&path_buf);
        (path_buf, snapshot)
    }
}
