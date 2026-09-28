//! The design system in code (`docs/design/design-system.md`): tokens, fonts, icons, the theme
//! and the components views are built from. Views on the system never hard-code a color, size
//! or padding; the test in `lint.rs` keeps it so.
//!
//! `ui` holds tokens, styles and stateless constructors of standard controls; `src/widgets/`
//! keeps frename's own stateful or custom-drawn widgets, which take their values from `tokens`.

pub mod badge;
pub mod button;
pub mod empty;
pub mod form;
pub mod icon_button;
pub mod icons;
pub mod layout;
pub mod legacy;
pub mod list;
pub mod menu;
pub mod palette;
pub mod scroll;
pub mod segmented;
pub mod style;
pub mod text;
mod theme;
pub mod tokens;
pub mod tooltip;

#[cfg(test)]
mod lint;

pub use theme::theme;
