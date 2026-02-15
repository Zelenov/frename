//! Folder feature - scans a directory and shows a file list.
//! Clicking a file selects it for playback and renaming.

mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub use state::FolderState;
