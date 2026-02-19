//! Messages for folder workspace feature

use std::path::PathBuf;

use frename_core::{File, FileSnapshot, FolderAndFile};

use super::Directory;
use crate::features::{file_name_panel, folder, tag_panel, video_player};

/// Key that triggered global focus (we emulate it into the search bar; Iced cannot replay the event).
#[derive(Debug, Clone)]
pub enum GlobalSearchKey {
    Char(char),
    Backspace,
    Delete,
}

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
        snapshot: FileSnapshot,
    },
    /// User selected a file in the list (from folder view)
    Folder(folder::Message),
    /// Video player messages
    VideoPlayer(video_player::Message),
    /// Tag panel messages
    TagPanel(tag_panel::Message),
    /// File name panel messages (wrap=true display in file workspace)
    FileNamePanel(file_name_panel::Message),
    /// Left splitter dragged (between video and folder) — absolute cursor X
    LeftSplitterDragged(f32),
    /// Right splitter dragged (between folder and rename panel) — absolute cursor X
    RightSplitterDragged(f32),
    /// Focus the search bar and emulate the triggering key into the filter (Iced cannot replay the event).
    FocusSearchBarAndKey(GlobalSearchKey),
    /// No-op (e.g. used when returning a focus operation from update).
    Noop,
    /// Scroll the tag list so the selected row is in view (deferred to next frame).
    ScrollTagListToSelection,
    /// Remove the selected tag (e.g. triggered by Delete key).
    RemoveTag,
}
