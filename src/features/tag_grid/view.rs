//! UI for the tag grid (design system §13.5.2): the folder's tags in their order, left to right,
//! top to bottom, in columns as wide as the widest chip; the cursor is the tag the keys act on.

use iced::widget::{column, container, row, stack, Column};
use iced::{Alignment, Element, Length};

use frename_core::{File, StoredTagStore, TagList};

use super::cell;
use super::layout::{self, Columns};
use crate::features::tag_panel::{
    Message, TagPanelState, GRID_CELL_HEIGHT, GRID_GAP, GRID_ROW_STRIDE, TAG_LIST_SCROLLABLE_ID,
};
use crate::ui::icons::Icon;
use crate::ui::palette::TagPalette;
use crate::ui::tokens::*;
use crate::ui::{empty, scroll, text};
use crate::widgets::bounds_reporter::BoundsReporter;

/// The width the grid's cells have to share: the area without the scroll gutter.
fn content_width(state: &TagPanelState) -> Option<f32> {
    state
        .panel_bounds()
        .map(|b| (b.width - SCROLL_GUTTER).max(0.0))
}

/// The grid's columns in the area the grid was last laid out in (one before its first layout).
pub fn grid_columns<S: StoredTagStore + Clone>(
    state: &TagPanelState,
    tag_list: &TagList<S>,
) -> Columns {
    content_width(state).map_or(
        Columns {
            count: 1,
            cell_width: layout::chip_width(0),
        },
        |width| layout::columns_for(tag_list, width),
    )
}

/// `cells` in rows of `columns`, `GRID_GAP` apart; cells never stretch.
pub fn rows<'a>(
    cells: impl IntoIterator<Item = Element<'a, Message>>,
    columns: Columns,
) -> Column<'a, Message> {
    let mut cells = cells.into_iter().peekable();
    let mut rows = Column::new().spacing(GRID_GAP);
    while cells.peek().is_some() {
        let line = row(cells.by_ref().take(columns.count as usize))
            .spacing(GRID_GAP)
            .height(GRID_CELL_HEIGHT);
        rows = rows.push(line);
    }
    rows
}

/// Render the tag grid for the open file; the search's "Create" cell comes first.
pub fn view<'a, S>(
    state: &'a TagPanelState,
    selected_file: Option<&'a File>,
    tag_list: &'a TagList<S>,
    tag_palette: TagPalette,
) -> Element<'a, Message>
where
    S: StoredTagStore + Clone,
{
    if selected_file.is_none() {
        return empty::pane(Icon::Tag, fl!("tag-grid-no-file"), None, None);
    }
    let filter = tag_list.filter_query().trim();
    let ids = tag_list.filtered_display_tag_ids();
    if ids.is_empty() && filter.is_empty() {
        return no_tags();
    }

    let columns = grid_columns(state, tag_list);
    let cursor = state.selected_tag_id();
    let cells = ids.iter().filter_map(|&id| {
        let tag = tag_list.get_tag(id)?;
        let color = tag_palette.color(tag.color_index());
        Some(cell::cell(tag, color, cursor == Some(id), columns))
    });
    let grid = scroll::vertical_with_id(TAG_LIST_SCROLLABLE_ID, rows(cells, columns)).on_scroll(
        |viewport| Message::TagListScrolled {
            scroll_y: viewport.absolute_offset().y,
            viewport_height: viewport.bounds().height,
        },
    );
    let with_bounds = stack![
        BoundsReporter::new(move |bounds| Message::PanelBounds {
            bounds,
            row_height: GRID_ROW_STRIDE,
            cols: columns.count,
            row_content_height: Some(GRID_CELL_HEIGHT),
        }),
        grid,
    ];

    // The "Create" cell stays above the rows, so the scroll arithmetic counts only tags.
    let create =
        (!filter.is_empty() && !tag_list.has_tag_with_name(filter)).then(|| cell::create(filter));
    column![]
        .push(create)
        .push(with_bounds)
        .spacing(GRID_GAP)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// The folder has no tags yet: the one hint that stays text, since typing goes into the tag
/// search right above it.
fn no_tags<'a>() -> Element<'a, Message> {
    let block = column![
        text::title(fl!("tag-grid-no-tags")),
        text::secondary(fl!("tag-grid-no-tags-hint")),
    ]
    .spacing(SPACE_XS)
    .align_x(Alignment::Center);
    container(block).center(Length::Fill).into()
}
