//! UI for file workspace: hosts the file name display and panels for the current file (tag panel and more in the future).
//!
//! Only this module knows the layout of the file workspace region: file name on top, search bar, tag panel below.

use iced::widget::{column, container};
use iced::{Element, Length};

use crate::features::tag_panel;
use crate::widgets;

use super::FileWorkspace;

/// Horizontal padding for the file workspace panel (same inset from splitter and window edge).
const PANEL_PADDING_X: f32 = 8.0;

/// Render the file workspace: file name on top, search bar, tag panel below (and later other panels).
/// Caller passes file workspace state, tag panel state, and global tag list; messages are tag panel messages.
pub fn view<'a, S>(
    file_workspace: &'a FileWorkspace<S>,
    tag_panel_state: &'a tag_panel::TagPanelState,
    tag_list: &'a frename_core::TagList<S>,
) -> Element<'a, tag_panel::Message>
where
    S: frename_core::StoredTagStore + Clone,
{
    let color_mapping = file_workspace.tag_color_mapping();
    let file_name = widgets::file_name_display::view_in_panel(
        file_workspace.tag_list().file_snapshot(),
        &color_mapping,
        widgets::file_name_display::DEFAULT_WRAP,
    );
    let search_bar = widgets::search_bar::view(
        tag_list.filter_query(),
        tag_panel::Message::SetFilter,
        || tag_panel::Message::SetFilter(String::new()),
    );
    let tag_panel = tag_panel::view::view(tag_panel_state, file_workspace.file(), tag_list);

    let content = column![file_name, search_bar, tag_panel]
        .spacing(4)
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([0.0, PANEL_PADDING_X])
        .into()
}
