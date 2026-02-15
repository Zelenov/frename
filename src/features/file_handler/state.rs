//! State for file handler feature

use std::path::PathBuf;

use iced::{Subscription, Task};

use crate::features::video_player::VideoPlayerState;

use super::Message;

/// Central file handling state - the core of the application
pub struct FileHandlerState {
    /// Currently open file path
    current_file: Option<PathBuf>,
    /// Video player (activated when the opened file is a video)
    video_player: VideoPlayerState,
}

impl Default for FileHandlerState {
    fn default() -> Self {
        Self {
            current_file: None,
            video_player: VideoPlayerState::default(),
        }
    }
}

impl FileHandlerState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile(path) => {
                log::info!("Opening file: {}", path.display());
                self.current_file = Some(path.clone());
                // For now, treat all files as video
                self.video_player.load_video(path, Message::VideoPlayer)
            }
            Message::VideoPlayer(msg) => {
                self.video_player.update(msg).map(Message::VideoPlayer)
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.video_player.subscription().map(Message::VideoPlayer)
    }

    /// Currently open file path
    pub fn current_file(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
    }

    /// Get reference to the video player state
    pub fn video_player(&self) -> &VideoPlayerState {
        &self.video_player
    }
}
