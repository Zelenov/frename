//! Menus (`docs/design/design-system.md` §8.14): a popup of commands 28 high, each with its icon
//! (or the icon's room), its name and its keys on the right; a latched one is marked with
//! `check`. The caller places the menu (a context menu opens under the pointer, see
//! `file_menu`) and closes it; [`size`] says how much room it takes, to keep it inside the window.

use iced::widget::{button, column, container, row, space, Column};
use iced::{Alignment, Element, Length, Padding, Size};

use super::badge::key_cap;
use super::icons::{icon, Icon};
use super::style::{self, ButtonKind};
use super::text;
use super::tokens::*;

/// One command of a menu.
pub struct MenuItem<M> {
    pub icon: Option<Icon>,
    pub label: String,
    /// The keys that do it anywhere, shown on the right.
    pub keys: Vec<&'static str>,
    /// Latched on (a list shown): marked with `check`.
    pub checked: bool,
    /// `None`: shown disabled.
    pub on_press: Option<M>,
}

/// Around the items, inside the edge.
const PADDING: f32 = SPACE_XS;

const ITEM_PADDING: Padding = Padding {
    top: (MENU_ITEM_HEIGHT - LINE_BODY) / 2.0,
    bottom: (MENU_ITEM_HEIGHT - LINE_BODY) / 2.0,
    left: SPACE_S,
    right: SPACE_S,
};

/// A menu item's row.
pub fn item<'a, M: Clone + 'a>(item: MenuItem<M>) -> Element<'a, M> {
    let color = if item.on_press.is_some() {
        TEXT
    } else {
        TEXT_DISABLED
    };
    let glyph: Element<'a, M> = match item.icon {
        Some(glyph) => icon(glyph, ICON_M, color).into(),
        None => space().width(ICON_M).into(),
    };
    let mark: Element<'a, M> = if item.checked {
        icon(Icon::Check, ICON_M, ACCENT_TEXT).into()
    } else {
        space().width(ICON_M).into()
    };
    let content = row![
        glyph,
        container(text::body(item.label).color(color)).width(Length::Fill)
    ]
    .extend(item.keys.into_iter().map(|key| key_cap(key, false)))
    .push(mark)
    .spacing(SPACE_S)
    .align_y(Alignment::Center);
    // Clipped: a long label never draws past the item's background.
    button(content)
        .clip(true)
        .width(Length::Fill)
        .padding(ITEM_PADDING)
        .on_press_maybe(item.on_press)
        .style(style::button(ButtonKind::MenuItem))
        .into()
}

/// A row of a menu that is not a command (a checkbox and a count): as high as an item.
pub fn row_of<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(content)
        .padding(ITEM_PADDING)
        .width(Length::Fill)
        .into()
}

/// The popup around `rows` (items, or a row of its own such as a slider), `width` wide.
///
/// Items fill the menu's width, and iced sizes a `Shrink` column by its children that are not
/// `Fill`: a menu of items only would be 0 px wide (just its padding). So `width` is `Shrink` only
/// when a row has a width of its own (the volume slider); otherwise a fixed width.
pub fn menu<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
    width: Length,
) -> Element<'a, M> {
    // Items touch: a menu is exactly its items and its edge high (see `size`).
    let rows: Column<'a, M> = column(rows);
    container(rows)
        .padding(PADDING)
        .width(width)
        .max_width(MENU_MAX_WIDTH)
        .style(style::popup)
        .into()
}

/// The size of a menu of `items` items, [`MENU_WIDTH`] wide (its edge is drawn inside it).
pub fn size(items: usize) -> Size {
    Size::new(MENU_WIDTH, items as f32 * MENU_ITEM_HEIGHT + 2.0 * PADDING)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_menu_is_as_high_as_its_items_and_its_edge() {
        let size = size(3);
        assert_eq!(size.width, MENU_WIDTH);
        assert_eq!(size.height, 3.0 * MENU_ITEM_HEIGHT + 2.0 * SPACE_XS);
    }
}
