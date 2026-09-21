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
    /// Copy tags from the file with this stable ID into the currently open file.
    CopyTagsFrom(frename_core::FileId),
    /// Open a native file picker dialog (from the controls bar).
    OpenFolder,
    /// Show only files without tags (true) or every file (false).
    SetUntaggedOnly(bool),
    /// Narrow the list to files whose name contains this text (empty = no filter).
    SetNameFilter(String),
}
