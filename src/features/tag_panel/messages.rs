//! Messages for tag panel feature

use frename_core::TagId;

/// Messages handled by the tag panel
#[derive(Debug, Clone)]
pub enum Message {
    /// User started dragging a tag row (identity by TagId).
    DragStarted(TagId),
    /// Cursor moved while dragging; used to compute drop target (put-before row).
    DragHoverCursor { x: f32, y: f32 },
    /// User released mouse; end drag. Workspace applies reorder (put dragged before target row) if target set.
    DragEnded,
    /// Tag list content bounds (for mapping cursor to row index). row_height/cols define layout; row_content_height is the visible row extent for scroll-into-view (None = use row_height).
    PanelBounds {
        bounds: iced::Rectangle,
        row_height: f32,
        cols: u32,
        /// Height of one row's content for scroll-into-view. None = list (row_height is content). Some = grid (content only, excludes spacing below row).
        row_content_height: Option<f32>,
    },
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
    /// Remove the tag from the store and UI (unselect first, then delete). Only for stored tags.
    DeleteTag(TagId),
    /// Remove the currently selected tag (no-op if none selected).
    DeleteSelectedTag,
    /// Save a snapshot-only tag to the store (add to DB). No-op if tag is already stored.
    SaveTag(TagId),
}
