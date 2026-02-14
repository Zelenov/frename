//! Messages for video controls feature

/// Messages handled by video controls
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle play/pause (from user click)
    TogglePlayPause,
    /// New video is ready - initialize controls
    VideoReady { duration_secs: f32 },
    /// Set playing state
    SetPlaying(bool),
    /// Update current position
    UpdatePosition(f32),
}
