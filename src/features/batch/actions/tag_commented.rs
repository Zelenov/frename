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

pub const LABEL: &str = "Tag commented videos";

/// Runs only while the commented tag is turned on in the settings.
pub fn operation() -> Option<super::Operation> {
    frename_core::commented_tag().map(|_| super::Operation::TagCommented)
}

/// What the action does with the tag from the settings, or that the tag is turned off there,
/// with a button to the settings, where the tag is named and turned on or off.
pub fn view<'a>() -> Element<'a, ActionMessage> {
    let tag = frename_core::commented_tag();
    let hint = match &tag {
        Some(tag) => format!(
            "Adds the “{tag}” tag to each checked video with a comment of yours (AI descriptions do \
             not count) and removes it from \
             those without one. Files whose tag changes are renamed."
        ),
        None => "Adds the tag for videos with a comment to each checked video with a comment of yours and removes it \
                 from those without one. The tag is turned off in the settings."
            .to_string(),
    };
    let status = match tag {
        Some(tag) => text(format!("Tag: {tag}")).size(13),
        None => text("Tag: off").size(13).color(theme::ERROR),
    };
    let settings = row![
        status,
        button(text("Tag settings…").size(12))
            .on_press(ActionMessage::OpenSettings)
            .padding([3, 10])
            .style(theme::icon_button_style(true)),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);
    super::panel(LABEL, hint, settings.into())
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
