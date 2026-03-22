//! Application-level messages

use iced::window;

use crate::features::{drag_drop, folder_workspace};

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    /// From iced Window::Opened event; triggers initialization (e.g. load last session).
    WindowReady,
    /// User requested window close (intercepted so we can unload GStreamer before exit).
    CloseRequested(window::Id),
    /// Window was moved; persist the new position.
    WindowMoved(f32, f32),
    /// Window was resized; persist the new size.
    WindowResized(f32, f32),
    DragDrop(drag_drop::Message),
    FolderWorkspace(folder_workspace::Message),
}
