//! Widget: squared trash zone; when dragging and cursor over trash, shows the dragged chip
//! in a tooltip overlay above (drawn on top of everything, not in layout).

use std::time::Duration;

use iced::widget::{container, space, stack, text, tooltip};
use iced::{Element, Length};

use frename_core::{StoredTagStore, TagList};

use crate::tag_colors::TagPalette;
use crate::theme;
use crate::widgets::bounds_reporter::BoundsReporter;
use crate::widgets::tag_chip;

use super::{FileNamePanelState, Message, TRASH_SIDE};

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
    let trash_icon = text("🗑").size(20).color(theme::TEXT_MUTED);
    let trash_bounds = BoundsReporter::new(Message::TrashBounds);
    let trash_square = container(stack![
        trash_bounds,
        container(trash_icon)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ])
    .width(Length::Fixed(TRASH_SIDE))
    .height(Length::Fixed(TRASH_SIDE))
    .style(theme::elevated_container_style);

    let show_trash_preview = state.is_dragging() && state.cursor_over_trash();
    let tooltip_content = show_trash_preview
        .then(|| state.dragging_tag_id())
        .flatten()
        .and_then(|id| tag_list.get_tag(id))
        .map(|tag| tag_chip::view_display_only(tag.tag(), tag_palette.color(tag.color_index())));

    let tooltip_body = match tooltip_content {
        Some(chip) => chip,
        None => container(space())
            .width(Length::Fixed(0.0))
            .height(Length::Fixed(0.0))
            .into(),
    };

    let trash_zone = container(
        tooltip(trash_square, tooltip_body, tooltip::Position::Top)
            .gap(4)
            .delay(Duration::ZERO)
            .snap_within_viewport(true),
    )
    .width(Length::Fixed(TRASH_SIDE));

    trash_zone.into()
}
