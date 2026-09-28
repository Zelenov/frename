//! Looks of surfaces (`docs/design/design-system.md` §3.1, §6, §8.12–8.18): windows, panels,
//! cards, popups, notices, badges and the lines between them.

use iced::widget::container::Style;
use iced::{border, Background, Border, Color, Theme};

use super::popup_shadow;
use crate::ui::tokens::*;

/// A surface of `color` with the primary text on it.
fn surface(color: Color) -> Style {
    Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(color)),
        ..Style::default()
    }
}

pub fn window(_theme: &Theme) -> Style {
    surface(BG_WINDOW)
}

pub fn panel(_theme: &Theme) -> Style {
    surface(BG_PANEL)
}

/// A raised card: the file name card, a key-value block (§13.5.6).
pub fn card(_theme: &Theme) -> Style {
    Style {
        border: border::rounded(RADIUS_M),
        ..surface(BG_RAISED)
    }
}

/// A raised strip without corners: the lock line over the file list.
pub fn raised(_theme: &Theme) -> Style {
    surface(BG_RAISED)
}

/// Behind the picture.
pub fn video(_theme: &Theme) -> Style {
    surface(VIDEO_BG)
}

/// The side list over the picture (§13.3.6).
pub fn overlay_list(_theme: &Theme) -> Style {
    surface(OVERLAY_LIST)
}

/// The fullscreen caption's pill (§13.3.8).
pub fn caption_pill(_theme: &Theme) -> Style {
    Style {
        border: border::rounded(RADIUS_L),
        ..surface(OVERLAY_CAPTION)
    }
}

/// A popup: a tooltip, a floating notice, a marker cluster's head.
pub fn popup(_theme: &Theme) -> Style {
    Style {
        border: Border {
            color: BORDER_SUBTLE,
            width: LINE,
            radius: RADIUS_M.into(),
        },
        shadow: popup_shadow(),
        ..surface(BG_OVERLAY)
    }
}

/// A 1-px line between regions.
pub fn divider(_theme: &Theme) -> Style {
    surface(BORDER_SUBTLE)
}

/// A bar or dot of `color`: the notice's edge, the selection bar, the update dot.
pub fn fill(color: Color) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        background: Some(Background::Color(color)),
        border: border::rounded(RADIUS_S),
        ..Style::default()
    }
}

/// A round dot of `color`.
pub fn dot(color: Color) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        background: Some(Background::Color(color)),
        border: border::rounded(RADIUS_FULL),
        ..Style::default()
    }
}

/// A selected row or cell (§3.2): the one selected look everywhere. `false` is transparent.
pub fn selectable(selected: bool) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        background: selected.then_some(Background::Color(SELECTED)),
        border: border::rounded(RADIUS_S),
        ..Style::default()
    }
}

/// A hover layer of `color` laid over a row or a chip.
pub fn layer(color: Color, radius: f32) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        background: Some(Background::Color(color)),
        border: border::rounded(radius),
        ..Style::default()
    }
}

/// A dashed-looking outline: the "create" cell, the trash; `danger` while something is over it.
pub fn outline(danger: bool) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        background: danger.then_some(Background::Color(DANGER_TINT)),
        border: Border {
            color: if danger { ERROR } else { BORDER_CONTROL },
            width: LINE,
            radius: RADIUS_S.into(),
        },
        ..Style::default()
    }
}

/// The body of a notice (§8.12): raised, or tinted red for an error.
pub fn notice(error: bool) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(if error { BG_ERROR } else { BG_RAISED })),
        border: border::rounded(RADIUS_M),
        ..Style::default()
    }
}

/// A badge's surface (§8.16).
pub fn badge(background: Color) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        background: Some(Background::Color(background)),
        border: border::rounded(RADIUS_S),
        ..Style::default()
    }
}

/// A key cap (§8.16); `on_fill` for one inside a filled button.
pub fn key_cap(on_fill: bool) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        background: (!on_fill).then_some(Background::Color(BG_RAISED)),
        border: Border {
            color: if on_fill {
                KEY_CAP_ON_FILL
            } else {
                BORDER_CONTROL
            },
            width: LINE,
            radius: RADIUS_CHECK.into(),
        },
        ..Style::default()
    }
}
