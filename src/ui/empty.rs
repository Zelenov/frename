//! Empty states (`docs/design/design-system.md` §8.20, §10.4, §13.7): what is missing in human
//! words, and the button that does the next step. Never a lone glyph, never directions.

use iced::widget::text::IntoFragment;
use iced::widget::{column, container, Column};
use iced::{Alignment, Color, Element, Length};

use super::icons::{icon, Icon};
use super::text;
use super::tokens::*;

/// The empty state of a pane or panel, centered in it: a 48-px icon, a title that names the next
/// step, an optional line of the other ways, and the action.
pub fn pane<'a, M: 'a>(
    glyph: Icon,
    title: impl IntoFragment<'a>,
    line: Option<String>,
    action: Option<Element<'a, M>>,
) -> Element<'a, M> {
    pane_in(glyph, TEXT_SECONDARY, title, line, action)
}

/// [`pane`] whose icon is in `color` (an error: `ERROR`).
pub fn pane_in<'a, M: 'a>(
    glyph: Icon,
    color: Color,
    title: impl IntoFragment<'a>,
    line: Option<String>,
    action: Option<Element<'a, M>>,
) -> Element<'a, M> {
    let block = column![icon(glyph, ICON_XL, color), text::title(title)]
        .push(line.map(text::secondary))
        .push(action.map(|a| {
            container(a).padding(iced::Padding {
                top: SPACE_S,
                ..iced::Padding::ZERO
            })
        }))
        .spacing(SPACE_S)
        .align_x(Alignment::Center);
    container(block).center(Length::Fill).into()
}

/// A small empty area (an empty list, a filter that hides everything): a line and, if there is
/// one, the button that fills it.
pub fn small<'a, M: 'a>(
    line: impl IntoFragment<'a>,
    action: Option<Element<'a, M>>,
) -> Column<'a, M> {
    column![text::body(line)]
        .push(action)
        .spacing(SPACE_S)
        .padding(SPACE_S)
        .width(Length::Fill)
}
