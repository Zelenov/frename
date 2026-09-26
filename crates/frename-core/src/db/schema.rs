//! Schema for the app state DB. Migrations are append-only: M1 is kept as it shipped, and
//! later migrations correct it, so an old database and a fresh one end up in the same shape.

/// Bootstrap: creates schema_version if not present, initializes to 0.
/// Always run first by migrations::current_version().
pub const BOOTSTRAP: &str = "
CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);
INSERT INTO schema_version (version) SELECT 0 WHERE NOT EXISTS (SELECT 1 FROM schema_version LIMIT 1);
";

/// Migration 1: full application schema (all tables, all columns).
pub const M1_FULL_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS folder_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    opened_at TEXT NOT NULL,
    folder_path TEXT NOT NULL UNIQUE,
    last_file_path TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS stored_tags (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    starred INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT stored_tags_name_unique UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS tag_color_mapping (
    tag_name TEXT NOT NULL PRIMARY KEY,
    color_index INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS window_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,
    is_maximized INTEGER NOT NULL DEFAULT 0,
    monitor_width REAL NOT NULL DEFAULT 0,
    monitor_height REAL NOT NULL DEFAULT 0,
    left_panel_width REAL DEFAULT 0,
    folder_panel_width REAL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS video_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    volume REAL NOT NULL DEFAULT 1.0
);
";

/// Migration 2: drop the tag tables. Tags live in each folder's own `.frename` file now, so
/// these have no reader left; the rows are not migrated, by design.
pub const M2_DROP_TAG_TABLES: &str = "
DROP TABLE IF EXISTS stored_tags;
DROP TABLE IF EXISTS tag_color_mapping;
";

/// Migration 3: user-facing app settings, edited in the settings window. One row, like
/// `video_settings`; a missing row means every setting is at its default.
pub const M3_APP_SETTINGS: &str = "
CREATE TABLE IF NOT EXISTS app_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    autoplay_video INTEGER NOT NULL DEFAULT 1,
    monochrome_tags INTEGER NOT NULL DEFAULT 0
);
";

/// Migration 4: where file comments are saved (`CommentStorage::as_str`). Existing users move
/// to comments inside the video too; their `.comment.txt` files are still read and move into
/// the video on save.
pub const M4_COMMENT_STORAGE: &str = "
ALTER TABLE app_settings ADD COLUMN comment_storage TEXT NOT NULL DEFAULT 'xmp';
";

/// Migration 5: where in/out points are saved (`InOutStorage::as_str`). Defaults to the file
/// name, where they always were.
pub const M5_IN_OUT_STORAGE: &str = "
ALTER TABLE app_settings ADD COLUMN in_out_storage TEXT NOT NULL DEFAULT 'file_name';
";

/// Migration 6: the tag added to commented videos while comments are stored inside them
/// (empty: off).
pub const M6_COMMENTED_TAG: &str = "
ALTER TABLE app_settings ADD COLUMN commented_tag TEXT NOT NULL DEFAULT 'Commented';
";

/// Migration 7: whether commented videos get the commented tag at all. On by default, as it
/// was; turning it off keeps the tag name for when it is turned back on.
pub const M7_COMMENTED_TAG_ENABLED: &str = "
ALTER TABLE app_settings ADD COLUMN commented_tag_enabled INTEGER NOT NULL DEFAULT 1;
";

/// Migration 8: whether file names put a space after each tag (`Food. clip.mp4`). Off.
pub const M8_SPACE_AFTER_TAGS: &str = "
ALTER TABLE app_settings ADD COLUMN space_after_tags INTEGER NOT NULL DEFAULT 0;
";

/// Migration 9: generating subtitles. The languages spoken in the footage, as comma-separated
/// hints (empty: detect automatically), and how long a cue may get (`CueLength::as_str`).
pub const M9_SUBTITLE_SETTINGS: &str = "
ALTER TABLE app_settings ADD COLUMN subtitle_languages TEXT NOT NULL DEFAULT 'en,ru';
ALTER TABLE app_settings ADD COLUMN subtitle_cue_length TEXT NOT NULL DEFAULT 'short';
";
