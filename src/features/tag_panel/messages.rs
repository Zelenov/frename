//! Messages for tag panel feature

/// Messages handled by the tag panel
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle a tag by its index (index into the full tag list)
    ToggleTag(usize),
    /// Set the filter query for the tag list (case-insensitive contains)
    SetFilter(String),
}
