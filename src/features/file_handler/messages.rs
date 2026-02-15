//! Messages for file handler feature

use std::path::PathBuf;

use crate::features::{folder, rename_panel, video_player};

/// Messages handled by the file handler
#[derive(Debug, Clone)]
pub enum Message {
    /// Open a file (set up video player + rename panel for the selected file)
    OpenFile(PathBuf),
    /// Folder feature messages (directory scan, file selection)
    Folder(folder::Message),
    /// Video player messages
    VideoPlayer(video_player::Message),
    /// Rename panel messages
    RenamePanel(rename_panel::Message),
    /// Left splitter dragged (between video and folder) — absolute cursor X
    LeftSplitterDragged(f32),
    /// Right splitter dragged (between folder and rename panel) — absolute cursor X
    RightSplitterDragged(f32),
}
