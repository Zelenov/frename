//! Settings feature - the app settings window: user-facing options persisted in the app database.

mod messages;
mod page;
mod state;
pub mod view;

pub use messages::{KeyMessage, Message};
pub use page::Page;
pub use state::SettingsState;
