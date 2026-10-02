//! Clip markers of the open video: the list over the picture (next to the subtitle list), its
//! rows, and the state of the row being edited. The markers themselves live in the open file's
//! `TagList`; the folder workspace applies these messages to it.

pub mod describe;
mod messages;
mod state;
pub mod view;

pub use describe::MomentOutcome;
pub use messages::Message;
pub use state::{MarkersState, MIN_RANGE_MS};

#[cfg(test)]
pub mod state_for_tests {
    pub use super::state::RANGE_HOLD;
}
