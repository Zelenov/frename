//! Design tokens: every color, space, size, radius, text size and font of the design system
//! (`docs/design/design-system.md` §3–6). Views take values from here, never literals.

use std::time::Duration;

use iced::{font, Color, Font};

// Surfaces, darkest to lightest: each layer is one step lighter, so depth reads without shadows.
/// Behind panels: the navigation column, button bars.
pub const BG_WINDOW: Color = Color::from_rgb8(0x13, 0x15, 0x19);
/// Panels and pages.
pub const BG_PANEL: Color = Color::from_rgb8(0x1A, 0x1D, 0x22);
/// Fields, secondary buttons, notices.
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

// Accent: "selected, focused, or the main action".
pub const ACCENT: Color = Color::from_rgb8(0x25, 0x63, 0xEB);
pub const ACCENT_HOVER: Color = Color::from_rgb8(0x2F, 0x6D, 0xF0);
pub const ACCENT_PRESSED: Color = Color::from_rgb8(0x1D, 0x56, 0xD6);
/// The accent as text or a line on dark: links, focus ring, selection bar.
pub const ACCENT_TEXT: Color = Color::from_rgb8(0x6C, 0xB0, 0xFF);

// Status.
pub const SUCCESS: Color = Color::from_rgb8(0x5C, 0xCB, 0x8F);
pub const WARNING: Color = Color::from_rgb8(0xE9, 0xB9, 0x55);
pub const ERROR: Color = Color::from_rgb8(0xFF, 0x7A, 0x7A);
/// Fill of the button that confirms a destructive action.
pub const DANGER: Color = Color::from_rgb8(0xC9, 0x34, 0x34);
pub const DANGER_HOVER: Color = Color::from_rgb8(0xD2, 0x3B, 0x3B);
/// Tint behind a danger-ghost button on hover.
pub const DANGER_TINT: Color = Color::from_rgba(1.0, 0.478, 0.478, 0.10);

/// The one shadow: under popups (menus, tooltips, dialogs) only.
pub const SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
pub const SHADOW_OFFSET_Y: f32 = 8.0;
pub const SHADOW_BLUR: f32 = 24.0;

/// A disabled filled button keeps its colors at this opacity.
pub const DISABLED_ALPHA: f32 = 0.4;

// Spacing: a 4-px base, each step at least 25 % bigger than the one before.
pub const SPACE_XXS: f32 = 2.0;
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_S: f32 = 8.0;
pub const SPACE_M: f32 = 12.0;
pub const SPACE_L: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;
/// Page padding: top and bottom.
pub const PAGE_PADDING_Y: f32 = 20.0;

// Sizes.
/// Buttons, fields, dropdowns.
pub const CONTROL_HEIGHT: f32 = 28.0;
/// The button bar at the bottom of a window.
pub const BUTTON_BAR_HEIGHT: f32 = 56.0;
/// A navigation item or a one-line list row.
pub const ROW_HEIGHT: f32 = 32.0;
/// Checkbox and radio.
pub const CHECK_SIZE: f32 = 16.0;
/// Width of the notice's colored edge.
pub const NOTICE_BAR: f32 = 3.0;
/// Width of the selected navigation item's bar.
pub const SELECTION_BAR: f32 = 2.0;
/// A line: dividers, borders.
pub const LINE: f32 = 1.0;
/// Settings navigation column.
pub const NAV_WIDTH: f32 = 188.0;
/// Settings label column.
pub const LABEL_WIDTH: f32 = 160.0;
/// Field widths that show the expected input: a tag, a language, a model with its prices.
pub const FIELD_WIDTH_S: f32 = 160.0;
pub const FIELD_WIDTH_M: f32 = 240.0;
pub const FIELD_WIDTH_L: f32 = 300.0;
/// One checkbox of a wrapping grid of short choices (languages).
pub const CHOICE_WIDTH: f32 = 120.0;
/// The scrollbar's rail and scroller, and the gap between it and the content.
pub const SCROLLBAR_WIDTH: f32 = 6.0;
pub const SCROLLBAR_GAP: f32 = 8.0;
/// Widest tooltip.
pub const TOOLTIP_MAX_WIDTH: f32 = 280.0;
/// The dot that says "an update is ready".
pub const DOT_SIZE: f32 = 7.0;

// Icons.
pub const ICON_S: f32 = 12.0;
pub const ICON_M: f32 = 16.0;

// Radii.
pub const RADIUS_S: f32 = 4.0;
pub const RADIUS_M: f32 = 6.0;
pub const RADIUS_L: f32 = 8.0;
/// Checkbox corners.
pub const RADIUS_CHECK: f32 = 3.0;

// Text sizes and line heights (logical px).
pub const TEXT_HEADING: f32 = 20.0;
pub const LINE_HEADING: f32 = 28.0;
pub const TEXT_BODY: f32 = 13.0;
pub const LINE_BODY: f32 = 20.0;
pub const TEXT_MONO: f32 = 12.0;
pub const TEXT_TOOLTIP: f32 = 12.0;
pub const LINE_TOOLTIP: f32 = 16.0;

// Fonts, bundled (assets/fonts, SIL OFL 1.1).
pub const FONT: Font = Font::with_name("Inter");
pub const FONT_STRONG: Font = Font {
    weight: font::Weight::Semibold,
    ..FONT
};
pub const FONT_MONO: Font = Font::with_name("JetBrains Mono NL");
/// The font files, loaded once at start-up.
pub const FONT_FILES: [&[u8]; 3] = [
    include_bytes!("../../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMonoNL-Regular.ttf"),
];

/// Hover time before a tooltip shows.
pub const TOOLTIP_DELAY: Duration = Duration::from_millis(500);

#[cfg(test)]
mod tests {
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

    fn contrast(a: Color, b: Color) -> f32 {
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
}
