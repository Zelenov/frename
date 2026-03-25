//! UI for the folder list. Only this module knows the list is scrollable and how rows look.
//!
//! Receives only data (directory, loading, tag color mapping) from workspace; selection from directory; no parent knows our layout or widgets.

use iced::widget::{column, container, mouse_area, row, scrollable, text, tooltip};
use iced::{mouse, Element, Length};

use crate::theme;
use crate::widgets;

use super::{FOLDER_LIST_SCROLLABLE_ID, FOLDER_ROW_HEIGHT};
use super::Message;

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
            .style(theme::panel_container_style)
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
        .files_in_order()
        .enumerate()
        .map(|(index, file_info)| {
            let is_selected = selected_index == Some(index);
            let name_display = widgets::file_name_display::view(
                file_info.snapshot().clone(),
                &tag_color_mapping,
                false,
            );

            let copy_btn: Element<'_, Message> = tooltip(
                mouse_area(
                    container(text("⎘").size(14).color(crate::theme::TEXT_MUTED))
                        .center_x(Length::Fixed(28.0))
                        .center_y(Length::Fill),
                )
                .on_press(Message::CopyTagsFrom(file_info.id()))
                .interaction(mouse::Interaction::Pointer),
                text("Paste tags"),
                tooltip::Position::Bottom,
            )
            .into();

            let comment = file_info.comment();
            let comment_icon: Element<'_, Message> = if !comment.is_empty() {
                tooltip(
                    container(text("💬").size(11).color(crate::theme::TEXT_MUTED))
                        .center_x(Length::Fixed(22.0))
                        .center_y(Length::Fill),
                    text(comment),
                    tooltip::Position::Right,
                )
                .into()
            } else {
                container(iced::widget::Space::new())
                    .width(Length::Fixed(22.0))
                    .into()
            };

            let name_area: Element<'_, Message> = mouse_area(
                container(name_display)
                    .padding(iced::Padding { top: 4.0, right: 8.0, bottom: 4.0, left: 0.0 })
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_y(Length::Fill),
            )
            .on_press(Message::SelectFile(index))
            .interaction(mouse::Interaction::Pointer)
            .into();

            container(
                row![copy_btn, comment_icon, name_area]
                    .align_y(iced::Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fixed(FOLDER_ROW_HEIGHT))
            .style(move |theme: &iced::Theme| theme::selectable_row_style(theme, is_selected))
            .into()
        })
        .collect();

    let list = scrollable(column(items).width(Length::Fill))
        .id(iced::widget::Id::new(FOLDER_LIST_SCROLLABLE_ID))
        .height(Length::Fill)
        .on_scroll(|viewport| {
            let offset = viewport.absolute_offset();
            Message::Scrolled {
                scroll_y: offset.y,
                viewport_height: viewport.bounds().height,
            }
        })
        .style(theme::dark_scrollable_style);

    container(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_container_style)
        .into()
}
