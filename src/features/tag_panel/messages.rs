//! Messages for tag panel feature

use frename_core::TagId;

/// Messages handled by the tag panel
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle a tag by its id
    ToggleTag(TagId),
    /// Set the filter query for the tag list (case-insensitive contains)
    SetFilter(String),
    /// Move list cursor up (keyboard)
    SelectUp,
    /// Move list cursor down (keyboard)
    SelectDown,
    /// Toggle the currently selected tag (Space key).
    ToggleSelectedTag,
    /// Tag list was scrolled; report viewport for scroll-into-view (selection).
    TagListScrolled {
        scroll_y: f32,
        viewport_height: f32,
    },
    /// Remove the tag from the store and UI (unselect first, then delete).
    DeleteTag(TagId),
    /// Remove the currently selected tag (no-op if none selected).
    DeleteSelectedTag,
    /// Mouse entered or left a tag row (for showing delete button on hover).
    TagHovered(Option<TagId>),
}
