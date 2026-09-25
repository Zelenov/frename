//! Messages for folder workspace feature

use std::path::PathBuf;

use iced::widget::text_editor;
use frename_core::{ConversionPlan, ConversionReport, File, FileId, FileSnapshot, FolderAndFile};

use super::Directory;
use crate::features::{file_name_panel, folder, media_viewer, sync_panel, tag_panel};

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
    /// Snapshot to persist: created by folder workspace when switching file.
    /// Uses the stable FileId so the correct file is found even if it was renamed.
    FileUpdated {
        id: FileId,
        snapshot: FileSnapshot,
    },
    /// User selected a file in the list (from folder view)
    Folder(folder::Message),
    /// Media viewer messages (video + image)
    MediaViewer(media_viewer::Message),
    /// Tag panel messages
    TagPanel(tag_panel::Message),
    /// File name panel messages (wrap=true display in file workspace)
    FileNamePanel(file_name_panel::Message),
    /// Sync panel messages
    SyncPanel(sync_panel::Message),
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
    /// Internal: update cached scroll Y after programmatic scroll (so next scroll-into-view uses correct viewport).
    TagListScrollAdjusted(f32),
    /// Remove the selected tag (e.g. triggered by Delete key).
    RemoveTag,
    /// Save the selected tag to the store (e.g. triggered by Enter key on an unsaved tag). No-op if already stored or none selected.
    SaveSelectedTag,
    /// Copy current file's tags (internal clipboard + OS clipboard with filename).
    CopyTags,
    /// Paste previously copied tags onto the current file (replace semantics).
    PasteTags,
    /// Undo the last undoable action (Ctrl+Z).
    Undo,
    /// Redo the last undone action (Ctrl+Y / Ctrl+Shift+Z).
    Redo,
    /// Scroll the folder file list so the selected file is visible (deferred to next frame).
    ScrollFolderListToSelected,
    /// Internal: update cached folder list scroll Y after programmatic scroll.
    FolderListScrollAdjusted(f32),
    /// Toggle fullscreen mode for the media viewer (F5).
    ToggleMediaFullscreen,
    /// Escape pressed globally: exits fullscreen if active, otherwise clears the search bar filter.
    EscapePressed,
    /// Set segment start marker at the current video position ([ key).
    SetSegmentStart,
    /// Set segment end marker at the current video position (] key).
    SetSegmentEnd,
    /// User interacted with the multiline comment editor.
    CommentAction(text_editor::Action),
    /// Screenshot captured at position (ms) with JPEG bytes.
    ScreenshotTaken(u64, Vec<u8>),
    /// Open a native file picker dialog so the user can choose a file to open.
    OpenFilePicker,
    /// Move the comments and in/out points of the plan's files into its storage.
    ConvertMetadata(ConversionPlan),
    /// Stop a running conversion after the batch in progress.
    CancelConversion,
    /// A batch of the running conversion finished (internal).
    ConversionBatchDone(ConversionReport),
    /// How far the running conversion is: `done` of `total` files. Read by the app, which
    /// shows it in the settings window.
    ConversionProgress { done: usize, total: usize },
    /// Background load of comments the folder scan deferred (internal). `generation` ties the
    /// batch to the folder it was started for; `results` are `(id, path loaded, snapshot)`.
    CommentBatchLoaded {
        generation: u64,
        results: Vec<(FileId, PathBuf, FileSnapshot)>,
    },
    /// Advance the loading spinner in the folder list (internal, only while files load).
    SpinnerTick,
    /// Folder conversion finished (internal): rescan `folder` and reselect `selected`,
    /// which is the selected file's path after the conversion.
    MetadataConverted {
        folder: PathBuf,
        selected: Option<PathBuf>,
        report: ConversionReport,
        /// The conversion was cancelled before its last file.
        cancelled: bool,
    },
}
