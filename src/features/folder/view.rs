//! UI for the folder list. Only this module knows the list is scrollable and how rows look.
//!
//! Receives only data (directory, loading, tag color mapping) from workspace; selection from directory; no parent knows our layout or widgets.

use iced::widget::{column, container, mouse_area, row, scrollable, text, text_input, tooltip};
use iced::{mouse, Element, Length};

use crate::tag_colors::TagPalette;
use crate::theme;
use crate::widgets;
use crate::widgets::search_bar::FILE_SEARCH_BAR_INPUT_ID;

use super::{InlineRename, FOLDER_LIST_SCROLLABLE_ID, FOLDER_RENAME_INPUT_ID, FOLDER_ROW_HEIGHT};
use super::Message;

const SUBTITLES_MARKER_WIDTH: f32 = 28.0;

/// Render the folder panel: a scrollable list of file names (tag chips + name.extension, no wrap).
/// Selection comes from the directory; view emits SelectFile/Previous/Next.
pub fn view<'a>(
    directory: Option<&'a crate::features::folder_workspace::Directory>,
    loading: bool,
    tag_color_mapping: &frename_core::TagColorMapping,
    tag_palette: TagPalette,
    rename: Option<&'a InlineRename>,
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
                file_info.snapshot(),
                tag_color_mapping,
                tag_palette,
                false,
            );

            let subtitles_icon: Element<'_, Message> = if file_info.has_subtitles() {
                tooltip(
                    container(text("SRT").size(9).color(theme::ACCENT))
                        .center_x(Length::Fixed(SUBTITLES_MARKER_WIDTH))
                        .center_y(Length::Fill),
                    container(text("Has subtitles")).padding([2, 6]).style(theme::elevated_container_style),
                    tooltip::Position::Right,
                )
                .into()
            } else {
                container(iced::widget::Space::new())
                    .width(Length::Fixed(SUBTITLES_MARKER_WIDTH))
                    .into()
            };

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

            let editing = rename.filter(|r| r.id == file_info.id());
            let name_area: Element<'_, Message> = if let Some(rename) = editing {
                rename_editor(rename)
            } else {
                mouse_area(
                    container(name_display)
                        .padding(iced::Padding { top: 4.0, right: 8.0, bottom: 4.0, left: 0.0 })
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_y(Length::Fill),
                )
                .on_press(Message::SelectFile(index))
                .on_double_click(Message::StartRename(index))
                .interaction(mouse::Interaction::Pointer)
                .into()
            };

            container(
                row![subtitles_icon, comment_icon, name_area]
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

    // A filter hides everything: say so instead of showing an empty scrollable.
    let body: Element<'_, Message> = if items.is_empty() {
        let icon = if dir.name_filter().trim().is_empty() { "✓" } else { "🔍" };
        container(text(icon).size(48).color(theme::TEXT_MUTED))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    } else {
        scrollable(column(items).width(Length::Fill))
            .id(iced::widget::Id::new(FOLDER_LIST_SCROLLABLE_ID))
            .height(Length::Fill)
            // Scrollbar beside the rows, not over them: long names and the rename field
            // would otherwise run underneath it.
            .spacing(2)
            .on_scroll(|viewport| {
                let offset = viewport.absolute_offset();
                Message::Scrolled {
                    scroll_y: offset.y,
                    viewport_height: viewport.bounds().height,
                }
            })
            .style(theme::dark_scrollable_style)
            .into()
    };

    let search = widgets::search_bar::view(
        FILE_SEARCH_BAR_INPUT_ID,
        dir.name_filter(),
        Message::SetNameFilter,
        || Message::SetNameFilter(String::new()),
        None::<fn(String) -> Message>,
    );

    container(column![search, body].spacing(4))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_container_style)
        .into()
}

/// The in-place rename editor that replaces a row's name: the whole file name in a text
/// field, with a red border and the reason next to it when Enter was refused.
fn rename_editor(rename: &InlineRename) -> Element<'_, Message> {
    let border = if rename.error.is_some() {
        theme::ERROR
    } else {
        theme::ACCENT
    };
    let input = text_input("", &rename.text)
        .id(iced::widget::Id::from(FOLDER_RENAME_INPUT_ID))
        .on_input(Message::RenameInput)
        .on_submit(Message::SubmitRename)
        .size(14)
        .padding([4, 6])
        .style(
            move |_theme: &iced::Theme, _status: text_input::Status| text_input::Style {
                background: iced::Background::Color(theme::BG_ELEVATED),
                border: iced::Border {
                    radius: 2.0.into(),
                    width: 1.0,
                    color: border,
                },
                icon: theme::TEXT_MUTED,
                placeholder: theme::TEXT_MUTED,
                value: theme::TEXT,
                selection: theme::ACCENT_SELECTED,
            },
        );
    let mut content = row![input].spacing(6).align_y(iced::Alignment::Center);
    if let Some(error) = rename.error {
        content = content.push(text(error).size(11).color(theme::ERROR));
    }
    container(content)
        .padding(iced::Padding {
            top: 0.0,
            right: 8.0,
            bottom: 0.0,
            left: 0.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
