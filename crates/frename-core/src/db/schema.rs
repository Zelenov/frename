//! Schema for the app state DB. Two migrations only (initial + folder history); no more until told.

/// Migration 1: table that tracks schema version.
pub const M1_SCHEMA_VERSION: &str = "
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);
INSERT INTO schema_version (version) SELECT 0 WHERE NOT EXISTS (SELECT 1 FROM schema_version LIMIT 1);
";

/// Migration 2: full folder history (last open folder and file). Preparation for version 1.0.
pub const M2_FOLDER_HISTORY: &str = "
CREATE TABLE IF NOT EXISTS folder_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    opened_at TEXT NOT NULL,
    folder_path TEXT NOT NULL UNIQUE,
    last_file_path TEXT NOT NULL
);
";
