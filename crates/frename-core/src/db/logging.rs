//! Logging decorator for app state store: delegates to inner and adds logging.

use crate::FolderAndFile;

use super::traits::{AppStateStore, Initializable};

/// Decorator that implements `AppStateStore` by delegating to an inner store
/// and adding logging. Keeps core logic in the inner type and logging in this wrapper.
#[derive(Clone, Debug)]
pub struct LoggingAppStateStore<S> {
    pub(super) inner: S,
}

impl<S> LoggingAppStateStore<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S: AppStateStore> AppStateStore for LoggingAppStateStore<S> {
    fn get_last_session(&self) -> Option<FolderAndFile> {
        log::debug!("AppStateStore::get_last_session()");
        let result = self.inner.get_last_session();
        log::debug!(
            "AppStateStore::get_last_session() -> {}",
            result
                .as_ref()
                .map(|s| format!("folder={}, file={:?}", s.folder().display(), s.file()))
                .unwrap_or_else(|| "None".to_string())
        );
        result
    }

    fn set_last_folder_and_file(&self, value: &FolderAndFile) {
        log::debug!(
            "AppStateStore::set_last_folder_and_file(folder={}, file={:?})",
            value.folder().display(),
            value.file()
        );
        self.inner.set_last_folder_and_file(value);
        log::debug!("AppStateStore::set_last_folder_and_file() done");
    }
}

impl<S: AppStateStore + Initializable> Initializable for LoggingAppStateStore<S> {
    fn initialize(&self) -> Result<(), rusqlite::Error> {
        log::debug!("AppStateStore::initialize()");
        match self.inner.initialize() {
            Ok(()) => {
                log::info!("AppStateStore::initialize() ok");
                Ok(())
            }
            Err(e) => {
                log::error!("AppStateStore::initialize() failed: {}", e);
                Err(e)
            }
        }
    }
}
