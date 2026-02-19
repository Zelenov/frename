//! Application database (SQLite): app state storage and future user data.

use std::path::PathBuf;

use rusqlite::Connection;
use uuid::Uuid;

use crate::{FolderAndFile, StoredTag, TagColorMapping};

use super::migrations;
use super::traits::{AppStateStore, Initializable, StoredTagStore};

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

impl StoredTagStore for AppDatabase {
    fn get_stored_tags(&self) -> Result<Vec<StoredTag>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = Connection::open(&self.path)?;
        let mut stmt = conn.prepare("SELECT id, name, sort_order FROM stored_tags ORDER BY sort_order")?;
        let tags = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let value: String = row.get(1)?;
                let sort_order: i64 = row.get(2)?;
                let id = Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok(StoredTag::with_sort_order(id, value, sort_order))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    fn get_tag_color_mapping(&self) -> Result<TagColorMapping, Box<dyn std::error::Error + Send + Sync>> {
        let conn = Connection::open(&self.path)?;
        let mut stmt = conn.prepare("SELECT tag_name, color_index FROM tag_color_mapping")?;
        let entries: Vec<(String, u8)> = stmt
            .query_map([], |row| {
                let name: String = row.get(0)?;
                let idx: i32 = row.get(1)?;
                Ok((name, idx as u8))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TagColorMapping::from_entries(entries))
    }

    fn add_stored_tag(
        &mut self,
        tag: StoredTag,
        color_index: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = Connection::open(&self.path)?;
        conn.execute(
            "INSERT INTO stored_tags (id, name, sort_order) VALUES (?1, ?2, ?3)",
            rusqlite::params![tag.id().to_string(), tag.value(), tag.sort_order()],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO tag_color_mapping (tag_name, color_index) VALUES (?1, ?2)",
            rusqlite::params![tag.value(), i32::from(color_index)],
        )?;
        Ok(())
    }

    fn save_tag(
        &mut self,
        tag: StoredTag,
        color_index: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = Connection::open(&self.path)?;
        let id_str = tag.id().to_string();
        let name = tag.value();
        let color = i32::from(color_index);
        // Remove old tag_color_mapping row for this id (if any) before updating name, so we don't leave a stale name.
        conn.execute(
            "DELETE FROM tag_color_mapping WHERE tag_name IN (SELECT name FROM stored_tags WHERE id = ?1)",
            [&id_str],
        )?;
        // Add or update stored_tags: insert with sort_order, or update name and sort_order on conflict.
        let sort_order = tag.sort_order();
        conn.execute(
            "INSERT INTO stored_tags (id, name, sort_order) VALUES (?1, ?2, ?3) ON CONFLICT(id) DO UPDATE SET name = excluded.name, sort_order = excluded.sort_order",
            rusqlite::params![id_str, name, sort_order],
        )?;
        // Add or replace color mapping for the current name.
        conn.execute(
            "INSERT OR REPLACE INTO tag_color_mapping (tag_name, color_index) VALUES (?1, ?2)",
            rusqlite::params![name, color],
        )?;
        Ok(())
    }

    fn remove_stored_tag_by_id(&mut self, tag_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = Connection::open(&self.path)?;
        let name: String = conn.query_row(
            "SELECT name FROM stored_tags WHERE id = ?1",
            [tag_id.to_string()],
            |row| row.get(0),
        )?;
        conn.execute("DELETE FROM tag_color_mapping WHERE tag_name = ?1", [&name])?;
        conn.execute("DELETE FROM stored_tags WHERE id = ?1", [tag_id.to_string()])?;
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
