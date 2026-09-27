//! The style functions behind the `ui` components: every look of the design system, built from
//! the tokens (`docs/design/design-system.md` §8).

use iced::widget::{self, container, overlay::menu};
use iced::{border, Background, Border, Color, Shadow, Theme, Vector};

use super::tokens::*;

/// The kinds of button (§8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Danger,
    DangerGhost,
    /// A navigation item; `true` when it is the page shown.
    Nav(bool),
}

fn faded(color: Color) -> Color {
    Color {
        a: color.a * DISABLED_ALPHA,
        ..color
    }
}

/// Lay a state overlay over `base`.
fn over(base: Color, layer: Color) -> Color {
    let a = layer.a;
    Color {
        r: base.r * (1.0 - a) + layer.r * a,
        g: base.g * (1.0 - a) + layer.g * a,
        b: base.b * (1.0 - a) + layer.b * a,
        a: base.a.max(a),
    }
}

pub fn button(
    kind: ButtonKind,
) -> impl Fn(&Theme, widget::button::Status) -> widget::button::Style + Clone {
    move |_theme, status| {
        use widget::button::Status::{Disabled, Hovered, Pressed};
        let (background, text_color, border_color) = match kind {
            ButtonKind::Primary => (
                match status {
                    Hovered => ACCENT_HOVER,
                    Pressed => ACCENT_PRESSED,
                    _ => ACCENT,
                },
                TEXT_ON_FILL,
                Color::TRANSPARENT,
            ),
            ButtonKind::Danger => (
                match status {
                    Hovered | Pressed => DANGER_HOVER,
                    _ => DANGER,
                },
                TEXT_ON_FILL,
                Color::TRANSPARENT,
            ),
            ButtonKind::Secondary => (
                match status {
                    Hovered => over(BG_RAISED, HOVER),
                    Pressed => over(BG_RAISED, PRESSED),
                    _ => BG_RAISED,
                },
                TEXT,
                BORDER_CONTROL,
            ),
            ButtonKind::DangerGhost => (
                match status {
                    Hovered | Pressed => DANGER_TINT,
                    _ => Color::TRANSPARENT,
                },
                ERROR,
                ERROR,
            ),
            ButtonKind::Nav(selected) => (
                match status {
                    _ if selected => SELECTED,
                    Hovered => HOVER,
                    Pressed => PRESSED,
                    _ => Color::TRANSPARENT,
                },
                if selected { TEXT } else { TEXT_SECONDARY },
                Color::TRANSPARENT,
            ),
        };
        let disabled = status == Disabled;
        let paint = |c: Color| if disabled { faded(c) } else { c };
        widget::button::Style {
            background: Some(Background::Color(paint(background))),
            text_color: paint(text_color),
            border: Border {
                color: paint(border_color),
                width: if border_color == Color::TRANSPARENT {
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

pub fn checkbox(_theme: &Theme, status: widget::checkbox::Status) -> widget::checkbox::Style {
    use widget::checkbox::Status::{Active, Disabled, Hovered};
    let (checked, hovered, disabled) = match status {
        Active { is_checked } => (is_checked, false, false),
        Hovered { is_checked } => (is_checked, true, false),
        Disabled { is_checked } => (is_checked, false, true),
    };
    let (background, edge) = match (checked, hovered, disabled) {
        (_, _, true) => (BG_PANEL, BORDER_SUBTLE),
        (true, true, _) => (ACCENT_HOVER, ACCENT_HOVER),
        (true, false, _) => (ACCENT, ACCENT),
        (false, true, _) => (BG_RAISED, TEXT_SECONDARY),
        (false, false, _) => (BG_RAISED, BORDER_CONTROL),
    };
    widget::checkbox::Style {
        background: Background::Color(background),
        icon_color: if disabled {
            TEXT_DISABLED
        } else {
            TEXT_ON_FILL
        },
        border: Border {
            color: edge,
            width: LINE,
            radius: RADIUS_CHECK.into(),
        },
        text_color: Some(if disabled { TEXT_DISABLED } else { TEXT }),
    }
}

pub fn radio(_theme: &Theme, status: widget::radio::Status) -> widget::radio::Style {
    let (selected, hovered) = match status {
        widget::radio::Status::Active { is_selected } => (is_selected, false),
        widget::radio::Status::Hovered { is_selected } => (is_selected, true),
    };
    widget::radio::Style {
        background: Background::Color(if selected { ACCENT } else { BG_RAISED }),
        dot_color: TEXT_ON_FILL,
        border_width: LINE,
        border_color: match (selected, hovered) {
            (true, _) => ACCENT,
            (false, true) => TEXT_SECONDARY,
            (false, false) => BORDER_CONTROL,
        },
        text_color: Some(TEXT),
    }
}

pub fn text_input(_theme: &Theme, status: widget::text_input::Status) -> widget::text_input::Style {
    let (background, edge, width, value) = match status {
        widget::text_input::Status::Active => (BG_RAISED, BORDER_CONTROL, LINE, TEXT),
        widget::text_input::Status::Hovered => (BG_RAISED, TEXT_SECONDARY, LINE, TEXT),
        widget::text_input::Status::Focused { .. } => (BG_RAISED, ACCENT_TEXT, 2.0 * LINE, TEXT),
        widget::text_input::Status::Disabled => (BG_PANEL, BORDER_SUBTLE, LINE, TEXT_DISABLED),
    };
    widget::text_input::Style {
        background: Background::Color(background),
        border: Border {
            color: edge,
            width,
            radius: RADIUS_S.into(),
        },
        icon: TEXT_SECONDARY,
        placeholder: TEXT_PLACEHOLDER,
        value,
        selection: SELECTED,
    }
}

pub fn pick_list(_theme: &Theme, status: widget::pick_list::Status) -> widget::pick_list::Style {
    let edge = match status {
        widget::pick_list::Status::Active => BORDER_CONTROL,
        widget::pick_list::Status::Hovered => TEXT_SECONDARY,
        widget::pick_list::Status::Opened { .. } => ACCENT_TEXT,
    };
    widget::pick_list::Style {
        text_color: TEXT,
        placeholder_color: TEXT_PLACEHOLDER,
        handle_color: TEXT_SECONDARY,
        background: Background::Color(BG_RAISED),
        border: Border {
            color: edge,
            width: LINE,
            radius: RADIUS_S.into(),
        },
    }
}

/// The list of a dropdown and every menu (§8.14).
pub fn menu(_theme: &Theme) -> menu::Style {
    menu::Style {
        background: Background::Color(BG_OVERLAY),
        border: Border {
            color: BORDER_SUBTLE,
            width: LINE,
            radius: RADIUS_M.into(),
        },
        text_color: TEXT,
        selected_text_color: TEXT,
        selected_background: Background::Color(SELECTED),
        shadow: popup_shadow(),
    }
}

fn popup_shadow() -> Shadow {
    Shadow {
        color: SHADOW,
        offset: Vector::new(0.0, SHADOW_OFFSET_Y),
        blur_radius: SHADOW_BLUR,
    }
}

pub fn tooltip(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(BG_OVERLAY)),
        border: Border {
            color: BORDER_SUBTLE,
            width: LINE,
            radius: RADIUS_M.into(),
        },
        shadow: popup_shadow(),
        snap: true,
    }
}

fn surface(color: Color) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(color)),
        ..container::Style::default()
    }
}

