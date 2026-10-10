//! Messages for folder workspace feature

use std::path::PathBuf;

use frename_core::{File, FileId, FileSnapshot, FolderAndFile};
use iced::widget::text_editor;

use super::Directory;
use crate::features::{
    batch, drag_out, file_menu, file_name_panel, folder, media_viewer, recent_folders, sync_panel,
    tag_panel, video_controls,
};

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
    /// Open a path the user gave (picker, drag-drop, command line): a folder opens as itself;
    /// a file opens its folder with the file selected (selected in place if already listed).
    OpenPath(PathBuf),
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
    FileUpdated { id: FileId, snapshot: FileSnapshot },
    /// User selected a file in the list (from folder view)
    Folder(folder::Message),
    /// Media viewer messages
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
    /// Demo mode only (a screenshot scenario's `describing`): show the open clip's marker of
    /// this name as being described, without sending a request.
    ShowDescribing(String),
    /// Demo (a scenario's `ai_retry_wait_s`): the batch job on the checked files, its first file
    /// waiting this many seconds before its request is sent again; nothing is sent.
    ShowBatchRetry(u64),
    /// Demo: the unnamed markers as "Describe N unnamed" leaves them, without a request.
    ShowDescribingUnnamed,
    /// Demo: "Describe N unnamed" asking whether to send, without a request.
    ShowConfirmingDescribeUnnamed,
    /// A marker's "Describe with AI" request came back (see `markers::describe`).
    MarkerDescribed {
        file: FileId,
        guid: String,
        /// The request's number (see `MarkersState::start_describing`).
        request: u64,
        outcome: crate::features::markers::MomentOutcome,
    },
    /// A marker's request has read its frames and let go of the clip; its answer is still on
    /// the way (see `markers::describe`).
    MarkerRequestReleased {
        guid: String,
        /// The request's number (see `MarkersState::start_describing`).
        request: u64,
    },
    /// A second passed: write the open clip's unsaved edits into the recovery journal, if they
    /// changed (see `frename_core::recovery`).
    JournalTick,
    /// A journal write finished (what was written, or why not).
    JournalWritten(
        frename_core::FileId,
        Option<(frename_core::recovery::Entry, Result<(), String>)>,
    ),
    /// Undo the last undoable action (Ctrl+Z).
    Undo,
    /// Redo the last undone action (Ctrl+Y / Ctrl+Shift+Z).
    Redo,
    /// Scroll the folder file list so the selected file is visible (deferred to next frame).
    ScrollFolderListToSelected,
    /// Internal: update cached folder list scroll Y after programmatic scroll.
    FolderListScrollAdjusted(f32),
    /// Fullscreen went on or off: put the marker list and the subtitle list back at the offsets
    /// they had (the view built them anew).
    RestoreListScrolls { markers_y: f32, cues_y: f32 },
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
    /// A click or a focus key somewhere: ask whether the comment box has the keys now.
    CheckCommentFocus,
    /// Whether the comment box has the keys, so its edge can show it.
    CommentFocused(bool),
    /// Resize the comment box, or let it take the whole panel.
    CommentLayout(crate::features::file_workspace::CommentLayout),
    /// Screenshot captured at position (ms) with JPEG bytes.
    ScreenshotTaken(u64, Vec<u8>),
    /// Turn the open video by this many quarter turns clockwise, -1 counter-clockwise
    /// (Ctrl+Alt+→ / Ctrl+Alt+←, or the ↻ / ↺ buttons).
    RotateVideo(i32),
    /// `Ctrl+Alt+←/→` while a text field had the keys: [`Message::RotateVideo`], unless the
    /// field is the comment box.
    RotateVideoWhileTyping(i32),
    /// `Alt+←/→`: one frame back or forward in the open video.
    StepFrame(video_controls::FrameStep),
    /// `Alt+←/→` while a text field had the keys: [`Message::StepFrame`], unless the field is
    /// the comment box.
    StepFrameWhileTyping(video_controls::FrameStep),
    /// `Home`: to the start of the open video, when a video is shown (#161).
    GoToStart,
    /// `Home` while a text field had the keys: to the start of the clip while the note that it
    /// continued is shown (it says Home starts it over), unless the field is the comment box or
    /// a name being edited; otherwise the field keeps the key (#161).
    GoToStartWhileTyping,
    /// The second step of [`Message::GoToStartWhileTyping`]: the file name being edited does not
    /// have the key, so only the comment box can still keep it.
    GoToStartUnlessWriting,
    /// The second step of `Shift+Space` with a rename open: the file name being edited does not
    /// have the key, so the tag under the tag cursor is toggled (#260).
    ToggleSelectedTagUnlessRenaming,
    /// Open a native folder picker dialog so the user can choose a folder to open.
    OpenFolderPicker,
    /// Open a native file picker dialog so the user can choose a file to open.
    OpenFilePicker,
    /// The recent folders list: the dropdown by the open button, and the empty screen's.
    RecentFolders(recent_folders::Message),
    /// Batch mode messages (batch panel, and folder list checks translated by the workspace).
    Batch(batch::Message),
    /// Enter batch mode with every listed file checked and the action set up for `operation`
    /// (from the settings window after a storage change).
    PrepareBatch(batch::Operation),
    /// A file of the running batch job is done (internal).
    BatchItemDone {
        id: FileId,
        /// Boxed: it is much larger than the other messages.
        result: Box<batch::ItemResult>,
    },
    /// The batch job ended, finished or cancelled (internal): the open file comes back.
    BatchFinished,
    /// Background load of comments the folder scan deferred (internal). `generation` ties the
    /// batch to the folder it was started for; `results` are `(id, path loaded, snapshot)`.
    CommentBatchLoaded {
        generation: u64,
        results: Vec<(FileId, PathBuf, FileSnapshot)>,
    },
    /// Advance the loading spinner in the folder list (internal, only while files load).
    SpinnerTick,
    /// Mouse events while a file row is pressed: may start a drag out of the window.
    DragOut(drag_out::Message),
    /// Drag these files out of the window. Handled by the app, which owns the window.
    StartDragOut(Vec<PathBuf>),
    /// The drag out of the window ended (the app sends it when the drag loop returns).
    DragOutFinished,
    /// The file context menu: opening, closing, and the item chosen (run by the workspace).
    FileMenu(file_menu::Message),
    /// A file menu key (`F11`, `Shift+F11`, `Ctrl+F11`): the action on the open file.
    FileAction(file_menu::FileAction),
    /// Run the action on the file now: its pending edits are on disk (internal).
    RunFileAction(FileId, file_menu::FileAction),
    /// The keyboard modifiers held right now (Ctrl/Shift), or none once the window loses focus.
    /// A mouse click carries no modifiers in iced, so a file-row click reads this instead.
    ModifiersChanged(iced::keyboard::Modifiers),
}
