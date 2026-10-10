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

use frename_core::ai::block::SuggestedTag;
use frename_core::{Segment, StoredTagStore, TagList};

use crate::ui::tokens::{CHIP_DRAG_LIFT, CHIP_DROP_ESTIMATE_WIDTH, CHIP_HEIGHT, SPACE_XS};

/// Layout shared by view and state (the drop-index grid): chips `TAG_CHIP_SPACING` apart, each in
/// a cell that leaves room for the lift of a dragged chip.
pub const TAG_CHIP_SPACING: f32 = SPACE_XS;
pub const TAG_CHIP_ESTIMATED_WIDTH: f32 = CHIP_DROP_ESTIMATE_WIDTH;
pub const TAG_CHIP_CELL_HEIGHT: f32 = CHIP_HEIGHT + CHIP_DRAG_LIFT;

/// Where the open clip's edits are, for the line at the foot of the file name card (§13.5.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStatus {
    /// The clip matches its file: nothing is waiting.
    Saved,
    /// The edits wait in the recovery journal and reach the file when the clip is left.
    InRecovery,
}

/// What the line says, from two facts about the open clip: its edits differ from its file
/// (`unsaved`), and the recovery journal holds exactly these edits (`in_journal`). `None` while
/// edits are not in the journal yet (a second at most, or a write that failed): saying "saved"
/// then would be false, and "saved to recovery" too.
pub fn save_status(unsaved: bool, in_journal: bool) -> Option<SaveStatus> {
    match (unsaved, in_journal) {
        (false, _) => Some(SaveStatus::Saved),
        (true, true) => Some(SaveStatus::InRecovery),
        (true, false) => None,
    }
}

/// The In and Out the open clip's AI description suggests, while they differ from the clip's
/// own: once applied, or set the same by hand, there is nothing left to offer.
pub fn suggested_in_out<S>(tag_list: &TagList<S>) -> Option<Segment>
where
    S: StoredTagStore + Clone,
{
    let current = Segment {
        start: tag_list.segment_start_secs(),
        end: tag_list.segment_end_secs(),
    };
    frename_core::ai::block::pending_in_out(tag_list.comment(), current)
}

/// The folder's tags the open clip's AI description suggests that the clip does not have yet,
/// most likely first: once added (or added by hand), a tag is no longer offered, and a tag the
/// folder no longer has is not offered at all.
pub fn pending_tag_suggestions<S>(tag_list: &TagList<S>) -> Vec<SuggestedTag>
where
    S: StoredTagStore + Clone,
{
    frename_core::ai::block::suggested_tags(tag_list.comment())
        .into_iter()
        .filter(|suggested| {
            tag_list
                .tag_id_by_name(&suggested.name)
                .and_then(|id| tag_list.get_tag(id))
                .is_some_and(|tag| tag.is_stored() && !tag.is_checked())
        })
        .collect()
}

/// The tags the open clip's AI description noticed that the folder does not have, leaving out
/// any the folder has got since.
pub fn tag_ideas<S>(tag_list: &TagList<S>) -> Vec<String>
where
    S: StoredTagStore + Clone,
{
    frename_core::ai::block::tag_ideas(tag_list.comment())
        .into_iter()
        .filter(|idea| {
            !tag_list
                .tag_id_by_name(idea)
                .and_then(|id| tag_list.get_tag(id))
                .is_some_and(|tag| tag.is_stored())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clip_that_matches_its_file_is_saved_whatever_the_journal_holds() {
        assert_eq!(save_status(false, false), Some(SaveStatus::Saved));
        assert_eq!(save_status(false, true), Some(SaveStatus::Saved));
    }

    #[test]
    fn edits_held_only_by_the_journal_are_in_recovery() {
        assert_eq!(save_status(true, true), Some(SaveStatus::InRecovery));
    }

    #[test]
    fn edits_the_journal_does_not_hold_yet_say_nothing() {
        assert_eq!(save_status(true, false), None);
    }
}
