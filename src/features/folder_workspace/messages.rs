//! Messages for folder workspace feature

use std::path::PathBuf;

use frename_core::Directory;
use crate::features::{folder, tag_panel, video_player};

/// Messages handled by the folder workspace (owns directory, selection, and all logic).
#[derive(Debug, Clone)]
pub enum Message {
    /// Open a file by path: if in current folder then select and open, else scan folder then open
    OpenFile(PathBuf),
    /// Scan a directory and auto-select the target file afterwards
    ScanFolder {
        directory: PathBuf,
        target_file: PathBuf,
    },
    /// Directory scan completed (internal)
    FolderLoaded {
        directory: Directory,
        target_file: PathBuf,
    },
    /// User selected a file in the list (from folder view)
    Folder(folder::Message),
    /// Video player messages
    VideoPlayer(video_player::Message),
    /// Rename panel messages
    TagPanel(tag_panel::Message),
    /// Left splitter dragged (between video and folder) — absolute cursor X
    LeftSplitterDragged(f32),
    /// Right splitter dragged (between folder and rename panel) — absolute cursor X
    RightSplitterDragged(f32),
}
