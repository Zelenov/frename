//! State for video player feature

use iced::Task;
use iced_video_player::Video;
use std::path::PathBuf;

use crate::features::video_controls::{self, VideoControlsState};
use super::Message;

/// Video player component state
pub struct VideoPlayerState {
    /// Current video
    current_video: Option<Video>,
    /// Whether a video is currently loading
    loading: bool,
    /// Path of the video being loaded or currently playing
    video_path: Option<PathBuf>,
    /// Controls state
    controls: VideoControlsState,
}

impl Default for VideoPlayerState {
    fn default() -> Self {
        Self {
            current_video: None,
            loading: false,
            video_path: None,
            controls: VideoControlsState::default(),
        }
    }
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
        self.video_path = Some(path.clone());
        self.controls = VideoControlsState::default();

        Task::future(async move {
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let Ok(url) = url::Url::from_file_path(&path) else {
                    log::warn!("Failed to create URL from path: {}", path.display());
                    let _ = tx.send(false);
                    return;
                };

                log::debug!("File URL created: {url}");

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

    /// Handle all video player messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VideoLoaded(success) => {
                self.loading = false;
                if success {
                    if let Some(path) = &self.video_path {
                        match url::Url::from_file_path(path) {
                            Ok(url) => match Video::new(&url) {
                                Ok(video) => {
                                    let duration_secs = video.duration().as_secs_f32();
                                    self.current_video = Some(video);
                                    return Task::done(Message::VideoReady { duration_secs });
                                }
                                Err(e) => {
                                    log::error!("Failed to reload video: {e}");
                                }
                            },
                            Err(()) => {
                                log::error!(
                                    "Failed to create URL from path: {}",
                                    path.display()
                                );
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::VideoReady { duration_secs } => Task::done(Message::Controls(
                video_controls::Message::VideoReady { duration_secs },
            )),
            Message::EndOfStream => Task::done(Message::Controls(
                video_controls::Message::SetPlaying(false),
            )),
            Message::TogglePause => {
                if let Some(video) = &mut self.current_video {
                    let paused = video.paused();
                    video.set_paused(!paused);
                }
                Task::none()
            }
            Message::Controls(ctrl_msg) => {
                self.controls.update(&ctrl_msg);

                // Controls actions that need video player response
                match ctrl_msg {
                    video_controls::Message::TogglePlayPause => {
                        Task::done(Message::TogglePause)
                    }
                    _ => Task::none(),
                }
            }
        }
    }

    /// Whether a video is currently loading
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Get reference to the current video
    pub fn current_video(&self) -> Option<&Video> {
        self.current_video.as_ref()
    }

    /// Get reference to the controls state
    pub fn controls(&self) -> &VideoControlsState {
        &self.controls
    }
}
