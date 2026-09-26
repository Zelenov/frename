//! Application database (SQLite): session, window geometry, video and app settings.
//!
//! Tags are not here — they live in each folder's own tag file (see `FolderTagStore`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use rusqlite::Connection;

use crate::ai::SummaryLanguage;
use crate::{CommentStorage, FolderAndFile, InOutStorage};

use super::migrations;
use super::traits::{AppSettings, AppStateStore, Initializable, VideoSettings, WindowGeometry};

/// Open connections, keyed by database path.
///
/// Opening a SQLite file costs ~2.5 ms on Windows (file open + journal setup), which is paid on
/// every call when a connection is short-lived. Panel-drag and window-move handlers write on every
/// mouse event, so connections are opened once per path and reused for the life of the process.
static CONNECTIONS: OnceLock<Mutex<HashMap<PathBuf, Arc<Mutex<Connection>>>>> = OnceLock::new();

/// Locks a connection, recovering the guard if another thread panicked while holding it.
/// A poisoned lock means a previous caller panicked mid-query, not that the connection is unusable.
fn lock_connection(conn: &Arc<Mutex<Connection>>) -> MutexGuard<'_, Connection> {
    conn.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Opens a connection and applies the pragmas that make repeated small writes cheap.
///
/// WAL keeps writers from rewriting the rollback journal on every commit, and `synchronous=NORMAL`
/// drops the per-commit fsync (a crash can lose the last transactions — only UI geometry and
/// volume, which are rewritten on the next interaction). Together with connection reuse this takes
/// a panel-width write from ~2.6 ms to ~0.03 ms.
fn open_tuned(path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;
    // journal_mode returns the resulting mode as a row, so it cannot go through pragma_update.
    let _: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(conn)
}

/// The application database. Holds app state (last folder/file), and will hold user data
/// and other application storage. SQLite-backed. Use `Initializable::initialize()` once at startup
/// (e.g. via `LoggingAppStateStore`); then use as an `AppStateStore` or for future storage.
#[derive(Clone, Debug)]
pub struct AppDatabase {
    path: PathBuf,
}

impl AppDatabase {
    /// Creates the database at the default path, `frename.db` in [`crate::app_data_dir`].
    #[allow(clippy::new_without_default)] // opens the database file; not a cheap default
    pub fn new() -> Self {
        Self {
            path: crate::app_data_dir().join("frename.db"),
        }
    }

    /// Creates the database at the given path (for tests or custom location).
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the shared connection for this database's path, opening it on first use.
    fn conn(&self) -> Result<Arc<Mutex<Connection>>, rusqlite::Error> {
        let cache = CONNECTIONS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut cache = cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(conn) = cache.get(&self.path) {
            return Ok(Arc::clone(conn));
        }
        let conn = Arc::new(Mutex::new(open_tuned(&self.path)?));
        cache.insert(self.path.clone(), Arc::clone(&conn));
        Ok(conn)
    }
}

impl Initializable for AppDatabase {
    fn initialize(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn()?;
        let conn = lock_connection(&conn);
        migrations::run(&conn)?;
        Ok(())
    }
}

impl AppStateStore for AppDatabase {
    fn get_last_session(&self) -> Option<FolderAndFile> {
        let conn = self.conn().ok()?;
        let conn = lock_connection(&conn);
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
        let Ok(conn) = self.conn() else { return };
        let conn = lock_connection(&conn);
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
        let conn = self.conn().ok()?;
        let conn = lock_connection(&conn);
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

    fn get_video_settings(&self) -> Option<VideoSettings> {
        let conn = self.conn().ok()?;
        let conn = lock_connection(&conn);
        conn.query_row(
            "SELECT volume FROM video_settings WHERE id = 1",
            [],
            |row| {
                Ok(VideoSettings {
                    volume: row.get::<_, f64>(0)? as f32,
                })
            },
        )
        .ok()
    }

    fn set_video_settings(&self, settings: VideoSettings) {
        if let Ok(conn) = self.conn() {
            let conn = lock_connection(&conn);
            let _ = conn.execute(
                "INSERT INTO video_settings (id, volume) VALUES (1, ?1)
                 ON CONFLICT(id) DO UPDATE SET volume = excluded.volume",
                rusqlite::params![settings.volume as f64],
            );
        }
    }

    fn get_app_settings(&self) -> Option<AppSettings> {
        let conn = self.conn().ok()?;
        let conn = lock_connection(&conn);
        conn.query_row(
            "SELECT autoplay_video, monochrome_tags, comment_storage, in_out_storage, commented_tag, commented_tag_enabled,
                    space_after_tags, summary_language
             FROM app_settings WHERE id = 1",
            [],
            |row| Ok(AppSettings {
                autoplay_video: row.get::<_, i64>(0)? != 0,
                monochrome_tags: row.get::<_, i64>(1)? != 0,
                comment_storage: CommentStorage::from_name(&row.get::<_, String>(2)?),
                in_out_storage: InOutStorage::from_name(&row.get::<_, String>(3)?),
                commented_tag: row.get::<_, String>(4)?,
                commented_tag_enabled: row.get::<_, i64>(5)? != 0,
                space_after_tags: row.get::<_, i64>(6)? != 0,
                summary_language: SummaryLanguage::from_name(&row.get::<_, String>(7)?),
            }),
        ).ok()
    }

    fn set_app_settings(&self, settings: AppSettings) {
        if let Ok(conn) = self.conn() {
            let conn = lock_connection(&conn);
            let _ = conn.execute(
                "INSERT INTO app_settings (id, autoplay_video, monochrome_tags, comment_storage, in_out_storage, commented_tag, commented_tag_enabled, space_after_tags, summary_language)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                     autoplay_video = excluded.autoplay_video,
                     monochrome_tags = excluded.monochrome_tags,
                     comment_storage = excluded.comment_storage,
                     in_out_storage = excluded.in_out_storage,
                     commented_tag = excluded.commented_tag,
                     commented_tag_enabled = excluded.commented_tag_enabled,
                     space_after_tags = excluded.space_after_tags,
                     summary_language = excluded.summary_language",
                rusqlite::params![
                    settings.autoplay_video,
                    settings.monochrome_tags,
                    settings.comment_storage.as_str(),
                    settings.in_out_storage.as_str(),
                    settings.commented_tag,
                    settings.commented_tag_enabled,
                    settings.space_after_tags,
                    settings.summary_language.as_str(),
                ],
            );
        }
    }

    fn set_window_state(&self, geometry: WindowGeometry) {
        if let Ok(conn) = self.conn() {
            let conn = lock_connection(&conn);
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
        if let Ok(conn) = self.conn() {
            let conn = lock_connection(&conn);
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
