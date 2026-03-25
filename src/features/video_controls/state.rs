//! State for video controls feature

use frename_core::{AppDatabase, AppStateStore, VideoSettings};
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
    /// Volume level 0.0..=1.0
    volume: f32,
}

impl Default for VideoControlsState {
    fn default() -> Self {
        let volume = AppDatabase::new()
            .get_video_settings()
            .unwrap_or_default()
            .volume;
        Self::with_volume(volume)
    }
}

impl VideoControlsState {
    /// Creates state with a specific volume (preserves it across video loads).
    pub fn with_volume(volume: f32) -> Self {
        Self {
            is_playing: false,
            duration_secs: 0.0,
            seeking: false,
            seek_position: 0.0,
            volume: volume.clamp(0.0, 1.0),
        }
    }
}

impl VideoControlsState {
    /// Handle all controls messages.
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
            Message::SeekBack10 | Message::SeekForward10 => {
                // No local state change; video player performs the seek
            }
            Message::SetSegmentStart | Message::SetSegmentEnd => {
                // No local state change; video player captures current position and emits SegmentMarked
            }
            Message::SetVolume(v) => {
                self.volume = v.clamp(0.0, 1.0);
                AppDatabase::new().set_video_settings(VideoSettings { volume: self.volume });
            }
            Message::TakeScreenshot => {
                // Handled by video player (needs current position). No local state change.
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

    /// Current volume level 0.0..=1.0
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Keyboard shortcuts for video controls: F1 = 10s back, F3 = 10s forward, F12 = screenshot
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                match key.as_ref() {
                    keyboard::Key::Named(keyboard::key::Named::F1) => Some(Message::SeekBack10),
                    keyboard::Key::Named(keyboard::key::Named::F3) => Some(Message::SeekForward10),
                    keyboard::Key::Named(keyboard::key::Named::F12) => Some(Message::TakeScreenshot),
                    _ => None,
                }
            }
            _ => None,
        })
    }
}
