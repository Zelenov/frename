//! The main window's own title bar: the window is drawn without the system's frame, so this bar
//! shows the folder, the app's controls and the minimize, maximize and close buttons; moving the
//! pointer with the button held on its empty part moves the window, and the window's edges
//! (`view::edges`) resize it.

mod messages;
mod state;
pub mod view;

pub use messages::{Edge, Message};
pub use state::{State, WindowOp};
