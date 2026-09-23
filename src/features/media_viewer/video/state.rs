//! State for the video player sub-feature.

use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video::VideoMeta;
use gst::prelude::*;
use iced::{Subscription, Task, time};
use iced_video_player::{Error as VideoError, Video};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::features::video_controls::{self, VideoControlsState};
use frename_core::{AppDatabase, AppStateStore};
use super::Message;

/// Video player component state.
pub struct VideoPlayerState {
    current_video: Option<Video>,
    loading: bool,
    load_failed: bool,
    controls: VideoControlsState,
    /// Start playback as soon as a video is opened; otherwise it opens paused.
    autoplay: bool,
}

impl Default for VideoPlayerState {
    fn default() -> Self {
        let autoplay = AppDatabase::new()
            .get_app_settings()
            .unwrap_or_default()
            .autoplay_video;
        Self {
            current_video: None,
            loading: false,
            load_failed: false,
            controls: VideoControlsState::default(),
            autoplay,
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
        self.controls = VideoControlsState::with_volume(self.controls.volume());
        let autoplay = self.autoplay;

        Task::future(async move {
            // The video is opened once, here on a blocking thread, and handed to the
            // update below. Opening it a second time on the update thread would stall
            // the UI for as long as the pipeline takes to preroll.
            let opened = tokio::task::spawn_blocking(move || {
                let Ok(url) = url::Url::from_file_path(&path) else {
                    log::warn!("Failed to create URL from path: {}", path.display());
                    return None;
                };
                log::debug!("File URL created: {url}");
                match open_video(&url) {
                    Ok(mut video) => {
                        log::info!("Video loaded successfully");
                        // Paused here, before the update thread sees it, so no audio slips out.
                        if !autoplay {
                            video.set_paused(true);
                        }
                        Some(video)
                    }
                    Err(e) => {
                        log::error!("Failed to load video: {e}");
                        None
                    }
                }
            })
            .await
            .unwrap_or(None);

            Message::VideoLoaded(Arc::new(Mutex::new(opened)))
        })
    }

    /// Handle all video player messages.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VideoLoaded(slot) => {
                self.loading = false;
                // A poisoned lock is as unusable as a failed open, so both land in the
                // error state rather than the neutral "nothing loaded" placeholder.
                let video = slot.lock().ok().and_then(|mut slot| slot.take());
                let Some(video) = video else {
                    self.load_failed = true;
                    log::info!("Video load failed; showing error state");
                    return Task::none();
                };
                self.load_failed = false;
                let duration_secs = video.duration().as_secs_f32();
                self.current_video = Some(video);
                Task::done(Message::VideoReady { duration_secs })
            }
            Message::VideoReady { duration_secs } => {
                let ready =
                    Task::done(Message::Controls(video_controls::Message::VideoReady { duration_secs }));
                // Controls assume playback on ready; a video opened paused has to say otherwise.
                let paused = self.current_video.as_ref().is_some_and(Video::paused);
                if !paused {
                    return ready;
                }
                ready.chain(Task::done(Message::Controls(video_controls::Message::SetPlaying(false))))
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
                self.loading = false;
                Task::done(Message::VideoUnloaded)
            }
            Message::VideoUnloaded => Task::none(),
            // Intercepted by media_viewer/folder_workspace; no-op here.
            Message::ToggleFullscreen => Task::none(),
            Message::SetAutoplay(autoplay) => {
                self.autoplay = autoplay;
                Task::none()
            }
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
        let Some(video) = self.current_video.as_ref() else {
            return Subscription::none();
        };
        // The tick exists only to advance the progress bar, so it is pointless while paused:
        // it used to force a full view rebuild 4x/second for as long as a video stayed open.
        // Iced re-evaluates subscriptions after every update, so pausing stops it immediately.
        let frame_tick = if video.paused() {
            Subscription::none()
        } else {
            time::every(Duration::from_millis(250)).map(|_| Message::NewFrame)
        };
        Subscription::batch([
            frame_tick,
            self.controls.subscription().map(Message::Controls),
        ])
    }
}

// ---------------------------------------------------------------------------
// Pipeline construction
// ---------------------------------------------------------------------------

/// How long the pipeline may take to reach `Playing` before the load is abandoned.
///
/// The 5 seconds `Video::new` allows is not enough for a file on cloud-backed
/// storage (Dropbox or OneDrive "online-only"): the provider downloads the whole file
/// on the first read, so a large 4K clip needs minutes before the demuxer sees a byte.
const PREROLL_TIMEOUT_SECS: u64 = 300;

