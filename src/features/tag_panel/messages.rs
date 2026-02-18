//! Messages for tag panel feature

use frename_core::TagId;

/// Messages handled by the tag panel
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle a tag by its id
    ToggleTag(TagId),
    /// Set the filter query for the tag list (case-insensitive contains)
    SetFilter(String),
}
