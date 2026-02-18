//! UI for the tag panel. Only this module knows how the tag list looks (scrollable, checkboxes).

use iced::widget::{checkbox, column, container, mouse_area, scrollable, text};
use iced::{mouse, Background, Border, Element, Length};

use frename_core::{File, StoredTagStore, TagList};

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

/// Render the tag panel: scrollable list of tag checkboxes with keyboard-selectable cursor.
/// `selected_file` is the file from the file workspace when ready (no file = placeholder).
/// `tag_list` is the workspace stored tags (value + checked); use workspace checked flag everywhere.
pub fn view<'a, S>(
    state: &'a TagPanelState,
    selected_file: Option<&'a File>,
    tag_list: &'a TagList<S>,
) -> Element<'a, Message>
where
    S: StoredTagStore + Clone,
{
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

    let tags = tag_list.tags();
    let filtered_indices = tag_list.filtered_indices();
    let selected_id = state.selected_tag_id();
    let tag_items: Vec<Element<'_, Message>> = filtered_indices
        .iter()
        .map(|&index| {
            let tag = &tags[index];
            let id = tag.id();
            let is_checked = tag.is_checked();
            let is_selected = selected_id == Some(id);
            let row_content = container(
                checkbox(is_checked)
                    .label(tag.tag())
                    .on_toggle(move |_| Message::ToggleTag(id))
                    .size(16)
                    .text_size(14)
                    .style(dark_checkbox_style),
            )
            .padding([4, 8])
            .width(Length::Fill)
            .id(iced::widget::Id::from(id.widget_id()))
            .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(if is_selected {
                    theme::ACCENT_SELECTED
                } else {
                    theme::BG_PANEL
                })),
                ..Default::default()
            });

            mouse_area(row_content)
                .on_press(Message::ToggleTag(id))
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
