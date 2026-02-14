//! Root application state and coordination

use iced::{Element, Task};

use crate::features::drag_drop;
use crate::ui;

/// Main application state
#[derive(Default)]
pub struct FrenameApp {
    drag_drop_state: drag_drop::DragDropState,
    video_player_state: ui::video_player::VideoPlayerState,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    DragDrop(drag_drop::Message),
    VideoPlayer(ui::video_player::Message),
}

impl FrenameApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DragDrop(drag_drop::Message::FileDropped(path)) => {
                self.drag_drop_state.handle_file_dropped(path.clone());
                self.video_player_state.load_video(path, Message::VideoPlayer)
            }
            Message::VideoPlayer(ui::video_player::Message::VideoLoaded(success)) => {
                if let Some(path) = &self.drag_drop_state.dropped_file {
                    self.video_player_state.handle_video_loaded(path, success);
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        ui::video_player::view(&self.video_player_state)
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
