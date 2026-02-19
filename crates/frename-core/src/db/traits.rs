//! Traits for app state storage: session read/write, stored tags, tag color mapping, and one-time initialization.

use crate::{FolderAndFile, StoredTag, TagColorMapping};

/// Interface for storing and restoring app state (last folder and file).
/// Implemented by the application database and by the test fake (e.g. `FakeAppStorage`).
/// Pass by value (e.g. `Box<dyn AppStateStore>`); no singleton, connection is opened per use.
pub trait AppStateStore: Send + Sync {
    /// Returns the last opened folder and file in it, if any.
    fn get_last_session(&self) -> Option<FolderAndFile>;

    /// Sets the last opened folder and file in it (inserts or updates).
    fn set_last_folder_and_file(&self, value: &FolderAndFile);
}

/// Interface for stored tags and tag color mapping. Tags are keyed by tag id (index); tag colors are keyed by tag name.
pub trait StoredTagStore: Send + Sync {
    /// Returns stored tags in index order (no color; use tag color mapping for colors).
    fn get_stored_tags(&self) -> Result<Vec<StoredTag>, Box<dyn std::error::Error + Send + Sync>>;

    /// Returns the tag name -> color index mapping (tag colors keyed by tag name).
    fn get_tag_color_mapping(&self) -> Result<TagColorMapping, Box<dyn std::error::Error + Send + Sync>>;

    /// Adds one stored tag and its color. Persists to stored_tags (by tag id) and tag_color_mapping (by tag name).
    fn add_stored_tag(
        &mut self,
        tag: StoredTag,
        color_index: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Removes a tag by tag id from stored_tags and its entry from tag_color_mapping (by tag name).
    fn remove_stored_tag_by_id(&mut self, tag_id: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// One-time setup (e.g. run migrations). Implemented by the database; the logging decorator wraps it.
pub trait Initializable: Send + Sync {
    /// Performs one-time setup. Call once at startup before using the store.
    fn initialize(&self) -> Result<(), rusqlite::Error>;
}
