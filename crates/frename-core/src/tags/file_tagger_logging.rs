//! Logging decorator for FileTagger: wraps parse/save with debug logging.

use std::path::{Path, PathBuf};

use super::{FileSnapshot, FileTagger, FolderInfo};

/// Delegates to `FileTagger` and adds logging around parse and save.
#[derive(Clone, Copy, Debug)]
pub struct LoggingFileTagger;

impl LoggingFileTagger {
    pub fn parse(path: &Path, folder_info: &FolderInfo) -> FileSnapshot {
        log::debug!("FileTagger::parse(path={})", path.display());
        let result = FileTagger::parse(path, folder_info);
        log::debug!(
            "FileTagger::parse() → tags={:?} name={} ext={}",
            result.tags(),
            result.name_without_extension(),
            result.extension()
        );
        result
    }

    pub fn save(snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        log::debug!(
            "FileTagger::save(path={}, tags={:?} name={} ext={})",
            path.display(),
            snapshot.tags(),
            snapshot.name_without_extension(),
            snapshot.extension()
        );
        let result = FileTagger::save(snapshot, path);
        log::debug!("FileTagger::save() → {:?}", result);
        result
    }
}
