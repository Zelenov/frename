//! Video player controls feature

mod fold;
mod messages;
mod progress_bar;
mod state;
pub mod view;

pub use fold::Fold;
pub use messages::Message;
pub use progress_bar::{bar_height, in_out_band, BarMarker};
pub use state::VideoControlsState;
pub use view::volume_after_scroll;
