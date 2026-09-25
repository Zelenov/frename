//! Messages from the folder list (user actions; workspace handles selection and notifies panels)

/// User actions in the folder list; workspace updates selected file and panels react.
#[derive(Debug, Clone)]
pub enum Message {
    /// User selected a file by its index in the list
    SelectFile(usize),
    /// User pressed previous file (folder controls)
    PreviousFile,
    /// User pressed next file (folder controls)
    NextFile,
    /// Folder list scrolled: current scroll offset and viewport height.
    Scrolled { scroll_y: f32, viewport_height: f32 },
    /// Scroll the folder list to the currently selected file.
    ScrollToSelected,
    /// Open a native file picker dialog (from the controls bar).
    OpenFolder,
    /// Show only files without tags (true) or every file (false).
    SetUntaggedOnly(bool),
    /// Show only files with a subtitle file (true) or every file (false).
    SetSubtitledOnly(bool),
    /// Show only files with a comment (true) or every file (false).
    SetCommentedOnly(bool),
    /// Narrow the list to files whose name contains this text (empty = no filter).
    SetNameFilter(String),
    /// Open the settings window (from the controls bar). Handled by the app.
    OpenSettings,
    /// Double-click on a row's name: edit that file's name in place.
    StartRename(usize),
    /// Text typed into the in-place rename editor.
    RenameInput(String),
    /// Enter in the in-place rename editor: rename the file.
    SubmitRename,
    /// Switch between editing the open file (false) and batch actions on checked files (true).
    SetBatchMode(bool),
    /// Check or uncheck a file for batch actions.
    ToggleChecked(frename_core::FileId),
    /// Header check box: check every listed file, or uncheck all when they all are.
    ToggleAllChecked,
    /// Flip the check of every listed file.
    InvertChecks,
}
