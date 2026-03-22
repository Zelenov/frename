//! Schema for the app state DB. Single combined migration + optional debug seed.

/// Bootstrap: creates schema_version if not present, initializes to 0.
/// Always run first by migrations::current_version().
pub const BOOTSTRAP: &str = "
CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);
INSERT INTO schema_version (version) SELECT 0 WHERE NOT EXISTS (SELECT 1 FROM schema_version LIMIT 1);
";

/// Migration 1: full application schema in final state (all tables).
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
    monitor_height REAL NOT NULL DEFAULT 0
);
";

/// Migration 2: add panel width columns to window_state.
pub const M2_PANEL_WIDTHS: &str = "
ALTER TABLE window_state ADD COLUMN left_panel_width REAL DEFAULT 0;
ALTER TABLE window_state ADD COLUMN folder_panel_width REAL DEFAULT 0;
";

/// Debug-only seed data: 100 tags with colors.
/// Uses INSERT OR IGNORE so re-running is safe.
/// IDs follow the pattern used by the original UUID migration.
/// Color index = 1 + (sort_order % 15).
pub const SEED_TAGS: &str = "
INSERT OR IGNORE INTO stored_tags (id, name, sort_order, starred) VALUES
('00000000-0000-0000-0000-000000000000','Action',0,0),
('00000000-0000-0000-0000-000000000001','Adventure',1,0),
('00000000-0000-0000-0000-000000000002','Animation',2,0),
('00000000-0000-0000-0000-000000000003','Architecture',3,0),
('00000000-0000-0000-0000-000000000004','Art',4,0),
('00000000-0000-0000-0000-000000000005','Astronomy',5,0),
('00000000-0000-0000-0000-000000000006','Biography',6,0),
('00000000-0000-0000-0000-000000000007','Blog',7,0),
('00000000-0000-0000-0000-000000000008','Business',8,0),
('00000000-0000-0000-0000-000000000009','Celebration',9,0),
('00000000-0000-0000-0000-00000000000a','Classic',10,0),
('00000000-0000-0000-0000-00000000000b','Comedy',11,0),
('00000000-0000-0000-0000-00000000000c','Concert',12,0),
('00000000-0000-0000-0000-00000000000d','Cooking',13,0),
('00000000-0000-0000-0000-00000000000e','Dance',14,0),
('00000000-0000-0000-0000-00000000000f','Design',15,0),
('00000000-0000-0000-0000-000000000010','Documentary',16,0),
('00000000-0000-0000-0000-000000000011','Drama',17,0),
('00000000-0000-0000-0000-000000000012','Education',18,0),
('00000000-0000-0000-0000-000000000013','Entertainment',19,0),
('00000000-0000-0000-0000-000000000014','Environment',20,0),
('00000000-0000-0000-0000-000000000015','Event',21,0),
('00000000-0000-0000-0000-000000000016','Experimental',22,0),
('00000000-0000-0000-0000-000000000017','Family',23,0),
('00000000-0000-0000-0000-000000000018','Fantasy',24,0),
('00000000-0000-0000-0000-000000000019','Fashion',25,0),
('00000000-0000-0000-0000-00000000001a','Finance',26,0),
('00000000-0000-0000-0000-00000000001b','Fitness',27,0),
('00000000-0000-0000-0000-00000000001c','Food',28,0),
('00000000-0000-0000-0000-00000000001d','Gaming',29,0),
('00000000-0000-0000-0000-00000000001e','Gardening',30,0),
('00000000-0000-0000-0000-00000000001f','Geography',31,0),
('00000000-0000-0000-0000-000000000020','Health',32,0),
('00000000-0000-0000-0000-000000000021','History',33,0),
('00000000-0000-0000-0000-000000000022','Holiday',34,0),
('00000000-0000-0000-0000-000000000023','Home Improvement',35,0),
('00000000-0000-0000-0000-000000000024','Horror',36,0),
('00000000-0000-0000-0000-000000000025','How-To',37,0),
('00000000-0000-0000-0000-000000000026','Humor',38,0),
('00000000-0000-0000-0000-000000000027','Indie',39,0),
('00000000-0000-0000-0000-000000000028','Industrial',40,0),
('00000000-0000-0000-0000-000000000029','Interview',41,0),
('00000000-0000-0000-0000-00000000002a','Journalism',42,0),
('00000000-0000-0000-0000-00000000002b','Kids',43,0),
('00000000-0000-0000-0000-00000000002c','Landscape',44,0),
('00000000-0000-0000-0000-00000000002d','Language',45,0),
('00000000-0000-0000-0000-00000000002e','Lecture',46,0),
('00000000-0000-0000-0000-00000000002f','Lifestyle',47,0),
('00000000-0000-0000-0000-000000000030','Literature',48,0),
('00000000-0000-0000-0000-000000000031','Live Stream',49,0),
('00000000-0000-0000-0000-000000000032','Mathematics',50,0),
('00000000-0000-0000-0000-000000000033','Medicine',51,0),
('00000000-0000-0000-0000-000000000034','Military',52,0),
('00000000-0000-0000-0000-000000000035','Motivation',53,0),
('00000000-0000-0000-0000-000000000036','Music',54,0),
('00000000-0000-0000-0000-000000000037','Mystery',55,0),
('00000000-0000-0000-0000-000000000038','Mythology',56,0),
('00000000-0000-0000-0000-000000000039','Nature',57,0),
('00000000-0000-0000-0000-00000000003a','News',58,0),
('00000000-0000-0000-0000-00000000003b','Outdoors',59,0),
('00000000-0000-0000-0000-00000000003c','Parody',60,0),
('00000000-0000-0000-0000-00000000003d','Performance',61,0),
('00000000-0000-0000-0000-00000000003e','Pets',62,0),
('00000000-0000-0000-0000-00000000003f','Philosophy',63,0),
('00000000-0000-0000-0000-000000000040','Photography',64,0),
('00000000-0000-0000-0000-000000000041','Physics',65,0),
('00000000-0000-0000-0000-000000000042','Podcast',66,0),
('00000000-0000-0000-0000-000000000043','Politics',67,0),
('00000000-0000-0000-0000-000000000044','Portrait',68,0),
('00000000-0000-0000-0000-000000000045','Presentation',69,0),
('00000000-0000-0000-0000-000000000046','Psychology',70,0),
('00000000-0000-0000-0000-000000000047','Puzzle',71,0),
('00000000-0000-0000-0000-000000000048','Reality',72,0),
('00000000-0000-0000-0000-000000000049','Religion',73,0),
('00000000-0000-0000-0000-00000000004a','Retro',74,0),
('00000000-0000-0000-0000-00000000004b','Review',75,0),
('00000000-0000-0000-0000-00000000004c','Romance',76,0),
('00000000-0000-0000-0000-00000000004d','Satire',77,0),
('00000000-0000-0000-0000-00000000004e','Science',78,0),
('00000000-0000-0000-0000-00000000004f','Science Fiction',79,0),
('00000000-0000-0000-0000-000000000050','Short Film',80,0),
('00000000-0000-0000-0000-000000000051','Social Media',81,0),
('00000000-0000-0000-0000-000000000052','Space',82,0),
('00000000-0000-0000-0000-000000000053','Sports',83,0),
('00000000-0000-0000-0000-000000000054','Suspense',84,0),
('00000000-0000-0000-0000-000000000055','Technology',85,0),
('00000000-0000-0000-0000-000000000056','Thriller',86,0),
('00000000-0000-0000-0000-000000000057','Time-Lapse',87,0),
('00000000-0000-0000-0000-000000000058','Travel',88,0),
('00000000-0000-0000-0000-000000000059','Tutorial',89,0),
('00000000-0000-0000-0000-00000000005a','Underwater',90,0),
('00000000-0000-0000-0000-00000000005b','Urban',91,0),
('00000000-0000-0000-0000-00000000005c','Vlog',92,0),
('00000000-0000-0000-0000-00000000005d','Weather',93,0),
('00000000-0000-0000-0000-00000000005e','Wedding',94,0),
('00000000-0000-0000-0000-00000000005f','Western',95,0),
('00000000-0000-0000-0000-000000000060','Wildlife',96,0),
('00000000-0000-0000-0000-000000000061','Workout',97,0),
('00000000-0000-0000-0000-000000000062','Workshop',98,0),
('00000000-0000-0000-0000-000000000063','Yoga',99,0);

