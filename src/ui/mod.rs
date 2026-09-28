//! The design system in code (`docs/design/design-system.md`): tokens, fonts, icons, the theme
//! and the components views are built from. Views on the system never hard-code a color, size
//! or padding; the test in `lint.rs` keeps it so.
//!
//! `ui` holds tokens, styles and stateless constructors of standard controls; `src/widgets/`
//! keeps frename's own stateful or custom-drawn widgets, which take their values from `tokens`.

pub mod button;
pub mod form;
pub mod icons;
pub mod layout;
pub mod legacy;
pub mod menu;
mod style;
pub mod text;
pub mod tokens;

#[cfg(test)]
mod lint;

use std::sync::OnceLock;

use iced::theme::{palette, Palette};
use iced::Theme;

use tokens::*;

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
        let mut extended = palette::Extended::generate(colors);
        extended.background.base = palette::Pair::new(BG_PANEL, TEXT);
        extended.background.weak = palette::Pair::new(BG_RAISED, TEXT);
        extended.background.strong = palette::Pair::new(BORDER_CONTROL, TEXT);
        extended.primary.base = palette::Pair::new(ACCENT, TEXT_ON_FILL);
        extended.primary.strong = palette::Pair::new(ACCENT_HOVER, TEXT_ON_FILL);
        extended
    })
}

/// The theme of the windows on the design system: its palette comes from the tokens, so the
/// parts of iced widgets the components do not style follow them too.
pub fn theme() -> Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(build_theme).clone()
}
