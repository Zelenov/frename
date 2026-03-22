//! Messages for the video player sub-feature.

use crate::features::video_controls;

/// Messages handled by the video player.
#[derive(Debug, Clone)]
pub enum Message {
    /// Video finished loading (success flag)
    VideoLoaded(bool),
    /// Video became available after loading
    VideoReady { duration_secs: f32 },
    /// New video frame rendered (triggers view refresh for progress bar)
    NewFrame,
    /// End of stream reached
    EndOfStream,
    /// Toggle pause state on the video
    TogglePause,
    /// Seek to position in seconds
    Seek(f32),
    /// Controls message
    Controls(video_controls::Message),
    /// Unload current video (e.g. before switching file). Emits VideoUnloaded when done.
    Unload,
    /// Current video has been unloaded; safe to persist file and load next.
    VideoUnloaded,
    /// User clicked the fullscreen toggle button.
    ToggleFullscreen,
}
