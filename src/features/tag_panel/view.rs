//! UI for the tag panel. Only this module knows how the tag list looks (scrollable, checkboxes).

use iced::widget::{checkbox, column, container, mouse_area, row, scrollable, text};
use iced::{mouse, Background, Border, Element, Length};

use frename_core::{File, StoredTagStore, TagList};

use crate::tag_colors;
use crate::theme;
use super::{Message, TagPanelState, TAG_LIST_SCROLLABLE_ID};

/// Tag list row height in pixels. Must match folder_workspace::TAG_ROW_HEIGHT for scroll-into-view.
const TAG_ROW_HEIGHT: f32 = 28.0;

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
        .style(theme::panel_container_style)
        .into();
    };

    let selected_id = state.selected_tag_id();
    let tag_items: Vec<Element<'_, Message>> = tag_list
        .filtered_tag_ids()
        .into_iter()
        .filter_map(|id| {
            let tag = tag_list.get_tag(id)?;
            let is_checked = tag.is_checked();
            let is_selected = selected_id == Some(id);
            let is_stored = tag.is_stored();
            let show_action = is_selected;
            let tag_color = tag_colors::TagColors::color(tag.color_index());
            let color_stripe = container(
                iced::widget::Space::new()
                    .width(Length::Fixed(4.))
                    .height(Length::Fill),
            )
            .width(Length::Fixed(4.))
            .height(Length::Fill)
            .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(tag_color)),
                ..Default::default()
            });
            let checkbox_content = container(
                checkbox(is_checked)
                    .label(tag.tag())
                    .on_toggle(move |_| Message::ToggleTag(id))
                    .size(16)
                    .text_size(14)
                    .style(dark_checkbox_style),
            )
            .padding([4, 8])
            .width(Length::Fill)
            .id(iced::widget::Id::from(id.widget_id()));
            let main_row = row![color_stripe, checkbox_content]
                .width(Length::Fill)
                .spacing(0);
            let main_cell = mouse_area(main_row)
                .on_press(Message::ToggleTag(id))
                .interaction(mouse::Interaction::Pointer);
            let action_slot: Element<'_, Message> = if show_action {
                let (label, msg) = if is_stored {
                    ("×", Message::DeleteTag(id))
                } else {
                    ("💾", Message::SaveTag(id))
                };
                mouse_area(
                    container(text(label).size(14).color(theme::TEXT_MUTED))
                        .center_y(Length::Fill)
                        .padding([0, 4]),
                )
                .on_press(msg)
                .into()
            } else {
                iced::widget::Space::new().into()
            };
            let row_height = Length::Fixed(TAG_ROW_HEIGHT);
            let right_margin = container(iced::widget::Space::new())
                .width(Length::Fixed(12.0))
                .height(row_height);
            let full_row = row![
                container(main_cell)
                    .width(Length::Fill)
                    .height(row_height),
                container(action_slot)
                    .width(Length::Fixed(24.0))
                    .height(row_height)
                    .center_y(Length::Fill),
                right_margin,
            ]
            .width(Length::Fill)
            .height(row_height)
            .spacing(0);
            let row_background = container(full_row)
                .height(row_height)
                .width(Length::Fill)
                .style(move |theme: &iced::Theme| theme::row_background_style(theme, is_selected));
            Some(row_background.into())
        })
        .collect();

    let tag_column = column(tag_items).width(Length::Fill);
    let tag_list = scrollable(tag_column)
        .id(iced::widget::Id::new(TAG_LIST_SCROLLABLE_ID))
        .height(Length::Fill)
        .on_scroll(|viewport| {
            let offset = viewport.absolute_offset();
            let scroll_y = offset.y;
            let viewport_height = viewport.bounds().height;
            Message::TagListScrolled {
                scroll_y,
                viewport_height,
            }
        })
        .style(theme::dark_scrollable_style);

    container(tag_list)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([4, 4])
        .style(theme::panel_container_style)
        .into()
}
