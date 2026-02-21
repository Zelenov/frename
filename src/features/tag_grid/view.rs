//! UI for the tag grid: two-column scrollable grid of tag chips (same content as tag panel).

use iced::widget::{checkbox, column, container, mouse_area, row, scrollable, stack, text};
use iced::{mouse, Alignment, Border, Element, Length};

use frename_core::{File, StoredTagStore, TagList};

use crate::tag_colors;
use crate::theme;
use crate::widgets::bounds_reporter::BoundsReporter;
use crate::widgets::tag_chip;
use crate::features::tag_panel::{Message, TagPanelState, TAG_LIST_SCROLLABLE_ID};

/// Height of one grid row: chip height + margin (separator between rows/columns).
const CHIP_HEIGHT: f32 = tag_chip::CHIP_ROW_HEIGHT;
const GRID_MARGIN: f32 = 4.0;
/// Row height in pixels. Used for layout and for cursor→index mapping in state.
pub const GRID_ROW_HEIGHT: f32 = CHIP_HEIGHT + GRID_MARGIN;

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

/// Render the tag grid: scrollable two-column grid of tag checkboxes (same data as tag panel).
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
    let ids: Vec<_> = tag_list.filtered_display_tag_ids().to_vec();
    let row_height = Length::Fixed(GRID_ROW_HEIGHT);

    let empty_cell = || {
        container(iced::widget::Space::new())
            .width(Length::Fill)
            .height(row_height)
            .into()
    };

    let grid_rows: Vec<Element<'_, Message>> = ids
        .chunks(2)
        .enumerate()
        .map(|(row_i, chunk)| {
            let mut cells: Vec<Element<'_, Message>> = chunk
                .iter()
                .enumerate()
                .filter_map(|(col_i, id)| {
                    let index = row_i * 2 + col_i;
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
                        CHIP_HEIGHT,
                        Some(checkbox_el),
                        None,
                        Some(Message::ToggleTag(id)),
                        false,
                        false,
                        Some(trailing_el),
                        true,
                        is_selected,
                    );
                    let chip_cell = container(chip)
                        .width(Length::Shrink)
                        .height(Length::Fixed(CHIP_HEIGHT))
                        .id(iced::widget::Id::from(id.widget_id()));
                    let main_cell = mouse_area(chip_cell)
                        .on_press(Message::ToggleTag(id))
                        .interaction(mouse::Interaction::Pointer);
                    let cell_style = is_selected || is_drop_target;
                    let cell = container(
                        container(main_cell)
                            .width(Length::Fill)
                            .height(row_height)
                            .align_x(Alignment::Start),
                    )
                    .width(Length::Fill)
                    .height(row_height)
                    .style(move |theme: &iced::Theme| theme::row_background_style(theme, cell_style));
                    Some(cell.into())
                })
                .collect();
            let first_cell = if cells.is_empty() {
                empty_cell()
            } else {
                cells.remove(0)
            };
            let second_cell = if cells.is_empty() {
                empty_cell()
            } else {
                cells.remove(0)
            };
            row![first_cell, second_cell]
                .spacing(GRID_MARGIN)
                .width(Length::Fill)
                .height(row_height)
                .into()
        })
        .collect();

    let tag_column = column(grid_rows)
        .spacing(GRID_MARGIN)
        .width(Length::Fill);
    let tag_scroll = scrollable(tag_column)
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

    let with_bounds = stack([
        BoundsReporter::new(|bounds| Message::PanelBounds {
            bounds,
            row_height: GRID_ROW_HEIGHT,
            cols: 2,
        })
        .into(),
        tag_scroll.into(),
    ]);

    container(with_bounds)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([4, 4])
        .style(theme::panel_container_style)
        .into()
}