/// Framerate written into the caps of a variable-framerate source.
///
/// GStreamer signals "variable framerate" as `framerate=0/1`, which many iPhone
/// recordings negotiate, and `Video::from_gst_pipeline` rejects any framerate of zero.
/// The value is informational: the crate only exposes it through `Video::framerate()`,
/// which frename never calls, and frame timing comes from buffer timestamps. So a
/// source that reports no fixed rate is relabelled rather than refused.
const VFR_NOMINAL_FRAMERATE: &str = "30/1";

/// Open a video for playback.
///
/// Variable-framerate sources are rejected outright by the player, so a load that fails
/// for that reason alone is retried once with the framerate relabelled. Everything else
/// takes the first path, which builds exactly the graph `Video::new` builds.
fn open_video(uri: &url::Url) -> Result<Video, VideoError> {
    gst::init()?;

    match open_pipeline(uri, false) {
        Err(VideoError::Framerate(rate)) => {
            log::info!(
                "Source reports framerate {rate} (variable); retrying as {VFR_NOMINAL_FRAMERATE}"
            );
            open_pipeline(uri, true)
        }
        other => other,
    }
}

/// Build a `playbin`, bring it to `Playing` and hand it to the player.
///
/// The preroll happens here rather than inside `Video::from_gst_pipeline`, which allows
/// only 5 seconds — too little for a cold file on cloud-backed storage (Dropbox or
/// OneDrive "online-only"), where the first read waits on a full download. Reaching
/// `Playing` up front means the wait inside the player returns immediately.
fn open_pipeline(uri: &url::Url, relabel_framerate: bool) -> Result<Video, VideoError> {
    let pipeline = gst::parse::launch(&description(uri, relabel_framerate))?
        .downcast::<gst::Pipeline>()
        .map_err(|_| VideoError::Cast)?;

    // Every failure past this point has to stop the pipeline, or playbin keeps the
    // audio device open and the sound of an abandoned load carries on in the background.
    if let Err(e) = preroll(&pipeline) {
        let _ = pipeline.set_state(gst::State::Null);
        return Err(e);
    }

    let (video_sink, text_sink) = match sinks(&pipeline) {
        Ok(sinks) => sinks,
        Err(e) => {
            let _ = pipeline.set_state(gst::State::Null);
            return Err(e);
        }
    };

    Video::from_gst_pipeline(pipeline, video_sink, Some(text_sink))
}

/// The `playbin` description, optionally rewriting the framerate on the way to the sink.
fn description(uri: &url::Url, relabel_framerate: bool) -> String {
    // capssetter has to sit behind the NV12 filter, not in front of it: offering its own
    // framerate to a filter that then has to negotiate it upstream collapses the whole
    // graph with "internal data stream error" — including on files that were fine.
    let video_sink = if relabel_framerate {
        format!(
            "videoscale ! videoconvert ! video/x-raw,format=NV12,pixel-aspect-ratio=1/1 \
             ! capssetter caps=video/x-raw,framerate={VFR_NOMINAL_FRAMERATE} \
             ! appsink name=iced_video drop=true"
        )
    } else {
        "videoscale ! videoconvert ! appsink name=iced_video drop=true \
         caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1"
            .to_string()
    };

    format!(
        "playbin uri=\"{}\" text-sink=\"appsink name=iced_text sync=true drop=true\" \
         video-sink=\"{video_sink}\"",
        uri.as_str()
    )
}

/// Bring the pipeline to `Playing`, giving a cold cloud-backed file time to download.
fn preroll(pipeline: &gst::Pipeline) -> Result<(), VideoError> {
    pipeline.set_state(gst::State::Playing)?;
    pipeline
        .state(gst::ClockTime::from_seconds(PREROLL_TIMEOUT_SECS))
        .0?;
    Ok(())
}

/// Pull the two appsinks that `Video::from_gst_pipeline` expects out of the playbin.
fn sinks(pipeline: &gst::Pipeline) -> Result<(gst_app::AppSink, gst_app::AppSink), VideoError> {
    // playbin wraps the video-sink description in a bin and exposes it through a
    // GhostPad, so the appsink has to be looked up by name inside that bin.
    let video_sink: gst::Element = pipeline.property("video-sink");
    let video_sink = video_sink
        .pads()
        .first()
        .cloned()
        .and_then(|pad| pad.dynamic_cast::<gst::GhostPad>().ok())
        .and_then(|pad| pad.parent_element())
        .and_then(|element| element.downcast::<gst::Bin>().ok())
        .and_then(|bin| bin.by_name("iced_video"))
        .and_then(|element| element.downcast::<gst_app::AppSink>().ok())
        .ok_or_else(|| VideoError::AppSink("iced_video".to_string()))?;

    let text_sink: gst::Element = pipeline.property("text-sink");
    let text_sink = text_sink
        .downcast::<gst_app::AppSink>()
        .map_err(|_| VideoError::AppSink("iced_text".to_string()))?;

    Ok((video_sink, text_sink))
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
