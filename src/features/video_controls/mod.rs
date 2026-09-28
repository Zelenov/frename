//! Video player controls feature

mod fold;
mod messages;
mod progress_bar;
mod state;
pub mod view;

pub use fold::Fold;
pub use messages::Message;
pub use progress_bar::{bar_height, BarMarker};
pub use state::VideoControlsState;
