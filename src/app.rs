//! Root application state and coordination

use iced::{Element, Task};

use crate::features::drag_drop;

/// Main application state
pub struct FrenameApp {
    drag_drop_state: drag_drop::DragDropState,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    DragDrop(drag_drop::Message),
}

impl FrenameApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DragDrop(drag_drop::Message::FileDropped(path)) => {
                self.drag_drop_state.handle_file_dropped(path);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::{container, text};
        use iced_video_player::VideoPlayer;

        // Show video player if video is loaded, otherwise show drop zone
        if let Some(video) = &self.drag_drop_state.current_video {
            VideoPlayer::new(video)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        } else {
            container(
                text("Drop a video file here to play")
                    .size(24)
            )
            .center(iced::Length::Fill)
            .into()
        }
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

impl Default for FrenameApp {
    fn default() -> Self {
        Self {
            drag_drop_state: Default::default(),
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
