//! Video player UI control

use iced::{Element, Task};
use iced_video_player::{Video, VideoPlayer};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Video player component state
#[derive(Default)]
pub struct VideoPlayerState {
    /// Current video
    pub current_video: Option<Arc<Video>>,
    /// Whether a video is currently loading
    pub loading: bool,
}

/// Messages for video player
#[derive(Debug, Clone)]
pub enum Message {
    /// Video finished loading
    VideoLoaded(bool),
}

impl VideoPlayerState {
    /// Load a video file asynchronously
    pub fn load_video<AppMessage>(
        &mut self,
        path: PathBuf,
        on_loaded: impl Fn(Message) -> AppMessage + 'static + Send + Sync,
    ) -> Task<AppMessage>
    where
        AppMessage: 'static,
    {
        log::info!("Starting video load: {}", path.display());
        self.loading = true;
        self.current_video = None;

        Task::future(async move {
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                // Convert path to URL
                let Ok(url) = url::Url::from_file_path(&path) else {
                    log::warn!("Failed to create URL from path: {}", path.display());
                    let _ = tx.send(false);
                    return;
                };

                log::debug!("File URL created: {url}");

                // Load video
                let success = match Video::new(&url) {
                    Ok(_video) => {
                        log::info!("Video loaded successfully");
                        true
                    }
                    Err(e) => {
                        log::error!("Failed to load video: {e}");
                        false
                    }
                };

                let _ = tx.send(success);
            });

            let success = rx.recv().unwrap_or(false);
            on_loaded(Message::VideoLoaded(success))
        })
    }

    /// Handle video loaded message
    pub fn handle_video_loaded(&mut self, path: &Path, success: bool) {
        self.loading = false;
        if success {
            // Reload the video synchronously for storage
            match url::Url::from_file_path(path) {
                Ok(url) => match Video::new(&url) {
                    Ok(video) => {
                        self.current_video = Some(Arc::new(video));
                    }
                    Err(e) => {
                        log::error!("Failed to reload video: {e}");
                    }
                },
                Err(()) => {
                    log::error!("Failed to create URL from path: {}", path.display());
                }
            }
        }
    }
}

/// Render the video player
pub fn view<'a, Message: 'a + Clone>(state: &'a VideoPlayerState) -> Element<'a, Message> {
    use iced::widget::{container, text};

    if let Some(video) = &state.current_video {
        VideoPlayer::new(video.as_ref())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    } else if state.loading {
        container(text("Loading video...").size(24))
            .center(iced::Length::Fill)
            .into()
    } else {
        container(text("Drop a video file here to play").size(24))
            .center(iced::Length::Fill)
            .into()
    }
}
