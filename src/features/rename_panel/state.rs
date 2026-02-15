//! State for the rename panel feature

use super::Message;

/// Rename panel state (UI-only; the selected file lives in the directory).
#[derive(Default)]
pub struct RenamePanelState {}

impl RenamePanelState {
    /// Handle rename panel messages (tag toggles are applied to the selected file in folder_workspace).
    pub fn update(&mut self, _message: &Message) {
        // ToggleTag is handled in file_handler by mutating directory.selected_file_mut()
    }
}
