//! Scroll areas (`docs/design/design-system.md` §8.19, §13.2): every one keeps a gutter for its
//! scrollbar on the right, always, so the scrollbar never lies over content and nothing jumps
//! when the content starts or stops overflowing.

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{container, scrollable, Id, Scrollable};
use iced::{Element, Length, Padding};

use super::style;
use super::tokens::*;

/// `content` scrolling vertically, with the gutter kept free on its right. The caller adds an
/// `id` and `on_scroll` when it scrolls the area itself.
pub fn vertical<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Scrollable<'a, M> {
    let bar = Scrollbar::new()
        .width(SCROLLBAR_WIDTH)
        .scroller_width(SCROLLBAR_WIDTH);
    let content = container(content).width(Length::Fill).padding(Padding {
        right: SCROLL_GUTTER,
        ..Padding::ZERO
    });
    scrollable(content)
        .direction(Direction::Vertical(bar))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::scrollable)
}

/// [`vertical`] with the id a task scrolls it by.
pub fn vertical_with_id<'a, M: 'a>(
    id: &'static str,
    content: impl Into<Element<'a, M>>,
) -> Scrollable<'a, M> {
    vertical(content).id(Id::new(id))
}
