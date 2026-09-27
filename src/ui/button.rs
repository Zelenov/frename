//! Buttons (`docs/design/design-system.md` §8.1): one primary per window, destructive ones quiet
//! until their confirmation. The caller adds `on_press`; without it the button is disabled.

use iced::widget::text::IntoFragment;
use iced::widget::{button, Button};
use iced::Padding;

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

fn make<'a, M>(label: impl IntoFragment<'a>, kind: ButtonKind) -> Button<'a, M> {
    let font = match kind {
        ButtonKind::Primary | ButtonKind::Danger => FONT_STRONG,
        _ => FONT,
    };
    let label = text::label(label, font);
    button(label).padding(PADDING).style(style::button(kind))
}

/// The one commit action of a window.
pub fn primary<'a, M>(label: impl IntoFragment<'a>) -> Button<'a, M> {
    make(label, ButtonKind::Primary)
}

/// Every other command; Close and Cancel.
pub fn secondary<'a, M>(label: impl IntoFragment<'a>) -> Button<'a, M> {
    make(label, ButtonKind::Secondary)
}

/// A minor action inside a row.
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
