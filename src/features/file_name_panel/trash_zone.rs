//! The trash at the end of the file name card (design system §13.5.6): a chip dropped on it is
//! taken off the clip. While a chip is dragged over it, it turns red and the dragged chip and
//! "Untag" show above it (drawn on top of everything, not in the layout).

use std::time::Duration;

use iced::widget::{column, container, space, stack, tooltip};
use iced::{Alignment, Element, Length};

use frename_core::{StoredTagStore, TagList};

use crate::ui::icons::{icon, Icon};
use crate::ui::palette::TagPalette;
use crate::ui::tokens::*;
use crate::ui::{style, text};
use crate::widgets::bounds_reporter::BoundsReporter;
use crate::widgets::tag_chip;

use super::{FileNamePanelState, Message};

/// Renders the trash zone: square panel with trash icon. When dragging and cursor over trash,
/// a tooltip overlay shows the dragged chip above the trash (on top of the UI, not in the layout).
pub fn view<'a, S>(
    state: &'a FileNamePanelState,
    tag_list: &'a TagList<S>,
    tag_palette: TagPalette,
) -> Element<'a, Message>
where
    S: StoredTagStore + Clone,
{
    let over = state.is_dragging() && state.cursor_over_trash();
    let color = if over { ERROR } else { TEXT_SECONDARY };
    let trash_bounds = BoundsReporter::new(Message::TrashBounds);
    let trash_square = container(stack![
        trash_bounds,
        container(icon(Icon::Trash, ICON_DROP, color)).center(Length::Fill),
    ])
    .width(Length::Fixed(TRASH_SIDE))
    .height(Length::Fixed(TRASH_SIDE))
    .style(style::outline(over));

    let tooltip_content = over
        .then(|| state.dragging_tag_id())
        .flatten()
        .and_then(|id| tag_list.get_tag(id))
        .map(|tag| {
            column![
                tag_chip::view_display_only(
                    tag.tag(),
                    tag_palette.color(tag.color_index(), tag.is_stored())
                ),
                text::caption_strong(fl!("file-name-panel-untag")).color(ERROR),
            ]
            .spacing(SPACE_XS)
            .align_x(Alignment::Center)
        });

    let tooltip_body: Element<'a, Message> = match tooltip_content {
        Some(chip) => chip.into(),
        None => space().into(),
    };

    let trash_zone = container(
        tooltip(trash_square, tooltip_body, tooltip::Position::Top)
            .gap(SPACE_XS)
            .delay(Duration::ZERO)
            .snap_within_viewport(true),
    )
    .width(Length::Fixed(TRASH_SIDE));

    trash_zone.into()
}
