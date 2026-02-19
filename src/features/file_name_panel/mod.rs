//! File name panel feature: wrapping row of tag chips (draggable, UI-only order) and below it
//! the initial name + extension (no dots). Folder list keeps using [crate::widgets::file_name_display].
//! Drag-and-drop uses our own logic; [dragking](https://github.com/airstrike/dragking) targets iced 0.13.

mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub use state::FileNamePanelState;

/// Layout constants shared by view and state (drop-index grid).
pub const TAG_CHIP_ROW_HEIGHT: f32 = 28.0;
pub const TAG_CHIP_SPACING: f32 = 4.0;
pub const TAG_CHIP_ESTIMATED_WIDTH: f32 = 64.0;
const DRAG_LIFT_PX: f32 = 6.0;
pub const TAG_CHIP_CELL_HEIGHT: f32 = TAG_CHIP_ROW_HEIGHT + DRAG_LIFT_PX;
