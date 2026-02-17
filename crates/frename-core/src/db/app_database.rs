//! Application database (SQLite): app state storage and future user data.

use std::path::PathBuf;

use rusqlite::Connection;

use crate::FolderAndFile;

use super::migrations;
use super::traits::{AppStateStore, Initializable};

/// The application database. Holds app state (last folder/file), and will hold user data
/// and other application storage. SQLite-backed. Use `Initializable::initialize()` once at startup
/// (e.g. via `LoggingAppStateStore`); then use as an `AppStateStore` or for future storage.
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
}

impl Initializable for AppDatabase {
    fn initialize(&self) -> Result<(), rusqlite::Error> {
        let conn = Connection::open(&self.path)?;
        migrations::run(&conn)?;
        Ok(())
    }
}

impl AppStateStore for AppDatabase {
    fn get_last_session(&self) -> Option<FolderAndFile> {
        let conn = Connection::open(&self.path).ok()?;
        let mut stmt = conn
            .prepare(
                "SELECT folder_path, last_file_path FROM folder_history ORDER BY opened_at DESC LIMIT 1",
            )
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
        let session = FolderAndFile::new(folder, file);
        Some(session)
    }

    fn set_last_folder_and_file(&self, value: &FolderAndFile) {
        let conn = match Connection::open(&self.path) {
            Ok(c) => c,
            Err(_) => return,
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
