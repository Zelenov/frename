//! State for drag and drop feature

use std::path::PathBuf;

/// State for tracking dropped files
#[derive(Default)]
pub struct DragDropState {
    /// Currently dropped file path
    pub dropped_file: Option<PathBuf>,
}

impl DragDropState {
    /// Handle file dropped
    pub fn handle_file_dropped(&mut self, path: PathBuf) {
        log::info!("File dropped: {}", path.display());
        self.dropped_file = Some(path);
    }
}
