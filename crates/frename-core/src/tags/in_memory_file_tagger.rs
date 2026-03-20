//! In-memory FileTagger: stores snapshots in a HashMap. Used in debug builds and
//! when --debug is passed on the command line. Never touches the disk.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::file_tagger_backend::FileTaggerBackend;
use super::file_snapshot::FileSnapshot;

/// Parses a file name into tags / name / extension without touching the disk.
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
        2 => (Vec::new(), parts[0].clone(), format!(".{}", parts[1])),
        n => (
            parts[..n - 2].to_vec(),
            parts[n - 2].clone(),
            format!(".{}", parts[n - 1]),
        ),
    };
    FileSnapshot::new(tags, name_without_extension, extension, name)
}

/// In-memory FileTagger: no disk access. Snapshots are stored in a HashMap
/// keyed by the computed new path (which includes tags in the file name).
#[derive(Default)]
pub struct InMemoryFileTagger {
    storage: Mutex<HashMap<PathBuf, FileSnapshot>>,
}

impl FileTaggerBackend for InMemoryFileTagger {
    fn parse(&self, path: &Path) -> FileSnapshot {
        let stored = self
            .storage
            .lock()
            .expect("in-memory tagger lock")
            .get(path)
            .cloned();
        stored.unwrap_or_else(|| parse_file_name(path))
    }

    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        let new_file_name = snapshot.file_name();
        let new_path = path
            .parent()
            .map(|p| p.join(&new_file_name))
            .unwrap_or_else(|| PathBuf::from(&new_file_name));
        self.storage
            .lock()
            .expect("in-memory tagger lock")
            .insert(new_path.clone(), snapshot.clone());
        new_path
    }
}
