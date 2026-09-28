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

    /// Monochrome color for a tag that is in the user's tag list. A darker, clearly visible gray
    /// distinct from [Self::PALETTE]'s index 0, which monochrome mode keeps for a tag that is not
    /// in the list (e.g. a stray word from a file name that was never added as a tag). Contrast
    /// with black text: PALETTE[0] ~11.5:1, this color ~5.3:1 (both above the 4.5:1 floor).
    pub const MONOCHROME_KNOWN: Color = Color::from_rgb(0.50, 0.50, 0.53);

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
    /// Every tag gets one of two neutral colors: [TagColors::MONOCHROME_KNOWN] when it is in the
    /// tag list, [TagColors::PALETTE]'s index 0 (light gray) when it is not, so the two stay
    /// visibly different even with colored mode off.
    Monochrome,
}

impl TagPalette {
    /// Palette for the monochrome tags setting.
    pub fn from_monochrome(monochrome: bool) -> Self {
        if monochrome {
            Self::Monochrome
        } else {
            Self::Colored
        }
    }

    /// Returns the color for a tag under this palette: `index` is the tag's palette color index
    /// (used only in [Self::Colored]); `in_list` says whether the tag is in the user's tag list
    /// (used only in [Self::Monochrome], to tell a known tag from one that is not, e.g. a stray
    /// word from a file name). Colored mode's result never depends on `in_list`.
    pub fn color(self, index: u8, in_list: bool) -> Color {
        match self {
            Self::Colored => TagColors::color(index),
            Self::Monochrome => {
                if in_list {
                    TagColors::MONOCHROME_KNOWN
                } else {
                    TagColors::PALETTE[0]
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Relative luminance of an sRGB color (WCAG 2.x formula).
    fn relative_luminance(color: Color) -> f32 {
        fn channel(c: f32) -> f32 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
    }

    /// WCAG contrast ratio between a color and black (#000000) text.
    fn contrast_with_black(color: Color) -> f32 {
        (relative_luminance(color) + 0.05) / 0.05
    }

    #[test]
    fn colored_mode_ignores_in_list_and_uses_the_index() {
        for index in 0..TagColors::PALETTE.len() as u8 {
            assert_eq!(
                TagPalette::Colored.color(index, true),
                TagColors::color(index)
            );
            assert_eq!(
                TagPalette::Colored.color(index, false),
                TagColors::color(index)
            );
        }
    }

    #[test]
    fn monochrome_mode_tells_a_listed_tag_from_an_unlisted_one() {
        let listed = TagPalette::Monochrome.color(3, true);
        let unlisted = TagPalette::Monochrome.color(3, false);
        assert_ne!(listed, unlisted, "monochrome must not collapse the two");
        assert_eq!(listed, TagColors::MONOCHROME_KNOWN);
        assert_eq!(unlisted, TagColors::PALETTE[0]);
    }

    #[test]
    fn monochrome_mode_ignores_the_color_index() {
        // Whether a tag is in the list is the only thing that should matter in monochrome mode;
        // its stored palette index (relevant only to colored mode) must not leak through.
        for index in 0..TagColors::PALETTE.len() as u8 {
            assert_eq!(
                TagPalette::Monochrome.color(index, true),
                TagColors::MONOCHROME_KNOWN
            );
            assert_eq!(
                TagPalette::Monochrome.color(index, false),
                TagColors::PALETTE[0]
            );
        }
    }

    #[test]
    fn both_monochrome_colors_keep_black_text_readable() {
        // WCAG AA for normal text: contrast ratio >= 4.5:1.
        let known_ratio = contrast_with_black(TagColors::MONOCHROME_KNOWN);
        let unlisted_ratio = contrast_with_black(TagColors::PALETTE[0]);
        assert!(
            known_ratio >= 4.5,
            "MONOCHROME_KNOWN contrast with black is {known_ratio:.2}:1, below 4.5:1"
        );
        assert!(
            unlisted_ratio >= 4.5,
            "PALETTE[0] contrast with black is {unlisted_ratio:.2}:1, below 4.5:1"
        );
    }
}
