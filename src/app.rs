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
        use iced::widget::container;

        // Empty window - entire area accepts drag and drop
        container("")
            .into()
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
