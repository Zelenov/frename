//! Messages for rename panel feature

/// Messages handled by the rename panel
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle a tag by its index
    ToggleTag(usize),
}
