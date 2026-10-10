//! Application database (SQLite): session, window geometry, video and app settings.
//!
//! Tags are not here — they live in each folder's own tag file (see `FolderTagStore`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use rusqlite::Connection;

use crate::ai::{self, SummaryLanguage};
use crate::recent_folders::{self, now_ms, RecentFolder};
use crate::{CommentStorage, CueLength, FolderAndFile, InOutStorage, MarkerStorage};

use super::migrations;
use super::traits::{
    AppSettings, AppStateStore, BatchRun, Initializable, UpdateCheckState, VideoSettings,
    WindowGeometry,
};

/// `key=value` pairs joined by `;`, for [`BatchRun::options`] in one TEXT column. Values are
/// always simple identifiers, model ids or `true`/`false`, so none of them ever holds `=` or `;`.
fn encode_options(options: &[(String, String)]) -> String {
    options
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(";")
}

/// The inverse of [`encode_options`]. A pair without `=`, from a future version's format, is
/// dropped rather than misread.
fn decode_options(text: &str) -> Vec<(String, String)> {
    text.split(';')
        .filter(|pair| !pair.is_empty())
        .filter_map(|pair| pair.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// The key a clip's playback position is kept under: its folder ([`folder_key`]) and its file
/// name, exactly.
fn clip_key(clip: &Path) -> Option<(String, String)> {
    let folder = folder_key(clip.parent()?);
    let file_name = clip.file_name()?.to_string_lossy().to_string();
    Some((folder, file_name))
}

/// The recent folders as stored, newest first.
fn read_recent_folders(conn: &Connection) -> Result<Vec<RecentFolder>, rusqlite::Error> {
    let rows: Vec<RecentFolder> = conn
        .prepare(
            "SELECT folder_path, opened_at_ms, last_file_path FROM recent_folders ORDER BY position",
        )?
        .query_map([], |row| {
            let last_file: String = row.get(2)?;
            Ok(RecentFolder {
                folder: PathBuf::from(row.get::<_, String>(0)?),
                opened_at_ms: row.get(1)?,
                last_file: (!last_file.is_empty()).then(|| PathBuf::from(last_file)),
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(recent_folders::tidy(rows))
}

/// Replace the stored recent folders with `list`, in one transaction.
fn write_recent_folders(conn: &Connection, list: &[RecentFolder]) -> Result<(), rusqlite::Error> {
    let transaction = conn.unchecked_transaction()?;
    transaction.execute("DELETE FROM recent_folders", [])?;
    for (position, entry) in list.iter().enumerate() {
        transaction.execute(
            "INSERT INTO recent_folders (position, folder_path, opened_at_ms, last_file_path)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                i64::try_from(position).unwrap_or(i64::MAX),
                entry.folder.to_string_lossy(),
                entry.opened_at_ms,
                entry
                    .last_file
                    .as_deref()
                    .map(|file| file.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ],
        )?;
    }
    transaction.commit()
}

/// A folder as playback positions are kept under it (see [`recent_folders::folder_key`]).
fn folder_key(folder: &Path) -> String {
    recent_folders::folder_key(folder)
}

/// Move the position kept under `from` to `to`, replacing what `to` had.
fn move_position(
    conn: &Connection,
    from: &(String, String),
    to: &(String, String),
) -> Result<(), rusqlite::Error> {
    let moved: Option<i64> = conn
        .query_row(
            "SELECT position_ms FROM playback_position WHERE folder = ?1 AND file_name = ?2",
            rusqlite::params![from.0, from.1],
            |row| row.get(0),
        )
        .ok();
    if moved.is_none() {
        return Ok(());
    }
    conn.execute(
        "DELETE FROM playback_position WHERE folder = ?1 AND file_name = ?2",
        rusqlite::params![to.0, to.1],
    )?;
    conn.execute(
        "UPDATE playback_position SET folder = ?3, file_name = ?4
         WHERE folder = ?1 AND file_name = ?2",
        rusqlite::params![from.0, from.1, to.0, to.1],
    )?;
    Ok(())
}

/// Apply [`crate::playback::tidy`] to `folder`'s positions, then drop the oldest beyond
/// [`crate::playback::MOST_KEPT`].
fn tidy_positions(
    conn: &Connection,
    folder: &str,
    names: &[String],
) -> Result<(), rusqlite::Error> {
    let remembered: Vec<String> = conn
        .prepare("SELECT file_name FROM playback_position WHERE folder = ?1")?
        .query_map([folder], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    for (name, change) in crate::playback::tidy(&remembered, names, cfg!(windows)) {
        match change {
            crate::playback::Tidy::Follow(renamed) => move_position(
                conn,
                &(folder.to_string(), name),
                &(folder.to_string(), renamed),
            )?,
            crate::playback::Tidy::Forget => {
                conn.execute(
                    "DELETE FROM playback_position WHERE folder = ?1 AND file_name = ?2",
                    rusqlite::params![folder, name],
                )?;
            }
        }
    }
    conn.execute(
        "DELETE FROM playback_position WHERE rowid NOT IN
             (SELECT rowid FROM playback_position ORDER BY saved_at_ms DESC LIMIT ?1)",
        [i64::try_from(crate::playback::MOST_KEPT).unwrap_or(i64::MAX)],
    )?;
    Ok(())
}

/// Open connections, keyed by database path.
///
/// Opening a SQLite file costs ~2.5 ms on Windows (file open + journal setup), which is paid on
/// every call when a connection is short-lived. Panel-drag and window-move handlers write on every
/// mouse event, so connections are opened once per path and reused for the life of the process.
static CONNECTIONS: OnceLock<Mutex<HashMap<PathBuf, Arc<Mutex<Connection>>>>> = OnceLock::new();

/// Locks a connection, recovering the guard if another thread panicked while holding it.
/// A poisoned lock means a previous caller panicked mid-query, not that the connection is unusable.
pub(super) fn lock_connection(conn: &Arc<Mutex<Connection>>) -> MutexGuard<'_, Connection> {
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

    /// Closes this database's cached connection, so SQLite folds its write-ahead log back in
    /// and the files can be deleted. A later call opens it again.
    pub fn close(&self) {
        let Some(cache) = CONNECTIONS.get() else {
            return;
        };
        cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&self.path);
    }

    /// Returns the shared connection for this database's path, opening it on first use.
    pub(super) fn conn(&self) -> Result<Arc<Mutex<Connection>>, rusqlite::Error> {
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
        drop(conn);
        self.record_recent_folder(value, now_ms());
    }

    fn get_recent_folders(&self) -> Vec<RecentFolder> {
        let Ok(conn) = self.conn() else {
            return Vec::new();
        };
        let conn = lock_connection(&conn);
        read_recent_folders(&conn).unwrap_or_default()
    }

    fn record_recent_folder(&self, value: &FolderAndFile, opened_at_ms: i64) {
        let Ok(conn) = self.conn() else { return };
        let conn = lock_connection(&conn);
        let Ok(mut list) = read_recent_folders(&conn) else {
            return;
        };
        recent_folders::record(&mut list, value, opened_at_ms);
        let _ = write_recent_folders(&conn, &list);
    }

    fn forget_recent_folder(&self, folder: &Path) {
        let Ok(conn) = self.conn() else { return };
        let conn = lock_connection(&conn);
        let Ok(mut list) = read_recent_folders(&conn) else {
            return;
        };
        recent_folders::forget(&mut list, folder);
        let _ = write_recent_folders(&conn, &list);
    }

    fn clear_recent_folders(&self) {
        let Ok(conn) = self.conn() else { return };
        let conn = lock_connection(&conn);
        let _ = conn.execute("DELETE FROM recent_folders", []);
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
                    space_after_tags, summary_language, ai_model, subtitle_languages, subtitle_cue_length,
                    marker_storage, ui_language, ai_moments, ai_tag_suggestions
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
                ai_model: row.get::<_, String>(8)?,
                subtitle_languages: row
                    .get::<_, String>(9)?
                    .split(',')
                    .filter(|code| !code.is_empty())
                    .map(str::to_string)
                    .collect(),
                subtitle_cue_length: CueLength::from_name(&row.get::<_, String>(10)?),
                marker_storage: MarkerStorage::from_name(&row.get::<_, String>(11)?),
                ui_language: row.get::<_, String>(12)?,
                ai_moments: ai::moments_from_name(&row.get::<_, String>(13)?),
                ai_tag_suggestions: row.get::<_, i64>(14)? != 0,
            }),
        ).ok()
    }

    fn set_app_settings(&self, settings: AppSettings) {
        if let Ok(conn) = self.conn() {
            let conn = lock_connection(&conn);
            let _ = conn.execute(
                "INSERT INTO app_settings (id, autoplay_video, monochrome_tags, comment_storage, in_out_storage, commented_tag, commented_tag_enabled, space_after_tags, summary_language, ai_model,
                                           subtitle_languages, subtitle_cue_length, marker_storage, ui_language, ai_moments, ai_tag_suggestions)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                 ON CONFLICT(id) DO UPDATE SET
                     autoplay_video = excluded.autoplay_video,
                     monochrome_tags = excluded.monochrome_tags,
                     comment_storage = excluded.comment_storage,
                     in_out_storage = excluded.in_out_storage,
                     commented_tag = excluded.commented_tag,
                     commented_tag_enabled = excluded.commented_tag_enabled,
                     space_after_tags = excluded.space_after_tags,
                     summary_language = excluded.summary_language,
                     ai_model = excluded.ai_model,
                     subtitle_languages = excluded.subtitle_languages,
                     subtitle_cue_length = excluded.subtitle_cue_length,
                     marker_storage = excluded.marker_storage,
                     ui_language = excluded.ui_language,
                     ai_moments = excluded.ai_moments,
                     ai_tag_suggestions = excluded.ai_tag_suggestions",
                rusqlite::params![
                    settings.autoplay_video,
                    settings.monochrome_tags,
                    settings.comment_storage.as_str(),
                    settings.in_out_storage.as_str(),
                    settings.commented_tag,
                    settings.commented_tag_enabled,
                    settings.space_after_tags,
                    settings.summary_language.as_str(),
                    settings.ai_model,
                    settings.subtitle_languages.join(","),
                    settings.subtitle_cue_length.as_str(),
                    settings.marker_storage.as_str(),
                    settings.ui_language,
                    ai::moments_as_str(settings.ai_moments),
                    settings.ai_tag_suggestions,
                ],
            );
        }
    }

    fn get_update_check(&self) -> Option<UpdateCheckState> {
        let conn = self.conn().ok()?;
        let conn = lock_connection(&conn);
        conn.query_row(
            "SELECT check_on_start, last_check, newest_version FROM update_check WHERE id = 1",
            [],
            |row| {
                Ok(UpdateCheckState {
                    check_on_start: row.get::<_, i64>(0)? != 0,
                    last_check: u64::try_from(row.get::<_, i64>(1)?).unwrap_or(0),
                    newest_version: row.get(2)?,
                })
            },
        )
        .ok()
    }

    fn set_update_check(&self, state: UpdateCheckState) {
        if let Ok(conn) = self.conn() {
            let conn = lock_connection(&conn);
            let _ = conn.execute(
                "INSERT INTO update_check (id, check_on_start, last_check, newest_version)
                 VALUES (1, ?1, ?2, ?3)
                 ON CONFLICT(id) DO UPDATE SET
                     check_on_start = excluded.check_on_start,
                     last_check = excluded.last_check,
                     newest_version = excluded.newest_version",
                rusqlite::params![
                    state.check_on_start,
                    i64::try_from(state.last_check).unwrap_or(i64::MAX),
                    state.newest_version,
                ],
            );
        }
    }

    fn get_batch_run(&self) -> Option<BatchRun> {
        let conn = self.conn().ok()?;
        let conn = lock_connection(&conn);
        conn.query_row(
            "SELECT action, options FROM batch_run WHERE id = 1",
            [],
            |row| {
                Ok(BatchRun {
                    action: row.get(0)?,
                    options: decode_options(&row.get::<_, String>(1)?),
                })
            },
        )
        .ok()
    }

    fn set_batch_run(&self, run: BatchRun) {
        if let Ok(conn) = self.conn() {
            let conn = lock_connection(&conn);
            let _ = conn.execute(
                "INSERT INTO batch_run (id, action, options) VALUES (1, ?1, ?2)
                 ON CONFLICT(id) DO UPDATE SET action = excluded.action, options = excluded.options",
                rusqlite::params![run.action, encode_options(&run.options)],
            );
        }
    }

    fn get_playback_position(&self, clip: &Path) -> Option<Duration> {
        let (folder, file_name) = clip_key(clip)?;
        let conn = self.conn().ok()?;
        let conn = lock_connection(&conn);
        conn.query_row(
            "SELECT position_ms FROM playback_position WHERE folder = ?1 AND file_name = ?2",
            rusqlite::params![folder, file_name],
            |row| row.get::<_, i64>(0),
        )
        .ok()
        .map(|ms| Duration::from_millis(u64::try_from(ms).unwrap_or(0)))
    }

    fn set_playback_position(&self, clip: &Path, position: Duration) {
        let Some((folder, file_name)) = clip_key(clip) else {
            return;
        };
        let Ok(conn) = self.conn() else { return };
        let conn = lock_connection(&conn);
        let result = if crate::playback::worth_remembering(position) {
            conn.execute(
                "INSERT INTO playback_position (folder, file_name, position_ms, saved_at_ms)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(folder, file_name) DO UPDATE SET
                     position_ms = excluded.position_ms,
                     saved_at_ms = excluded.saved_at_ms",
                rusqlite::params![
                    folder,
                    file_name,
                    i64::try_from(position.as_millis()).unwrap_or(i64::MAX),
                    now_ms(),
                ],
            )
        } else {
            conn.execute(
                "DELETE FROM playback_position WHERE folder = ?1 AND file_name = ?2",
                rusqlite::params![folder, file_name],
            )
        };
        if let Err(e) = result {
            log::warn!("set_playback_position failed: {e}");
        }
    }

    fn move_playback_position(&self, from: &Path, to: &Path) {
        let (Some(from), Some(to)) = (clip_key(from), clip_key(to)) else {
            return;
        };
        if from == to {
            return;
        }
        let Ok(conn) = self.conn() else { return };
        let conn = lock_connection(&conn);
        if let Err(e) = move_position(&conn, &from, &to) {
            log::warn!("move_playback_position failed: {e}");
        }
    }

    fn tidy_playback_positions(&self, folder: &Path, names: &[String]) {
        let folder = folder_key(folder);
        let Ok(conn) = self.conn() else { return };
        let conn = lock_connection(&conn);
        if let Err(e) = tidy_positions(&conn, &folder, names) {
            log::warn!("tidy_playback_positions failed: {e}");
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, migrated database in the temp folder, named for the test.
    fn temp_database(name: &str) -> AppDatabase {
        let path = std::env::temp_dir().join(format!("frename-{name}-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = AppDatabase::with_path(&path);
        db.initialize().expect("migrate");
        db
    }

    fn recent_names(db: &AppDatabase) -> Vec<String> {
        db.get_recent_folders().iter().map(|e| e.name()).collect()
    }

    #[test]
    fn opening_folders_lists_them_newest_first_with_their_last_file() {
        let db = temp_database("recent-order");
        assert!(db.get_recent_folders().is_empty());
        db.set_last_folder_and_file(&FolderAndFile::new("/shoots/a", Some("/shoots/a/x.mp4")));
        db.set_last_folder_and_file(&FolderAndFile::new("/shoots/b", None::<&str>));
        // The folder is opened again, then a file is picked in it, as a session goes.
        db.set_last_folder_and_file(&FolderAndFile::new("/shoots/a", None::<&str>));
        assert_eq!(recent_names(&db), ["a", "b"]);
        assert_eq!(
            db.get_recent_folders()[0].last_file.as_deref(),
            Some(Path::new("/shoots/a/x.mp4")),
            "opening the folder keeps the file it had"
        );
        db.set_last_folder_and_file(&FolderAndFile::new("/shoots/a", Some("/shoots/a/y.mp4")));
        assert_eq!(
            db.get_recent_folders()[0].last_file.as_deref(),
            Some(Path::new("/shoots/a/y.mp4"))
        );
    }

    #[test]
    fn the_recent_folders_keep_ten_and_survive_a_restart() {
        let db = temp_database("recent-ten");
        for n in 0..12 {
            db.record_recent_folder(&FolderAndFile::new(format!("/s/{n}"), None::<&str>), n);
        }
        let reopened = AppDatabase::with_path(&db.path);
        let names = recent_names(&reopened);
        assert_eq!(names.len(), crate::recent_folders::MAX_RECENT_FOLDERS);
        assert_eq!((names[0].as_str(), names[9].as_str()), ("11", "2"));
        assert_eq!(reopened.get_recent_folders()[0].opened_at_ms, 11);
    }

    #[test]
    fn one_folder_can_be_forgotten_and_the_list_cleared() {
        let db = temp_database("recent-forget");
        for name in ["a", "b", "c"] {
            db.set_last_folder_and_file(&FolderAndFile::new(
                format!("/shoots/{name}"),
                None::<&str>,
            ));
        }
        db.forget_recent_folder(Path::new("/shoots/b"));
        assert_eq!(recent_names(&db), ["c", "a"]);
        db.clear_recent_folders();
        assert!(db.get_recent_folders().is_empty());
        db.set_last_folder_and_file(&FolderAndFile::new("/shoots/d", None::<&str>));
        assert_eq!(recent_names(&db), ["d"]);
    }

    #[test]
    fn the_last_session_is_still_the_last_folder_opened() {
        let db = temp_database("recent-session");
        db.set_last_folder_and_file(&FolderAndFile::new("/shoots/a", Some("/shoots/a/x.mp4")));
        db.forget_recent_folder(Path::new("/shoots/a"));
        assert_eq!(
            db.get_last_session().map(|s| s.folder),
            Some(PathBuf::from("/shoots/a")),
            "taking a folder off the list does not forget the session"
        );
    }

    #[test]
    fn a_closed_database_leaves_no_files_behind() {
        let folder = std::env::temp_dir().join(format!("frename-db-close-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).expect("create folder");
        let db = AppDatabase::with_path(folder.join("app.db"));
        db.initialize().expect("migrate");
        db.set_playback_position(Path::new("C:/clips/a.mp4"), Duration::from_secs(40));

        db.close();

        std::fs::remove_dir_all(&folder).expect("nothing holds the files open");
        assert!(!folder.exists());
    }

    #[test]
    fn the_update_check_state_round_trips() {
        let path =
            std::env::temp_dir().join(format!("frename-update-check-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = AppDatabase::with_path(&path);
        db.initialize().expect("migrate");
        assert_eq!(db.get_update_check(), None, "never saved");

        let state = UpdateCheckState {
            check_on_start: false,
            last_check: 1_790_000_000,
            newest_version: "0.68.0".to_string(),
        };
        db.set_update_check(state.clone());
        assert_eq!(db.get_update_check(), Some(state));
    }

    #[test]
    fn app_settings_round_trip_with_the_subtitle_settings() {
        let path = std::env::temp_dir().join(format!(
            "frename-subtitle-settings-{}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let db = AppDatabase::with_path(&path);
        db.initialize().expect("migrate");

        let mut settings = AppSettings::default();
        assert_eq!(settings.subtitle_languages, ["en", "ru"]);
        settings.subtitle_languages = vec!["de".into(), "fr".into()];
        settings.subtitle_cue_length = CueLength::Sentence;
        db.set_app_settings(settings.clone());
        assert_eq!(db.get_app_settings(), Some(settings.clone()));

        // No language checked: detect automatically.
        settings.subtitle_languages.clear();
        db.set_app_settings(settings.clone());
        assert_eq!(db.get_app_settings(), Some(settings));
    }

    #[test]
    fn app_settings_round_trip_with_the_ui_language() {
        let path =
            std::env::temp_dir().join(format!("frename-ui-language-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = AppDatabase::with_path(&path);
        db.initialize().expect("migrate");

        let mut settings = AppSettings::default();
        assert_eq!(settings.ui_language, "", "System by default");
        settings.ui_language = "ru".to_string();
        db.set_app_settings(settings.clone());
        assert_eq!(db.get_app_settings(), Some(settings));
    }

    #[test]
    fn app_settings_round_trip_with_the_ai_moments() {
        let db = database("ai-moments");
        let mut settings = AppSettings::default();
        assert_eq!(
            settings.ai_moments,
            ai::MomentsMode::Important,
            "only what stands out by default"
        );
        settings.ai_moments = ai::MomentsMode::Full;
        db.set_app_settings(settings.clone());
        assert_eq!(db.get_app_settings(), Some(settings));
    }

    #[test]
    fn tag_suggestions_are_on_by_default_and_can_be_turned_off() {
        let db = database("ai-tag-suggestions");
        let mut settings = AppSettings::default();
        assert!(settings.ai_tag_suggestions);
        settings.ai_tag_suggestions = false;
        db.set_app_settings(settings.clone());
        assert_eq!(db.get_app_settings(), Some(settings));
    }

    /// A migrated database of its own, for one test.
    fn database(name: &str) -> AppDatabase {
        let path = std::env::temp_dir().join(format!("frename-{name}-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = AppDatabase::with_path(&path);
        db.initialize().expect("migrate");
        db
    }

    fn secs(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }

    #[test]
    fn a_playback_position_round_trips_and_survives_a_restart() {
        let db = database("playback-round-trip");
        let clip = Path::new("/footage/day1/MVI_0410.mp4");
        assert_eq!(db.get_playback_position(clip), None, "never saved");
        db.set_playback_position(clip, Duration::from_millis(40_250));
        assert_eq!(
            db.get_playback_position(clip),
            Some(Duration::from_millis(40_250))
        );
        // A later save replaces it.
        db.set_playback_position(clip, secs(75));
        assert_eq!(db.get_playback_position(clip), Some(secs(75)));
        // Another clip of the same name elsewhere is another clip.
        assert_eq!(
            db.get_playback_position(Path::new("/footage/day2/MVI_0410.mp4")),
            None
        );
        // What a new start of frename reads.
        let reopened = AppDatabase::with_path(&db.path);
        reopened.initialize().expect("migrate again");
        assert_eq!(reopened.get_playback_position(clip), Some(secs(75)));
    }

    #[test]
    fn a_position_near_the_start_forgets_the_clip() {
        let db = database("playback-near-start");
        let clip = Path::new("/footage/MVI_0410.mp4");
        db.set_playback_position(clip, secs(40));
        db.set_playback_position(clip, secs(1));
        assert_eq!(db.get_playback_position(clip), None);
    }

    #[test]
    fn a_rename_carries_the_position_along() {
        let db = database("playback-rename");
        let before = Path::new("/footage/MVI_0410.mp4");
        let after = Path::new("/footage/pick.MVI_0410.mp4");
        db.set_playback_position(before, secs(40));
        // A stale position under the new name is replaced, not kept.
        db.set_playback_position(after, secs(90));
        db.move_playback_position(before, after);
        assert_eq!(db.get_playback_position(before), None);
        assert_eq!(db.get_playback_position(after), Some(secs(40)));
        // Moving a clip with no position leaves the other one alone.
        db.move_playback_position(Path::new("/footage/other.mp4"), after);
        assert_eq!(db.get_playback_position(after), Some(secs(40)));
    }

    #[test]
    fn a_folder_spelled_another_way_is_the_same_folder() {
        let db = database("playback-folder-key");
        db.set_playback_position(Path::new("/footage/day1/clip.mp4"), secs(40));
        assert_eq!(
            db.get_playback_position(Path::new("/footage/day1//clip.mp4")),
            Some(secs(40))
        );
        if cfg!(windows) {
            db.set_playback_position(Path::new(r"C:\Footage\Day1\clip.mp4"), secs(50));
            assert_eq!(
                db.get_playback_position(Path::new("c:/footage/day1/clip.mp4")),
                Some(secs(50))
            );
            assert_eq!(
                db.get_playback_position(Path::new(r"C:\Footage\Day1\CLIP.mp4")),
                None,
                "the file name is kept exactly"
            );
        }
    }

    #[test]
    fn listing_a_folder_forgets_gone_files_and_follows_outside_renames() {
        let db = database("playback-tidy");
        let folder = Path::new("/footage");
        db.set_playback_position(&folder.join("kept.mp4"), secs(10));
        db.set_playback_position(&folder.join("gone.mp4"), secs(20));
        db.set_playback_position(&folder.join("MVI_0410.mp4"), secs(30));
        let elsewhere = Path::new("/other/gone.mp4");
        db.set_playback_position(elsewhere, secs(40));

        let listed = ["kept.mp4", "skip.MVI_0410.mp4", "new.mp4"].map(str::to_string);
        db.tidy_playback_positions(folder, &listed);

        assert_eq!(
            db.get_playback_position(&folder.join("kept.mp4")),
            Some(secs(10))
        );
        assert_eq!(db.get_playback_position(&folder.join("gone.mp4")), None);
        assert_eq!(
            db.get_playback_position(&folder.join("skip.MVI_0410.mp4")),
            Some(secs(30))
        );
        assert_eq!(
            db.get_playback_position(elsewhere),
            Some(secs(40)),
            "another folder is left alone"
        );
    }

    #[test]
    fn decode_options_drops_a_pair_without_an_equals_sign() {
        assert_eq!(
            decode_options("a=1;bogus;b=2"),
            vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string())
            ]
        );
        assert_eq!(decode_options(""), Vec::<(String, String)>::new());
    }

    #[test]
    fn the_batch_run_round_trips_with_its_options() {
        let path =
            std::env::temp_dir().join(format!("frename-batch-run-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = AppDatabase::with_path(&path);
        db.initialize().expect("migrate");
        assert_eq!(db.get_batch_run(), None, "never saved");

        let run = BatchRun {
            action: "describe_ai".to_string(),
            options: vec![
                ("language".to_string(), "ru".to_string()),
                ("redo".to_string(), "false".to_string()),
            ],
        };
        db.set_batch_run(run.clone());
        assert_eq!(db.get_batch_run(), Some(run));

        // A later save replaces the row instead of adding another.
        let run = BatchRun {
            action: "rotate".to_string(),
            options: vec![("turn".to_string(), "left".to_string())],
        };
        db.set_batch_run(run.clone());
        assert_eq!(db.get_batch_run(), Some(run));
    }

    #[test]
    fn a_batch_run_with_no_options_round_trips_too() {
        let path = std::env::temp_dir().join(format!(
            "frename-batch-run-no-options-{}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let db = AppDatabase::with_path(&path);
        db.initialize().expect("migrate");

        let run = BatchRun {
            action: "tag_commented".to_string(),
            options: Vec::new(),
        };
        db.set_batch_run(run.clone());
        assert_eq!(db.get_batch_run(), Some(run));
    }
}
