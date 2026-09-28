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
    /// Every tag gets the neutral first palette color.
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

    /// The chip color of a tag with the stored color `index` (wrapping around the palette).
    pub fn color(self, index: u8) -> Color {
        match self {
            Self::Colored => TAG_PALETTE[usize::from(index) % TAG_PALETTE.len()],
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
        assert_eq!(TagPalette::Colored.color(1), TAG_PALETTE[1]);
        assert_eq!(TagPalette::Colored.color(17), TAG_PALETTE[1]);
    }

    #[test]
    fn monochrome_gives_every_tag_the_same_color() {
        assert_eq!(
            TagPalette::Monochrome.color(5),
            TagPalette::Monochrome.color(9)
        );
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
