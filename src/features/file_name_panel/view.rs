//! UI for the file name panel: composes chips panel, trash zone, and file name line.

use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

use frename_core::{StoredTagStore, TagList};

use crate::theme;

use super::chips_panel;
use super::file_name_line;
use super::trash_zone;
use super::{FileNamePanelState, Message, TRASH_SPACING};

/// Renders the file name panel: chips (with bounds) | trash zone on top row, file name below.
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

    let name_line = file_name_line::view(name_ext);

    let inner = container(
        column![top_row, name_line]
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
