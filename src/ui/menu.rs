//! Menus (`docs/design/design-system.md` §8.14): a popup surface with items 28 high, each a
//! label with its keys on the right. The caller places the menu (a context menu opens under the
//! pointer) and closes it; [`size`] says how much room it takes, to keep it inside the window.

use iced::widget::{button, container, row, Column};
use iced::{Element, Length, Padding, Size};

use super::style::{self, ButtonKind};
use super::text;
use super::tokens::*;

/// Around the items, inside the edge.
const PADDING: f32 = SPACE_XS;

/// An item: 28 px high with a 20-px line of text, 8 px at the sides.
const ITEM_PADDING: Padding = Padding {
    top: CONTROL_PADDING_Y,
    bottom: CONTROL_PADDING_Y,
    left: SPACE_S,
    right: SPACE_S,
};

/// A menu of `items` (built with [`item`]), [`MENU_WIDTH`] wide.
pub fn menu<'a, M: 'a>(items: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    container(Column::with_children(items).width(Length::Fill))
        .padding(PADDING)
        .width(MENU_WIDTH)
        .style(style::popup)
        .into()
}

/// One item: `label`, and on the right the keys that do the same without the menu.
pub fn item<'a, M: Clone + 'a>(label: String, keys: &'a str, on_press: M) -> Element<'a, M> {
    let content = row![
        text::label(label, FONT).width(Length::Fill),
        text::secondary(keys),
    ]
    .spacing(SPACE_M);
    button(content)
        .padding(ITEM_PADDING)
        .width(Length::Fill)
        .style(style::button(ButtonKind::MenuItem))
        .on_press(on_press)
        .into()
}

/// The size of a menu of `items` items (its edge is drawn inside it).
pub fn size(items: usize) -> Size {
    Size::new(MENU_WIDTH, items as f32 * CONTROL_HEIGHT + 2.0 * PADDING)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_menu_is_as_high_as_its_items_and_its_edge() {
        let size = size(3);
        assert_eq!(size.width, MENU_WIDTH);
        assert_eq!(size.height, 3.0 * CONTROL_HEIGHT + 2.0 * SPACE_XS);
    }
}
