//! UI for the folder list. Only this module knows the list is scrollable and how rows look.
//!
//! Receives only data (directory, loading, tag color mapping) from workspace; selection from directory; no parent knows our layout or widgets.

use iced::widget::{column, container, mouse_area, row, scrollable, text};
use iced::{mouse, Background, Element, Length};

use crate::theme;
use crate::widgets;

use super::Message;

const FOLDER_LIST_SCROLLABLE_ID: &str = "folder-file-list";

/// Render the folder panel: a scrollable list of file names (tag chips + name.extension, no wrap).
/// Selection comes from the directory; view emits SelectFile/Previous/Next.
pub fn view<'a>(
    directory: Option<&'a crate::features::folder_workspace::Directory>,
    loading: bool,
    tag_color_mapping: frename_core::TagColorMapping,
) -> Element<'a, Message> {
    let placeholder_icon = |icon: &'static str| {
        container(text(icon).size(48).color(theme::TEXT_MUTED))
            .padding([8, 8])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(theme::BG_PANEL)),
                ..Default::default()
            })
    };

    if loading {
        return placeholder_icon("⏳").into();
    }

    let Some(dir) = directory else {
        return placeholder_icon("📂").into();
    };

    if dir.is_empty() {
        return placeholder_icon("📭").into();
    }

    let selected_index = dir.selected_index();

    let items: Vec<Element<'_, Message>> = dir
        .files()
        .iter()
        .enumerate()
        .map(|(index, file_info)| {
            let is_selected = selected_index == Some(index);
            let name_display = widgets::file_name_display::view(
                file_info.snapshot().clone(),
                &tag_color_mapping,
                false,
            );
            let row_content = row![name_display].align_y(iced::Alignment::Center);

            let row = container(row_content)
                .padding([4, 8])
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                    background: Some(Background::Color(if is_selected {
                        theme::ACCENT_SELECTED
                    } else {
                        iced::Color::TRANSPARENT
                    })),
                    ..Default::default()
                });

            mouse_area(row)
                .on_press(Message::SelectFile(index))
                .interaction(mouse::Interaction::Pointer)
                .into()
        })
        .collect();

    let list = scrollable(column(items).width(Length::Fill))
        .id(iced::widget::Id::new(FOLDER_LIST_SCROLLABLE_ID))
        .height(Length::Fill)
        .style(theme::dark_scrollable_style);

    container(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_PANEL)),
            ..Default::default()
        })
        .into()
}
