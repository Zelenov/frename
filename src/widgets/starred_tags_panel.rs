//! The starred strip above the tag grid (design system §13.5.3): the starred tags again, in cells
//! like the grid's and in its columns, so the few tags used on almost every clip are always in
//! the same place. It follows the search, takes no space when empty, and shows at most three
//! rows. It is for the mouse: the keyboard cursor never enters it, but a starred cursor tag shows
//! its ring in both places.

use iced::widget::{column, container, space};
use iced::{Element, Length};

use frename_core::{StoredTagStore, TagId, TagList};

use crate::features::tag_grid::cell;
use crate::features::tag_grid::layout::Columns;
use crate::features::tag_grid::view::rows;
use crate::features::tag_panel::Message;
use crate::ui::palette::TagPalette;
use crate::ui::tokens::*;
use crate::ui::{style, text};

/// The strip never grows past this many rows.
const MAX_ROWS: usize = 3;

/// Render the starred strip in the grid's `columns`, or `None` when no starred tag matches the
/// search. `cursor` is the tag grid's cursor.
pub fn view<'a, S>(
    tag_list: &'a TagList<S>,
    columns: Columns,
    cursor: Option<TagId>,
    tag_palette: TagPalette,
) -> Option<Element<'a, Message>>
where
    S: StoredTagStore + Clone,
{
    let starred = tag_list.starred_tags_in_display_order();
    if starred.is_empty() {
        return None;
    }
    let shown = MAX_ROWS * columns.count as usize;
    let hidden = starred.len().saturating_sub(shown);
    let cells = starred.into_iter().take(shown).map(|tag| {
        let color = tag_palette.color(tag.color_index());
        cell::cell(tag, color, cursor == Some(tag.id()), columns)
    });
    let more = (hidden > 0).then(|| text::caption(fl!("tag-grid-more", count = (hidden as i64))));
    let line = container(space())
        .width(Length::Fill)
        .height(LINE)
        .style(style::divider);
    Some(
        column![rows(cells, columns)]
            .push(more)
            .push(line)
            .spacing(SPACE_XS)
            .width(Length::Fill)
            .into(),
    )
}
