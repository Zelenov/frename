//! Video player controls feature

mod messages;
mod progress_bar;
mod state;
pub mod view;

pub use messages::Message;
pub use progress_bar::{bar_height, BarMarker};
pub use state::VideoControlsState;
pub use view::MIN_CONTROLS_WIDTH;
