//! State for drag and drop feature

use iced::{event, window, Subscription};
use std::path::PathBuf;

use super::Message;

/// State for tracking dropped files
#[derive(Default)]
pub struct DragDropState {
    /// Currently dropped file path
    pub dropped_file: Option<PathBuf>,
}

impl DragDropState {
    /// Handle file dropped
    pub fn handle_file_dropped(&mut self, path: PathBuf) {
        log::info!("File dropped: {}", path.display());
        self.dropped_file = Some(path);
    }

    /// Listen for file drop window events
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| match event {
            iced::Event::Window(window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(path))
            }
            _ => None,
        })
    }
}
