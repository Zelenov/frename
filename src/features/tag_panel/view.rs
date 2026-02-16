//! UI for the tag panel. Only this module knows how the tag list looks (scrollable, checkboxes).

use iced::widget::{checkbox, column, container, mouse_area, scrollable, text};
use iced::{mouse, Background, Border, Element, Length};

use frename_core::{File, TagList};

use crate::theme;
use super::{Message, TagPanelState};

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

/// Render the tag panel: scrollable list of tag checkboxes.
/// `selected_file` is the file from the file workspace when ready (no file = placeholder).
/// `tag_list` is the workspace stored tags (value + checked); use workspace checked flag everywhere.
pub fn view<'a>(
    _state: &'a TagPanelState,
    selected_file: Option<&'a File>,
    tag_list: &'a TagList,
) -> Element<'a, Message> {
    let Some(_file) = selected_file else {
        return container(
            text("📄")
                .size(48)
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

    let tag_items: Vec<Element<'_, Message>> = tag_list
        .tags()
        .iter()
        .enumerate()
        .map(|(index, tag)| {
            let is_checked = tag.is_checked();
            let row_content = container(
                checkbox(is_checked)
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

    container(tag_list)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([4, 4])
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_PANEL)),
            ..Default::default()
        })
        .into()
}
