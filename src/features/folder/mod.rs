//! Folder list view: displays the file list and sends user actions (SelectFile, PreviousFile, NextFile).
//! Workspace owns folder structure and selection; it passes selected file and panels show it.

mod messages;
pub mod view;

pub use messages::Message;

/// Scrollable ID for the folder file list (shared between view and workspace for scroll-to-selected).
pub const FOLDER_LIST_SCROLLABLE_ID: &str = "folder-file-list";

/// Approximate rendered height of one file row (pixels), used to compute scroll position.
pub const FOLDER_ROW_HEIGHT: f32 = 28.0;
