//! Schema for the app state DB. Single combined migration + optional debug seed.

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

/// Demo seed data: practical tags for video review workflows.
/// Uses INSERT OR IGNORE so re-running is safe.
pub const SEED_TAGS: &str = "
INSERT OR IGNORE INTO stored_tags (id, name, sort_order, starred) VALUES
-- Picks
('00000000-0000-0000-0000-000000000000','pick',0,1),
('00000000-0000-0000-0000-000000000001','skip',1,1),
('00000000-0000-0000-0000-000000000002','review',2,1),
-- Shot type
('00000000-0000-0000-0000-000000000003','wide',3,0),
('00000000-0000-0000-0000-000000000004','medium',4,0),
('00000000-0000-0000-0000-000000000005','close',5,0),
-- Camera movement
('00000000-0000-0000-0000-000000000006','handheld',6,0),
('00000000-0000-0000-0000-000000000007','gimbal',7,0),
('00000000-0000-0000-0000-000000000008','tripod',8,0),
('00000000-0000-0000-0000-000000000009','drone',9,0),
-- Lighting
('00000000-0000-0000-0000-00000000000a','golden-hour',10,0),
('00000000-0000-0000-0000-00000000000b','sunrise',11,0),
('00000000-0000-0000-0000-00000000000c','sunset',12,0),
('00000000-0000-0000-0000-00000000000d','night',13,0),
('00000000-0000-0000-0000-00000000000e','overcast',14,0),
-- Subject
('00000000-0000-0000-0000-00000000000f','people',15,0),
('00000000-0000-0000-0000-000000000010','wildlife',16,0),
('00000000-0000-0000-0000-000000000011','landscape',17,0),
('00000000-0000-0000-0000-000000000012','city',18,0),
('00000000-0000-0000-0000-000000000013','action',19,0),
-- Usage
('00000000-0000-0000-0000-000000000014','b-roll',20,0),
('00000000-0000-0000-0000-000000000015','interview',21,0),
('00000000-0000-0000-0000-000000000016','timelapse',22,0);

INSERT OR IGNORE INTO tag_color_mapping (tag_name, color_index) VALUES
('pick',9),('skip',7),('review',10),
('wide',3),('medium',3),('close',3),
('handheld',5),('gimbal',5),('tripod',5),('drone',6),
('golden-hour',12),('sunrise',12),('sunset',11),('night',8),('overcast',4),
('people',2),('wildlife',13),('landscape',14),('city',1),('action',15),
('b-roll',4),('interview',2),('timelapse',6);
";
