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

impl TagPalette {
    /// Palette for the monochrome tags setting.
    pub fn from_monochrome(monochrome: bool) -> Self {
        if monochrome {
            Self::Monochrome
        } else {
            Self::Colored
        }
    }

    /// Returns the color for the given tag color index under this palette.
    pub fn color(self, index: u8) -> Color {
        match self {
            Self::Colored => TagColors::color(index),
            Self::Monochrome => TagColors::PALETTE[0],
        }
    }
}
