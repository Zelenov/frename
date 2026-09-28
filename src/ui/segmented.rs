//! The segmented control (`docs/design/design-system.md` §8.4): 2–4 segments that show a mode or a
//! view, joined in one outlined box with a line between them; the current one is filled,
//! SemiBold and underlined, so it does not rest on color.

use iced::border::Radius;
use iced::widget::{button, column, container, row, space, Row};
use iced::{Alignment, Color, Element, Length, Padding};

use super::icons::{icon, Icon};
use super::style::{self, ButtonKind};
use super::text;
use super::tokens::*;

/// One segment: its icon, its words, an optional count after them.
pub struct Segment<M> {
    pub icon: Option<Icon>,
    pub label: String,
    pub count: Option<usize>,
    pub selected: bool,
    pub on_press: M,
}

const PADDING: Padding = Padding {
    top: CONTROL_PADDING_Y,
    bottom: 0.0,
    left: SPACE_M,
    right: SPACE_M,
};

/// Where a segment sits: only the ends are rounded, on their outer side, so a selected fill
/// meets the box's corners and the joins between segments are straight.
fn corners(first: bool, last: bool) -> Radius {
    let round = |on: bool| if on { RADIUS_S - LINE } else { 0.0 };
    Radius {
        top_left: round(first),
        bottom_left: round(first),
        top_right: round(last),
        bottom_right: round(last),
    }
}

fn segment<'a, M: Clone + 'a>(s: Segment<M>, first: bool, last: bool) -> Element<'a, M> {
    let color = if s.selected { TEXT } else { TEXT_SECONDARY };
    let label = if s.selected {
        text::strong(s.label)
    } else {
        text::body(s.label).color(color)
    };
    let words = row![]
        .push(s.icon.map(|glyph| icon(glyph, ICON_M, color)))
        .push(label)
        .push(s.count.map(|n| text::caption(n.to_string())))
        .spacing(SPACE_TIGHT)
        .align_y(Alignment::Center);
    let underline = container(space())
        .width(Length::Fill)
        .height(RING)
        .style(style::fill(if s.selected {
            ACCENT_TEXT
        } else {
            Color::TRANSPARENT
        }));
    // The line under the words takes the bottom padding's place, so the segment fills the box.
    let content = column![
        words,
        space().height(CONTROL_PADDING_Y - RING - LINE),
        underline
    ]
    .width(Length::Shrink)
    .align_x(Alignment::Center);
    let kind = ButtonKind::Segment(s.selected);
    let radius = corners(first, last);
    button(content)
        .padding(PADDING)
        .height(CONTROL_HEIGHT - 2.0 * LINE)
        .on_press(s.on_press)
        .style(move |theme, status| {
            let mut look = style::button(kind)(theme, status);
            // The box draws the edge; the segment only its fill.
            look.border.width = 0.0;
            look.border.radius = radius;
            look
        })
        .into()
}

/// The segments side by side in one box, a line between each two.
pub fn segmented<'a, M: Clone + 'a>(
    segments: impl IntoIterator<Item = Segment<M>>,
) -> Element<'a, M> {
    let segments: Vec<Segment<M>> = segments.into_iter().collect();
    let count = segments.len();
    let mut joined: Row<'a, M> = Row::new().align_y(Alignment::Center);
    for (i, s) in segments.into_iter().enumerate() {
        if i > 0 {
            joined = joined.push(
                container(space())
                    .width(LINE)
                    .height(Length::Fill)
                    .style(style::fill(BORDER_CONTROL)),
            );
        }
        joined = joined.push(segment(s, i == 0, i + 1 == count));
    }
    container(joined.height(CONTROL_HEIGHT - 2.0 * LINE))
        .padding(LINE)
        .style(style::outline(false))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_outer_corners_of_the_ends_are_round() {
        let first = corners(true, false);
        assert!(first.top_left > 0.0 && first.bottom_left > 0.0);
        assert_eq!((first.top_right, first.bottom_right), (0.0, 0.0));
        let middle = corners(false, false);
        assert_eq!(middle, Radius::from(0.0));
    }
}
