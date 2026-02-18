//! Traits for app state storage: session read/write, stored tags, and one-time initialization.

use crate::{FolderAndFile, StoredTag};

/// Interface for storing and restoring app state (last folder and file).
/// Implemented by the application database and by the test fake (e.g. `FakeAppStorage`).
/// Pass by value (e.g. `Box<dyn AppStateStore>`); no singleton, connection is opened per use.
pub trait AppStateStore: Send + Sync {
    /// Returns the last opened folder and file in it, if any.
    fn get_last_session(&self) -> Option<FolderAndFile>;

    /// Sets the last opened folder and file in it (inserts or updates).
    fn set_last_folder_and_file(&self, value: &FolderAndFile);
}

/// Interface for stored tags. Implemented by the database and by the test fake. Used by `TagList<S>`.
pub trait StoredTagStore: Send + Sync {
    /// Returns stored tags in index order.
    fn get_stored_tags(&self) -> Result<Vec<StoredTag>, Box<dyn std::error::Error + Send + Sync>>;

    /// Adds one stored tag.
    fn add_stored_tag(&mut self, tag: StoredTag) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// One-time setup (e.g. run migrations). Implemented by the database; the logging decorator wraps it.
pub trait Initializable: Send + Sync {
    /// Performs one-time setup. Call once at startup before using the store.
    fn initialize(&self) -> Result<(), rusqlite::Error>;
}
