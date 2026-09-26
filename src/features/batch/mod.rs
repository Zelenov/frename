//! Batch mode: actions over the files checked in the folder list.
//!
//! Every action runs through the same job: one file at a time on a blocking thread, with
//! progress, cancel between files, and a per-file outcome shown in the folder list. An action
//! only says what it does to one file ([`Operation::run`]); each lives in its own module under
//! [`actions`].

mod actions;
mod messages;
mod state;
pub mod view;

pub use actions::{Action, ActionMessage, Operation};
pub use messages::Message;
pub use state::{BatchState, ItemResult, ItemStatus};
