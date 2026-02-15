//! Root application state and coordination

use iced::{Element, Subscription, Task};

use crate::features::{drag_drop, file_handler};

use super::Message;

/// Main application state
#[derive(Default)]
pub struct FrenameApp {
    drag_drop_state: drag_drop::DragDropState,
    file_handler_state: file_handler::FileHandlerState,
}

impl FrenameApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DragDrop(drag_drop::Message::FileDropped(path)) => {
                self.drag_drop_state.handle_file_dropped(path.clone());
                Task::done(Message::FileHandler(file_handler::Message::OpenFile(path)))
            }
            Message::FileHandler(msg) => {
                self.file_handler_state.update(msg).map(Message::FileHandler)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        file_handler::view::view(&self.file_handler_state).map(Message::FileHandler)
    }

    /// Feature subscriptions (file drop, keyboard, timers, etc.)
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.drag_drop_state.subscription().map(Message::DragDrop),
            self.file_handler_state
                .subscription()
                .map(Message::FileHandler),
        ])
    }

    /// Get the window title based on the currently open file
    pub fn title(&self) -> String {
        if let Some(file) = self.file_handler_state.current_file() {
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
