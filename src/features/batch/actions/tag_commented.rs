//! "Tag commented videos": puts the commented tag (see the settings) on each file with a
//! comment and takes it off each file without one. No options of its own: the tag is named and
//! turned on or off in the settings, which its panel opens.

use std::path::Path;

use frename_core::{FileTagger, MoveOutcome};
use iced::widget::{button, row, text};
use iced::Element;

use super::super::ItemResult;
use super::ActionMessage;
use crate::theme;

pub fn label() -> String {
    fl!("batch-action-tag-commented")
}

/// Runs only while the commented tag is turned on in the settings.
pub fn operation() -> Option<super::Operation> {
    frename_core::commented_tag().map(|_| super::Operation::TagCommented)
}

/// What the action does with the tag from the settings, or that the tag is turned off there,
/// with a button to the settings, where the tag is named and turned on or off.
pub fn view<'a>() -> Element<'a, ActionMessage> {
    let tag = frename_core::commented_tag();
    let hint = match &tag {
        Some(tag) => fl!("batch-action-tag-commented-hint", tag = tag.as_str()),
        None => fl!("batch-action-tag-commented-hint-off"),
    };
    let status = match tag {
        Some(tag) => text(fl!("batch-action-tag-commented-status", tag = tag)).size(13),
        None => text(fl!("batch-action-tag-commented-status-off"))
            .size(13)
            .color(theme::ERROR),
    };
    let settings = row![
        status,
        button(text(fl!("batch-action-tag-commented-settings")).size(12))
            .on_press(ActionMessage::OpenSettings)
            .padding([3, 10])
            .style(theme::icon_button_style(true)),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);
    super::panel(label(), hint, settings.into())
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
