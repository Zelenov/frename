//! Application-level messages

use crate::features::{drag_drop, file_handler};

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    DragDrop(drag_drop::Message),
    FileHandler(file_handler::Message),
}
