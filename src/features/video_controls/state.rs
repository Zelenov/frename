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
    /// Whether the user is currently dragging the progress bar
    seeking: bool,
}

impl Default for VideoControlsState {
    fn default() -> Self {
        Self {
            is_playing: false,
            position_secs: 0.0,
            duration_secs: 0.0,
            seeking: false,
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
            Message::VideoReady { duration_secs } => {
                self.duration_secs = *duration_secs;
                self.position_secs = 0.0;
                self.is_playing = true;
                self.seeking = false;
            }
            Message::SetPlaying(playing) => {
                if self.is_playing != *playing {
                    self.is_playing = *playing;
                }
            }
            Message::UpdatePosition(pos) => {
                // Ignore position updates while user is dragging
                if !self.seeking && (self.position_secs - pos).abs() > 0.05 {
                    self.position_secs = *pos;
                }
            }
            Message::Seek(pos) => {
                self.seeking = true;
                self.position_secs = *pos;
            }
            Message::SeekReleased => {
                self.seeking = false;
            }
        }
    }

    /// Whether the video is currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Current position in seconds
    pub fn position_secs(&self) -> f32 {
        self.position_secs
    }

    /// Total duration in seconds
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }
}
