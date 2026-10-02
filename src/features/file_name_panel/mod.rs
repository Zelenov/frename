//! File name panel feature: wrapping row of tag chips (draggable, UI-only order) and below it
//! the initial name + extension (no dots). Folder list keeps using [crate::widgets::file_name_display].
//! Drag-and-drop uses our own logic; [dragking](https://github.com/airstrike/dragking) targets iced 0.13.

mod chips_panel;
mod file_name_line;
mod messages;
mod state;
mod trash_zone;
pub mod view;

pub use messages::Message;
pub use state::FileNamePanelState;

use frename_core::{Segment, StoredTagStore, TagList};

use crate::ui::tokens::{CHIP_DRAG_LIFT, CHIP_DROP_ESTIMATE_WIDTH, CHIP_HEIGHT, SPACE_XS};

/// Layout shared by view and state (the drop-index grid): chips `TAG_CHIP_SPACING` apart, each in
/// a cell that leaves room for the lift of a dragged chip.
pub const TAG_CHIP_SPACING: f32 = SPACE_XS;
pub const TAG_CHIP_ESTIMATED_WIDTH: f32 = CHIP_DROP_ESTIMATE_WIDTH;
pub const TAG_CHIP_CELL_HEIGHT: f32 = CHIP_HEIGHT + CHIP_DRAG_LIFT;

/// The In and Out the open clip's AI description suggests, while they differ from the clip's
/// own: once applied, or set the same by hand, there is nothing left to offer.
pub fn suggested_in_out<S>(tag_list: &TagList<S>) -> Option<Segment>
where
    S: StoredTagStore + Clone,
{
    let suggested = frename_core::ai::block::suggested_in_out(tag_list.comment())?;
    let current = Segment {
        start: tag_list.segment_start_secs(),
        end: tag_list.segment_end_secs(),
    };
    (suggested != current).then_some(suggested)
}
