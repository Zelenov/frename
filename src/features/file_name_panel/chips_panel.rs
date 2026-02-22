//! Widget: wrapping row of tag chips (draggable) with bounds reporter for drop index.

use iced::widget::{container, row, stack};
use iced::{Alignment, Element, Length};

use frename_core::{StoredTagStore, TagList};

use crate::tag_colors;
use crate::widgets::bounds_reporter::BoundsReporter;
use crate::widgets::tag_chip;

use super::{
    FileNamePanelState, Message, TAG_CHIP_CELL_HEIGHT, TAG_CHIP_ROW_HEIGHT, TAG_CHIP_SPACING,
};

/// Renders the chips panel: wrapping row of tag chips with drop-index bounds reporting.
pub fn view<'a, S>(
    state: &'a FileNamePanelState,
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
        let chip = tag_chip::view_with_leading(
            tag_name,
            tag_color,
            TAG_CHIP_ROW_HEIGHT,
            None,
            Some((TAG_CHIP_CELL_HEIGHT, is_dragging)),
            Some(Message::DragStarted {
                tag_id,
                initial_index: idx,
            }),
            false,
            false, // no hover feedback
            None,
            false,
            false,
        );
        chip_elements.push(chip.into());
    }

    let tag_row = row(chip_elements)
        .spacing(TAG_CHIP_SPACING)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .wrap()
        .vertical_spacing(TAG_CHIP_SPACING)
        .align_x(Alignment::Start);

    let content_bounds = BoundsReporter::new(Message::PanelBounds);
    // Wrap BoundsReporter in a Fixed height to prevent it (Fill x Fill) from propagating
    // Fill height up through the Shrink chain in file_name_panel → file_workspace column.
    let content_bounds = container(content_bounds)
        .width(Length::Fill)
        .height(Length::Fixed(TAG_CHIP_CELL_HEIGHT));
    let chips_cell = container(
        stack![content_bounds, tag_row]
            .width(Length::Fill),
    )
    .width(Length::Fill);

    chips_cell.into()
}
