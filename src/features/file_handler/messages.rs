//! Messages for file handler feature

use std::path::PathBuf;

use crate::features::{rename_panel, video_player};

/// Messages handled by the file handler
#[derive(Debug, Clone)]
pub enum Message {
    /// Open a file (from any source: drag-drop, dialog, CLI, etc.)
    OpenFile(PathBuf),
    /// Video player messages
    VideoPlayer(video_player::Message),
    /// Rename panel messages
    RenamePanel(rename_panel::Message),
    /// Splitter dragged — absolute cursor X position
    SplitterDragged(f32),
}
