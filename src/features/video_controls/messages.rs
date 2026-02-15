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
}
