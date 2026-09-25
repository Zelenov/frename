//! Application-level messages

use iced::{window, Size};

use crate::features::{drag_drop, folder_workspace, settings};

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    /// Main window opened (iced Window::Opened); triggers initialization (e.g. load last session).
    WindowReady,
    /// User requested window close (intercepted so we can unload GStreamer before exit).
    CloseRequested(window::Id),
    /// A window has closed: the main one ends the app, the settings one just goes away.
    WindowClosed(window::Id),
    /// Open the settings window, or bring it to the front if it is already open.
    OpenSettings,
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
    Settings(settings::Message),
    Noop,
}
