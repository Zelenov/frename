//! Messages of the file context menu.

use frename_core::FileId;
use iced::Point;

use super::FileAction;

#[derive(Debug, Clone)]
pub enum Message {
    /// The right mouse button went down at this point of the window: a menu asked for next opens
    /// there, and an open menu closes (a right-click elsewhere).
    RightPressed(Point),
    /// Open the menu for this file where the right button went down (a right-click on its row
    /// in the file list, or on the open file's name).
    Open(FileId),
    /// Close without choosing: Esc, or a click outside the menu.
    Close,
    /// An item was chosen: the menu closes, and the workspace runs the action on the file.
    Choose(FileId, FileAction),
}
