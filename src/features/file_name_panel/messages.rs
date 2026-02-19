//! Messages for file name panel feature (wrap=true display in file workspace).
//! Drag reorder is UI-only; does not change TagList or stored order.

use frename_core::TagId;

#[derive(Debug, Clone)]
pub enum Message {
    /// User started dragging the tag chip (identity by TagId so list changes don't desync).
    DragStarted {
        tag_id: TagId,
        initial_index: usize,
    },
    /// Cursor moved while dragging; used to compute drop slot (panel bounds set by BoundsReporter).
    DragHoverCursor { x: f32, y: f32 },
    /// User released mouse; end drag (order not persisted).
    DragEnded,
    /// Panel content bounds on screen (for mapping cursor to chip index).
    PanelBounds(iced::Rectangle),
}
