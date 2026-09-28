//! Tag panel feature: the state and messages of the tags of the open file. The tag grid
//! (`features::tag_grid`) and the starred strip draw them.

#![allow(dead_code)]

mod messages;
mod state;

pub use messages::Message;
pub use state::TagPanelState;

/// Widget id for the tag list scrollable (for scroll-to-selection operations).
pub const TAG_LIST_SCROLLABLE_ID: &str = "tag-list-scrollable";

/// The grid's sizes are tokens; the view lays rows out by the stride and the scroll-into-view
/// arithmetic counts by it.
pub use crate::ui::tokens::{GRID_CELL_HEIGHT, GRID_CELL_INSET, GRID_GAP, GRID_ROW_STRIDE};
