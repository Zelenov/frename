//! State for file handler feature

use std::path::PathBuf;

use iced::{Subscription, Task};

use crate::features::rename_panel::RenamePanelState;
use crate::features::video_player::VideoPlayerState;

use super::Message;

/// Central file handling state - the core of the application
pub struct FileHandlerState {
    /// Currently open file path
    current_file: Option<PathBuf>,
    /// Video player (activated when the opened file is a video)
    video_player: VideoPlayerState,
    /// Rename panel (right side: file name display + tag list)
    rename_panel: RenamePanelState,
}

impl Default for FileHandlerState {
    fn default() -> Self {
        Self {
            current_file: None,
            video_player: VideoPlayerState::default(),
            rename_panel: RenamePanelState::default(),
        }
    }
}

impl FileHandlerState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile(path) => {
                log::info!("Opening file: {}", path.display());
                self.current_file = Some(path.clone());

                // Set the initial file name from the dropped file (stem without extension)
                let file_stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                self.rename_panel.set_file_name(file_stem);

                // For now, treat all files as video
                self.video_player.load_video(path, Message::VideoPlayer)
            }
            Message::VideoPlayer(msg) => {
                self.video_player.update(msg).map(Message::VideoPlayer)
            }
            Message::RenamePanel(msg) => {
                self.rename_panel.update(&msg);
                Task::none()
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

    /// Get reference to the rename panel state
    pub fn rename_panel(&self) -> &RenamePanelState {
        &self.rename_panel
    }
}
