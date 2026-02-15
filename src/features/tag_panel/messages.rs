//! Messages for tag panel feature

/// Messages handled by the tag panel
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle a tag by its index
    ToggleTag(usize),
}
