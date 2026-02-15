//! UI rendering for the rename panel feature

use iced::widget::{checkbox, column, container, mouse_area, scrollable, text};
use iced::{mouse, Background, Border, Element, Length};

use frename_core::File;

use crate::theme;
use super::{Message, RenamePanelState};

/// Dark checkbox style: dark background, light text, accent when checked.
fn dark_checkbox_style(
    _theme: &iced::Theme,
    status: iced::widget::checkbox::Status,
) -> iced::widget::checkbox::Style {
    let is_checked = match status {
        iced::widget::checkbox::Status::Active { is_checked }
        | iced::widget::checkbox::Status::Hovered { is_checked }
        | iced::widget::checkbox::Status::Disabled { is_checked } => is_checked,
    };
    let (background, border_color) = match status {
        iced::widget::checkbox::Status::Hovered { .. } => (
            if is_checked {
                theme::ACCENT
            } else {
                theme::SPLITTER_ACTIVE
            },
            theme::TEXT_MUTED,
        ),
        _ => (
            if is_checked {
                theme::ACCENT
            } else {
                theme::TRACK
            },
            theme::TEXT_MUTED,
        ),
    };
    iced::widget::checkbox::Style {
        background: iced::Background::Color(background),
        icon_color: theme::TEXT,
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: border_color,
        },
        text_color: Some(theme::TEXT),
    }
}

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
                .color(theme::TEXT_MUTED)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .padding([8, 8])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_PANEL)),
            ..Default::default()
        })
        .into();
    };

    // File name display at the top
    let file_name_display = {
        let label = if file.file_name().is_empty() {
            "(no tags selected)"
        } else {
            file.file_name()
        };

        container(
            text(label)
                .size(14)
                .color(theme::TEXT)
                .wrapping(text::Wrapping::WordOrGlyph),
        )
        .padding([8, 8])
        .width(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_ELEVATED)),
            ..Default::default()
        })
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
                    .text_size(14)
                    .style(dark_checkbox_style),
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
    let tag_list = scrollable(tag_column)
        .height(Length::Fill)
        .style(theme::dark_scrollable_style);

    let panel = column![file_name_display, tag_list]
        .spacing(4)
        .width(Length::Fill)
        .height(Length::Fill);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([4, 4])
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_PANEL)),
            ..Default::default()
        })
        .into()
}
