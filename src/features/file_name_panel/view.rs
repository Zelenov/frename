//! UI for the file name panel: composes chips panel, trash zone, and file name line.

use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

use frename_core::{StoredTagStore, TagList};

use crate::theme;

use crate::widgets::timecode_badge;

use super::chips_panel;
use super::file_name_line;
use super::trash_zone;
use super::{FileNamePanelState, Message, TRASH_SPACING};

/// Renders the file name panel:
/// - Top: chips (wrapping) | trash (right, centered)
/// - Divider
/// - Bottom: [IN×] [OUT×] (optional) | filename.ext
pub fn view<'a, S>(
    state: &'a FileNamePanelState,
    tag_list: &'a TagList<S>,
) -> Element<'a, Message>
where
    S: StoredTagStore + Clone,
{
    let snapshot = tag_list.file_snapshot();
    let name = snapshot.name_without_extension().to_string();
    let ext = snapshot.extension().to_string();
    let name_ext = file_name_line::name_ext_from_parts(&name, &ext);

    let chips = chips_panel::view(state, tag_list);
    let trash = trash_zone::view(state, tag_list);
    let top_row = row![chips, trash]
        .spacing(TRASH_SPACING)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let seg_start = tag_list.segment_start_secs();
    let seg_end = tag_list.segment_end_secs();

    let mut bottom_items: Vec<Element<'_, Message>> = Vec::new();
    if let Some(s) = seg_start {
        bottom_items.push(timecode_badge::view("IN", timecode_badge::fmt_timecode(s), Message::ClearSegmentStart));
    }
    if let Some(e) = seg_end {
        bottom_items.push(timecode_badge::view("OUT", timecode_badge::fmt_timecode(e), Message::ClearSegmentEnd));
    }
    bottom_items.push(file_name_line::view(name_ext).into());

    let bottom_row = row(bottom_items)
        .spacing(6)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let inner = container(
        column![top_row, bottom_row]
            .spacing(8)
            .width(Length::Fill),
    )
    .padding([8, 8])
    .width(Length::Fill)
    .style(theme::elevated_container_style);

    container(inner)
        .width(Length::Fill)
        .into()
}
