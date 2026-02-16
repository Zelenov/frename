//! File workspace: the file currently being edited and the panels that work on it.
//!
//! When a file is opened, the folder workspace sets the file here. This module hosts the
//! file name display, tag panel (and will host more panels). Global tags live in the folder workspace.

mod state;
pub mod view;

pub use state::FileWorkspace;
