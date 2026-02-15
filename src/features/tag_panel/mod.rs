//! Tag panel feature - list of tags for the current file.
//! Shows the tag list for building the file name. File name display is a separate component in the file workspace.

mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub use state::TagPanelState;
