//! Settings feature - the app settings window: user-facing options persisted in the app database.

mod messages;
mod page;
mod state;
pub mod view;

pub use messages::{KeyMessage, Message, TopUpMessage};
pub use page::Page;
pub use state::SettingsState;

/// The page's scrollable content; showing a page scrolls it to its top.
pub const SETTINGS_SCROLLABLE_ID: &str = "settings-content";
