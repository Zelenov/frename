//! UI rendering for the rename panel feature

use iced::widget::{checkbox, column, container, scrollable, text};
use iced::{Element, Length};

use super::{Message, RenamePanelState};

const PANEL_WIDTH: f32 = 200.0;

/// Render the full rename panel: file name display on top, tag list below.
pub fn view(state: &RenamePanelState) -> Element<'_, Message> {
    let core = state.rename_core();

    // File name display at the top
    let file_name_display = {
        let label = if core.file_name().is_empty() {
            "(no tags selected)"
        } else {
            core.file_name()
        };

        container(text(label).size(14).wrapping(text::Wrapping::WordOrGlyph))
            .padding([8, 8])
            .width(Length::Fill)
    };

    // Scrollable tag list with checkboxes
    let tag_items: Vec<Element<'_, Message>> = core
        .tags()
        .iter()
        .enumerate()
        .map(|(index, tag)| {
            container(
                checkbox(tag.is_checked())
                    .label(tag.tag())
                    .on_toggle(move |_| Message::ToggleTag(index))
                    .size(16)
                    .text_size(14),
            )
            .padding([4, 8])
            .width(Length::Fill)
            .into()
        })
        .collect();

    let tag_column = column(tag_items).spacing(2).width(Length::Fill);

    let tag_list = scrollable(tag_column).height(Length::Fill);

    // Combine: file name on top, tags below
    let panel = column![file_name_display, tag_list]
        .spacing(4)
        .width(Length::Fill)
        .height(Length::Fill);

    container(panel)
        .width(PANEL_WIDTH)
        .height(Length::Fill)
        .padding([4, 4])
        .into()
}
