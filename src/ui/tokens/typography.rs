//! Text sizes, line heights and fonts (`docs/design/design-system.md` §4). Only the styles of
//! `ui::text` use them.

use iced::{font, Font};

// Sizes and line heights (logical px): hierarchy by weight and color first, size second.
pub const TEXT_HEADING: f32 = 20.0;
pub const LINE_HEADING: f32 = 28.0;
pub const TEXT_TITLE: f32 = 15.0;
pub const LINE_TITLE: f32 = 20.0;
pub const TEXT_BODY: f32 = 13.0;
pub const LINE_BODY: f32 = 20.0;
pub const TEXT_CAPTION: f32 = 11.0;
pub const LINE_CAPTION: f32 = 16.0;
pub const TEXT_TOOLTIP: f32 = 12.0;
pub const LINE_TOOLTIP: f32 = 16.0;
/// JetBrains Mono's x-height is larger than Inter's: 12 px mono matches 13 px Inter.
pub const TEXT_MONO: f32 = 12.0;
/// One character of `mono` text: JetBrains Mono advances 0.6 em, so a name can be cut exactly.
pub const MONO_CHAR_WIDTH: f32 = TEXT_MONO * 0.6;
/// The average width of one character of `body` text, to guess how many lines a text wraps to.
pub const BODY_CHAR_WIDTH: f32 = 6.8;
/// A generous average width of one character of `caption` text (counts, badges).
pub const CAPTION_CHAR_WIDTH: f32 = 6.5;
/// A mini chip in a file list row.
pub const TEXT_CHIP_MINI: f32 = 12.0;
pub const LINE_CHIP_MINI: f32 = 16.0;
/// The subtitle strip under the windowed video.
pub const TEXT_SUBTITLE: f32 = 15.0;
pub const LINE_SUBTITLE: f32 = 20.0;
/// The subtitle caption over fullscreen video.
pub const TEXT_VIDEO_CAPTION: f32 = 28.0;
pub const LINE_VIDEO_CAPTION: f32 = 36.0;

// Fonts, bundled (assets/fonts, SIL OFL 1.1).
pub const FONT: Font = Font::with_name("Inter");
pub const FONT_STRONG: Font = Font {
    weight: font::Weight::Semibold,
    ..FONT
};
pub const FONT_MONO: Font = Font::with_name("JetBrains Mono NL");
/// The font files, loaded once at start-up.
pub const FONT_FILES: [&[u8]; 3] = [
    include_bytes!("../../../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../../../assets/fonts/JetBrainsMonoNL-Regular.ttf"),
];
