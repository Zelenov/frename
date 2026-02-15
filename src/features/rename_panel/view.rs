//! UI rendering for the rename panel feature

use iced::widget::{checkbox, column, container, mouse_area, scrollable, text};
use iced::{mouse, Element, Length};

use super::{Message, RenamePanelState};

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

    // Scrollable tag list with checkboxes.
    // Each row is wrapped in a mouse_area so the entire row is clickable.
    // The checkbox keeps on_toggle for its own click area; the mouse_area
    // handles clicks on the rest of the row (iced does not double-fire).
    let tag_items: Vec<Element<'_, Message>> = core
        .tags()
        .iter()
        .enumerate()
        .map(|(index, tag)| {
            let row_content = container(
                checkbox(tag.is_checked())
                    .label(tag.tag())
                    .on_toggle(move |_| Message::ToggleTag(index))
                    .size(16)
                    .text_size(14),
            )
            .padding([4, 8])
            .width(Length::Fill);

            mouse_area(row_content)
                .on_press(Message::ToggleTag(index))
                .interaction(mouse::Interaction::Pointer)
                .into()
        })
        .collect();

    let tag_column = column(tag_items).width(Length::Fill);

    let tag_list = scrollable(tag_column).height(Length::Fill);

    // Combine: file name on top, tags below
    let panel = column![file_name_display, tag_list]
        .spacing(4)
        .width(Length::Fill)
        .height(Length::Fill);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([4, 4])
        .into()
}
