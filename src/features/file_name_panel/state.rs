//! State for the file name panel feature (wrap=true display in file workspace).
//! Drag reorder state is UI-only; does not persist to TagList.
//! Handles its own messages (bounds, drag, drop-index).

use iced::{event, mouse, Rectangle, Subscription};

use frename_core::{StoredTagStore, TagId, TagList};

use super::{
    Message, TAG_CHIP_CELL_HEIGHT, TAG_CHIP_ESTIMATED_WIDTH, TAG_CHIP_SPACING,
};

#[derive(Default)]
pub struct FileNamePanelState {
    /// Tag ID being dragged (identity across list changes).
    dragging_tag_id: Option<TagId>,
    /// Index where the dragged chip would drop (for display order).
    drop_target_index: Option<usize>,
    /// Panel content bounds (from BoundsReporter) for mapping cursor to drop index.
    bounds: Option<Rectangle>,
}

const PADDING: f32 = 8.0;

impl FileNamePanelState {
    pub fn is_dragging(&self) -> bool {
        self.dragging_tag_id.is_some()
    }

    /// Tag ID being dragged (for view to resolve to current index when list changes).
    pub fn dragging_tag_id(&self) -> Option<TagId> {
        self.dragging_tag_id
    }

    pub fn drop_target_index(&self) -> Option<usize> {
        self.drop_target_index
    }

    fn set_dragging(&mut self, tag_id: TagId, initial_index: usize) {
        self.dragging_tag_id = Some(tag_id);
        self.drop_target_index = Some(initial_index);
    }

    /// Current index of the dragged tag in the tag list's checked order (same as file name chips).
    fn resolved_dragging_index<S: StoredTagStore + Clone>(&self, tag_list: &TagList<S>) -> Option<usize> {
        self.dragging_tag_id
            .and_then(|id| tag_list.checked_index_of(id))
    }

    fn set_drop_target_index(&mut self, index: Option<usize>) {
        self.drop_target_index = index;
    }

    fn clear_drag(&mut self) {
        self.dragging_tag_id = None;
        self.drop_target_index = None;
    }

    /// Subscription for drag: cursor position and release while dragging. Inactive when not dragging.
    pub fn subscription(&self) -> Subscription<Message> {
        if !self.is_dragging() {
            return Subscription::none();
        }
        event::listen_with(|ev, _status, _id| match ev {
            iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(Message::DragHoverCursor {
                    x: position.x,
                    y: position.y,
                })
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(Message::DragEnded)
            }
            _ => None,
        })
    }

    /// Handle file name panel messages. Checked tag list is read from `tag_list` (via snapshot for count).
    pub fn update<S: StoredTagStore + Clone>(&mut self, message: Message, tag_list: &TagList<S>) {
        let tag_count = tag_list.file_snapshot().tags().len();
        match message {
            Message::DragStarted { tag_id, initial_index } => {
                self.set_dragging(tag_id, initial_index.min(tag_count.saturating_sub(1)));
            }
            Message::DragHoverCursor { x, y } => {
                let index = self
                    .drop_index(x, y, tag_count)
                    .or_else(|| self.resolved_dragging_index(tag_list));
                if let Some(idx) = index {
                    self.set_drop_target_index(Some(idx.min(tag_count)));
                } else {
                    // Dragged tag no longer in list (e.g. unchecked); end drag.
                    self.clear_drag();
                }
            }
            Message::DragEnded => self.clear_drag(),
            Message::PanelBounds(bounds) => self.bounds = Some(bounds),
        }
    }

    /// Chip index from cursor for wrapping tag row (left→right, top→bottom).
    /// Returns None when cursor is outside the panel content rectangle (caller uses start position).
    fn drop_index(&self, cursor_x: f32, cursor_y: f32, tag_count: usize) -> Option<usize> {
        let bounds = self.bounds?;
        if tag_count == 0 {
            return None;
        }
        let content_x = bounds.x + PADDING;
        let content_y = bounds.y + PADDING;
        let col_total = TAG_CHIP_ESTIMATED_WIDTH + TAG_CHIP_SPACING;
        let row_total = TAG_CHIP_CELL_HEIGHT + TAG_CHIP_SPACING;
        let content_width = (bounds.width - 2.0 * PADDING).max(col_total);
        let cols_per_row = (content_width / col_total).floor() as usize;
        if cols_per_row == 0 {
            return None;
        }
        let rows = (tag_count + cols_per_row - 1) / cols_per_row;
        let content_height = (rows as f32) * row_total;
        // Outside panel => None so caller keeps drop target at drag start.
        if cursor_x < content_x
            || cursor_x >= content_x + content_width
            || cursor_y < content_y
            || cursor_y >= content_y + content_height
        {
            return None;
        }
        let rel_x = cursor_x - content_x;
        let rel_y = cursor_y - content_y;
        let col = (rel_x / col_total).floor() as usize;
        let row = (rel_y / row_total).floor() as usize;
        let index = row * cols_per_row + col;
        Some(index.min(tag_count))
    }
}
