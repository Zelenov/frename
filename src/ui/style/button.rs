//! Button looks (`docs/design/design-system.md` §8.1, §8.4, §8.9, §8.21): one function for every
//! kind, so states (hover, pressed, disabled, selected) work the same on all of them.

use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Shadow, Theme};

use crate::ui::tokens::*;

/// The kinds of button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    /// The one commit action of a window or panel.
    Primary,
    /// Every other command; Close and Cancel.
    Secondary,
    /// A minor command that acts right here, in a row or a list's header.
    Ghost,
    /// Confirms a destructive action.
    Danger,
    /// The first step of a destructive action.
    DangerGhost,
    /// Words that open something to look at (§8.21).
    Link,
    /// A navigation item or a list row; `true` when it is the one selected.
    Nav(bool),
    /// An icon button: transparent; `true` when it is latched on (§8.1).
    Icon(bool),
    /// An icon button over the video, where `HOVER` does not show.
    OverlayIcon,
    /// A segment of a segmented control; `true` when it is the current one (§8.4).
    Segment(bool),
}

impl ButtonKind {
    /// Background, text and edge, before the disabled fade.
    fn colors(self, status: Status) -> (Color, Color, Color) {
        use Status::{Hovered, Pressed};
        let layer = |base: Color| match status {
            Hovered => over(base, HOVER),
            Pressed => over(base, PRESSED),
            _ => base,
        };
        let clear = Color::TRANSPARENT;
        match self {
            ButtonKind::Primary => (
                match status {
                    Hovered => ACCENT_HOVER,
                    Pressed => ACCENT_PRESSED,
                    _ => ACCENT,
                },
                TEXT_ON_FILL,
                clear,
            ),
            ButtonKind::Danger => (
                match status {
                    Hovered | Pressed => DANGER_HOVER,
                    _ => DANGER,
                },
                TEXT_ON_FILL,
                clear,
            ),
            ButtonKind::Secondary => (layer(BG_RAISED), TEXT, BORDER_CONTROL),
            ButtonKind::Ghost | ButtonKind::Icon(false) => (layer(clear), TEXT, clear),
            ButtonKind::DangerGhost => (
                match status {
                    Hovered | Pressed => DANGER_TINT,
                    _ => clear,
                },
                ERROR,
                ERROR,
            ),
            ButtonKind::Link => (clear, ACCENT_TEXT, clear),
            ButtonKind::Nav(true) | ButtonKind::Icon(true) => (SELECTED, TEXT, clear),
            ButtonKind::Nav(false) => (layer(clear), TEXT_SECONDARY, clear),
            ButtonKind::OverlayIcon => (
                match status {
                    Hovered | Pressed => OVERLAY_HOVER,
                    _ => clear,
                },
                TEXT,
                clear,
            ),
            ButtonKind::Segment(true) => (SELECTED, TEXT, BORDER_CONTROL),
            ButtonKind::Segment(false) => (layer(BG_RAISED), TEXT_SECONDARY, BORDER_CONTROL),
        }
    }
}

/// The style function of a button of `kind`.
pub fn button(kind: ButtonKind) -> impl Fn(&Theme, Status) -> Style + Clone {
    move |_theme, status| {
        let (background, text_color, edge) = kind.colors(status);
        let paint = |c: Color| {
            if status == Status::Disabled {
                faded(c, DISABLED_ALPHA)
            } else {
                c
            }
        };
        Style {
            background: Some(Background::Color(paint(background))),
            text_color: paint(text_color),
            border: Border {
                color: paint(edge),
                width: if edge == Color::TRANSPARENT {
                    0.0
                } else {
                    LINE
                },
                radius: RADIUS_S.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style(kind: ButtonKind, status: Status) -> Style {
        button(kind)(&Theme::Dark, status)
    }

    #[test]
    fn hover_is_a_layer_and_never_looks_selected() {
        let idle = style(ButtonKind::Nav(false), Status::Active);
        let hovered = style(ButtonKind::Nav(false), Status::Hovered);
        let selected = style(ButtonKind::Nav(true), Status::Active);
        assert_ne!(idle.background, hovered.background);
        assert_ne!(hovered.background, selected.background);
    }

    #[test]
    fn a_disabled_button_keeps_its_look_faded() {
        let active = style(ButtonKind::Primary, Status::Active);
        let disabled = style(ButtonKind::Primary, Status::Disabled);
        assert_eq!(disabled.text_color.a, active.text_color.a * DISABLED_ALPHA);
    }

    #[test]
    fn only_controls_with_an_edge_draw_one() {
        assert_eq!(
            style(ButtonKind::Secondary, Status::Active).border.width,
            LINE
        );
        assert_eq!(
            style(ButtonKind::Icon(false), Status::Active).border.width,
            0.0
        );
    }
}
