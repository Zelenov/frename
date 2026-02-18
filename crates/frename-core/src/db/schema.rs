//! Schema for the app state DB. SQL migrations: schema and seed data.

/// Migration 1: table that tracks schema version.
pub const M1_SCHEMA_VERSION: &str = "
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);
INSERT INTO schema_version (version) SELECT 0 WHERE NOT EXISTS (SELECT 1 FROM schema_version LIMIT 1);
";

/// Migration 2: full folder history (last open folder and file).
pub const M2_FOLDER_HISTORY: &str = "
CREATE TABLE IF NOT EXISTS folder_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    opened_at TEXT NOT NULL,
    folder_path TEXT NOT NULL UNIQUE,
    last_file_path TEXT NOT NULL
);
";

/// Migration 3: stored tags table and seed data (SQL only).
pub const M3_STORED_TAGS: &str = "
CREATE TABLE IF NOT EXISTS stored_tags (
    sort_order INTEGER NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY (name),
    CONSTRAINT stored_tags_name_unique UNIQUE (name)
);
INSERT OR IGNORE INTO stored_tags (sort_order, name) VALUES
(0, 'Action'),
(1, 'Adventure'),
(2, 'Animation'),
(3, 'Architecture'),
(4, 'Art'),
(5, 'Astronomy'),
(6, 'Biography'),
(7, 'Blog'),
(8, 'Business'),
(9, 'Celebration'),
(10, 'Classic'),
(11, 'Comedy'),
(12, 'Concert'),
(13, 'Cooking'),
(14, 'Dance'),
(15, 'Design'),
(16, 'Documentary'),
(17, 'Drama'),
(18, 'Education'),
(19, 'Entertainment'),
(20, 'Environment'),
(21, 'Event'),
(22, 'Experimental'),
(23, 'Family'),
(24, 'Fantasy'),
(25, 'Fashion'),
(26, 'Finance'),
(27, 'Fitness'),
(28, 'Food'),
(29, 'Gaming'),
(30, 'Gardening'),
(31, 'Geography'),
(32, 'Health'),
(33, 'History'),
(34, 'Holiday'),
(35, 'Home Improvement'),
(36, 'Horror'),
(37, 'How-To'),
(38, 'Humor'),
(39, 'Indie'),
(40, 'Industrial'),
(41, 'Interview'),
(42, 'Journalism'),
(43, 'Kids'),
(44, 'Landscape'),
(45, 'Language'),
(46, 'Lecture'),
(47, 'Lifestyle'),
(48, 'Literature'),
(49, 'Live Stream'),
(50, 'Mathematics'),
(51, 'Medicine'),
(52, 'Military'),
(53, 'Motivation'),
(54, 'Music'),
(55, 'Mystery'),
(56, 'Mythology'),
(57, 'Nature'),
(58, 'News'),
(59, 'Outdoors'),
(60, 'Parody'),
(61, 'Performance'),
(62, 'Pets'),
(63, 'Philosophy'),
(64, 'Photography'),
(65, 'Physics'),
(66, 'Podcast'),
(67, 'Politics'),
(68, 'Portrait'),
(69, 'Presentation'),
(70, 'Psychology'),
(71, 'Puzzle'),
(72, 'Reality'),
(73, 'Religion'),
(74, 'Retro'),
(75, 'Review'),
(76, 'Romance'),
(77, 'Satire'),
(78, 'Science'),
(79, 'Science Fiction'),
(80, 'Short Film'),
(81, 'Social Media'),
(82, 'Space'),
(83, 'Sports'),
(84, 'Suspense'),
(85, 'Technology'),
(86, 'Thriller'),
(87, 'Time-Lapse'),
(88, 'Travel'),
(89, 'Tutorial'),
(90, 'Underwater'),
(91, 'Urban'),
(92, 'Vlog'),
(93, 'Weather'),
(94, 'Wedding'),
(95, 'Western'),
(96, 'Wildlife'),
(97, 'Workout'),
(98, 'Workshop'),
(99, 'Yoga');
";

/// Migration 4: add color_index to stored_tags and assign palette indices (sort_order % 16).
pub const M4_TAG_COLOR_INDEX: &str = "
ALTER TABLE stored_tags ADD COLUMN color_index INTEGER NOT NULL DEFAULT 0;
UPDATE stored_tags SET color_index = (sort_order % 16);
";

/// Migration 5: separate tag color storage. Colors keyed by tag name (not tag ID).
/// tag_color_mapping is the only source for tag colors; stored_tags keeps sort_order and name only for tag identity.
pub const M5_TAG_COLOR_MAPPING: &str = "
CREATE TABLE IF NOT EXISTS tag_color_mapping (
    tag_name TEXT NOT NULL PRIMARY KEY,
    color_index INTEGER NOT NULL
);
INSERT OR IGNORE INTO tag_color_mapping (tag_name, color_index) SELECT name, color_index FROM stored_tags;
";
