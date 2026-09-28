//! Text buttons (`docs/design/design-system.md` §8.1, §8.21): one primary per window, destructive
//! ones quiet until their confirmation, links for side trips. The caller adds `on_press`; without
//! it the button is disabled. Icon-only buttons are [`super::icon_button`].

use iced::widget::text::IntoFragment;
use iced::widget::{button, row, Button};
use iced::{Alignment, Color, Font, Padding};

use super::icons::{icon, Icon};
use super::style::{self, ButtonKind};
use super::text;
use super::tokens::*;

/// 28 px high with a 20-px line of text: 4 px above and below, 12 px at the sides.
const PADDING: Padding = Padding {
    top: CONTROL_PADDING_Y,
    bottom: CONTROL_PADDING_Y,
    left: SPACE_M,
    right: SPACE_M,
};

/// The padding of a button of `kind`: a link has no box, only its words.
fn padding(kind: ButtonKind) -> Padding {
    if kind == ButtonKind::Link {
        Padding::ZERO
    } else {
        PADDING
    }
}

fn font(kind: ButtonKind) -> Font {
    match kind {
        ButtonKind::Primary | ButtonKind::Danger => FONT_STRONG,
        _ => FONT,
    }
}

fn make<'a, M>(label: impl IntoFragment<'a>, kind: ButtonKind) -> Button<'a, M> {
    button(text::label(label, font(kind)))
        .padding(padding(kind))
        .style(style::button(kind))
}

/// The color an icon inside a button of `kind` takes, so it matches the label.
fn icon_color(kind: ButtonKind, enabled: bool) -> Color {
    let color = match kind {
        ButtonKind::Primary | ButtonKind::Danger => TEXT_ON_FILL,
        ButtonKind::DangerGhost => ERROR,
        ButtonKind::Link => ACCENT_TEXT,
        _ => TEXT,
    };
    if enabled {
        color
    } else {
        faded(color, DISABLED_ALPHA)
    }
}

/// The one commit action of a window.
pub fn primary<'a, M>(label: impl IntoFragment<'a>) -> Button<'a, M> {
    make(label, ButtonKind::Primary)
}

/// Every other command; Close and Cancel.
pub fn secondary<'a, M>(label: impl IntoFragment<'a>) -> Button<'a, M> {
    make(label, ButtonKind::Secondary)
}

/// A minor command that acts right here, inside a row or a list's header: *Invert*, *Undo*.
pub fn ghost<'a, M>(label: impl IntoFragment<'a>) -> Button<'a, M> {
    make(label, ButtonKind::Ghost)
}

/// The first step of a destructive action.
pub fn danger_ghost<'a, M>(label: impl IntoFragment<'a>) -> Button<'a, M> {
    make(label, ButtonKind::DangerGhost)
}

/// Confirms a destructive action; only inside its confirmation.
pub fn danger<'a, M>(label: impl IntoFragment<'a>) -> Button<'a, M> {
    make(label, ButtonKind::Danger)
}

/// Words that open something to look at, or adjust a detail shown next to them (§8.21). Never
/// the only way out of a dead end.
pub fn link<'a, M>(label: impl IntoFragment<'a>) -> Button<'a, M> {
    make(label, ButtonKind::Link)
}

/// A button of `kind` with a 16-px `glyph` before its words, 6 px apart. `enabled` says whether
/// the caller gives it `on_press`, so the icon fades with the label.
pub fn with_icon<'a, M: 'a>(
    kind: ButtonKind,
    glyph: Icon,
    label: impl IntoFragment<'a>,
    enabled: bool,
) -> Button<'a, M> {
    let content = row![
        icon(glyph, ICON_M, icon_color(kind, enabled)),
        text::label(label, font(kind)),
    ]
    .spacing(SPACE_TIGHT)
    .align_y(Alignment::Center);
    button(content)
        .padding(padding(kind))
        .style(style::button(kind))
}
