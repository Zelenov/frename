//! Folder workspace: the open folder and its panels (folder list, video, rename).
//! This is the main content of the app once a folder is chosen.

mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub use state::FolderWorkspace;
