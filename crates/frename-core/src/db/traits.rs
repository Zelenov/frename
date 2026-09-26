//! Traits for app state storage: session read/write, stored tags, tag color mapping, and one-time initialization.

use uuid::Uuid;

use crate::ai::SummaryLanguage;
use crate::{CommentStorage, FolderAndFile, InOutStorage, StoredTag, TagColorMapping};

/// Saved window position and size (logical pixels).
#[derive(Debug, Clone, Copy)]
pub struct WindowGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Whether the window was maximized when last saved.
    pub is_maximized: bool,
    /// Width of the monitor the window was on (logical pixels); 0.0 if unknown.
    pub monitor_width: f32,
    /// Height of the monitor the window was on (logical pixels); 0.0 if unknown.
    pub monitor_height: f32,
    /// Width of the left (media) panel; 0.0 means use default.
    pub left_panel_width: f32,
    /// Width of the folder list (middle) panel; 0.0 means use default.
    pub folder_panel_width: f32,
}

/// Saved video player settings.
#[derive(Debug, Clone, Copy)]
pub struct VideoSettings {
    /// Volume level 0.0..=1.0. Defaults to 1.0.
    pub volume: f32,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self { volume: 1.0 }
    }
}

/// User-facing app settings, edited in the settings window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    /// Start playing a video as soon as it is opened. Defaults to true.
    pub autoplay_video: bool,
    /// Draw every tag in one neutral color instead of its own palette color. Defaults to false.
    pub monochrome_tags: bool,
    /// Where file comments are saved. Defaults to inside the video file.
    pub comment_storage: CommentStorage,
    /// Where in/out points are saved. Defaults to the file name.
    pub in_out_storage: InOutStorage,
    /// Tag checked on videos that get a comment while comments are stored inside them; empty
    /// turns it off. Defaults to [`crate::DEFAULT_COMMENTED_TAG`].
    pub commented_tag: String,
    /// Whether commented videos get [`Self::commented_tag`] at all. Defaults to true.
    pub commented_tag_enabled: bool,
    /// Whether file names put a space after each tag (`Food. clip.mp4`). Defaults to false.
    pub space_after_tags: bool,
    /// The language AI descriptions are written in. Defaults to the subtitles' language.
    pub summary_language: SummaryLanguage,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autoplay_video: true,
            monochrome_tags: false,
            comment_storage: CommentStorage::default(),
            in_out_storage: InOutStorage::default(),
            commented_tag: crate::DEFAULT_COMMENTED_TAG.to_string(),
            commented_tag_enabled: true,
            space_after_tags: false,
            summary_language: SummaryLanguage::default(),
        }
    }
}

impl AppSettings {
    /// The commented tag as core should use it (see [`crate::set_commented_tag`]): empty while
    /// turned off.
    pub fn effective_commented_tag(&self) -> &str {
        if self.commented_tag_enabled {
            &self.commented_tag
        } else {
            ""
        }
    }
}

/// The update check's saved state (Settings -> Updates).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCheckState {
    /// Check for updates in the background when frename starts. Defaults to true.
    pub check_on_start: bool,
    /// When the last successful check ran, in seconds since the Unix epoch; 0 for never.
    pub last_check: u64,
    /// The newest version the last check found (`0.68.0`); empty when none was found.
    pub newest_version: String,
}

impl Default for UpdateCheckState {
    fn default() -> Self {
        Self {
            check_on_start: true,
            last_check: 0,
            newest_version: String::new(),
        }
    }
}

/// Interface for storing and restoring app state (last folder and file, window geometry).
/// Implemented by the application database and by the test fake (e.g. `FakeAppStorage`).
/// Pass by value (e.g. `Box<dyn AppStateStore>`); no singleton, connection is opened per use.
pub trait AppStateStore: Send + Sync {
    /// Returns the last opened folder and file in it, if any.
    fn get_last_session(&self) -> Option<FolderAndFile>;

    /// Sets the last opened folder and file in it (inserts or updates).
    fn set_last_folder_and_file(&self, value: &FolderAndFile);

    /// Returns the saved window geometry, if any.
    fn get_window_state(&self) -> Option<WindowGeometry> {
        None
    }

    /// Saves the window geometry (position + size).
    fn set_window_state(&self, _geometry: WindowGeometry) {}

    /// Returns the saved video settings, if any.
    fn get_video_settings(&self) -> Option<VideoSettings> {
        None
    }

    /// Saves the video settings.
    fn set_video_settings(&self, _settings: VideoSettings) {}

    /// Returns the saved app settings, if any.
    fn get_app_settings(&self) -> Option<AppSettings> {
        None
    }

    /// Saves the app settings.
    fn set_app_settings(&self, _settings: AppSettings) {}

    /// Returns the saved update check state, if any.
    fn get_update_check(&self) -> Option<UpdateCheckState> {
        None
    }

    /// Saves the update check state.
    fn set_update_check(&self, _state: UpdateCheckState) {}
}

/// Interface for stored tags and tag color mapping. Tags are keyed by tag id (UUID); tag colors are keyed by tag name.
pub trait StoredTagStore: Send + Sync {
    /// Returns stored tags in sort order (no color; use tag color mapping for colors).
    fn get_stored_tags(&self) -> Result<Vec<StoredTag>, Box<dyn std::error::Error + Send + Sync>>;

    /// Returns the tag name -> color index mapping (tag colors keyed by tag name).
    fn get_tag_color_mapping(
        &self,
    ) -> Result<TagColorMapping, Box<dyn std::error::Error + Send + Sync>>;

    /// Saves or updates a stored tag by id: insert if id not present, else update name and color.
    fn save_tag(
        &mut self,
        tag: StoredTag,
        color_index: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Removes a tag by tag id (UUID) from stored_tags and its entry from tag_color_mapping (by tag name).
    fn remove_stored_tag_by_id(
        &mut self,
        tag_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Updates sort_order for the given stored tags (e.g. after display collection rebalance). Ids not present in the store are ignored.
    fn update_tag_orders(
        &mut self,
        tag_orders: &[(Uuid, i64)],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// One-time setup (e.g. run migrations). Implemented by the database; the logging decorator wraps it.
pub trait Initializable: Send + Sync {
    /// Performs one-time setup. Call once at startup before using the store.
    fn initialize(&self) -> Result<(), rusqlite::Error>;
}
