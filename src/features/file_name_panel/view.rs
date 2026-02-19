//! UI for the file name panel: wrapping row of tag chips (left→right, top→bottom), then name+extension below.
//! No dots. Drag reorder is UI-only; does not change TagList.
//! Extension already includes a dot when needed (see name_ext_from_parts).

use iced::widget::{column, container, mouse_area, row, space, stack, text};
use iced::{mouse, Alignment, Background, Element, Length, Shadow, Vector};

use frename_core::{StoredTagStore, TagList};

use crate::tag_colors;
use crate::theme;
use crate::widgets::bounds_reporter::BoundsReporter;

use super::{Message, TAG_CHIP_CELL_HEIGHT, TAG_CHIP_ROW_HEIGHT, TAG_CHIP_SPACING};

/// Renders: (1) vertical list of tag chips (draggable, UI order only), (2) below: initial name + extension.
/// No separators/dots. Name+extension always visible. Data comes from `tag_list` (checked tags, name, extension).
pub fn view<'a, S>(
    state: &'a super::FileNamePanelState,
    tag_list: &'a TagList<S>,
) -> Element<'a, Message>
where
    S: StoredTagStore + Clone,
{
    let snapshot = tag_list.file_snapshot();
    let tag_count = snapshot.tags().len();
    let dragging_index = state.dragging_tag_id().and_then(|id| tag_list.checked_index_of(id));
    let drop_target_index = state.drop_target_index();
    let display_order: Vec<usize> = match (dragging_index, drop_target_index) {
        (Some(drag_i), Some(drop_i)) => {
            let mut indices: Vec<usize> = (0..tag_count).filter(|&i| i != drag_i).collect();
            indices.insert(drop_i.min(indices.len()), drag_i);
            indices
        }
        (Some(drag_i), None) => {
            let mut indices: Vec<usize> = (0..tag_count).filter(|&i| i != drag_i).collect();
            indices.push(drag_i);
            indices
        }
        _ => (0..tag_count).collect(),
    };

    let name = snapshot.name_without_extension().to_string();
    let ext = snapshot.extension().to_string();
    let name_ext = name_ext_from_parts(&name, &ext);

    let mut chip_elements: Vec<Element<'_, Message>> = Vec::with_capacity(display_order.len());
    for &idx in &display_order {
        let tag_id = match tag_list.checked_tag_id_at(idx) {
            Some(id) => id,
            None => continue,
        };
        let tag = match tag_list.get_tag(tag_id) {
            Some(t) => t,
            None => continue,
        };
        let tag_name = tag.tag().to_string();
        let tag_color = tag_colors::TagColors::color(tag.color_index());
        let is_dragging = dragging_index == Some(idx);
        let chip_inner = container(
            text(tag_name)
                .size(14)
                .color(iced::Color::from_rgb(0.0, 0.0, 0.0)),
        )
        .padding([4, 6]);
        let chip_style = move |_theme: &_| {
            let shadow = if is_dragging {
                Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 10.0,
                }
            } else {
                Shadow::default()
            };
            iced::widget::container::Style {
                background: Some(Background::Color(tag_color)),
                border: iced::border::rounded(2),
                shadow,
                ..Default::default()
            }
        };
        let chip = container(chip_inner)
            .height(Length::Fixed(TAG_CHIP_ROW_HEIGHT))
            .style(chip_style);
        let chip = mouse_area(chip)
            .on_press(Message::DragStarted {
                tag_id,
                initial_index: idx,
            })
            .interaction(mouse::Interaction::Pointer);
        let chip: Element<'_, Message> = chip.into();
        let lift_px = TAG_CHIP_CELL_HEIGHT - TAG_CHIP_ROW_HEIGHT;
        let chip = if is_dragging {
            column![
                chip,
                container(space()).height(Length::Fixed(lift_px)),
            ]
        } else {
            column![
                container(space()).height(Length::Fixed(lift_px)),
                chip,
            ]
        }
        .into();
        chip_elements.push(chip);
    }

    let tag_row = row(chip_elements)
        .spacing(TAG_CHIP_SPACING)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .wrap()
        .vertical_spacing(TAG_CHIP_SPACING)
        .align_x(Alignment::Start);

    let name_ext_line = container(
        text(name_ext)
            .size(14)
            .color(theme::TEXT),
    )
    .width(Length::Fill)
    .padding([6, 0]);

    let content = column![tag_row, name_ext_line]
        .spacing(8)
        .width(Length::Fill);

    let inner = container(content)
        .padding([8, 8])
        .width(Length::Fill)
        .style(theme::elevated_container_style);

    let bounds_reporter = BoundsReporter::new(Message::PanelBounds);
    let with_bounds = stack![bounds_reporter, inner]
        .width(Length::Fill)
        .height(Length::Fill);

    container(with_bounds)
        .width(Length::Fill)
        .into()
}

fn name_ext_from_parts(name: &str, ext: &str) -> String {
    if name.is_empty() {
        ext.to_string()
    } else if ext.is_empty() {
        name.to_string()
    } else {
        format!("{}{}", name, ext)
    }
}
