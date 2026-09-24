//! Folder list view: displays the file list and sends user actions (SelectFile, PreviousFile, NextFile).
//! Workspace owns folder structure and selection; it passes selected file and panels show it.

mod messages;
pub mod view;

pub use messages::Message;

/// Scrollable ID for the folder file list (shared between view and workspace for scroll-to-selected).
pub const FOLDER_LIST_SCROLLABLE_ID: &str = "folder-file-list";

/// Fixed height of one file row (pixels). Applied in view and used to compute scroll position.
pub const FOLDER_ROW_HEIGHT: f32 = 40.0;

/// Text input ID of the in-place rename editor (shared between view and workspace for focus).
pub const FOLDER_RENAME_INPUT_ID: &str = "folder-rename-input";

/// A file being renamed in place in the folder list (double-click on its name).
#[derive(Debug, Clone)]
pub struct InlineRename {
    /// The file being renamed.
    pub id: frename_core::FileId,
    /// The name as typed so far: the whole file name, extension included.
    pub text: String,
    /// Why the last Enter was refused; cleared by the next edit.
    pub error: Option<&'static str>,
}
