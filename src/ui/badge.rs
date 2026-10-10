//! Badges, key caps and timecodes (`docs/design/design-system.md` §8.16): small facts drawn next
//! to what they are about.

use iced::widget::text::{IntoFragment, Wrapping};
use iced::widget::{container, mouse_area, row};
use iced::{mouse, Alignment, Color, Element, Padding};

use super::icons::{icon, Icon};
use super::style;
use super::text;
use super::tokens::*;
use super::tooltip;

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
    /// A count that is on: the filters on.
    Accent,
    /// Needs attention: "No key".
    Warning,
}

impl BadgeKind {
    fn colors(self) -> (Color, Color) {
        match self {
            BadgeKind::Neutral => (BG_RAISED, TEXT_SECONDARY),
            BadgeKind::Accent => (ACCENT_TINT, ACCENT_TEXT),
            BadgeKind::Warning => (WARNING_TINT, WARNING),
        }
    }
}

/// A badge: a fact in `caption` SemiBold.
pub fn badge<'a, M: 'a>(kind: BadgeKind, content: impl IntoFragment<'a>) -> Element<'a, M> {
    let (background, color) = kind.colors();
    // A badge is one fact on one line: it never wraps, so its surface always covers its words.
    container(
        text::caption_strong(content)
            .color(color)
            .wrapping(Wrapping::None),
    )
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
/// clears it when `on_clear` is given, with what the `x` does as its tooltip.
pub fn timecode<'a, M: Clone + 'a>(
    name: &'a str,
    time: String,
    on_clear: Option<(M, String)>,
) -> Element<'a, M> {
    let clear = on_clear.map(|(message, tip)| {
        tooltip::tip_text(
            mouse_area(icon(Icon::X, ICON_S, TEXT_SECONDARY))
                .on_press(message)
                .interaction(mouse::Interaction::Pointer),
            tip,
            tooltip::Position::Top,
        )
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

/// A suggested timecode range: `mono` on the accent tint, with who suggests it (`name`, "AI")
/// before it and a `check` after it. The whole pill applies it on a click, with what that does
/// as its tooltip.
pub fn suggested_timecode<'a, M: Clone + 'a>(
    name: String,
    time: String,
    on_apply: M,
    tip: String,
) -> Element<'a, M> {
    let pill = container(
        row![
            icon(Icon::Sparkles, ICON_S, ACCENT_TEXT),
            text::caption(name).color(ACCENT_TEXT),
            text::mono(time).color(TEXT),
            icon(Icon::Check, ICON_S, ACCENT_TEXT),
        ]
        .spacing(SPACE_XS)
        .align_y(Alignment::Center),
    )
    .padding(BADGE_PADDING)
    .style(style::badge(ACCENT_TINT));
    tooltip::tip_text(
        mouse_area(pill)
            .on_press(on_apply)
            .interaction(mouse::Interaction::Pointer),
        tip,
        tooltip::Position::Top,
    )
}

/// A tag the AI suggests (§8.16): a `plus`, its name and how sure the AI is, on the accent tint.
/// The whole pill adds it on a click, with what that does (and its key) as the tooltip.
pub fn suggested_tag<'a, M: Clone + 'a>(
    name: String,
    percent: Option<u8>,
    on_add: M,
    tip: tooltip::Tip,
) -> Element<'a, M> {
    let sure = percent.map(|p| text::caption(format!("{p}%")).color(TEXT_SECONDARY));
    let pill = container(
        row![
            icon(Icon::Plus, ICON_S, ACCENT_TEXT),
            text::caption_strong(name)
                .color(TEXT)
                .wrapping(Wrapping::None),
        ]
        .push(sure)
        .spacing(SPACE_XS)
        .align_y(Alignment::Center),
    )
    .padding(BADGE_PADDING)
    .style(style::badge(ACCENT_TINT));
    tooltip::tip(
        mouse_area(pill)
            .on_press(on_add)
            .interaction(mouse::Interaction::Pointer),
        tip,
        tooltip::Position::Top,
    )
}

/// What applies every suggestion at once (§8.16): a `check` and `label` in the accent text, on
/// no surface of its own, after the suggestions it applies.
pub fn apply_all<'a, M: Clone + 'a>(
    label: String,
    on_apply: M,
    tip: tooltip::Tip,
) -> Element<'a, M> {
    let link = container(
        row![
            icon(Icon::Check, ICON_S, ACCENT_TEXT),
            text::caption_strong(label)
                .color(ACCENT_TEXT)
                .wrapping(Wrapping::None),
        ]
        .spacing(SPACE_XS)
        .align_y(Alignment::Center),
    )
    .padding(BADGE_PADDING);
    tooltip::tip(
        mouse_area(link)
            .on_press(on_apply)
            .interaction(mouse::Interaction::Pointer),
        tip,
        tooltip::Position::Top,
    )
}
