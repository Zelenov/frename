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
                AppDatabase::new().set_video_settings(VideoSettings {
                    volume: self.volume,
                });
            }
            Message::TakeScreenshot
            | Message::AddMarker
            | Message::DeleteMarker
            | Message::PreviousMarker
            | Message::NextMarker
            | Message::EditMarker(_) => {
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

    /// Keyboard shortcuts for video controls: F1 = 10s back, F3 = 10s forward, F12 = save the
    /// frame, F2 = add a marker; with Shift, F1 / F3 = previous / next marker, F2 = delete the
    /// marker under the playhead. They work while a text field has focus, like the F-keys did.
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                shortcut(&key, modifiers)
            }
            _ => None,
        })
    }
}

/// The controls message of a key press, if it is one of the video shortcuts.
fn shortcut(key: &keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
    use keyboard::key::Named;
    let keyboard::Key::Named(named) = key else {
        return None;
    };
    if modifiers.command() || modifiers.alt() {
        return None;
    }
    match (named, modifiers.shift()) {
        (Named::F1, false) => Some(Message::SeekBack10),
        (Named::F3, false) => Some(Message::SeekForward10),
        (Named::F12, false) => Some(Message::TakeScreenshot),
        (Named::F2, false) => Some(Message::AddMarker),
        (Named::F1, true) => Some(Message::PreviousMarker),
        (Named::F3, true) => Some(Message::NextMarker),
        (Named::F2, true) => Some(Message::DeleteMarker),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyboard::key::Named;

    fn key(named: Named, modifiers: keyboard::Modifiers) -> Option<Message> {
        shortcut(&keyboard::Key::Named(named), modifiers)
    }

    #[test]
    fn shift_turns_the_seek_keys_into_marker_jumps() {
        let none = keyboard::Modifiers::empty();
        let shift = keyboard::Modifiers::SHIFT;
        assert!(matches!(key(Named::F1, none), Some(Message::SeekBack10)));
        assert!(matches!(
            key(Named::F1, shift),
            Some(Message::PreviousMarker)
        ));
        assert!(matches!(key(Named::F3, none), Some(Message::SeekForward10)));
        assert!(matches!(key(Named::F3, shift), Some(Message::NextMarker)));
        assert!(matches!(key(Named::F2, none), Some(Message::AddMarker)));
        assert!(matches!(key(Named::F2, shift), Some(Message::DeleteMarker)));
        assert!(matches!(
            key(Named::F12, none),
            Some(Message::TakeScreenshot)
        ));
        assert!(key(Named::F12, shift).is_none());
        assert!(key(Named::F2, keyboard::Modifiers::CTRL).is_none());
    }
}
