//! State for video controls feature

use iced::{event, keyboard, Subscription};

use super::Message;

/// Video player controls state
pub struct VideoControlsState {
    /// Whether the video is currently playing
    is_playing: bool,
    /// Total duration in seconds
    duration_secs: f32,
    /// Whether the user is currently dragging the progress bar
    seeking: bool,
    /// Position while user is dragging (only meaningful when seeking == true)
    seek_position: f32,
}

impl Default for VideoControlsState {
    fn default() -> Self {
        Self {
            is_playing: false,
            duration_secs: 0.0,
            seeking: false,
            seek_position: 0.0,
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
                self.is_playing = true;
                self.seeking = false;
                self.seek_position = 0.0;
            }
            Message::SetPlaying(playing) => {
                if self.is_playing != *playing {
                    self.is_playing = *playing;
                }
            }
            Message::Seek(pos) => {
                self.seeking = true;
                self.seek_position = *pos;
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

    /// Total duration in seconds
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }

    /// Whether the user is currently seeking (dragging the progress bar)
    pub fn is_seeking(&self) -> bool {
        self.seeking
    }

    /// Seek position in seconds (only meaningful while seeking)
    pub fn seek_position_secs(&self) -> f32 {
        self.seek_position
    }

    /// Keyboard shortcuts for video controls (Space = play/pause)
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Space),
                ..
            }) => Some(Message::TogglePlayPause),
            _ => None,
        })
    }
}
