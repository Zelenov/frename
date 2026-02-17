//! Application-level messages

use crate::features::{drag_drop, folder_workspace};

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    /// From iced Window::Opened event; triggers initialization (e.g. load last session).
    WindowReady,
    DragDrop(drag_drop::Message),
    FolderWorkspace(folder_workspace::Message),
}
