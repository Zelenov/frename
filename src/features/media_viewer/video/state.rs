//! State for the video player sub-feature.

use iced::{Subscription, Task, time};
use iced_video_player::Video;
use std::path::PathBuf;
use std::time::Duration;

use crate::features::video_controls::{self, VideoControlsState};
use super::Message;

/// Video player component state.
pub struct VideoPlayerState {
    current_video: Option<Video>,
    loading: bool,
    load_failed: bool,
    video_path: Option<PathBuf>,
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
    /// Load a video file asynchronously.
    pub fn load_video(&mut self, path: PathBuf) -> Task<Message> {
        log::info!("Starting video load: {}", path.display());
        self.loading = true;
        self.load_failed = false;
        self.current_video = None;
        self.video_path = Some(path.clone());
        self.controls = VideoControlsState::default();

        Task::future(async move {
            let success = tokio::task::spawn_blocking(move || {
                let Ok(url) = url::Url::from_file_path(&path) else {
                    log::warn!("Failed to create URL from path: {}", path.display());
                    return false;
                };
                log::debug!("File URL created: {url}");
                match Video::new(&url) {
                    Ok(_) => { log::info!("Video loaded successfully"); true }
                    Err(e) => { log::error!("Failed to load video: {e}"); false }
                }
            })
            .await
            .unwrap_or(false);

            Message::VideoLoaded(success)
        })
    }

    /// Handle all video player messages.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VideoLoaded(success) => {
                self.loading = false;
                self.load_failed = !success;
                if !success {
                    log::info!("Video load failed; showing error state");
                    return Task::none();
                }
                let Some(path) = self.video_path.as_ref() else {
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
            Message::VideoReady { duration_secs } => {
                Task::done(Message::Controls(video_controls::Message::VideoReady { duration_secs }))
            }
            Message::NewFrame => Task::none(),
            Message::EndOfStream => {
                Task::done(Message::Controls(video_controls::Message::SetPlaying(false)))
            }
            Message::TogglePause => {
                if let Some(video) = &mut self.current_video {
                    let paused = video.paused();
                    video.set_paused(!paused);
                }
                Task::none()
            }
            Message::Seek(position_secs) => {
                if let Some(video) = &mut self.current_video {
                    let duration = Duration::from_secs_f64(position_secs as f64);
                    if let Err(e) = video.seek(duration, false) {
                        log::error!("Failed to seek: {e}");
                    }
                }
                Task::none()
            }
            Message::Controls(ctrl_msg) => {
                self.controls.update(&ctrl_msg);
                match ctrl_msg {
                    video_controls::Message::TogglePlayPause => Task::done(Message::TogglePause),
                    video_controls::Message::Seek(pos) => Task::done(Message::Seek(pos)),
                    video_controls::Message::SeekBack10 => {
                        if let Some(video) = self.current_video.as_ref() {
                            let new_pos = (video.position().as_secs_f32() - 10.0).max(0.0);
                            Task::done(Message::Seek(new_pos))
                        } else {
                            Task::none()
                        }
                    }
                    video_controls::Message::SeekForward10 => {
                        if let Some(video) = self.current_video.as_ref() {
                            let pos = video.position().as_secs_f32();
                            let dur = video.duration().as_secs_f32();
                            Task::done(Message::Seek((pos + 10.0).min(dur)))
                        } else {
                            Task::none()
                        }
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
            // Intercepted by media_viewer/folder_workspace; no-op here.
            Message::ToggleFullscreen => Task::none(),
        }
    }

    pub fn is_loading(&self) -> bool { self.loading }
    pub fn load_failed(&self) -> bool { self.load_failed }
    pub fn current_video(&self) -> Option<&Video> { self.current_video.as_ref() }
    pub fn controls(&self) -> &VideoControlsState { &self.controls }

    /// True when a video is loaded or in the process of loading.
    pub fn is_active(&self) -> bool {
        self.current_video.is_some() || self.loading
    }

    /// Subscriptions active while a video is loaded.
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
