//! Logging decorator for FileTagger: delegates to FileTagger and adds logging.

use std::path::Path;

use super::{FileSnapshot, FileTagger};

/// Wrapper that delegates to [FileTagger] and adds logging around parse and save.
#[derive(Clone, Copy, Debug)]
pub struct LoggingFileTagger;

impl LoggingFileTagger {
    /// Parse the file at the given path and return the file tag list. Logs before and after.
    pub fn parse(path: &Path) -> FileSnapshot {
        log::debug!("FileTagger::parse(path={})", path.display());
        let result = FileTagger::parse(path);
        log::debug!(
            "FileTagger::parse() -> tags={:?} name={} ext={}",
            result.tags(),
            result.name_without_extension(),
            result.extension()
        );
        result
    }

    /// Save the given file tag list for the file at the given path. Logs before and after.
    pub fn save(snapshot: &FileSnapshot, path: &Path) -> String {
        log::debug!(
            "FileTagger::save(path={}, tags={:?} name={} ext={})",
            path.display(),
            snapshot.tags(),
            snapshot.name_without_extension(),
            snapshot.extension()
        );
        let result = FileTagger::save(snapshot, path);
        log::debug!("FileTagger::save() -> {:?}", result);
        result
    }
}
