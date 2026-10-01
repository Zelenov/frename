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

/// Where a list scrolled to `offset`, in a viewport `viewport` tall, should be so that the row
/// spanning `top..bottom` is shown: `offset` itself when it is, else `fallback`.
pub fn keep_row_in_view(offset: f32, viewport: f32, top: f32, bottom: f32, fallback: f32) -> f32 {
    if top >= offset && bottom <= offset + viewport {
        offset
    } else {
        fallback
    }
}

/// A list being put back at an offset after the view built it anew (fullscreen). A fresh
/// scrollable first reports offset 0, then the offset the restore reached, which may be less
/// than asked for when the content is shorter: the report that matters is the first one that is
/// not the fresh 0 (or any, when 0 was asked for). It carries the new viewport's height.
#[derive(Debug, Default)]
pub struct ScrollRestore {
    offset: f32,
    armed: bool,
}

impl ScrollRestore {
    /// The list is being put back at `offset`.
    pub fn arm(&mut self, offset: f32) {
        self.offset = offset;
        self.armed = true;
    }

    /// A scroll report at `offset`; true when it is the one that ends the restore.
    pub fn report(&mut self, offset: f32) -> bool {
        let ends = self.armed && (offset != 0.0 || self.offset == 0.0);
        if ends {
            self.armed = false;
        }
        ends
    }
}

/// [`vertical`] with the id a task scrolls it by.
pub fn vertical_with_id<'a, M: 'a>(
    id: &'static str,
    content: impl Into<Element<'a, M>>,
) -> Scrollable<'a, M> {
    vertical(content).id(Id::new(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_restore_ends_on_the_first_report_that_is_not_the_fresh_zero() {
        let mut restore = ScrollRestore::default();
        assert!(!restore.report(120.0), "not armed: nothing to end");
        restore.arm(120.0);
        assert!(!restore.report(0.0), "the fresh list's own report");
        assert!(restore.report(90.0), "clamped: shorter than asked for");
        assert!(!restore.report(90.0), "done");
    }

    #[test]
    fn a_restore_to_the_top_ends_on_any_report() {
        let mut restore = ScrollRestore::default();
        restore.arm(0.0);
        assert!(restore.report(0.0));
    }
}
