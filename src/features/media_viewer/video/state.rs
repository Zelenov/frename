//! State for the video player sub-feature.

use gstreamer as gst;
use gstreamer_video::VideoMeta;
use gst::prelude::*;
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
        self.controls = VideoControlsState::with_volume(self.controls.volume());

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
                    video_controls::Message::SetSegmentStart => self.capture_segment_start(),
                    video_controls::Message::SetSegmentEnd => self.capture_segment_end(),
                    video_controls::Message::TakeScreenshot => self.capture_screenshot(),
                    video_controls::Message::SetVolume(v) => {
                        if let Some(video) = &mut self.current_video {
                            video.set_volume(v as f64);
                        }
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
            Message::CaptureSegmentStart => self.capture_segment_start(),
            Message::CaptureSegmentEnd => self.capture_segment_end(),
            Message::SegmentStartMarked(_) | Message::SegmentEndMarked(_) => Task::none(), // bubbles up via media_viewer
            Message::ScreenshotTaken(_, _) => Task::none(),
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

    fn capture_segment_start(&self) -> Task<Message> {
        let Some(video) = self.current_video.as_ref() else { return Task::none(); };
        let secs = video.position().as_secs_f32().floor();
        Task::done(Message::SegmentStartMarked(secs))
    }

    fn capture_segment_end(&self) -> Task<Message> {
        let Some(video) = self.current_video.as_ref() else { return Task::none(); };
        let secs = video.position().as_secs_f32().ceil();
        Task::done(Message::SegmentEndMarked(secs))
    }

    fn capture_screenshot(&self) -> Task<Message> {
        let Some(video) = self.current_video.as_ref() else {
            log::warn!("capture_screenshot: no video loaded");
            return Task::none();
        };
        let position_ms = video.position().as_millis() as u64;
        log::info!("capture_screenshot: position_ms={position_ms}");
        let jpeg = capture_jpeg(video);
        log::info!("capture_screenshot: jpeg={:?}", jpeg.as_ref().map(|v| v.len()));
        let jpeg = jpeg.unwrap_or_default();
        Task::done(Message::ScreenshotTaken(position_ms, jpeg))
    }

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

// ---------------------------------------------------------------------------
// Frame capture helpers
// ---------------------------------------------------------------------------

/// Capture the current video frame as JPEG bytes.
///
/// Reads the `last-sample` property of the iced_video AppSink.
/// This holds the most recently delivered NV12 frame and does not
/// compete with the worker thread's continuous pull loop.
fn capture_jpeg(video: &Video) -> Option<Vec<u8>> {
    let pipeline = video.pipeline();

    // Locate the iced_video appsink inside the video-sink bin.
    let video_sink: gst::Element = pipeline.property("video-sink");
    let appsink_el = if let Ok(bin) = video_sink.clone().downcast::<gst::Bin>() {
        bin.by_name("iced_video")?
    } else {
        // video-sink wraps its sink pad in a GhostPad; parent of that pad is the bin.
        video_sink.pads().into_iter()
            .find_map(|p| p.dynamic_cast::<gst::GhostPad>().ok())
            .and_then(|gp| gp.parent_element())
            .and_then(|e| e.downcast::<gst::Bin>().ok())
            .and_then(|b| b.by_name("iced_video"))?
    };

    // last-sample holds the most recently delivered frame — no queue contention.
    let sample: Option<gst::Sample> = appsink_el.property("last-sample");
    let sample = match sample {
        Some(s) => s,
        None => {
            log::warn!("last-sample is None — no frame delivered yet");
            return None;
        }
    };

    let caps = sample.caps()?;
    let s = caps.structure(0)?;
    let width = s.get::<i32>("width").ok()? as u32;
    let height = s.get::<i32>("height").ok()? as u32;

    let buffer = sample.buffer()?;
    let map = buffer.map_readable().ok()?;

    // Stride from VideoMeta when available; otherwise assume stride == width.
    let stride = buffer
        .meta::<VideoMeta>()
        .map(|m| m.stride()[0] as u32)
        .unwrap_or(width);

    let rgb = nv12_to_rgb(map.as_slice(), width, height, stride);
    drop(map);

    let img = image::RgbImage::from_raw(width, height, rgb)?;
    let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
    img.write_to(&mut cursor, image::ImageFormat::Jpeg).ok()?;
    Some(cursor.into_inner())
}

fn nv12_to_rgb(yuv: &[u8], width: u32, height: u32, stride: u32) -> Vec<u8> {
    let uv_start = (stride * height) as usize;
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            let y_val = yuv[(y * stride + x) as usize] as f32;
            let uv_off = uv_start + ((y / 2) * stride + (x / 2) * 2) as usize;
            let u = yuv[uv_off] as f32;
            let v = yuv[uv_off + 1] as f32;
            let r = (1.164 * (y_val - 16.0) + 1.596 * (v - 128.0)).clamp(0.0, 255.0) as u8;
            let g = (1.164 * (y_val - 16.0) - 0.813 * (v - 128.0) - 0.391 * (u - 128.0)).clamp(0.0, 255.0) as u8;
            let b = (1.164 * (y_val - 16.0) + 2.018 * (u - 128.0)).clamp(0.0, 255.0) as u8;
            rgb.extend_from_slice(&[r, g, b]);
        }
    }
    rgb
}
