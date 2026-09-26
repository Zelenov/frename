//! "Apply tag spacing": renames each file to the tag spacing chosen in the settings, with or
//! without a space after each tag (`Food. Goat. clip.mp4` / `Food.Goat.clip.mp4`). No options
//! of its own: the spacing is a setting, which its panel opens.

use std::path::Path;

use frename_core::FileTagger;
use iced::widget::{button, row, text};
use iced::Element;

use super::super::ItemResult;
use super::ActionMessage;
use crate::theme;

pub fn label() -> String {
    fl!("batch-action-respace-tags")
}

pub fn view<'a>() -> Element<'a, ActionMessage> {
    let (hint, status) = if frename_core::space_after_tags() {
        (
            fl!("batch-action-respace-tags-hint-space"),
            fl!("batch-action-respace-tags-status-space"),
        )
    } else {
        (
            fl!("batch-action-respace-tags-hint-no-space"),
            fl!("batch-action-respace-tags-status-no-space"),
        )
    };
    let settings = row![
        text(status).size(13),
        button(text(fl!("batch-action-respace-tags-settings")).size(12))
            .on_press(ActionMessage::OpenSettings)
            .padding([3, 10])
            .style(theme::icon_button_style(true)),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);
    super::panel(label(), hint, settings.into())
}

/// Rename the file at `path` to the chosen tag spacing.
pub fn run(path: &Path) -> ItemResult {
    super::item_result(FileTagger::respace_tags(path))
}
