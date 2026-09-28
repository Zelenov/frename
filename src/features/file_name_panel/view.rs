//! UI for the file name card: its chips, the trash, the timecodes and the name line.

use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

use frename_core::{StoredTagStore, TagList};

use crate::ui::badge;
use crate::ui::palette::TagPalette;
use crate::ui::style;
use crate::ui::tokens::*;

use super::chips_panel;
use super::file_name_line;
use super::trash_zone;
use super::{FileNamePanelState, Message};

/// Renders the file name card (design system §13.5.6):
/// - first line: the checked tags as chips in the file's order | the trash
/// - second line: the IN and OUT timecodes (when set) | the name without the tags
pub fn view<'a, S>(
    state: &'a FileNamePanelState,
    tag_list: &'a TagList<S>,
    tag_palette: TagPalette,
) -> Element<'a, Message>
where
    S: StoredTagStore + Clone,
{
    let snapshot = tag_list.file_snapshot();
    let name = snapshot.name_without_extension().to_string();
    let ext = snapshot.extension().to_string();
    let name_ext = file_name_line::name_ext_from_parts(&name, &ext);

    let chips = chips_panel::view(state, tag_list, tag_palette);
    let trash = trash_zone::view(state, tag_list, tag_palette);
    let top_row = row![chips, trash]
        .spacing(SPACE_S)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let seg_start = tag_list.segment_start_secs();
    let seg_end = tag_list.segment_end_secs();

    let mut bottom_items: Vec<Element<'_, Message>> = Vec::new();
    if let Some(s) = seg_start {
        bottom_items.push(badge::timecode(
            "IN",
            file_name_line::fmt_timecode(s),
            Some(Message::ClearSegmentStart),
        ));
    }
    if let Some(e) = seg_end {
        bottom_items.push(badge::timecode(
            "OUT",
            file_name_line::fmt_timecode(e),
            Some(Message::ClearSegmentEnd),
        ));
    }
    bottom_items.push(file_name_line::view(name_ext));

    let bottom_row = row(bottom_items)
        .spacing(SPACE_TIGHT)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let inner = container(
        column![top_row, bottom_row]
            .spacing(SPACE_S)
            .width(Length::Fill),
    )
    .padding(SPACE_S)
    .width(Length::Fill)
    .style(style::card);

    container(inner).width(Length::Fill).into()
}
