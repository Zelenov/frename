//! UI for the tag grid: scrollable grid of tag chips (same content as tag panel).
//! Column count is dynamic: panel width / longest chip width.

use iced::widget::{checkbox, column, container, mouse_area, row, scrollable, stack, text, tooltip};
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
/// Row height in pixels (content only). Used for each row widget height.
pub const GRID_ROW_HEIGHT: f32 = CHIP_HEIGHT + GRID_MARGIN;
/// Vertical stride per grid row: row height + column spacing. Must match scrollable layout for scroll-into-view.
const GRID_ROW_STRIDE: f32 = GRID_ROW_HEIGHT + GRID_MARGIN;

/// Horizontal padding of the panel container (each side).
const PANEL_PADDING_X: f32 = 4.0;

/// Estimated width of a tag chip from tag name length (14px font, ~8px per character).
fn estimated_chip_width(tag_name_len: usize) -> f32 {
    let base = 2.0 * tag_chip::CHIP_PADDING_HORIZONTAL
        + tag_chip::LEADING_SLOT_WIDTH
        + tag_chip::LEADING_TO_LABEL_SPACING
        + tag_chip::LABEL_TO_TRAILING_SPACING
        + tag_chip::TRAILING_SLOT_WIDTH;
    const ESTIMATED_CHAR_WIDTH: f32 = 8.0;
    base + ESTIMATED_CHAR_WIDTH * (tag_name_len as f32)
}

/// Minimum chip width when there are no tags (avoid divide by zero).
const MIN_CHIP_WIDTH: f32 = 60.0;

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
    let ids: Vec<_> = tag_list.filtered_display_tag_ids().to_vec();
    let row_height = Length::Fixed(GRID_ROW_HEIGHT);

    let max_chip_width = tag_list
        .longest_display_tag_id()
        .and_then(|id| tag_list.get_tag(id))
        .map(|t| estimated_chip_width(t.tag().len()))
        .unwrap_or(MIN_CHIP_WIDTH)
        .max(MIN_CHIP_WIDTH);

    let content_width = state
        .panel_bounds()
        .map(|b| (b.width - 2.0 * PANEL_PADDING_X).max(0.0));

    let cols = content_width
        .map(|w| {
            let divisor = max_chip_width + GRID_MARGIN;
            if divisor <= 0.0 {
                1
            } else {
                ((w + GRID_MARGIN) / divisor).floor() as u32
            }
        })
        .unwrap_or(1)
        .max(1);

    let cols_usize = cols as usize;

    let empty_cell = || {
        container(iced::widget::Space::new())
            .width(Length::Fill)
            .height(row_height)
            .into()
    };

    let grid_rows: Vec<Element<'_, Message>> = ids
        .chunks(cols_usize)
        .map(|chunk| {
            let mut cells: Vec<Element<'_, Message>> = chunk
                .iter()
                .filter_map(|id| {
                    let id = *id;
                    let tag = tag_list.get_tag(id)?;
                    let is_checked = tag.is_checked();
                    let is_selected = selected_id == Some(id);
                    let is_stored = tag.is_stored();
                    let tag_color = tag_colors::TagColors::color(tag.color_index());
                    let checkbox_el = checkbox(is_checked)
                        .on_toggle(move |_| Message::ToggleTag(id))
                        .size(16)
                        .spacing(0)
                        .style(dark_checkbox_style)
                        .into();
                    let trailing_el: Element<'_, Message> = {
                        let (label, msg, tip) = match is_stored {
                            true => ("×", Message::DeleteTag(id), "Delete"),
                            false => ("○", Message::SaveTag(id), "Enter"),
                        };
                        let btn = mouse_area(
                            container(text(label).size(13).color(iced::Color::from_rgb(0.0, 0.0, 0.0)))
                                .center_y(Length::Fill)
                                .padding([0, 4]),
                        )
                        .on_press(msg);
                        if is_selected {
                            tooltip(btn, text(tip), tooltip::Position::Top).into()
                        } else {
                            btn.into()
                        }
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
                    let cell = container(
                        container(main_cell)
                            .width(Length::Fill)
                            .height(row_height)
                            .align_x(Alignment::Start)
                            .center_y(Length::Fill),
                    )
                    .width(Length::Fill)
                    .height(row_height)
                    .style(move |theme: &iced::Theme| theme::tag_row_background_style(theme, is_selected));
                    Some(cell.into())
                })
                .collect();
            while cells.len() < cols_usize {
                cells.push(empty_cell());
            }
            row(cells)
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
        BoundsReporter::new(move |bounds| Message::PanelBounds {
            bounds,
            row_height: GRID_ROW_STRIDE,
            cols,
            row_content_height: Some(GRID_ROW_HEIGHT),
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
