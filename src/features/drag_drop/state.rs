//! State for drag and drop feature

use std::path::PathBuf;

/// State for tracking dropped files
#[derive(Debug, Clone, Default)]
pub struct DragDropState {
    /// Currently dropped file path
    pub dropped_file: Option<PathBuf>,
}

impl DragDropState {
    /// Update state when a file is dropped
    pub fn handle_file_dropped(&mut self, path: PathBuf) {
        self.dropped_file = Some(path);
    }
}
