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
