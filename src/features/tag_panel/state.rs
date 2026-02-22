//! State for the tag panel feature

use iced::Rectangle;

use frename_core::{StoredTagStore, TagId, TagList};

use super::Message;

const DEFAULT_ROW_HEIGHT: f32 = 28.0;
const DEFAULT_COLS: u32 = 1;

/// Tag panel state (UI-only; the selected file and tag toggles live in the file workspace).
pub struct TagPanelState {
    /// Tag list cursor: which tag row is selected (for keyboard Up/Down, delete button, Delete key).
    selected_tag_id: Option<TagId>,
    /// Tag list content bounds (from BoundsReporter) and scroll Y for mapping cursor to row index.
    bounds: Option<Rectangle>,
    scroll_y: f32,
    /// Row height in pixels / stride (list or grid). Used for cursor→index mapping and row_top.
    row_height: f32,
    /// Number of columns (1 = list, 2 = grid).
    cols: u32,
    /// Height of row content for scroll-into-view. None = use row_height (list). Some = grid content only.
    row_content_height: Option<f32>,
}

impl Default for TagPanelState {
    fn default() -> Self {
        Self {
            selected_tag_id: None,
            bounds: None,
            scroll_y: 0.0,
            row_height: DEFAULT_ROW_HEIGHT,
            cols: DEFAULT_COLS,
            row_content_height: None,
        }
    }
}

const PADDING: f32 = 4.0;

impl TagPanelState {
    /// Panel content bounds when set by BoundsReporter (for dynamic column count in grid).
    pub fn panel_bounds(&self) -> Option<Rectangle> {
        self.bounds
    }

    /// Row height in pixels (list or grid). Used for scroll-into-view.
    pub fn row_height(&self) -> f32 {
        self.row_height
    }

    /// Number of columns (1 = list, ≥1 = grid). Used for selection step and scroll-into-view.
    pub fn cols(&self) -> u32 {
        self.cols.max(1)
    }

    /// Height of one row's content for scroll-into-view. None = use row_height (list). Some = grid content only.
    pub fn row_content_height(&self) -> Option<f32> {
        self.row_content_height
    }

    /// Which tag is currently selected in the list (None = no selection).
    pub fn selected_tag_id(&self) -> Option<TagId> {
        self.selected_tag_id
    }

    /// Set the selected tag (called from folder_workspace when handling SelectUp/SelectDown/ToggleTag).
    pub fn set_selected(&mut self, id: Option<TagId>) {
        self.selected_tag_id = id;
    }

    /// Handle tag panel messages.
    pub fn update<S: StoredTagStore + Clone>(&mut self, message: &Message, _tag_list: &TagList<S>) {
        match message {
            Message::PanelBounds {
                bounds,
                row_height,
                cols,
                row_content_height,
            } => {
                self.bounds = Some(*bounds);
                self.row_height = *row_height;
                self.cols = (*cols).max(1);
                self.row_content_height = *row_content_height;
            }
            Message::TagListScrolled { scroll_y, .. } => {
                self.scroll_y = *scroll_y;
            }
            _ => {}
        }
    }

    /// Row index under the cursor (0..len). None if cursor outside list. Uses stored row_height and cols (list=1, grid=2).
    pub fn row_index_at_cursor(&self, cursor_x: f32, cursor_y: f32, row_count: usize) -> Option<usize> {
        let bounds = self.bounds?;
        if row_count == 0 {
            return None;
        }
        let content_y = bounds.y + PADDING;
        let row_height = if self.row_height > 0.0 {
            self.row_height
        } else {
            DEFAULT_ROW_HEIGHT
        };
        let cols = self.cols.max(1) as usize;
        if cursor_x < bounds.x
            || cursor_x >= bounds.x + bounds.width
            || cursor_y < content_y
        {
            return None;
        }
        let rel_y = cursor_y - content_y + self.scroll_y;
        let rel_x = cursor_x - bounds.x;
        let row = (rel_y / row_height).floor() as usize;
        let col_width = bounds.width / cols as f32;
        let col = (rel_x / col_width).floor().min((cols - 1) as f32) as usize;
        let index = row * cols + col;
        Some(index.min(row_count.saturating_sub(1)))
    }
}
