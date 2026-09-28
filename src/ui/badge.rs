//! Badges, key caps and timecodes (`docs/design/design-system.md` §8.16): small facts drawn next
//! to what they are about.

use iced::widget::text::IntoFragment;
use iced::widget::{container, mouse_area, row};
use iced::{mouse, Alignment, Color, Element, Padding};

use super::icons::{icon, Icon};
use super::style;
use super::text;
use super::tokens::*;

/// A badge's inset: 0 × 6.
const BADGE_PADDING: Padding = Padding {
    top: 0.0,
    bottom: 0.0,
    left: SPACE_TIGHT,
    right: SPACE_TIGHT,
};

/// A key cap's inset: 0 × 4.
const KEY_PADDING: Padding = Padding {
    top: 0.0,
    bottom: 0.0,
    left: SPACE_XS,
    right: SPACE_XS,
};

/// How a badge is tinted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    /// `SRT`, "3 markers": raised, secondary text.
    Neutral,
    /// A count that is on: the filters on, the checked files.
    Accent,
    /// Needs attention: "No key".
    Warning,
    /// "Not saved".
    Error,
}

impl BadgeKind {
    fn colors(self) -> (Color, Color) {
        match self {
            BadgeKind::Neutral => (BG_RAISED, TEXT_SECONDARY),
            BadgeKind::Accent => (ACCENT_TINT, ACCENT_TEXT),
            BadgeKind::Warning => (WARNING_TINT, WARNING),
            BadgeKind::Error => (ERROR_TINT, ERROR),
        }
    }
}

/// A badge: a fact in `caption` SemiBold.
pub fn badge<'a, M: 'a>(kind: BadgeKind, content: impl IntoFragment<'a>) -> Element<'a, M> {
    let (background, color) = kind.colors();
    container(text::caption_strong(content).color(color))
        .padding(BADGE_PADDING)
        .style(style::badge(background))
        .into()
}

/// One key as printed on the keyboard (`Ctrl`, `F2`); one cap per key, no "+" between them.
/// `on_fill` draws it inside a filled button.
pub fn key_cap<'a, M: 'a>(key: &'a str, on_fill: bool) -> Element<'a, M> {
    let color = if on_fill {
        TEXT_ON_FILL
    } else {
        TEXT_SECONDARY
    };
    container(text::caption(key).color(color))
        .padding(KEY_PADDING)
        .style(style::key_cap(on_fill))
        .into()
}

/// A timecode: `mono` on a raised pill, with the `name` ("IN", "OUT") before it and an `x` that
/// clears it when `on_clear` is given.
pub fn timecode<'a, M: Clone + 'a>(
    name: &'a str,
    time: String,
    on_clear: Option<M>,
) -> Element<'a, M> {
    let clear = on_clear.map(|message| {
        mouse_area(icon(Icon::X, ICON_S, TEXT_SECONDARY))
            .on_press(message)
            .interaction(mouse::Interaction::Pointer)
    });
    container(
        row![text::caption(name), text::mono(time).color(TEXT)]
            .push(clear)
            .spacing(SPACE_XS)
            .align_y(Alignment::Center),
    )
    .padding(BADGE_PADDING)
    .style(style::badge(BG_OVERLAY))
    .into()
}
