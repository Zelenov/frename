//! The segmented control (`docs/design/design-system.md` §8.4): 2–4 segments that show a mode or a
//! view; the current one is filled, SemiBold and underlined, so it does not rest on color.

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

fn segment<'a, M: Clone + 'a>(s: Segment<M>) -> Element<'a, M> {
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
    // The line under the words takes the bottom padding's place, so the segment is 28 high.
    let content = column![words, space().height(CONTROL_PADDING_Y - RING), underline]
        .width(Length::Shrink)
        .align_x(Alignment::Center);
    button(content)
        .padding(PADDING)
        .height(CONTROL_HEIGHT)
        .on_press(s.on_press)
        .style(style::button(ButtonKind::Segment(s.selected)))
        .into()
}

/// The segments side by side.
pub fn segmented<'a, M: Clone + 'a>(segments: impl IntoIterator<Item = Segment<M>>) -> Row<'a, M> {
    Row::with_children(segments.into_iter().map(segment)).align_y(Alignment::Center)
}
