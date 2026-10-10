//! Messages for file name panel feature (wrap=true display in file workspace).
//! On DragEnded the workspace applies the new order to TagList and persists stored tags.

use frename_core::TagId;

#[derive(Debug, Clone)]
pub enum Message {
    /// User started dragging the tag chip (identity by TagId so list changes don't desync).
    DragStarted { tag_id: TagId, initial_index: usize },
    /// Cursor moved while dragging; used to compute drop slot (panel bounds set by BoundsReporter).
    DragHoverCursor { x: f32, y: f32 },
    /// User released mouse; end drag. Workspace applies reorder to TagList and persists.
    DragEnded,
    /// Panel content bounds on screen (for mapping cursor to chip index).
    PanelBounds(iced::Rectangle),
    /// Trash zone bounds (drop here to unselect the dragged tag).
    TrashBounds(iced::Rectangle),
    /// Middle-click on a tag chip: uncheck the tag.
    RemoveTag(TagId),
    /// Clear the segment start marker (× on the IN badge).
    ClearSegmentStart,
    /// Clear the segment end marker (× on the OUT badge).
    ClearSegmentEnd,
    /// Set the in and out points to the ones the AI description suggests.
    ApplySuggestedInOut,
    /// Add the tag the AI description suggests (by name) to the open clip.
    AddSuggestedTag(String),
    /// Add the most likely of the tags the AI description suggests (F6).
    AddFirstSuggestedTag,
    /// Add every tag the AI description suggests, as one undo step (Shift+F6).
    AddAllSuggestedTags,
    /// Right-click on the file name: open the file menu for the open file.
    OpenFileMenu,
}
