//! Scroll areas (`docs/design/design-system.md` §8.19, §13.2): every one keeps a gutter for its
//! scrollbar on the right, always, so the scrollbar never lies over content and nothing jumps
//! when the content starts or stops overflowing.

use std::time::{Duration, Instant};

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
    armed_at: Option<Instant>,
}

/// How long after a restore a report still counts as its own: a list whose content fits its
/// viewport reports nothing, so the wait must not outlive the toggle.
const RESTORE_WINDOW: Duration = Duration::from_millis(750);

impl ScrollRestore {
    /// The list is being put back at `offset`.
    pub fn arm(&mut self, offset: f32) {
        self.offset = offset;
        self.armed_at = Some(Instant::now());
    }

    /// A scroll report at `offset`; true when it is the one that ends the restore.
    pub fn report(&mut self, offset: f32) -> bool {
        self.report_at(offset, Instant::now())
    }

    fn report_at(&mut self, offset: f32, now: Instant) -> bool {
        let Some(armed_at) = self.armed_at else {
            return false;
        };
        if now.duration_since(armed_at) > RESTORE_WINDOW {
            self.armed_at = None;
            return false;
        }
        let ends = offset != 0.0 || self.offset == 0.0;
        if ends {
            self.armed_at = None;
        }
        ends
    }
}

/// The row a list last scrolled to while following playback; the list scrolls only when the row
/// changes, so a list scrolled by hand is not fought on every tick. A click on a row that seeks
/// (§13.2: a click never scrolls) calls [`Self::clicked`]: the frames that follow, which may
/// still show the old position until the seek lands, do not move the list either.
#[derive(Debug, Default)]
pub struct FollowedRow {
    row: Option<usize>,
    clicked: Option<(Option<usize>, Instant)>,
}

/// How long after a click the frames of the old position are ignored.
const CLICK_WINDOW: Duration = Duration::from_millis(1000);

impl FollowedRow {
    /// Playback is at `row`; true when the list has to scroll to it.
    pub fn follow(&mut self, row: Option<usize>) -> bool {
        self.follow_at(row, Instant::now())
    }

    fn follow_at(&mut self, row: Option<usize>, now: Instant) -> bool {
        if let Some((clicked, at)) = self.clicked {
            if now.duration_since(at) <= CLICK_WINDOW {
                // The seek's own follow and the frames before it landed all come inside the
                // window: whichever row they show, the list stays.
                self.row = clicked;
                return false;
            }
            self.clicked = None;
        }
        std::mem::replace(&mut self.row, row) != row
    }

    /// The list is scrolled to `row` by other means (shown afresh, put back).
    pub fn set(&mut self, row: Option<usize>) {
        self.clicked = None;
        self.row = row;
    }

    /// Another seek than a click: the list follows it.
    pub fn unpin(&mut self) {
        self.clicked = None;
    }

    /// A click on a row sought to `row`: it is in view where the pointer is, leave the list be.
    pub fn clicked(&mut self, row: Option<usize>) {
        self.row = row;
        self.clicked = Some((row, Instant::now()));
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
    fn a_report_long_after_the_restore_is_not_its_own() {
        let mut restore = ScrollRestore::default();
        restore.arm(120.0);
        let later = Instant::now() + RESTORE_WINDOW * 2;
        assert!(
            !restore.report_at(40.0, later),
            "a list that fits never reported"
        );
        assert!(!restore.report(40.0), "and the wait is over");
    }

    #[test]
    fn a_restore_to_the_top_ends_on_any_report() {
        let mut restore = ScrollRestore::default();
        restore.arm(0.0);
        assert!(restore.report(0.0));
    }

    #[test]
    fn a_click_keeps_the_list_where_it_is_until_playback_moves_on() {
        let mut row = FollowedRow::default();
        let at = Instant::now();
        assert!(row.follow_at(Some(2), at), "playback reaches a row: scroll");
        assert!(!row.follow_at(Some(2), at), "same row: no scroll");
        row.clicked(Some(7));
        assert!(
            !row.follow_at(Some(6), at),
            "a frame before the seek landed"
        );
        assert!(!row.follow_at(Some(7), at), "the seek landed on the row");
        assert!(
            !row.follow_at(Some(6), at),
            "a late frame of the old position"
        );
        assert!(!row.follow_at(Some(7), at), "and the landed one again");
        let later = at + CLICK_WINDOW + Duration::from_millis(1);
        assert!(row.follow_at(Some(8), later), "playback moves on: scroll");
    }

    #[test]
    fn another_seek_is_followed_at_once() {
        let mut row = FollowedRow::default();
        row.clicked(Some(7));
        row.unpin();
        assert!(row.follow_at(Some(2), Instant::now()));
    }

    #[test]
    fn a_click_does_not_pin_the_list_for_long() {
        let mut row = FollowedRow::default();
        row.clicked(Some(7));
        let later = Instant::now() + CLICK_WINDOW + Duration::from_millis(1);
        assert!(
            row.follow_at(Some(3), later),
            "the seek went elsewhere: follow it"
        );
    }

    #[test]
    fn a_list_shown_afresh_follows_from_there() {
        let mut row = FollowedRow::default();
        row.clicked(Some(7));
        row.set(Some(1));
        assert!(row.follow_at(Some(2), Instant::now()));
    }
}
