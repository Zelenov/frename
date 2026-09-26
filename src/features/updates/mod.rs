//! Updates feature - the Updates section of the settings window: the running version, a check
//! for a newer release on GitHub, a background check at start-up, and "Update and restart".
//! Velopack does the checking, downloading and applying (`source`).

mod messages;
mod source;
mod state;
pub mod view;

pub use messages::{Message, Release};
pub use source::apply_on_exit;
pub use state::UpdatesState;
