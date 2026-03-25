//! FileTagger facade: routes parse/save to the installed backend.
//!
//! Call `install_file_tagger(backend)` once at startup (in main.rs) before any
//! file operations. If no backend is installed the default is InMemoryFileTagger,
//! which is always correct for tests and debug builds.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::FileTaggerBackend;
use super::in_memory_file_tagger::InMemoryFileTagger;

static BACKEND: OnceLock<Box<dyn FileTaggerBackend>> = OnceLock::new();

/// Install the file tagger backend. Must be called before any file operations.
/// Subsequent calls are ignored (OnceLock semantics).
pub fn install_file_tagger(backend: Box<dyn FileTaggerBackend>) {
    let _ = BACKEND.set(backend);
}

fn backend() -> &'static dyn FileTaggerBackend {
    BACKEND.get_or_init(|| Box::new(InMemoryFileTagger::default())).as_ref()
}

/// Thin facade over the installed `FileTaggerBackend`.
pub struct FileTagger;

impl FileTagger {
    pub fn parse(path: &Path) -> FileSnapshot {
        backend().parse(path)
    }

    /// Returns the new path (may differ from `path` after a disk rename).
    pub fn save(snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        backend().save(snapshot, path)
    }

    /// Returns true if `path` is a sidecar file that should be hidden from the file list.
    pub fn is_sidecar_file(path: &Path) -> bool {
        backend().is_sidecar_file(path)
    }
}

/// Extension trait: save this snapshot then re-parse from the new path.
/// Returns `(new_path, new_snapshot)`.
pub trait SaveAndReparse {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, FileSnapshot);
}

impl SaveAndReparse for FileSnapshot {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, FileSnapshot) {
        let new_path = FileTagger::save(self, path);
        let snapshot = FileTagger::parse(&new_path);
        (new_path, snapshot)
    }
}
