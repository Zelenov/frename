//! UI for the file name panel: wrapping row of tag chips (left→right, top→bottom), then name+extension below.
//! No dots. Drag reorder is UI-only; does not change TagList.
//! Extension already includes a dot when needed (see name_ext_from_parts).

use iced::widget::{checkbox, column, container, mouse_area, row, stack, text};
use iced::{mouse, Alignment, Element, Length};

use frename_core::{StoredTagStore, TagList};

use crate::tag_colors;
use crate::theme;
use crate::widgets::bounds_reporter::BoundsReporter;
use crate::widgets::tag_chip;

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
        let checkbox_el = checkbox(true)
            .on_toggle(move |_| Message::UnselectTag(tag_id))
            .size(16)
            .spacing(0)
            .into();
        let is_hovered = state.hovered_tag_id() == Some(tag_id);
        let chip = tag_chip::view_with_leading(
            tag_name,
            tag_color,
            TAG_CHIP_ROW_HEIGHT,
            Some(checkbox_el),
            Some((TAG_CHIP_CELL_HEIGHT, is_dragging)),
            Some(Message::DragStarted {
                tag_id,
                initial_index: idx,
            }),
            true,  // leading_visible_on_hover_only
            is_hovered,
            None,  // trailing: no action icons in file name panel
            false,
            false,
        );
        let chip_with_hover = mouse_area(chip)
            .on_enter(Message::ChipHovered(Some(tag_id)))
            .on_exit(Message::ChipHovered(None))
            .interaction(mouse::Interaction::Pointer);
        chip_elements.push(chip_with_hover.into());
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
