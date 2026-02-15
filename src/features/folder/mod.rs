//! Folder list view: displays the file list and sends user actions (SelectFile, PreviousFile, NextFile).
//! Workspace owns folder structure and selection; it passes selected file and panels show it.

mod messages;
pub mod view;

pub use messages::Message;
