//! Text sizes, line heights and fonts (`docs/design/design-system.md` §4). Only the styles of
//! `ui::text` use them.

use std::sync::RwLock;

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
/// The subtitle strip under the windowed video, at the medium subtitle size; the setting scales
/// it and the fullscreen caption alike (`media_viewer::video::view::subtitle_scale`).
pub const TEXT_SUBTITLE: f32 = 15.0;
pub const LINE_SUBTITLE: f32 = 20.0;
/// The subtitle caption over fullscreen video.
pub const TEXT_VIDEO_CAPTION: f32 = 28.0;
pub const LINE_VIDEO_CAPTION: f32 = 36.0;

// Fonts, bundled (assets/fonts, SIL OFL 1.1).
pub const FONT_DEFAULT: Font = Font::with_name("Inter");
pub const FONT_MONO: Font = Font::with_name("JetBrains Mono NL");
/// The font files, loaded once at start-up.
pub const FONT_FILES: [&[u8]; 3] = [
    include_bytes!("../../../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../../../assets/fonts/JetBrainsMonoNL-Regular.ttf"),
];

/// Noto Sans CJK subsets (`scripts/subset-cjk.py`): the UI language, its family and its two files.
/// The family is named by the text styles, not left to glyph fallback: cosmic-text picks a
/// system "Noto Sans CJK SC" for Han text by the OS locale before it looks at any other font, so
/// Japanese kanji would come out in Chinese forms.
const CJK_FONTS: [(&str, &str, [&[u8]; 2]); 3] = [
    (
        "ja",
        "Noto Sans CJK JP",
        [
            include_bytes!("../../../assets/fonts/NotoSansCJK-JP-Regular.otf"),
            include_bytes!("../../../assets/fonts/NotoSansCJK-JP-Bold.otf"),
        ],
    ),
    (
        "ko",
        "Noto Sans CJK KR",
        [
            include_bytes!("../../../assets/fonts/NotoSansCJK-KR-Regular.otf"),
            include_bytes!("../../../assets/fonts/NotoSansCJK-KR-Bold.otf"),
        ],
    ),
    (
        "zh-Hans",
        "Noto Sans CJK SC",
        [
            include_bytes!("../../../assets/fonts/NotoSansCJK-SC-Regular.otf"),
            include_bytes!("../../../assets/fonts/NotoSansCJK-SC-Bold.otf"),
        ],
    ),
];

/// Every CJK font file, loaded at start-up next to [`FONT_FILES`].
pub fn cjk_font_files() -> impl Iterator<Item = &'static [u8]> {
    CJK_FONTS
        .iter()
        .flat_map(|(_, _, files)| files.iter().copied())
}

/// The family of the UI language when it is a CJK one; the text styles use it instead of Inter.
static CJK_FAMILY: RwLock<Option<&'static str>> = RwLock::new(None);

/// The CJK family the UI of `language` is drawn in, `None` for the languages Inter covers.
fn cjk_family(language: &str) -> Option<&'static str> {
    CJK_FONTS
        .iter()
        .find(|(code, _, _)| *code == language)
        .map(|(_, family, _)| *family)
}

/// Draw the UI in the font of `language` (a code of `i18n::LANGUAGES`). Only `i18n::apply` calls
/// it, with the language being shown, next to switching the text.
pub fn set_language(language: &str) {
    let family = cjk_family(language);
    match CJK_FAMILY.write() {
        Ok(mut current) => *current = family,
        Err(poisoned) => *poisoned.into_inner() = family,
    }
}

fn family() -> Option<&'static str> {
    match CJK_FAMILY.read() {
        Ok(current) => *current,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

fn regular_of(family: Option<&'static str>) -> Font {
    family.map_or(FONT_DEFAULT, Font::with_name)
}

fn strong_of(family: Option<&'static str>) -> Font {
    Font {
        weight: if family.is_some() {
            font::Weight::Bold
        } else {
            font::Weight::Semibold
        },
        ..regular_of(family)
    }
}

/// The UI text font: Inter, or the CJK family of the UI language.
pub fn font() -> Font {
    regular_of(family())
}

/// The UI font for emphasis: Inter Semibold, or the CJK Bold.
pub fn font_strong() -> Font {
    strong_of(family())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A CJK character (U+2E80 and up; the test is limited to those on purpose: letters of other scripts fall back to Inter) the interface
    /// shows in a CJK language must be in that language's font: an edit to `i18n/<lang>/frename.ftl` that adds one needs `scripts/subset-cjk.py` run again.
    #[test]
    fn cjk_fonts_cover_their_texts() {
        for (language, _, files) in CJK_FONTS {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("i18n")
                .join(language)
                .join("frename.ftl");
            let text = std::fs::read_to_string(&path).expect("the language file");
            for (weight, data) in ["Regular", "Bold"].iter().zip(files) {
                let face = ttf_parser::Face::parse(data, 0).expect("a font file");
                let missing: String = text
                    .chars()
                    .filter(|c| *c >= '\u{2E80}' && face.glyph_index(*c).is_none())
                    .collect();
                assert!(
                    missing.is_empty(),
                    "{language} {weight} lacks {missing:?}: run scripts/subset-cjk.py {language}"
                );
            }
        }
    }

    /// Pure on purpose: the global is written by every test that builds the app.
    #[test]
    fn the_ui_font_follows_the_language() {
        assert_eq!(cjk_family("ja"), Some("Noto Sans CJK JP"));
        assert_eq!(cjk_family("de"), None);
        assert_eq!(
            regular_of(cjk_family("ja")),
            Font::with_name("Noto Sans CJK JP")
        );
        assert_eq!(strong_of(cjk_family("ja")).weight, font::Weight::Bold);
        assert_eq!(regular_of(None), FONT_DEFAULT);
        assert_eq!(strong_of(None).weight, font::Weight::Semibold);
    }
}
