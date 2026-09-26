//! State for the video player sub-feature.

use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video::VideoMeta;
use iced::{time, Subscription, Task};
use iced_video_player::{Error as VideoError, Video};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::view::{CUE_LIST_SCROLLABLE_ID, CUE_ROW_PITCH};
use super::Message;
use crate::features::markers;
use crate::features::video_controls::{self, VideoControlsState};
use frename_core::{load_subtitles, AppDatabase, AppStateStore, Subtitles};

/// How long a note in the controls bar stays.
const NOTICE_DURATION: Duration = Duration::from_secs(2);

/// What the list over the right of the picture shows. One list at a time, so a windowed
/// video is not covered twice. Kept across files, like volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overlay {
    /// No list; fullscreen still shows the subtitle list when there are subtitles.
    #[default]
    Closed,
    Subtitles,
    Markers,
}

/// Video player component state.
pub struct VideoPlayerState {
    current_video: Option<Video>,
    loading: bool,
    load_failed: bool,
    controls: VideoControlsState,
    /// Start playback as soon as a video is opened; otherwise it opens paused.
    autoplay: bool,
    /// Whether playback is paused, as last requested. Kept here because
    /// `Video::paused` reads the pipeline's current state, which a flushing seek drops
    /// to `Paused` until preroll completes: sampled then, it switched the playback tick
    /// off for good and froze the progress bar and the subtitle highlight.
    paused: bool,
    /// Path of the video being loaded or shown; lets a late subtitle load for a
    /// previous file be recognized and dropped.
    current_path: Option<PathBuf>,
    /// Subtitles found next to the current video, if any.
    subtitles: Option<Arc<Subtitles>>,
    /// The list shown over the picture; see [`Overlay`].
    overlay: Overlay,
    /// Note shown in the controls bar and its number, so an older timer does not hide a newer
    /// note.
    notice: Option<(String, u64)>,
    notice_count: u64,
    /// Playback position the whole view agrees on: progress bar, caption and list.
    ///
    /// Stored rather than queried at view time: the pipeline query fails while a seek is
    /// in flight (the player reports that as 0) and lags behind a seek made while paused,
    /// so querying per view made the caption, list and bar each show a different moment.
    /// Refreshed on every playback tick; set to the target the moment a seek is made.
    position: Duration,
    /// Cue the list last scrolled to; the list follows only when this changes, so a
    /// user scrolling it by hand is not fought on every tick.
    followed_cue: Option<usize>,
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
            paused: !autoplay,
            current_path: None,
            subtitles: None,
            overlay: Overlay::Closed,
            notice: None,
            notice_count: 0,
            position: Duration::ZERO,
            followed_cue: None,
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
        self.current_path = Some(path.clone());
        self.subtitles = None;
        self.position = Duration::ZERO;
        self.followed_cue = None;
        let autoplay = self.autoplay;
        self.paused = !autoplay;

