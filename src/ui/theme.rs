//! The theme of the windows (`docs/design/design-system.md` §15.1): a palette made from the
//! tokens, so the parts of iced widgets the components do not style follow them too.

use std::sync::OnceLock;

use iced::theme::{palette as iced_palette, Palette};
use iced::Theme;

use super::tokens::*;

fn build_theme() -> Theme {
    let colors = Palette {
        background: BG_PANEL,
        text: TEXT,
        primary: ACCENT,
        success: SUCCESS,
        warning: WARNING,
        danger: DANGER,
    };
    Theme::custom_with_fn("frename".to_string(), colors, |colors| {
        let mut extended = iced_palette::Extended::generate(colors);
        extended.background.base = iced_palette::Pair::new(BG_PANEL, TEXT);
        extended.background.weak = iced_palette::Pair::new(BG_RAISED, TEXT);
        extended.background.strong = iced_palette::Pair::new(BORDER_CONTROL, TEXT);
        extended.primary.base = iced_palette::Pair::new(ACCENT, TEXT_ON_FILL);
        extended.primary.strong = iced_palette::Pair::new(ACCENT_HOVER, TEXT_ON_FILL);
        extended
    })
}

/// The theme of the windows on the design system: its palette comes from the tokens, so the
/// parts of iced widgets the components do not style follow them too.
pub fn theme() -> Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(build_theme).clone()
}
