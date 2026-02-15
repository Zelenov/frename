//! UI rendering for the rename panel feature

use iced::widget::{checkbox, column, container, mouse_area, scrollable, text};
use iced::{mouse, Element, Length};

use frename_core::File;

use super::{Message, RenamePanelState};

/// Render the full rename panel: file name display on top, tag list below.
/// `selected_file` is the currently selected file from the directory (tags and name are shown/edited there).
pub fn view<'a>(
    _state: &'a RenamePanelState,
    selected_file: Option<&'a File>,
) -> Element<'a, Message> {
    let Some(file) = selected_file else {
        return container(
            text("Select a file")
                .size(14)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .padding([8, 8])
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    };

    // File name display at the top
    let file_name_display = {
        let label = if file.file_name().is_empty() {
            "(no tags selected)"
        } else {
            file.file_name()
        };

        container(text(label).size(14).wrapping(text::Wrapping::WordOrGlyph))
            .padding([8, 8])
            .width(Length::Fill)
    };

    // Scrollable tag list with checkboxes
    let tag_items: Vec<Element<'_, Message>> = file
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
