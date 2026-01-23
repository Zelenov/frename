//! frename - File Renaming Utility for Windows
//! 
//! A GUI-based file renaming tool with preview and batch operations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console in release mode

use iced::{Element, Task, window};

fn main() -> iced::Result {
    iced::application(
        || (FrenameApp::default(), Task::none()),
        FrenameApp::update,
        FrenameApp::view,
    )
    .window(window::Settings {
        size: iced::Size::new(800.0, 600.0),
        resizable: true,
        ..window::Settings::default()
    })
    .antialiasing(false)
    .run()
}

/// Main application state
struct FrenameApp {
    // Application state will be added here
}

/// Application messages
#[derive(Debug, Clone)]
enum Message {
    // Messages will be added here
}

impl FrenameApp {
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Empty window - no UI components
        iced::widget::container(iced::widget::text(""))
            .into()
    }
}

impl Default for FrenameApp {
    fn default() -> Self {
        Self {}
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
