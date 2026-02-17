//! No-op app state store for tests.

use crate::FolderAndFile;

use super::traits::AppStateStore;

/// No-op store: never returns a session, records nothing.
#[derive(Clone, Debug, Default)]
pub(crate) struct EmptyAppStateStore;

impl EmptyAppStateStore {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl AppStateStore for EmptyAppStateStore {
    fn get_last_session(&self) -> Option<FolderAndFile> {
        None
    }

    fn set_last_folder_and_file(&self, _value: &FolderAndFile) {}
}
