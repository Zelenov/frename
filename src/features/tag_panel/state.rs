//! State for the tag panel feature

use frename_core::TagId;

use super::Message;

/// Tag panel state (UI-only; the selected file and tag toggles live in the file workspace).
#[derive(Default)]
pub struct TagPanelState {
    /// Tag list cursor: which tag row is selected (for keyboard Up/Down and selected bar).
    selected_tag_id: Option<TagId>,
    /// Tag row under the mouse (for showing delete button on hover).
    hovered_tag_id: Option<TagId>,
}

impl TagPanelState {
    /// Which tag is currently selected in the list (None = no selection).
    pub fn selected_tag_id(&self) -> Option<TagId> {
        self.selected_tag_id
    }

    /// Set the selected tag (called from folder_workspace when handling SelectUp/SelectDown/ToggleTag).
    pub fn set_selected(&mut self, id: Option<TagId>) {
        self.selected_tag_id = id;
    }

    /// Which tag row is hovered (for showing delete button).
    pub fn hovered_tag_id(&self) -> Option<TagId> {
        self.hovered_tag_id
    }

    /// Set the hovered tag (called from folder_workspace when handling TagHovered).
    pub fn set_hovered(&mut self, id: Option<TagId>) {
        self.hovered_tag_id = id;
    }

    /// Handle tag panel messages (tag toggles and selection are applied in folder_workspace).
    pub fn update(&mut self, _message: &Message) {
        // ToggleTag, SelectUp, SelectDown are handled in folder_workspace
    }
}
