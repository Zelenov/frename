//! UI for the tag panel. Only this module knows how the tag list looks (scrollable, tag chips with checkboxes).

use iced::widget::{checkbox, column, container, mouse_area, row, scrollable, stack, text};
use iced::{mouse, Border, Element, Length};

use frename_core::{File, StoredTagStore, TagList};

use crate::tag_colors;
use crate::theme;
use crate::widgets::bounds_reporter::BoundsReporter;
use crate::widgets::tag_chip;
use super::{Message, TagPanelState, TAG_LIST_SCROLLABLE_ID};

/// Tag list row height in pixels. Must match folder_workspace::TAG_ROW_HEIGHT for scroll-into-view.
pub const TAG_ROW_HEIGHT: f32 = 28.0;

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
    let drop_target_index = state.drop_target_index();
    let tag_items: Vec<Element<'_, Message>> = tag_list
        .filtered_display_tag_ids()
        .iter()
        .enumerate()
        .filter_map(|(index, id)| {
            let id = *id;
            let tag = tag_list.get_tag(id)?;
            let is_checked = tag.is_checked();
            let is_selected = selected_id == Some(id);
            let is_stored = tag.is_stored();
            let is_drop_target = drop_target_index == Some(index);
            let tag_color = tag_colors::TagColors::color(tag.color_index());
            let checkbox_el = checkbox(is_checked)
                .on_toggle(move |_| Message::ToggleTag(id))
                .size(16)
                .spacing(0)
                .style(dark_checkbox_style)
                .into();
            let trailing_el = {
                let (label, msg) = match is_stored {
                    true => ("×", Message::DeleteTag(id)),
                    false => ("○", Message::SaveTag(id)),
                };
                mouse_area(
                    container(text(label).size(13).color(iced::Color::from_rgb(0.0, 0.0, 0.0)))
                        .center_y(Length::Fill)
                        .padding([0, 4]),
                )
                .on_press(msg)
                .into()
            };
            let chip = tag_chip::view_with_leading(
                tag.tag(),
                tag_color,
                TAG_ROW_HEIGHT,
                Some(checkbox_el),
                None,
                Some(Message::ToggleTag(id)),
                false, // leading always visible in tag list
                false,
                Some(trailing_el),
                true,  // trailing_visible_on_selection_only: X/💾 only when selected
                is_selected,
            );
            let chip_cell = container(chip)
                .width(Length::Shrink)
                .height(Length::Fixed(TAG_ROW_HEIGHT))
                .id(iced::widget::Id::from(id.widget_id()));
            let main_cell = mouse_area(chip_cell)
                .on_press(Message::ToggleTag(id))
                .interaction(mouse::Interaction::Pointer);
            let row_height = Length::Fixed(TAG_ROW_HEIGHT);
            let right_margin = container(iced::widget::Space::new())
                .width(Length::Fixed(12.0))
                .height(row_height);
            let full_row = row![
                container(main_cell)
                    .width(Length::Fill)
                    .height(row_height),
                right_margin,
            ]
            .width(Length::Fill)
            .height(row_height)
            .spacing(0);
            let row_style = is_selected || is_drop_target;
            let row_background = container(full_row)
                .height(row_height)
                .width(Length::Fill)
                .style(move |theme: &iced::Theme| theme::row_background_style(theme, row_style));
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
    // List on top so clicks (checkboxes) reach it; BoundsReporter underneath for cursor→row mapping.
    let with_bounds = stack([
        BoundsReporter::new(|bounds| Message::PanelBounds {
            bounds,
            row_height: TAG_ROW_HEIGHT,
            cols: 1,
        })
        .into(),
        tag_list.into(),
    ]);

    container(with_bounds)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([4, 4])
        .style(theme::panel_container_style)
        .into()
}
