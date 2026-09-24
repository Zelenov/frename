//! Settings feature - the app settings window: user-facing options persisted in the app database.

mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub use state::{FolderConversion, SettingsState};
