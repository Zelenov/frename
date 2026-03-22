//! Application database (SQLite): app state storage and future user data.

use std::path::PathBuf;

use rusqlite::Connection;
use uuid::Uuid;

use crate::{FolderAndFile, StoredTag, TagColorMapping};

use super::migrations;
use super::schema;
use super::traits::{AppStateStore, Initializable, StoredTagStore, WindowGeometry};

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

    /// Inserts the built-in tag seed data (INSERT OR IGNORE — safe to call multiple times).
    /// Call this only in debug/development mode.
    pub fn seed_debug_tags(&self) -> Result<(), rusqlite::Error> {
        let conn = Connection::open(&self.path)?;
        conn.execute_batch(schema::SEED_TAGS)
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
        let mut stmt = conn.prepare("SELECT id, name, sort_order, starred FROM stored_tags ORDER BY sort_order")?;
        let tags = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let value: String = row.get(1)?;
                let sort_order: i64 = row.get(2)?;
                let starred: i64 = row.get(3)?;
                let id = Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok(StoredTag::with_all(id, value, sort_order, starred != 0))
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
        // Add or update stored_tags: insert with sort_order and starred, or update on conflict.
        let sort_order = tag.sort_order();
        let starred = tag.starred() as i32;
        conn.execute(
            "INSERT INTO stored_tags (id, name, sort_order, starred) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(id) DO UPDATE SET name = excluded.name, sort_order = excluded.sort_order, starred = excluded.starred",
            rusqlite::params![id_str, name, sort_order, starred],
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

    fn update_tag_orders(
        &mut self,
        tag_orders: &[(Uuid, i64)],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if tag_orders.is_empty() {
            return Ok(());
        }
        let mut conn = Connection::open(&self.path)?;
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare("UPDATE stored_tags SET sort_order = ?1 WHERE id = ?2")?;
            for (id, sort_order) in tag_orders {
                stmt.execute(rusqlite::params![sort_order, id.to_string()])?;
            }
        }
        tx.commit()?;
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

    fn get_window_state(&self) -> Option<WindowGeometry> {
        let conn = Connection::open(&self.path).ok()?;
        conn.query_row(
            "SELECT x, y, width, height, is_maximized, monitor_width, monitor_height, left_panel_width, folder_panel_width FROM window_state WHERE id = 1",
            [],
            |row| Ok(WindowGeometry {
                x: row.get(0)?,
                y: row.get(1)?,
                width: row.get(2)?,
                height: row.get(3)?,
                is_maximized: row.get::<_, i64>(4)? != 0,
                monitor_width: row.get(5)?,
                monitor_height: row.get(6)?,
                left_panel_width: row.get::<_, f64>(7)? as f32,
                folder_panel_width: row.get::<_, f64>(8)? as f32,
            }),
        ).ok()
    }

    fn set_window_state(&self, geometry: WindowGeometry) {
        if let Ok(conn) = Connection::open(&self.path) {
            let _ = conn.execute(
                "INSERT INTO window_state (id, x, y, width, height, is_maximized, monitor_width, monitor_height, left_panel_width, folder_panel_width)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(id) DO UPDATE SET
                     x = excluded.x, y = excluded.y,
                     width = excluded.width, height = excluded.height,
                     is_maximized = excluded.is_maximized,
                     monitor_width = excluded.monitor_width,
                     monitor_height = excluded.monitor_height,
                     left_panel_width = excluded.left_panel_width,
                     folder_panel_width = excluded.folder_panel_width",
                rusqlite::params![
                    geometry.x, geometry.y, geometry.width, geometry.height,
                    geometry.is_maximized as i64,
                    geometry.monitor_width, geometry.monitor_height,
                    geometry.left_panel_width as f64, geometry.folder_panel_width as f64,
                ],
            );
        }
    }

}

impl AppDatabase {
    /// Updates only the panel width columns in window_state (row must already exist).
    pub fn set_panel_widths(&self, left_panel_width: f32, folder_panel_width: f32) {
        if let Ok(conn) = Connection::open(&self.path) {
            match conn.execute(
                "UPDATE window_state SET left_panel_width = ?1, folder_panel_width = ?2 WHERE id = 1",
                rusqlite::params![left_panel_width as f64, folder_panel_width as f64],
            ) {
                Ok(rows) => log::debug!("set_panel_widths: left={left_panel_width}, folder={folder_panel_width} ({rows} rows updated)"),
                Err(e) => log::warn!("set_panel_widths failed: {e}"),
            }
        }
    }
}
