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
    /// Resize the comment box, or let it take the whole panel.
    CommentLayout(CommentLayout),
    /// "Describe with AI" for this clip, from the comment's corner.
    DescribeClip,
    /// "Generate subtitles" for this clip, from the comment's corner.
    SubtitlesClip,
    /// "Suggest tags" for this clip, from the comment's corner.
    SuggestTagsClip,
}

/// Where a paid action of the open clip stands, for its button in the comment's corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipAction {
    /// Not asked for: the button starts it.
    Idle,
    /// Asked for; the job in work comes first.
    Queued,
    /// Its clip is in work.
    Running,
}

/// The buttons of the clip's paid actions.
#[derive(Debug, Clone, Copy)]
pub struct ClipActions {
    pub describe: ClipAction,
    pub subtitles: ClipAction,
    pub suggest_tags: ClipAction,
}

/// How much room the comment box takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommentLayout {
    /// Make the box this many pixels taller (shorter when negative), from the handle above it.
    Grow(f32),
    /// Let the box take the whole panel instead of the tags, or give the tags back.
    ToggleExpanded,
}
