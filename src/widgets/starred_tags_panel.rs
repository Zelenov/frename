//! Panel showing all starred tags in a flow grid layout.
//! Located between the search bar and the tag grid in the file workspace.
//! Narrowed by the search filter like the tag grid.
//! Returns None when there are no starred tags (panel occupies no space).

use iced::widget::{checkbox, column, container, mouse_area, row, text, tooltip};
use iced::{mouse, Alignment, Border, Element, Length};

use frename_core::{StoredTagStore, TagId, TagList};

use crate::features::tag_panel::Message;
use crate::tag_colors::TagPalette;
use crate::theme;
use crate::widgets::tag_chip;

const CHIP_HEIGHT: f32 = tag_chip::CHIP_ROW_HEIGHT;
const GRID_MARGIN: f32 = 4.0;
const ROW_HEIGHT: f32 = CHIP_HEIGHT + GRID_MARGIN;
const MIN_CHIP_WIDTH: f32 = 60.0;

fn estimated_chip_width(tag_name_len: usize) -> f32 {
    let base = 2.0 * tag_chip::CHIP_PADDING_HORIZONTAL
        + tag_chip::LEADING_SLOT_WIDTH
        + tag_chip::LEADING_TO_LABEL_SPACING
        + tag_chip::LABEL_TO_TRAILING_SPACING
        + tag_chip::TRAILING_SLOT_WIDTH;
    base + 8.0 * tag_name_len as f32
}

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
            if is_checked { theme::ACCENT } else { theme::SPLITTER_ACTIVE },
            theme::TEXT_MUTED,
        ),
        _ => (
            if is_checked { theme::ACCENT } else { theme::TRACK },
            theme::TEXT_MUTED,
        ),
    };
    iced::widget::checkbox::Style {
        background: iced::Background::Color(background),
        icon_color: theme::TEXT,
        border: Border { radius: 2.0.into(), width: 1.0, color: border_color },
        text_color: Some(theme::TEXT),
    }
}

/// Render the starred tags panel.
/// Returns `None` when there are no starred tags (panel takes no space).
/// `panel_content_width` is the inner content width (panel width minus padding), used to
/// calculate the number of columns — same formula as the tag grid.
/// `selected_id` mirrors the tag grid cursor so the delete button shows consistently.
pub fn view<'a, S>(
    tag_list: &'a TagList<S>,
    panel_content_width: Option<f32>,
    selected_id: Option<TagId>,
    tag_palette: TagPalette,
) -> Option<Element<'a, Message>>
where
    S: StoredTagStore + Clone,
{
    let starred = tag_list.starred_tags_in_display_order();
    if starred.is_empty() {
        return None;
    }

    let max_chip_width = starred
        .iter()
        .map(|t| estimated_chip_width(t.tag().len()))
        .fold(MIN_CHIP_WIDTH, f32::max);

    let cols = panel_content_width
        .map(|w| {
            let divisor = max_chip_width + GRID_MARGIN;
            if divisor <= 0.0 {
                1usize
            } else {
                ((w + GRID_MARGIN) / divisor).floor() as usize
            }
        })
        .unwrap_or(1)
        .max(1);

    let row_height = Length::Fixed(ROW_HEIGHT);

    let empty_cell = || -> Element<'_, Message> {
        container(iced::widget::Space::new())
            .width(Length::Fill)
            .height(row_height)
            .into()
    };

    let grid_rows: Vec<Element<'_, Message>> = starred
        .chunks(cols)
        .map(|chunk| {
            let mut cells: Vec<Element<'_, Message>> = chunk
                .iter()
                .filter_map(|tag| {
                    let id = tag.id();
                    let is_checked = tag.is_checked();
                    let is_selected = selected_id == Some(id);
                    let tag_color = tag_palette.color(tag.color_index());

                    let checkbox_el: Element<'static, Message> = checkbox(is_checked)
                        .on_toggle(move |_| Message::ToggleTag(id))
                        .size(16)
                        .spacing(0)
                        .style(dark_checkbox_style)
                        .into();

                    // Star: always filled gold — clicking unstarres the tag.
                    let star_part: Element<'static, Message> = mouse_area(
                        container(
                            text("★")
                                .size(13)
                                .color(iced::Color::from_rgb(1.0, 0.8, 0.0)),
                        )
                        .width(Length::Fixed(20.0))
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                    )
                    .on_press(Message::ToggleStar(id))
                    .interaction(mouse::Interaction::Pointer)
                    .into();

                    // Delete button: only shown when tag is selected.
                    let action_part: Element<'static, Message> = if is_selected {
                        let btn = mouse_area(
                            container(
                                text("×")
                                    .size(13)
                                    .color(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                            )
                            .width(Length::Fixed(16.0))
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill),
                        )
                        .on_press(Message::DeleteTag(id));
                        tooltip(btn, text("Delete"), tooltip::Position::Top).into()
                    } else {
                        container(iced::widget::Space::new())
                            .width(Length::Fixed(16.0))
                            .into()
                    };

                    let trailing_el: Element<'static, Message> = row![star_part, action_part]
                        .spacing(0)
                        .align_y(Alignment::Center)
                        .into();

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
                        false,
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
                    .style(move |theme: &iced::Theme| {
                        theme::tag_row_background_style(theme, is_selected)
                    });

                    Some(cell.into())
                })
                .collect();

            while cells.len() < cols {
                cells.push(empty_cell());
            }

            row(cells)
                .spacing(GRID_MARGIN)
                .width(Length::Fill)
                .height(row_height)
                .into()
        })
        .collect();

    let tag_column = column(grid_rows).spacing(GRID_MARGIN).width(Length::Fill);

    Some(
        container(tag_column)
            .width(Length::Fill)
            .padding([4, 4])
            .style(theme::panel_container_style)
            .into(),
    )
}