pub fn window(_theme: &Theme) -> container::Style {
    surface(BG_WINDOW)
}

pub fn panel(_theme: &Theme) -> container::Style {
    surface(BG_PANEL)
}

/// A 1-px line between regions.
pub fn divider(_theme: &Theme) -> container::Style {
    surface(BORDER_SUBTLE)
}

/// A bar of `color`: the notice's edge, the selected navigation item's mark, the update dot.
pub fn fill(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        border: border::rounded(RADIUS_S),
        ..container::Style::default()
    }
}

/// The body of a notice (§8.12): raised, or tinted red for an error.
pub fn notice(error: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(if error { BG_ERROR } else { BG_RAISED })),
        border: border::rounded(RADIUS_M),
        ..container::Style::default()
    }
}

pub fn scrollable(_theme: &Theme, status: widget::scrollable::Status) -> widget::scrollable::Style {
    let rail = |active: bool| widget::scrollable::Rail {
        background: Some(Background::Color(BG_RAISED)),
        border: border::rounded(RADIUS_CHECK),
        scroller: widget::scrollable::Scroller {
            background: Background::Color(if active { ACCENT_TEXT } else { BORDER_CONTROL }),
            border: border::rounded(RADIUS_CHECK),
        },
    };
    let (vertical, horizontal) = match status {
        widget::scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_hovered,
            ..
        } => (
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_hovered,
        ),
        widget::scrollable::Status::Dragged {
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_dragged,
            ..
        } => (
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_dragged,
        ),
        widget::scrollable::Status::Active { .. } => (false, false),
    };
    widget::scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail(vertical),
        horizontal_rail: rail(horizontal),
        gap: None,
        auto_scroll: widget::scrollable::AutoScroll {
            background: Background::Color(BG_OVERLAY),
            border: border::rounded(RADIUS_L).width(LINE).color(BORDER_SUBTLE),
            shadow: Shadow::default(),
            icon: TEXT_SECONDARY,
        },
    }
}
