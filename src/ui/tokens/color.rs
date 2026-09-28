//! Colors of the chrome (`docs/design/design-system.md` §3): four surfaces, text, one accent,
//! status. The colors of the content (video, tags, markers) are in `content.rs`.

use iced::Color;

// Surfaces, darkest to lightest: each layer is one step lighter, so depth reads without shadows.
/// Behind panels: the navigation column, button bars.
pub const BG_WINDOW: Color = Color::from_rgb8(0x13, 0x15, 0x19);
/// Panels and pages.
pub const BG_PANEL: Color = Color::from_rgb8(0x1A, 0x1D, 0x22);
/// Fields, secondary buttons, notices, cards.
pub const BG_RAISED: Color = Color::from_rgb8(0x23, 0x27, 0x2E);
/// Menus, dropdown lists, tooltips, dialogs.
pub const BG_OVERLAY: Color = Color::from_rgb8(0x2C, 0x31, 0x39);
/// The surface of an error notice.
pub const BG_ERROR: Color = Color::from_rgb8(0x2A, 0x1F, 0x22);

// States, laid over any surface.
pub const HOVER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.06);
pub const PRESSED: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.12);
/// The selected row, category, cell or segment.
pub const SELECTED: Color = Color::from_rgb8(0x1D, 0x31, 0x5A);
/// Over the window while something is dragged over it (§13.4.4): `BG_WINDOW` at 80 %.
pub const SCRIM_DROP: Color = Color::from_rgba(0.075, 0.082, 0.098, 0.8);

// Lines.
/// Dividers and popup outlines (decorative, no contrast minimum).
pub const BORDER_SUBTLE: Color = Color::from_rgb8(0x30, 0x35, 0x3D);
/// The edge of a field, checkbox, radio or secondary button: at least 3 : 1 on panel and raised.
pub const BORDER_CONTROL: Color = Color::from_rgb8(0x6A, 0x72, 0x80);

// Text: at least 4.5 : 1 on every surface it is used on.
pub const TEXT: Color = Color::from_rgb8(0xE6, 0xE8, 0xEB);
pub const TEXT_SECONDARY: Color = Color::from_rgb8(0xA3, 0xAA, 0xB5);
pub const TEXT_PLACEHOLDER: Color = Color::from_rgb8(0x8F, 0x97, 0xA3);
pub const TEXT_DISABLED: Color = Color::from_rgb8(0x5C, 0x63, 0x6E);
/// Text on `ACCENT` and `DANGER` fills.
pub const TEXT_ON_FILL: Color = Color::from_rgb8(0xFF, 0xFF, 0xFF);
/// The edge of a key cap on a filled button (§13.6.7).
pub const KEY_CAP_ON_FILL: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.6);

// Accent: "selected, focused, or the main action".
pub const ACCENT: Color = Color::from_rgb8(0x25, 0x63, 0xEB);
pub const ACCENT_HOVER: Color = Color::from_rgb8(0x2F, 0x6D, 0xF0);
pub const ACCENT_PRESSED: Color = Color::from_rgb8(0x1D, 0x56, 0xD6);
/// The accent as text or a line on dark: links, focus ring, selection bar.
pub const ACCENT_TEXT: Color = Color::from_rgb8(0x6C, 0xB0, 0xFF);
/// Behind an accent badge: `ACCENT_TEXT` at 16 %.
pub const ACCENT_TINT: Color = Color::from_rgba(0.424, 0.690, 1.0, 0.16);

// Status.
pub const SUCCESS: Color = Color::from_rgb8(0x5C, 0xCB, 0x8F);
pub const WARNING: Color = Color::from_rgb8(0xE9, 0xB9, 0x55);
pub const ERROR: Color = Color::from_rgb8(0xFF, 0x7A, 0x7A);
/// Behind an error badge: `ERROR` at 16 %.
pub const ERROR_TINT: Color = Color::from_rgba(1.0, 0.478, 0.478, 0.16);
/// Behind a warning badge: `WARNING` at 16 %.
pub const WARNING_TINT: Color = Color::from_rgba(0.914, 0.725, 0.333, 0.16);
/// Fill of the button that confirms a destructive action.
pub const DANGER: Color = Color::from_rgb8(0xC9, 0x34, 0x34);
pub const DANGER_HOVER: Color = Color::from_rgb8(0xD2, 0x3B, 0x3B);
/// Tint behind a danger-ghost button on hover, and behind the trash while a chip is over it.
pub const DANGER_TINT: Color = Color::from_rgba(1.0, 0.478, 0.478, 0.10);

/// The one shadow: under popups (menus, tooltips, dialogs) and dragged chips only.
pub const SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);

/// A disabled filled button keeps its colors at this opacity.
pub const DISABLED_ALPHA: f32 = 0.4;

/// `color` at `alpha` of its own opacity: a disabled fill, an idle band.
pub const fn faded(color: Color, alpha: f32) -> Color {
    Color {
        a: color.a * alpha,
        ..color
    }
}

/// `layer` laid over `base`: hover and pressed states on any surface.
pub fn over(base: Color, layer: Color) -> Color {
    let a = layer.a;
    Color {
        r: base.r * (1.0 - a) + layer.r * a,
        g: base.g * (1.0 - a) + layer.g * a,
        b: base.b * (1.0 - a) + layer.b * a,
        a: base.a.max(a),
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    fn luminance(c: Color) -> f32 {
        let channel = |v: f32| {
            if v <= 0.039_28 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
    }

    /// WCAG 2 contrast of two opaque colors.
    pub fn contrast(a: Color, b: Color) -> f32 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    #[test]
    fn text_is_readable_on_every_surface_it_is_used_on() {
        for surface in [BG_WINDOW, BG_PANEL, BG_RAISED, BG_OVERLAY, SELECTED] {
            for text in [TEXT, TEXT_SECONDARY, ACCENT_TEXT, SUCCESS, WARNING, ERROR] {
                assert!(contrast(text, surface) >= 4.5, "{text:?} on {surface:?}");
            }
        }
        assert!(contrast(TEXT_PLACEHOLDER, BG_RAISED) >= 4.5);
        assert!(contrast(ERROR, BG_ERROR) >= 4.5);
        assert!(contrast(TEXT, BG_ERROR) >= 4.5);
        for fill in [ACCENT, ACCENT_HOVER, ACCENT_PRESSED, DANGER, DANGER_HOVER] {
            assert!(contrast(TEXT_ON_FILL, fill) >= 4.5, "white on {fill:?}");
        }
    }

    #[test]
    fn control_edges_stand_out_from_their_surface() {
        for surface in [BG_PANEL, BG_RAISED] {
            assert!(contrast(BORDER_CONTROL, surface) >= 3.0, "{surface:?}");
        }
        assert!(contrast(ACCENT, BG_PANEL) >= 3.0);
    }

    #[test]
    fn a_state_layer_lightens_its_surface() {
        let hovered = over(BG_RAISED, HOVER);
        assert!(hovered.r > BG_RAISED.r && hovered.a == 1.0);
        assert_eq!(faded(ACCENT, 0.5).a, 0.5);
    }
}
