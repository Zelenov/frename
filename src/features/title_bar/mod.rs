//! The main window's own title bar (prototype): the window is drawn without the system's frame,
//! so this bar shows the title and the minimize, maximize and close buttons, and moving the
//! pointer with the button held on its empty part moves the window.

mod messages;
pub mod view;

pub use messages::Message;
