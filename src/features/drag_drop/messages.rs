//! Messages for drag and drop feature

use std::path::PathBuf;
use std::time::Instant;

/// Messages handled by the drag and drop feature
#[derive(Debug, Clone)]
pub enum Message {
    /// A file or folder was dropped into the window (one per item of a drop).
    FileDropped(PathBuf),
    /// Timer tick while dropped items wait: once no item has arrived for a moment, the drop is
    /// complete and one of its items opens.
    Tick(Instant),
}
