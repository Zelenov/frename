//! Messages for folder workspace feature

use std::path::PathBuf;

use frename_core::{File, FileTag, FolderAndFile};

use super::Directory;
use crate::features::{folder, tag_panel, video_player};

/// Messages handled by the folder workspace (owns directory, selection, and all logic).
#[derive(Debug, Clone)]
pub enum Message {
    /// Open a file by path: if in current folder then select and open, else scan folder then open
    OpenFile(PathBuf),
    /// Initialize: load last session from state store and open that folder/file if any.
    LoadLastSession,
    /// Scan a directory and auto-select the target file afterwards
    ScanFolder(FolderAndFile),
    /// Directory scan completed (internal). target_file = which file to select and open, if any.
    FolderLoaded {
        directory: Directory,
        target_file: Option<PathBuf>,
    },
    /// Directory scan failed; keep previous state (no directory replaced).
    FolderLoadFailed,
    /// File was selected (by directory). Apply snapshot, set file workspace, load/unload video.
    FileOpened(File),
    /// Snapshot to persist: created by folder workspace when switching file (from file workspace get_snapshot). Save to disk then update directory.
    FileUpdated {
        path: PathBuf,
        new_tags: Vec<FileTag>,
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
