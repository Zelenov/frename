//! Message type for the file workspace region (tag panel + file name panel).
//! Caller (folder_workspace) maps these to its own Message.

use crate::features::{file_name_panel, sync_panel, tag_panel};
use iced::widget::text_editor;

#[derive(Debug, Clone)]
pub enum Message {
    TagPanel(tag_panel::Message),
    FileNamePanel(file_name_panel::Message),
    SyncPanel(sync_panel::Message),
    /// User interacted with the multiline comment editor.
    CommentAction(text_editor::Action),
    /// The read-only AI description under the comment.
    AiBlock(AiBlockMessage),
}

/// What can be done with the AI description of the open file's comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiBlockMessage {
    /// Show or hide its segments.
    ToggleSegments,
    /// Ask to remove it (comment edits are not undoable, so it asks first).
    AskRemove,
    ConfirmRemove,
    CancelRemove,
}
