//! "Tag commented videos": puts the commented tag (see the settings) on each file with a
//! comment and takes it off each file without one. No options of its own: the tag is named and
//! turned on or off in the settings, which its panel opens.

use std::path::Path;

use frename_core::{FileTagger, MoveOutcome};
use iced::Element;

use super::super::page::{self, Change};
use super::super::ItemResult;
use super::ActionMessage;
use crate::ui::button;
use crate::ui::layout::{self, NoticeKind};

pub fn label() -> String {
    fl!("batch-action-tag-commented")
}

/// Runs only while the commented tag is turned on in the settings.
pub fn operation() -> Option<super::Operation> {
    frename_core::commented_tag().map(|_| super::Operation::TagCommented)
}

/// What the action does with the tag from the settings, or, when the tag is turned off there,
/// the notice with the button that turns it on (the Saving page of Settings, at the tag row).
pub fn view<'a>() -> Element<'a, ActionMessage> {
    let (hint, part) = match frename_core::commented_tag() {
        Some(tag) => (
            fl!("batch-action-tag-commented-hint", tag = tag.clone()),
            page::linked_row(
                fl!("batch-option-tag"),
                tag,
                fl!("batch-ai-change"),
                ActionMessage::OpenSettings,
            ),
        ),
        None => (
            fl!("batch-action-tag-commented-hint-off"),
            layout::notice(
                NoticeKind::Warning,
                fl!("batch-action-tag-commented-off"),
                None,
                [button::secondary(fl!("batch-action-tag-commented-choose"))
                    .on_press(ActionMessage::OpenSettings)
                    .into()],
            ),
        ),
    };
    page::page(label(), hint, &[Change::Renames], [part])
}

/// Bring the commented tag of the file at `path` in line with its comment.
pub fn run(path: &Path) -> ItemResult {
    // Turned off in the settings since the job started: nothing to do.
    let outcome = match frename_core::commented_tag() {
        Some(tag) => FileTagger::sync_commented_tag(path, &tag),
        None => MoveOutcome::NothingToMove,
    };
    super::item_result(outcome)
}
