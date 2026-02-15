//! State for the tag panel feature

use super::Message;

/// Tag panel state (UI-only; the selected file and tag toggles live in the file workspace).
#[derive(Default)]
pub struct TagPanelState {}

impl TagPanelState {
    /// Handle tag panel messages (tag toggles are applied to the file workspace in folder_workspace).
    pub fn update(&mut self, _message: &Message) {
        // ToggleTag is handled in folder_workspace by mutating file_workspace
    }
}
