//! Messages for the marker list and the marker keys.

use frename_core::MarkerColor;
use iced::widget::text_editor;

/// What the user did to the markers. The video player adds the playhead position before it
/// reaches the folder workspace, which owns the markers.
#[derive(Debug, Clone)]
pub enum Message {
    /// `F2` or `◆+`: add a marker at the playhead, or open the one just added / under it.
    Add,
    /// `Shift+F2`: delete the marker under the playhead.
    DeleteAtPlayhead,
    /// `Shift+F1`: jump to the previous marker.
    Previous,
    /// `Shift+F3`: jump to the next marker.
    Next,
    /// A row's time was clicked: jump to this time (ms).
    JumpTo(u64),
    /// Open the row of the marker with this GUID for editing.
    Open(String),
    /// Close the open row (`Enter` in its name, `Esc`, or its `✓`).
    Close,
    /// Typing in the open row's name.
    NameInput(String),
    /// Typing in the open row's comment.
    CommentAction(text_editor::Action),
    /// Show or hide the color picker of a row.
    ToggleColorPicker(String),
    SetColor(String, MarkerColor),
    /// Set the marker's end to the playhead, making it a ranged marker.
    SetEndHere(String),
    /// Make a ranged marker a point marker again.
    ClearEnd(String),
    Delete(String),
}
