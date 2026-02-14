//! Root application state and coordination

use iced::{Element, Task};

use crate::features::{drag_drop, video_player};

/// Main application state
#[derive(Default)]
pub struct FrenameApp {
    drag_drop_state: drag_drop::DragDropState,
    video_player_state: video_player::VideoPlayerState,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    DragDrop(drag_drop::Message),
    VideoPlayer(video_player::Message),
}

impl FrenameApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DragDrop(drag_drop::Message::FileDropped(path)) => {
                self.drag_drop_state.handle_file_dropped(path.clone());
                self.video_player_state.load_video(path, Message::VideoPlayer)
            }
            Message::VideoPlayer(msg) => {
                self.video_player_state.update(msg).map(Message::VideoPlayer)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        video_player::view::view(&self.video_player_state).map(Message::VideoPlayer)
    }

    /// Get the window title based on dropped file
    pub fn title(&self) -> String {
        if let Some(file) = &self.drag_drop_state.dropped_file {
            file.display().to_string()
        } else {
            String::from("frename")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let _app = FrenameApp::default();
        // Basic smoke test - app can be created
    }
}