        let subtitles_task = Self::load_subtitles(path.clone());
        let video_task = Task::future(async move {
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
        });
        Task::batch([video_task, subtitles_task])
    }

    /// Read the `.srt` next to the video on a blocking thread.
    fn load_subtitles(video_path: PathBuf) -> Task<Message> {
        Task::future(async move {
            let path = video_path.clone();
            let subtitles = tokio::task::spawn_blocking(move || load_subtitles(&path))
                .await
                .unwrap_or(None)
                .map(Arc::new);
            Message::SubtitlesLoaded {
                video_path,
                subtitles,
            }
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
                let ready = Task::done(Message::Controls(video_controls::Message::VideoReady {
                    duration_secs,
                }));
                // Controls assume playback on ready; a video opened paused has to say otherwise.
                if !self.paused {
                    return ready;
                }
                ready.chain(Task::done(Message::Controls(
                    video_controls::Message::SetPlaying(false),
                )))
            }
            Message::NewFrame => {
                if let Some(position) = self.current_video.as_ref().and_then(query_position) {
                    self.position = position;
                }
                self.follow_cue(false)
            }
            Message::EndOfStream => {
                // The pipeline stays in Playing at the end; the next play restarts the stream.
                self.paused = true;
                Task::done(Message::Controls(video_controls::Message::SetPlaying(
                    false,
                )))
            }
            Message::TogglePause => {
                // Pausing stops the ticks, so take the exact position playback stopped at.
                if let Some(position) = self.current_video.as_ref().and_then(query_position) {
                    self.position = position;
                }
                if let Some(video) = &mut self.current_video {
                    self.paused = !self.paused;
                    video.set_paused(self.paused);
                }
                Task::none()
            }
            Message::Seek(position_secs) => self.seek_to(
                Duration::from_secs_f64(position_secs.max(0.0) as f64),
                false,
            ),
            Message::Controls(ctrl_msg) => {
                self.controls.update(&ctrl_msg);
                match ctrl_msg {
                    video_controls::Message::TogglePlayPause => Task::done(Message::TogglePause),
                    video_controls::Message::Seek(pos) => Task::done(Message::Seek(pos)),
                    video_controls::Message::SeekBack10 => {
                        if self.current_video.is_none() {
                            return Task::none();
                        }
                        Task::done(Message::Seek((self.position.as_secs_f32() - 10.0).max(0.0)))
                    }
                    video_controls::Message::SeekForward10 => {
                        let Some(video) = self.current_video.as_ref() else {
                            return Task::none();
                        };
                        let dur = video.duration().as_secs_f32();
                        Task::done(Message::Seek((self.position.as_secs_f32() + 10.0).min(dur)))
                    }
                    video_controls::Message::SetSegmentStart => self.capture_segment_start(),
                    video_controls::Message::SetSegmentEnd => self.capture_segment_end(),
                    video_controls::Message::TakeScreenshot => self.capture_screenshot(),
                    video_controls::Message::AddMarker => {
                        Task::done(Message::Markers(markers::Message::Add))
                    }
                    video_controls::Message::DeleteMarker => {
                        Task::done(Message::Markers(markers::Message::DeleteAtPlayhead))
                    }
                    video_controls::Message::PreviousMarker => {
                        Task::done(Message::Markers(markers::Message::Previous))
                    }
                    video_controls::Message::NextMarker => {
                        Task::done(Message::Markers(markers::Message::Next))
                    }
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
            Message::SubtitlesLoaded {
                video_path,
                subtitles,
            } => {
                if self.current_path.as_ref() == Some(&video_path) {
                    self.subtitles = subtitles;
                }
                Task::none()
            }
            Message::ToggleCueList => {
                self.overlay = if self.overlay == Overlay::Subtitles {
                    Overlay::Closed
                } else {
                    Overlay::Subtitles
                };
                // The list is built afresh at the top; bring the current cue into view.
                self.follow_cue(true)
            }
            Message::ToggleMarkerList => {
                self.overlay = if self.overlay == Overlay::Markers {
                    Overlay::Closed
                } else {
                    Overlay::Markers
                };
                Task::none()
            }
            Message::ShowMarkerList => {
                self.overlay = Overlay::Markers;
                Task::none()
            }
            Message::ShowOverlay(overlay) => {
                self.overlay = overlay;
                self.follow_cue(true)
            }
            // Bubbles up via media_viewer, which adds the playhead.
            Message::Markers(_) => Task::none(),
            Message::SeekExact(ms) => self.seek_to(Duration::from_millis(ms), true),
            Message::ShowNotice(text) => {
                self.notice_count += 1;
                let number = self.notice_count;
                self.notice = Some((text, number));
                Task::future(async move {
                    tokio::time::sleep(NOTICE_DURATION).await;
                    Message::ClearNotice(number)
                })
            }
            Message::ClearNotice(number) => {
                if self.notice.as_ref().is_some_and(|(_, n)| *n == number) {
                    self.notice = None;
                }
                Task::none()
            }
            Message::SeekToCue(index) => {
                let Some(cue) = self.subtitles.as_ref().and_then(|s| s.cues().get(index)) else {
                    return Task::none();
                };
                // Exact, not keyframe: a keyframe before the cue would land playback on
                // the previous cue, and the highlight would jump back to it.
                let start = cue.start;
                self.seek_to(start, true)
            }
            Message::Unload => {
                self.current_video = None;
                self.loading = false;
                self.current_path = None;
                self.subtitles = None;
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

    pub fn is_loading(&self) -> bool {
        self.loading
    }
    pub fn load_failed(&self) -> bool {
        self.load_failed
    }
    pub fn current_video(&self) -> Option<&Video> {
        self.current_video.as_ref()
    }
    pub fn controls(&self) -> &VideoControlsState {
        &self.controls
    }
    pub fn subtitles(&self) -> Option<&Subtitles> {
        self.subtitles.as_deref()
    }
    /// Whether the subtitle list is open (windowed mode; fullscreen shows it anyway).
    pub fn show_cue_list(&self) -> bool {
        self.overlay == Overlay::Subtitles
    }

    /// Whether the marker list is open.
    pub fn show_marker_list(&self) -> bool {
        self.overlay == Overlay::Markers
    }

    /// The note to show in the controls bar, if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_ref().map(|(text, _)| text.as_str())
    }

    /// The playhead in milliseconds, as the view shows it.
    pub fn position_ms(&self) -> u64 {
        u64::try_from(self.display_position().as_millis()).unwrap_or(u64::MAX)
    }

    /// Position to draw: the drag position while the progress bar is held, otherwise the
    /// stored playback position. Everything position-dependent in the view reads this.
    pub fn display_position(&self) -> Duration {
        if self.controls.is_seeking() {
            Duration::from_secs_f32(self.controls.seek_position_secs().max(0.0))
        } else {
            self.position
        }
    }

    /// Seek and adopt the target as the position right away, so the view shows where
    /// playback is going rather than wherever the pipeline is mid-seek.
    fn seek_to(&mut self, target: Duration, accurate: bool) -> Task<Message> {
        let Some(video) = self.current_video.as_mut() else {
            return Task::none();
        };
        if let Err(e) = video.seek(target, accurate) {
            log::error!("Failed to seek: {e}");
            return Task::none();
        }
        self.position = target.min(video.duration());
        self.follow_cue(false)
    }

    /// Scroll the subtitle list so the current cue sits near its top. Only when that cue
    /// changed since the last follow, unless `force`.
    fn follow_cue(&mut self, force: bool) -> Task<Message> {
        let Some(subtitles) = self.subtitles.as_ref() else {
            return Task::none();
        };
        let cue = subtitles.last_started_index(self.display_position());
        if cue == self.followed_cue && !force {
            return Task::none();
        }
        self.followed_cue = cue;
        // One cue of context above the current one.
        let rows_above = cue.unwrap_or(0).saturating_sub(1);
        let offset = iced::widget::scrollable::AbsoluteOffset {
            x: None,
            y: Some(rows_above as f32 * CUE_ROW_PITCH),
        };
        iced::widget::operation::scroll_to::<()>(
            iced::widget::Id::new(CUE_LIST_SCROLLABLE_ID),
            offset,
        )
        .discard()
    }

    fn capture_segment_start(&self) -> Task<Message> {
        let Some(video) = self.current_video.as_ref() else {
            return Task::none();
        };
        let secs = video.position().as_secs_f32().floor();
        Task::done(Message::SegmentStartMarked(secs))
    }

    fn capture_segment_end(&self) -> Task<Message> {
        let Some(video) = self.current_video.as_ref() else {
            return Task::none();
        };
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
        log::info!(
            "capture_screenshot: jpeg={:?}",
            jpeg.as_ref().map(|v| v.len())
        );
        let jpeg = jpeg.unwrap_or_default();
        Task::done(Message::ScreenshotTaken(position_ms, jpeg))
    }

    /// True when a video is loaded or in the process of loading.
    pub fn is_active(&self) -> bool {
        self.current_video.is_some() || self.loading
    }

    /// Subscriptions active while a video is loaded.
    pub fn subscription(&self) -> Subscription<Message> {
        if self.current_video.is_none() {
            return Subscription::none();
        }
        // The tick exists only to advance the progress bar, so it is pointless while paused:
        // it used to force a full view rebuild 4x/second for as long as a video stayed open.
        // Iced re-evaluates subscriptions after every update, so pausing stops it immediately.
        let frame_tick = if self.paused {
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

/// Where the pipeline is now; `None` when it cannot say (mid-seek, state change).
/// `Video::position` reports that as 0, which would flash the view back to the start.
fn query_position(video: &Video) -> Option<Duration> {
    video
        .pipeline()
        .query_position::<gst::ClockTime>()
        .map(|t| Duration::from_nanos(t.nseconds()))
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
    open_video_with(uri, None).map_err(|failure| {
        for problem in &failure.problems {
            log::warn!("GStreamer: {problem}");
        }
        failure.error
    })
}

/// Why a video could not be opened: the player's error plus the errors and warnings GStreamer
/// posted on the way. A missing decoder, for one, is only a warning followed by the end of the
/// stream, so the error alone does not name it.
pub(crate) struct OpenFailure {
    pub error: VideoError,
    pub problems: Vec<String>,
}

impl std::fmt::Display for OpenFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)?;
        for problem in &self.problems {
            write!(f, "; {problem}")?;
        }
        Ok(())
    }
}

impl From<VideoError> for OpenFailure {
    fn from(error: VideoError) -> Self {
        Self {
            error,
            problems: Vec::new(),
        }
    }
}

/// [`open_video`], with the sound sent to `audio_sink` (a `playbin` sink description) instead
/// of the default device when one is given.
fn open_video_with(uri: &url::Url, audio_sink: Option<&str>) -> Result<Video, OpenFailure> {
    gst::init().map_err(VideoError::from)?;

    match open_pipeline(uri, false, audio_sink) {
        Err(OpenFailure {
            error: VideoError::Framerate(rate),
            ..
        }) => {
            log::info!(
                "Source reports framerate {rate} (variable); retrying as {VFR_NOMINAL_FRAMERATE}"
            );
            open_pipeline(uri, true, audio_sink)
        }
        other => other,
    }
}

/// Open `uri` exactly as the player does, with the sound going nowhere, and wait until a
/// decoded frame has reached the player's video sink. Used by `--self-test`.
pub(crate) fn check_decodes(uri: &url::Url, timeout: Duration) -> Result<(), String> {
    let video = open_video_with(uri, Some("fakesink")).map_err(|failure| failure.to_string())?;
    let sink = player_video_sink(&video.pipeline())
        .ok_or_else(|| VideoError::AppSink("iced_video".to_string()).to_string())?;
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let sample: Option<gst::Sample> = sink.property("last-sample");
        if sample.is_some() {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!("no frame reached the player within {timeout:?}"));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Build a `playbin`, bring it to `Playing` and hand it to the player.
///
/// The preroll happens here rather than inside `Video::from_gst_pipeline`, which allows
/// only 5 seconds — too little for a cold file on cloud-backed storage (Dropbox or
/// OneDrive "online-only"), where the first read waits on a full download. Reaching
/// `Playing` up front means the wait inside the player returns immediately.
fn open_pipeline(
    uri: &url::Url,
    relabel_framerate: bool,
    audio_sink: Option<&str>,
) -> Result<Video, OpenFailure> {
    let pipeline = gst::parse::launch(&description(uri, relabel_framerate, audio_sink))
        .map_err(VideoError::from)?
        .downcast::<gst::Pipeline>()
        .map_err(|_| VideoError::Cast)?;

    // Every failure past this point has to stop the pipeline, or playbin keeps the
    // audio device open and the sound of an abandoned load carries on in the background.
    // What GStreamer said is read first: stopping the pipeline flushes its bus.
    let fail = |error: VideoError| {
        let problems = bus_problems(&pipeline);
        let _ = pipeline.set_state(gst::State::Null);
        OpenFailure { error, problems }
    };

    preroll(&pipeline).map_err(fail)?;
    let (video_sink, text_sink) = sinks(&pipeline).map_err(fail)?;
    // A stream that ended without a video frame (no decoder for it, or no video at all)
    // prerolls fine but leaves the sink without caps; the player would refuse it with the
    // same error after tearing the pipeline down, and the reason with it.
    if video_sink
        .static_pad("sink")
        .and_then(|pad| pad.current_caps())
        .is_none()
    {
        return Err(fail(VideoError::Caps));
    }

    Ok(Video::from_gst_pipeline(
        pipeline,
        video_sink,
        Some(text_sink),
    )?)
}

/// The errors and warnings waiting on the pipeline's bus, as text. Reads them off the bus, so
/// only for a pipeline that is being given up.
fn bus_problems(pipeline: &gst::Pipeline) -> Vec<String> {
    let Some(bus) = pipeline.bus() else {
        return Vec::new();
    };
    std::iter::from_fn(|| bus.pop())
        .filter_map(|message| match message.view() {
            gst::MessageView::Error(e) => Some(e.error().to_string()),
            gst::MessageView::Warning(w) => Some(w.error().to_string()),
            _ => None,
        })
        .collect()
}

/// The `playbin` description, optionally rewriting the framerate on the way to the sink and
/// sending the sound to `audio_sink` instead of the default device.
fn description(uri: &url::Url, relabel_framerate: bool, audio_sink: Option<&str>) -> String {
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

    let audio_sink = audio_sink
        .map(|sink| format!(" audio-sink=\"{sink}\""))
        .unwrap_or_default();
    format!(
        "playbin uri=\"{}\" text-sink=\"appsink name=iced_text sync=true drop=true\" \
         video-sink=\"{video_sink}\"{audio_sink}",
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

/// The `iced_video` appsink of a running player pipeline, found inside playbin's video-sink bin.
fn player_video_sink(pipeline: &gst::Pipeline) -> Option<gst::Element> {
    let video_sink: gst::Element = pipeline.property("video-sink");
    if let Ok(bin) = video_sink.clone().downcast::<gst::Bin>() {
        return bin.by_name("iced_video");
    }
    // video-sink wraps its sink pad in a GhostPad; parent of that pad is the bin.
    video_sink
        .pads()
        .into_iter()
        .find_map(|p| p.dynamic_cast::<gst::GhostPad>().ok())
        .and_then(|gp| gp.parent_element())
        .and_then(|e| e.downcast::<gst::Bin>().ok())
        .and_then(|b| b.by_name("iced_video"))
}

/// Capture the current video frame as JPEG bytes.
///
/// Reads the `last-sample` property of the iced_video AppSink.
/// This holds the most recently delivered NV12 frame and does not
/// compete with the worker thread's continuous pull loop.
fn capture_jpeg(video: &Video) -> Option<Vec<u8>> {
    let appsink_el = player_video_sink(&video.pipeline())?;

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
            let g = (1.164 * (y_val - 16.0) - 0.813 * (v - 128.0) - 0.391 * (u - 128.0))
                .clamp(0.0, 255.0) as u8;
            let b = (1.164 * (y_val - 16.0) + 2.018 * (u - 128.0)).clamp(0.0, 255.0) as u8;
            rgb.extend_from_slice(&[r, g, b]);
        }
    }
    rgb
}
