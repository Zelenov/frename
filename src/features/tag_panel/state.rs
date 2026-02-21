//! State for the tag panel feature

use iced::{event, mouse, Rectangle, Subscription};

use frename_core::{StoredTagStore, TagId, TagList};

use super::Message;

/// Row height for cursor-to-row mapping. Must match view::TAG_ROW_HEIGHT.
const TAG_ROW_HEIGHT: f32 = 28.0;

/// Tag panel state (UI-only; the selected file and tag toggles live in the file workspace).
#[derive(Default)]
pub struct TagPanelState {
    /// Tag list cursor: which tag row is selected (for keyboard Up/Down, delete button, Delete key).
    selected_tag_id: Option<TagId>,
    /// Tag being dragged (for reorder).
    dragging_tag_id: Option<TagId>,
    /// Row index where the dragged tag will be dropped (put before the tag at this index). None when outside list.
    drop_target_index: Option<usize>,
    /// Tag list content bounds (from BoundsReporter) and scroll Y for mapping cursor to row index.
    bounds: Option<Rectangle>,
    scroll_y: f32,
}

const PADDING: f32 = 4.0;

impl TagPanelState {
    /// Which tag is currently selected in the list (None = no selection).
    pub fn selected_tag_id(&self) -> Option<TagId> {
        self.selected_tag_id
    }

    /// Set the selected tag (called from folder_workspace when handling SelectUp/SelectDown/ToggleTag).
    pub fn set_selected(&mut self, id: Option<TagId>) {
        self.selected_tag_id = id;
    }

    /// Tag ID being dragged. None when not dragging.
    pub fn dragging_tag_id(&self) -> Option<TagId> {
        self.dragging_tag_id
    }

    /// Row index to drop at (put dragged tag before the tag at this index). None when outside or not dragging.
    pub fn drop_target_index(&self) -> Option<usize> {
        self.drop_target_index
    }

    /// Subscription for drag: cursor and release while dragging.
    pub fn subscription(&self) -> Subscription<Message> {
        if self.dragging_tag_id.is_none() {
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

    /// Handle tag panel messages. Pass tag_list for drag drop target resolution.
    pub fn update<S: StoredTagStore + Clone>(&mut self, message: &Message, tag_list: &TagList<S>) {
        match message {
            Message::DragStarted(tag_id) => {
                self.dragging_tag_id = Some(*tag_id);
                self.drop_target_index = tag_list
                    .filtered_display_tag_ids()
                    .iter()
                    .position(|id| id == tag_id)
                    .or(Some(0));
            }
            Message::DragHoverCursor { x, y } => {
                self.drop_target_index = self.row_index_at_cursor(*x, *y, tag_list.filtered_display_tag_ids().len());
            }
            Message::DragEnded => {
                self.dragging_tag_id = None;
                self.drop_target_index = None;
            }
            Message::PanelBounds(bounds) => {
                self.bounds = Some(*bounds);
            }
            Message::TagListScrolled { scroll_y, .. } => {
                self.scroll_y = *scroll_y;
            }
            _ => {}
        }
    }

    /// Row index under the cursor (0..len). None if cursor outside list.
    fn row_index_at_cursor(&self, cursor_x: f32, cursor_y: f32, row_count: usize) -> Option<usize> {
        let bounds = self.bounds?;
        if row_count == 0 {
            return None;
        }
        let content_y = bounds.y + PADDING;
        let row_height = TAG_ROW_HEIGHT;
        if cursor_x < bounds.x
            || cursor_x >= bounds.x + bounds.width
            || cursor_y < content_y
        {
            return None;
        }
        let rel_y = cursor_y - content_y + self.scroll_y;
        let row = (rel_y / row_height).floor() as usize;
        Some(row.min(row_count.saturating_sub(1)))
    }
}
