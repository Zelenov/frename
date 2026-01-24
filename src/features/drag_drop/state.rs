//! State for drag and drop feature

use std::path::PathBuf;
use iced_video_player::Video;

/// State for tracking dropped files
#[derive(Default)]
pub struct DragDropState {
    /// Currently dropped file path
    pub dropped_file: Option<PathBuf>,
    /// Current video player
    pub current_video: Option<Video>,
}

impl DragDropState {
    /// Update state when a file is dropped
    pub fn handle_file_dropped(&mut self, path: PathBuf) {
        log::info!("File dropped: {:?}", path);
        
        self.dropped_file = Some(path.clone());
        
        // Try to load the file as a video
        let Ok(url) = url::Url::from_file_path(&path) else {
            log::warn!("Failed to create file URL from path: {:?}", path);
            return;
        };
        
        log::debug!("File URL created: {}", url);
        match Video::new(&url) {
            Ok(video) => {
                log::info!("Video loaded successfully");
                self.current_video = Some(video);
            }
            Err(e) => {
                log::error!("Failed to load video: {}", e);
                eprintln!("Failed to load video: {}", e);
                self.current_video = None;
            }
        }
    }
}
