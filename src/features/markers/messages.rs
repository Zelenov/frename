//! Messages for the marker list and the marker keys.

use frename_core::MarkerColor;

/// What the user did to the markers. The video player adds the playhead position before it
/// reaches the folder workspace, which owns the markers.
#[derive(Debug, Clone)]
pub enum Message {
    /// `📍`: add a marker at the playhead, or open the one just added / under it.
    Add,
    /// `F2` down: as [`Message::Add`], and a marker it adds grows into a range while `F2` is
    /// held.
    KeyDown,
    /// `F2` up: the held marker ends at the playhead.
    KeyUp,
    /// Set a marker's start and end (ms): a handle drag, or a range made a point.
    SetSpan(String, u64, u64),
    /// `Alt`+drag on the bar drew a new range: start, end (ms).
    AddRange(u64, u64),
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
    NameAction(iced::widget::text_editor::Action),
    /// Show or hide the color picker of a row.
    ToggleColorPicker(String),
    SetColor(String, MarkerColor),
    Delete(String),
    /// A row's ✨: name and describe this marker with AI, in the background.
    Describe(String),
    /// `Ctrl+F2`: [`Message::Describe`] the marker the playhead is on.
    DescribeAtPlayhead,
    /// Stop describing this marker.
    StopDescribing(String),
    /// The list scrolled: its offset from the top and its viewport's height.
    Scrolled(f32, f32),
}
