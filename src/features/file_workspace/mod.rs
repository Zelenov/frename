//! File workspace: the file currently being edited and the panels that work on it.
//!
//! When a file is opened, the folder workspace sets the file here. This module hosts the
//! file name panel (wrap=true), tag panel (and will host more panels). Global tags live in the folder workspace.

mod messages;
mod state;
pub mod view;

pub use messages::{AiBlockMessage, Message};
pub use state::FileWorkspace;
