//! UI for file workspace: hosts the file name display and panels for the current file (tag panel and more in the future).
//!
//! Only this module knows the layout of the file workspace region: file name on top, tag panel below.

use iced::widget::{column, container};
use iced::{Element, Length};

use crate::features::tag_panel;

use super::file_name_display;
use super::FileWorkspace;

/// Render the file workspace: file name display on top, tag panel below (and later other panels).
/// Caller passes file workspace state and tag panel state; messages are tag panel messages.
pub fn view<'a>(
    file_workspace: &'a FileWorkspace,
    tag_panel_state: &'a tag_panel::TagPanelState,
) -> Element<'a, tag_panel::Message> {
    let file_name = file_name_display::view(file_workspace.file());
    let tag_panel = tag_panel::view::view(tag_panel_state, file_workspace.file());

    let content = column![file_name, tag_panel]
        .spacing(4)
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
