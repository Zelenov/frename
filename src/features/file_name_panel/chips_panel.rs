//! The first line of the file name card (design system §13.5.6): the file's checked tags as
//! chips in the file's order, the order of the new name. A chip is dragged to reorder it and
//! middle-clicked to take it off the clip; a bounds reporter maps the pointer to a drop index.

use iced::widget::{container, mouse_area, row, stack};
use iced::{mouse, Alignment, Element, Length};

use frename_core::{StoredTagStore, TagList};

use crate::ui::palette::TagPalette;
use crate::widgets::bounds_reporter::BoundsReporter;
use crate::widgets::tag_chip;

use crate::ui::text;

use super::{FileNamePanelState, Message, TAG_CHIP_CELL_HEIGHT, TAG_CHIP_SPACING};

/// Renders the chips panel: wrapping row of tag chips with drop-index bounds reporting.
pub fn view<'a, S>(
    state: &'a FileNamePanelState,
    tag_list: &'a TagList<S>,
    tag_palette: TagPalette,
) -> Element<'a, Message>
where
    S: StoredTagStore + Clone,
{
    let snapshot = tag_list.file_snapshot();
    let tag_count = snapshot.tags().len();
    let dragging_index = state
        .dragging_tag_id()
        .and_then(|id| tag_list.checked_index_of(id));
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

    if tag_count == 0 {
        return container(text::secondary(fl!("file-name-panel-no-tags")))
            .width(Length::Fill)
            .height(TAG_CHIP_CELL_HEIGHT)
            .center_y(TAG_CHIP_CELL_HEIGHT)
            .into();
    }

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
        let tag_color = tag_palette.color(tag.color_index(), tag.is_stored());
        let is_dragging = dragging_index == Some(idx);
        let chip = mouse_area(tag_chip::draggable(tag.tag(), tag_color, is_dragging))
            .on_press(Message::DragStarted {
                tag_id,
                initial_index: idx,
            })
            .on_middle_press(Message::RemoveTag(tag_id))
            .interaction(mouse::Interaction::Grab);
        chip_elements.push(chip.into());
    }

    let tag_row = row(chip_elements)
        .spacing(TAG_CHIP_SPACING)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .wrap()
        .vertical_spacing(TAG_CHIP_SPACING)
        .align_x(Alignment::Start);

    // The chips are the stack's first layer, so they give it its height: wrapped onto more
    // lines, the card grows instead of the second line running over the name below. The bounds
    // reporter fills that height on top; it takes no events, so the chips still get the clicks.
    let content_bounds = BoundsReporter::new(Message::PanelBounds);
    let chips_cell =
        container(stack![tag_row, content_bounds].width(Length::Fill)).width(Length::Fill);

    chips_cell.into()
}
