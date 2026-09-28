//! List rows and cells (`docs/design/design-system.md` §3.2, §8.9, §13.2): hover is a layer only,
//! the selected row has one look everywhere, and where several things could look selected a bar
//! marks it too, so the state never rests on color alone.

use iced::widget::{container, hover, row, space};
use iced::{Color, Element, Length, Padding};

use super::style;
use super::tokens::*;

/// `content` that brightens under the pointer with `layer` (`HOVER`, `OVERLAY_HOVER` over the
/// video, `CHIP_HOVER` on a chip); corners of `radius`.
pub fn hoverable<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    layer: Color,
    radius: f32,
) -> Element<'a, M> {
    let top = container(space())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::layer(layer, radius));
    hover(content, top)
}

/// The 2-px bar on the left edge of a selected row; empty, it keeps its place.
pub fn selection_bar<'a, M: 'a>(selected: bool) -> Element<'a, M> {
    let color = if selected {
        ACCENT_TEXT
    } else {
        Color::TRANSPARENT
    };
    container(space())
        .width(SELECTION_BAR)
        .height(Length::Fill)
        .style(style::fill(color))
        .into()
}

/// A row of a list: `selected` fills it and draws the bar on its left; hover brightens it.
/// `content` gets the row's inset (0 × 8 after the bar).
pub fn row_item<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    selected: bool,
    hover_layer: Color,
) -> Element<'a, M> {
    let body = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            left: SPACE_S - SELECTION_BAR,
            right: SPACE_S,
            ..Padding::ZERO
        });
    let item = container(row![selection_bar(selected), body].height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::selectable(selected));
    hoverable(item, hover_layer, RADIUS_S)
}