INSERT OR IGNORE INTO tag_color_mapping (tag_name, color_index) VALUES
('Action',1),('Adventure',2),('Animation',3),('Architecture',4),('Art',5),
('Astronomy',6),('Biography',7),('Blog',8),('Business',9),('Celebration',10),
('Classic',11),('Comedy',12),('Concert',13),('Cooking',14),('Dance',15),
('Design',1),('Documentary',2),('Drama',3),('Education',4),('Entertainment',5),
('Environment',6),('Event',7),('Experimental',8),('Family',9),('Fantasy',10),
('Fashion',11),('Finance',12),('Fitness',13),('Food',14),('Gaming',15),
('Gardening',1),('Geography',2),('Health',3),('History',4),('Holiday',5),
('Home Improvement',6),('Horror',7),('How-To',8),('Humor',9),('Indie',10),
('Industrial',11),('Interview',12),('Journalism',13),('Kids',14),('Landscape',15),
('Language',1),('Lecture',2),('Lifestyle',3),('Literature',4),('Live Stream',5),
('Mathematics',6),('Medicine',7),('Military',8),('Motivation',9),('Music',10),
('Mystery',11),('Mythology',12),('Nature',13),('News',14),('Outdoors',15),
('Parody',1),('Performance',2),('Pets',3),('Philosophy',4),('Photography',5),
('Physics',6),('Podcast',7),('Politics',8),('Portrait',9),('Presentation',10),
('Psychology',11),('Puzzle',12),('Reality',13),('Religion',14),('Retro',15),
('Review',1),('Romance',2),('Satire',3),('Science',4),('Science Fiction',5),
('Short Film',6),('Social Media',7),('Space',8),('Sports',9),('Suspense',10),
('Technology',11),('Thriller',12),('Time-Lapse',13),('Travel',14),('Tutorial',15),
('Underwater',1),('Urban',2),('Vlog',3),('Weather',4),('Wedding',5),
('Western',6),('Wildlife',7),('Workout',8),('Workshop',9),('Yoga',10);
";
