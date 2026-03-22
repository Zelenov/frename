//! Application-level messages

use iced::{window, Size};

use crate::features::{drag_drop, folder_workspace};

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    /// From iced Window::Opened event; triggers initialization (e.g. load last session).
    WindowReady(window::Id),
    /// User requested window close (intercepted so we can unload GStreamer before exit).
    CloseRequested(window::Id),
    /// Window was moved; persist the new position.
    WindowMoved(f32, f32),
    /// Window was resized; persist the new size.
    WindowResized(f32, f32),
    /// Async result of window::is_maximized query.
    WindowMaximizedFetched(bool),
    /// Async result of window::monitor_size query.
    WindowMonitorSizeFetched(Option<Size>),
    DragDrop(drag_drop::Message),
    FolderWorkspace(folder_workspace::Message),
}
