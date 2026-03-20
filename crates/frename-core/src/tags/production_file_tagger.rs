//! Production FileTagger: renames files on disk. Only used in release builds
//! when --debug is NOT passed on the command line.

use std::path::{Path, PathBuf};

use super::file_tagger_backend::FileTaggerBackend;
use super::file_snapshot::FileSnapshot;

/// Parses the file name at `path` by splitting on dots (identical to InMemoryFileTagger::parse_file_name).
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

/// Production FileTagger: actually renames files on disk when the name changes.
/// `parse` reads the snapshot from the current file name (the ground truth after a rename).
pub struct ProductionFileTagger;

impl FileTaggerBackend for ProductionFileTagger {
    /// Parse from the actual file name on disk (the ground truth after a rename).
    fn parse(&self, path: &Path) -> FileSnapshot {
        parse_file_name(path)
    }

    /// Rename the file on disk when the name changes. Returns the new path.
    /// Falls back to the old path and logs an error if the rename fails (e.g. file locked).
    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        let new_file_name = snapshot.file_name();
        let new_path = path
            .parent()
            .map(|p| p.join(&new_file_name))
            .unwrap_or_else(|| PathBuf::from(&new_file_name));

        if new_path != path {
            if let Err(e) = std::fs::rename(path, &new_path) {
                log::error!(
                    "ProductionFileTagger: rename {:?} → {:?} failed: {}",
                    path, new_path, e
                );
                return path.to_path_buf();
            }
            log::info!("Renamed on disk: {:?} → {:?}", path, new_path);
        }
        new_path
    }
}
