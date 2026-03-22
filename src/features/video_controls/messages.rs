//! Messages for video controls feature

/// Messages handled by video controls
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle play/pause (from user click or keyboard)
    TogglePlayPause,
    /// New video is ready - initialize controls
    VideoReady { duration_secs: f32 },
    /// Set playing state
    SetPlaying(bool),
    /// User is seeking to a position in seconds
    Seek(f32),
    /// User released the seek bar
    SeekReleased,
    /// Seek 10 seconds backward (button or F1)
    SeekBack10,
    /// Seek 10 seconds forward (button or F3)
    SeekForward10,
    /// Set segment start marker at the current video position ([ key or button)
    SetSegmentStart,
    /// Set segment end marker at the current video position (] key or button)
    SetSegmentEnd,
    /// Set volume (0.0 = silent, 1.0 = full)
    SetVolume(f32),
}
