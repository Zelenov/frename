//! State for video controls feature

use super::Message;

/// Video player controls state
pub struct VideoControlsState {
    /// Whether the video is currently playing
    is_playing: bool,
    /// Current playback position in seconds
    position_secs: f32,
    /// Total duration in seconds
    duration_secs: f32,
}

impl Default for VideoControlsState {
    fn default() -> Self {
        Self {
            is_playing: false,
            position_secs: 0.0,
            duration_secs: 0.0,
        }
    }
}

impl VideoControlsState {
    /// Handle all controls messages
    pub fn update(&mut self, message: &Message) {
        match message {
            Message::TogglePlayPause => {
                self.is_playing = !self.is_playing;
            }
            Message::SetPlaying(playing) => {
                if self.is_playing != *playing {
                    self.is_playing = *playing;
                }
            }
            Message::VideoReady { duration_secs } => {
                self.duration_secs = *duration_secs;
                self.position_secs = 0.0;
                self.is_playing = true;
            }
            Message::UpdatePosition(pos) => {
                if (self.position_secs - pos).abs() > 0.05 {
                    self.position_secs = *pos;
                }
            }
        }
    }

    /// Whether the video is currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
}
