//! Menus (`docs/design/design-system.md` §8.14): a popup of commands, each with its icon (or the
//! icon's room), its name and its keys on the right; a latched one is marked with `check`.

use iced::widget::{button, column, container, row, space, Column};
use iced::{Alignment, Element, Length, Padding};

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
        text::body(item.label).color(color),
        space::horizontal()
    ]
    .extend(item.keys.into_iter().map(|key| key_cap(key, false)))
    .push(mark)
    .spacing(SPACE_S)
    .align_y(Alignment::Center);
    button(content)
        .width(Length::Fill)
        .padding(ITEM_PADDING)
        .on_press_maybe(item.on_press)
        .style(style::button(ButtonKind::Ghost))
        .into()
}

/// The popup around `rows` (items, or a row of its own such as a slider).
pub fn menu<'a, M: 'a>(rows: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    let rows: Column<'a, M> = column(rows).spacing(SPACE_XXS);
    container(rows)
        .padding(SPACE_XS)
        .width(Length::Shrink)
        .max_width(MENU_MAX_WIDTH)
        .style(style::popup)
        .into()
}
