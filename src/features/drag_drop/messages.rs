//! Messages for drag and drop feature

use std::path::PathBuf;

/// Messages handled by the drag and drop feature
#[derive(Debug, Clone)]
pub enum Message {
    /// File was dropped into the window
    FileDropped(PathBuf),
}
