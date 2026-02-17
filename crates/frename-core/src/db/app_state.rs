//! Application database and app state storage.
//!
//! The application has one database (`AppDatabase`) that holds app state, and will
//! hold user data and other application storage. It is not only an "app state store".
//! The trait `AppStateStore` is the interface for reading/writing last session; the
//! database implements it. `initialize()` is a method of the database, not of the trait.

use std::path::PathBuf;

use crate::FolderAndFile;

use super::migrations;

/// Interface for storing and restoring app state (last folder and file).
/// Implemented by the application database and by test doubles (e.g. `EmptyAppStateStore`).
/// Pass by value (e.g. `Box<dyn AppStateStore>`); no singleton, connection is opened per use.
pub trait AppStateStore: Send + Sync {
    /// Returns the last opened folder and file in it, if any.
    fn get_last_session(&self) -> Option<FolderAndFile>;

    /// Sets the last opened folder and file in it (inserts or updates).
    fn set_last_folder_and_file(&self, value: &FolderAndFile);
}

// ---------------------------------------------------------------------------
// Empty store (e.g. for tests)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Application database (SQLite)
// ---------------------------------------------------------------------------

use rusqlite::Connection;

/// The application database. Holds app state (last folder/file), and will hold user data
/// and other application storage. SQLite-backed. Call `initialize()` once at startup
/// (e.g. next to GStreamer init); then use as an `AppStateStore` or for future storage.
#[derive(Clone, Debug)]
pub struct AppDatabase {
    path: PathBuf,
}

impl AppDatabase {
    /// Creates the database using the default path (next to the executable, or temp dir if unavailable).
    pub fn new() -> Self {
        let path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("frename.db")))
            .unwrap_or_else(|| std::env::temp_dir().join("frename.db"));
        Self { path }
    }

    /// Creates the database at the given path (for tests or custom location).
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// One-time setup (run migrations). Call at startup before using the database.
    pub fn initialize(&self) {
        let Ok(conn) = Connection::open(&self.path) else {
            return;
        };
        let _ = migrations::run(&conn);
    }
}

impl AppStateStore for AppDatabase {
    fn get_last_session(&self) -> Option<FolderAndFile> {
        let conn = Connection::open(&self.path).ok()?;
        let mut stmt = conn
            .prepare("SELECT folder_path, last_file_path FROM folder_history ORDER BY opened_at DESC LIMIT 1")
            .ok()?;
        let mut rows = stmt.query([]).ok()?;
        let row = rows.next().ok()??;
        let folder: String = row.get(0).ok()?;
        let file: String = row.get(1).ok()?;
        let file = if file.is_empty() {
            None
        } else {
            Some(PathBuf::from(file))
        };
        Some(FolderAndFile::new(folder, file))
    }

    fn set_last_folder_and_file(&self, value: &FolderAndFile) {
        let Ok(conn) = Connection::open(&self.path) else {
            return;
        };
        let folder_str = value.folder().to_string_lossy().to_string();
        let file_str = value
            .file()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let _ = conn.execute(
            "INSERT INTO folder_history (opened_at, folder_path, last_file_path) VALUES (datetime('now'), ?1, ?2)
             ON CONFLICT(folder_path) DO UPDATE SET last_file_path = excluded.last_file_path, opened_at = datetime('now')",
            rusqlite::params![folder_str, file_str],
        );
    }
}
