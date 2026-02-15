//! State for the rename panel feature

use frename_core::RenameCore;

use super::Message;

/// Rename panel state - the entire right side of the application.
/// Owns the core rename engine and coordinates tag selection
/// with file name generation.
pub struct RenamePanelState {
    /// Core rename engine (tags + concatenated file name)
    rename_core: RenameCore,
}

impl Default for RenamePanelState {
    fn default() -> Self {
        Self {
            rename_core: RenameCore::new(),
        }
    }
}

impl RenamePanelState {
    /// Handle rename panel messages
    pub fn update(&mut self, message: &Message) {
        match message {
            Message::ToggleTag(index) => {
                self.rename_core.toggle_tag(*index);
            }
        }
    }

    /// Set the initial file name (e.g. from a dropped file).
    pub fn set_file_name(&mut self, name: impl Into<String>) {
        self.rename_core.set_file_name(name);
    }

    /// Get reference to the rename core
    pub fn rename_core(&self) -> &RenameCore {
        &self.rename_core
    }
}
