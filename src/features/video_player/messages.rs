//! Messages for video player feature

use crate::features::video_controls;

/// Messages handled by the video player
#[derive(Debug, Clone)]
pub enum Message {
    /// Video finished loading
    VideoLoaded(bool),
    /// Video became available after loading
    VideoReady { duration_secs: f32 },
    /// End of stream reached
    EndOfStream,
    /// Toggle pause state on the video
    TogglePause,
    /// Controls message
    Controls(video_controls::Message),
}
