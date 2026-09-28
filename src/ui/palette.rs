//! The colors of the content that the data picks (`docs/design/design-system.md` §3.3): a tag's
//! chip color from its stored index, a marker's color from its Premiere color.

use frename_core::MarkerColor;
use iced::Color;

use super::tokens::*;

/// How tag colors are resolved. Chosen from the monochrome tags setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagPalette {
    /// Each tag gets its own palette color.
    #[default]
    Colored,
    /// Every tag gets one of two grays: `TAG_MONO_KNOWN` when it is in the tag list, the palette's
    /// light gray when it is not (a stray word from a file name), so the two stay apart.
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

    /// The chip color of a tag: `index` is its stored palette color (colored mode only, wrapping
    /// around the palette); `in_list` whether it is in the tag list (monochrome mode only).
    pub fn color(self, index: u8, in_list: bool) -> Color {
        match self {
            Self::Colored => TAG_PALETTE[usize::from(index) % TAG_PALETTE.len()],
            Self::Monochrome if in_list => TAG_MONO_KNOWN,
            Self::Monochrome => TAG_PALETTE[0],
        }
    }
}

/// How a clip marker's color is drawn: Premiere's marker colors, gray for a value frename does
/// not know.
pub fn marker_color(color: MarkerColor) -> Color {
    match color {
        MarkerColor::Green => MARKER_GREEN,
        MarkerColor::Red => MARKER_RED,
        MarkerColor::Orange => MARKER_ORANGE,
        MarkerColor::Yellow => MARKER_YELLOW,
        MarkerColor::White => MARKER_WHITE,
        MarkerColor::Blue => MARKER_BLUE,
        MarkerColor::Cyan => MARKER_CYAN,
        MarkerColor::Lavender => MARKER_LAVENDER,
        MarkerColor::Magenta => MARKER_MAGENTA,
        MarkerColor::Other(_) => TEXT_SECONDARY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_color_index_wraps_around_the_palette() {
        assert_eq!(TagPalette::Colored.color(1, true), TAG_PALETTE[1]);
        assert_eq!(TagPalette::Colored.color(17, false), TAG_PALETTE[1]);
    }

    #[test]
    fn monochrome_tells_a_listed_tag_from_an_unlisted_one_and_ignores_the_index() {
        for index in 0..TAG_PALETTE.len() as u8 {
            assert_eq!(TagPalette::Monochrome.color(index, true), TAG_MONO_KNOWN);
            assert_eq!(TagPalette::Monochrome.color(index, false), TAG_PALETTE[0]);
        }
        assert_ne!(TAG_MONO_KNOWN, TAG_PALETTE[0]);
        assert_eq!(TagPalette::from_monochrome(true), TagPalette::Monochrome);
        assert_eq!(TagPalette::from_monochrome(false), TagPalette::Colored);
    }

    #[test]
    fn every_premiere_color_has_its_own_look() {
        let known: Vec<Color> = MarkerColor::ALL.into_iter().map(marker_color).collect();
        for (i, a) in known.iter().enumerate() {
            assert!(known[i + 1..].iter().all(|b| b != a), "{a:?} twice");
        }
    }
}
