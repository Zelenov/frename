//! Indexed palette of tag colors. Tags store a color index; UI looks up the color here.
//! Colors are light enough for black text on the chip (file name display and tag list stripe).

use iced::Color;

/// Tag color palette. Resolve a stored index via [TagColors::color].
pub struct TagColors;

impl TagColors {
    /// Fixed palette (16 colors). Index wraps with modulo when resolving.
    pub const PALETTE: [Color; 16] = [
        Color::from_rgb(0.75, 0.75, 0.78), // 0  light gray
        Color::from_rgb(0.45, 0.75, 0.95), // 1  light blue
        Color::from_rgb(0.65, 0.85, 0.55), // 2  soft green
        Color::from_rgb(0.90, 0.75, 0.45), // 3  amber
        Color::from_rgb(0.75, 0.55, 0.90), // 4  lavender
        Color::from_rgb(0.55, 0.85, 0.80), // 5  teal
        Color::from_rgb(0.95, 0.65, 0.75), // 6  pink
        Color::from_rgb(0.70, 0.80, 0.95), // 7  pale blue
        Color::from_rgb(0.85, 0.70, 0.50), // 8  tan
        Color::from_rgb(0.60, 0.90, 0.70), // 9  mint
        Color::from_rgb(0.90, 0.60, 0.55), // 10 salmon
        Color::from_rgb(0.55, 0.70, 0.95), // 11 periwinkle
        Color::from_rgb(0.80, 0.65, 0.90), // 12 violet
        Color::from_rgb(0.65, 0.90, 0.60), // 13 lime
        Color::from_rgb(0.95, 0.80, 0.50), // 14 gold
        Color::from_rgb(0.70, 0.85, 0.85), // 15 cyan
    ];

    /// Returns the color for the given tag color index (wraps with modulo palette length).
    pub fn color(index: u8) -> Color {
        Self::PALETTE[index as usize % Self::PALETTE.len()]
    }
}

/// How tag colors are resolved. Chosen from the monochrome tags setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagPalette {
    /// Each tag gets its own palette color.
    #[default]
    Colored,
    /// Every tag gets the neutral first palette color.
    Monochrome,
}

/// Monochrome's own color for a tag that is in the tag list. Distinct from `TagColors::PALETTE[0]`
/// (monochrome's color for a tag that is not), which `color_index == 0` cannot reliably stand in
/// for: the sequential color counter can hand a real stored tag index 0 too (issue #53).
const MONOCHROME_STORED: Color = Color::from_rgb(0.55, 0.55, 0.58);

impl TagPalette {
    /// Palette for the monochrome tags setting.
    pub fn from_monochrome(monochrome: bool) -> Self {
        if monochrome {
            Self::Monochrome
        } else {
            Self::Colored
        }
    }

    /// Returns the color for a tag with the given color index and stored-ness under this
    /// palette. `stored` is ignored in `Colored` mode (each tag already has its own color there);
    /// in `Monochrome` mode it is the only thing that decides which of monochrome's two looks a
    /// tag gets, never the color index (see [`MONOCHROME_STORED`]).
    pub fn color(self, index: u8, stored: bool) -> Color {
        match self {
            Self::Colored => TagColors::color(index),
            Self::Monochrome if stored => MONOCHROME_STORED,
            Self::Monochrome => TagColors::PALETTE[0],
        }
    }
}

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

    fn contrast_with_black(c: Color) -> f32 {
        (luminance(c) + 0.05) / 0.05
    }

    /// Issue #53: monochrome must key its two looks on whether a tag is stored, not on its color
    /// index — a real stored tag can land on index 0 too (the sequential color counter wraps).
    #[test]
    fn monochrome_keys_its_two_looks_on_stored_not_on_color_index() {
        let stored_at_zero = TagPalette::Monochrome.color(0, true);
        let unstored_at_zero = TagPalette::Monochrome.color(0, false);
        assert_ne!(
            stored_at_zero, unstored_at_zero,
            "index 0 alone must not decide the look"
        );
        // Whatever index an unstored tag happens to have, it still gets the "not in the list"
        // look, and a stored tag still gets the "in the list" look.
        assert_eq!(TagPalette::Monochrome.color(7, false), unstored_at_zero);
        assert_eq!(TagPalette::Monochrome.color(7, true), stored_at_zero);
    }

    /// Colored mode is unaffected by stored-ness: it already gives every tag its own color.
    #[test]
    fn colored_mode_ignores_stored_ness() {
        assert_eq!(
            TagPalette::Colored.color(3, true),
            TagPalette::Colored.color(3, false)
        );
    }

    #[test]
    fn both_monochrome_looks_keep_black_text_readable() {
        assert!(contrast_with_black(TagPalette::Monochrome.color(0, false)) >= 4.5);
        assert!(contrast_with_black(TagPalette::Monochrome.color(0, true)) >= 4.5);
    }
}
