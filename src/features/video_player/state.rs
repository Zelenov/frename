//! State for video player feature

use iced::{Subscription, Task, time};
use iced_video_player::Video;
use std::path::PathBuf;
use std::time::Duration;

use crate::features::video_controls::{self, VideoControlsState};
use super::Message;

/// Video player component state
pub struct VideoPlayerState {
    /// Current video
    current_video: Option<Video>,
    /// Whether a video is currently loading
    loading: bool,
    /// Last load attempt failed (show error X in view)
    load_failed: bool,
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
            load_failed: false,
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
        self.load_failed = false;
        self.current_video = None;
        self.video_path = Some(path.clone());
        self.controls = VideoControlsState::default();

        Task::future(async move {
            // Run blocking video load on a thread pool so the async executor
            // can yield and the UI can redraw "Loading video..." while loading.
            let success = tokio::task::spawn_blocking(move || {
                let Ok(url) = url::Url::from_file_path(&path) else {
                    log::warn!("Failed to create URL from path: {}", path.display());
                    return false;
                };

                log::debug!("File URL created: {url}");

                match Video::new(&url) {
                    Ok(_video) => {
                        log::info!("Video loaded successfully");
                        true
                    }
                    Err(e) => {
                        log::error!("Failed to load video: {e}");
                        false
                    }
                }
            })
            .await
            .unwrap_or(false);

            on_loaded(Message::VideoLoaded(success))
        })
    }

    /// Handle all video player messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VideoLoaded(success) => {
                self.loading = false;
                self.load_failed = !success;
                if !success {
                    log::info!("Video load failed; showing error state (load_failed=true)");
                }

                let Some(path) = self.video_path.as_ref().filter(|_| success) else {
                    return Task::none();
                };

                let Ok(url) = url::Url::from_file_path(path) else {
                    log::error!("Failed to create URL from path: {}", path.display());
                    return Task::none();
                };

                let Ok(video) = Video::new(&url) else {
                    log::error!("Failed to reload video from: {url}");
                    return Task::none();
                };

                let duration_secs = video.duration().as_secs_f32();
                self.current_video = Some(video);
                Task::done(Message::VideoReady { duration_secs })
            }
            Message::VideoReady { duration_secs } => Task::done(Message::Controls(
                video_controls::Message::VideoReady { duration_secs },
            )),
            // NewFrame triggers update → view cycle so the progress bar
            // reads fresh position from the video. No state change needed.
            Message::NewFrame => Task::none(),
            Message::EndOfStream => Task::done(Message::Controls(
                video_controls::Message::SetPlaying(false),
            )),
            Message::TogglePause => {
                let Some(video) = &mut self.current_video else {
                    return Task::none();
                };
                let paused = video.paused();
                video.set_paused(!paused);
                Task::none()
            }
            Message::Seek(position_secs) => {
                let Some(video) = &mut self.current_video else {
                    return Task::none();
                };
                let duration = Duration::from_secs_f64(position_secs as f64);
                if let Err(e) = video.seek(duration, false) {
                    log::error!("Failed to seek: {e}");
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
                    video_controls::Message::Seek(pos) => {
                        Task::done(Message::Seek(pos))
                    }
                    _ => Task::none(),
                }
            }
            Message::Unload => {
                self.current_video = None;
                self.video_path = None;
                self.loading = false;
                Task::done(Message::VideoUnloaded)
            }
            Message::VideoUnloaded => Task::none(),
        }
    }

    /// Whether a video is currently loading
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Whether the last video load attempt failed
    pub fn load_failed(&self) -> bool {
        self.load_failed
    }

    /// Get reference to the current video
    pub fn current_video(&self) -> Option<&Video> {
        self.current_video.as_ref()
    }

    /// Get reference to the controls state
    pub fn controls(&self) -> &VideoControlsState {
        &self.controls
    }

    /// Subscriptions active when a video is loaded:
    /// - periodic tick for progress bar updates
    /// - keyboard shortcuts from controls (Space = play/pause)
    pub fn subscription(&self) -> Subscription<Message> {
        if self.current_video.is_some() {
            Subscription::batch([
                time::every(Duration::from_millis(250)).map(|_| Message::NewFrame),
                self.controls.subscription().map(Message::Controls),
            ])
        } else {
            Subscription::none()
        }
    }
}
