//! State for the video player sub-feature.

use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video::VideoMeta;
use iced::{time, Subscription, Task};
use iced_video_player::{Error as VideoError, Video};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::frame_step::{self, ShownFrame, StepTarget};
use super::view::{cue_offset, CUE_LIST_SCROLLABLE_ID};
use super::Message;
use crate::features::markers;
use crate::features::video_controls::{self, VideoControlsState};
use crate::ui::scroll::FollowedRow;
use crate::ui::tokens::{SPACE_XXS, SPINNER_TICK};
use frename_core::{
    load_subtitles, AppDatabase, AppStateStore, FileTagger, Rotation, RotationError, Subtitles,
};

/// How long a note over the picture stays.
const NOTICE_DURATION: Duration = Duration::from_secs(2);
/// How long a note that must be read stays: the report of a crash and what was restored.
const LONG_NOTICE_DURATION: Duration = Duration::from_secs(20);
/// How often a frame step looks whether its frame is on screen.
const FRAME_STEP_TICK: Duration = Duration::from_millis(15);
/// How long a frame step waits for a new frame before taking what is on screen: at the first
/// frame (a clip whose first timestamp is not zero) or past the last one, none comes.
const FRAME_STEP_TIMEOUT: Duration = Duration::from_millis(500);

/// What the list over the right of the picture shows. One list at a time, so a windowed
/// video is not covered twice. Kept across files, like volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overlay {
    /// No list.
    #[default]
    Closed,
    Subtitles,
    Markers,
}

/// A range playing from its start to its end.
#[derive(Debug, Clone, Copy)]
struct RangePlay {
    start: Duration,
    end: Duration,
    /// Playback was seen inside the range. Until then a tick may still report the position
    /// before the seek to its start landed, which can be past its end: pausing on that, mid
    /// seek, was lost by the pipeline, and the video played on with the bar stopped.
    entered: bool,
}

impl RangePlay {
    /// Whether playback at `position` has played the range to its end.
    fn done_at(&mut self, position: Duration) -> bool {
        if (self.start..self.end).contains(&position) {
            self.entered = true;
        }
        self.entered && position >= self.end
    }
}

/// A frame step on its way: the frame it started from, which way, and when.
#[derive(Debug, Clone, Copy)]
struct PendingStep {
    from: ShownFrame,
    step: video_controls::FrameStep,
    since: Instant,
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
    /// Note shown over the picture and its number, so an older timer does not hide a newer
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
    followed_cue: FollowedRow,
    /// Where the subtitle list is scrolled to; put back when fullscreen rebuilds the list.
    cue_scroll_y: f32,
    /// Set when the list was just put back at this offset; see `CueListScrolled`.
    cue_restore: crate::ui::scroll::ScrollRestore,
    /// A clicked range is playing: pause when playback reaches its end. Any seek or pause by
    /// the user forgets it.
    play_until: Option<RangePlay>,
    /// Where to seek once the video being loaded is open: it is the shown video, reopened.
    resume_at: Option<Duration>,
    /// The rotation the video was opened with, or why it has none; see [`Self::rotation`].
    rotation: Option<Result<Rotation, RotationError>>,
    /// Number of the latest load; a `VideoLoaded` of an older one is dropped.
    load_generation: u64,
    /// Loads started (on a blocking thread, opening the file) whose `VideoLoaded` has not
    /// landed yet. While GStreamer is inside that open, it holds the file open on the OS
    /// level; `Unload` must not answer `VideoUnloaded` until this reaches zero, or a save
    /// right after can hit a sharing violation on Windows (issue #91).
    loads_in_flight: u64,
    /// `Unload` arrived while a load was still in flight: `VideoUnloaded` is held back until
    /// every such load has landed (and, being stale, been dropped) — see `loads_in_flight`.
    unloading: bool,
    /// **More** is open over the picture.
    more_open: bool,
    /// Steps of the spinner clock since the load started: turns the spinner, and after a while
    /// says the file is slow to come.
    loading_ticks: usize,
    /// A frame step whose frame is not on screen yet. Further steps wait for it: a held key
    /// would otherwise step again from the frame still shown and stand still.
    frame_step: Option<PendingStep>,
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
            followed_cue: FollowedRow::default(),
            cue_scroll_y: 0.0,
            cue_restore: Default::default(),
            play_until: None,
            resume_at: None,
            rotation: None,
            load_generation: 0,
            loads_in_flight: 0,
            unloading: false,
            more_open: false,
            loading_ticks: 0,
            frame_step: None,
        }
    }
}

impl VideoPlayerState {
    /// Load a video file asynchronously.
    pub fn load_video(&mut self, path: PathBuf) -> Task<Message> {
        self.resume_at = None;
        self.open(path, !self.autoplay)
    }

    /// Open the shown video again at the same moment and in the same play state, e.g. after
    /// its rotation changed: the player takes the picture size only when a video opens.
    pub fn reload_video(&mut self) -> Task<Message> {
        let Some(path) = self.current_path.clone() else {
            return Task::none();
        };
        // Reopened again before the last reopen landed: its moment is still the one to go back
        // to (the position was reset when it started loading).
        let position = self.resume_at.unwrap_or(self.position);
        let task = self.open(path, self.paused);
        self.resume_at = Some(position);
        task
    }

    /// The rotation the shown video was opened with, or why frename cannot read or change it;
    /// `None` while it loads.
    pub fn rotation(&self) -> Option<&Result<Rotation, RotationError>> {
        self.rotation.as_ref()
    }

    /// Scroll the subtitle list to `y`.
    fn scroll_cue_list_to(y: f32) -> Task<Message> {
        iced::widget::operation::scroll_to::<()>(
            iced::widget::Id::new(CUE_LIST_SCROLLABLE_ID),
            iced::widget::scrollable::AbsoluteOffset {
                x: None,
                y: Some(y),
            },
        )
        .discard()
    }

    /// How many loads have started (see `load_generation`), for tests of reopens.
    #[cfg(test)]
    pub fn loads_started(&self) -> u64 {
        self.load_generation
    }

    fn open(&mut self, path: PathBuf, paused: bool) -> Task<Message> {
        log::info!("Starting video load: {}", path.display());
        // Unknown until the load reads it: a reopen after a turn or an undo may see another.
        self.rotation = None;
        self.loading = true;
        self.loading_ticks = 0;
        self.load_failed = false;
        self.current_video = None;
        self.controls = VideoControlsState::with_volume(self.controls.volume());
        self.current_path = Some(path.clone());
        self.subtitles = None;
        self.position = Duration::ZERO;
        self.followed_cue = FollowedRow::default();
        self.cue_scroll_y = 0.0;
        self.cue_restore = Default::default();
        self.play_until = None;
        self.frame_step = None;
        self.paused = paused;
        self.load_generation = self.load_generation.wrapping_add(1);
        let generation = self.load_generation;
        self.loads_in_flight += 1;

        let subtitles_task = Self::load_subtitles(path.clone());
        let video_task = Task::future(async move {
            // The video is opened once, here on a blocking thread, and handed to the
            // update below. Opening it a second time on the update thread would stall
            // the UI for as long as the pipeline takes to preroll.
            let opened = tokio::task::spawn_blocking(move || {
                let Ok(url) = url::Url::from_file_path(&path) else {
                    log::warn!("Failed to create URL from path: {}", path.display());
                    return (None, None);
                };
                log::debug!("File URL created: {url}");
                // Read by frename rather than left to GStreamer's tag, so a debug build shows
                // a turn it keeps in memory.
                let rotation = FileTagger::video_rotation(&path);
                match open_video(&url, rotation.as_ref().ok().copied()) {
                    Ok(mut video) => {
                        log::info!("Video loaded successfully");
                        // Paused here, before the update thread sees it, so no audio slips out.
                        if paused {
                            video.set_paused(true);
                        }
                        (Some(video), Some(rotation))
                    }
                    Err(e) => {
                        log::error!("Failed to load video: {e}");
                        (None, Some(rotation))
                    }
                }
            })
            .await
            .unwrap_or((None, None));

            Message::VideoLoaded {
                video: Arc::new(Mutex::new(opened.0)),
                rotation: opened.1,
                generation,
            }
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
            Message::VideoLoaded {
                video: slot,
                rotation,
                generation,
            } => {
                // This load has landed: whatever file handle GStreamer held open for it is
                // dropped by the end of this arm (with the video, if any). If `Unload` came in
                // while this was still in flight, only now is it safe to say so (issue #91).
                self.loads_in_flight = self.loads_in_flight.saturating_sub(1);
                let unloaded = if self.unloading && self.loads_in_flight == 0 {
                    self.unloading = false;
                    // The wait `Unload` set up is over: `loading` (kept true so the view showed
                    // its spinner rather than "nothing loaded" for the wait) is now false, same
                    // as `Unload` itself would have set it had no load been in flight.
                    self.loading = false;
                    Task::done(Message::VideoUnloaded)
                } else {
                    Task::none()
                };
                // A load started before the latest one (the video was reopened meanwhile): its
                // video, dropped here, may show an older rotation.
                if generation != self.load_generation {
                    log::debug!("Dropping a video load that a newer one replaced");
                    return unloaded;
                }
                self.loading = false;
                self.rotation = rotation;
                let resume_at = self.resume_at.take();
                // A poisoned lock is as unusable as a failed open, so both land in the
                // error state rather than the neutral "nothing loaded" placeholder.
                let video = slot.lock().ok().and_then(|mut slot| slot.take());
                let Some(video) = video else {
                    self.load_failed = true;
                    log::info!("Video load failed; showing error state");
                    return unloaded;
                };
                self.load_failed = false;
                let duration_secs = video.duration().as_secs_f32();
                self.current_video = Some(video);
                let ready = Task::done(Message::VideoReady { duration_secs });
                let loaded = match resume_at {
                    // Reopened: back to the moment it showed.
                    Some(position) => Task::batch([ready, self.seek_to(position, true)]),
                    None => ready,
                };
                Task::batch([unloaded, loaded])
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
                let follow = self.follow_cue(false);
                let position = self.position;
                if !self
                    .play_until
                    .as_mut()
                    .is_some_and(|range| range.done_at(position))
                {
                    return follow;
                }
                // The clicked range has played: stop at its end.
                self.play_until = None;
                self.paused = true;
                if let Some(video) = &mut self.current_video {
                    video.set_paused(true);
                }
                Task::batch([
                    follow,
                    Task::done(Message::Controls(video_controls::Message::SetPlaying(
                        false,
                    ))),
                ])
            }
            Message::EndOfStream => {
                // The pipeline stays in Playing at the end; the next play restarts the stream.
                self.play_until = None;
                self.paused = true;
                Task::done(Message::Controls(video_controls::Message::SetPlaying(
                    false,
                )))
            }
            Message::TogglePause => {
                self.play_until = None;
                self.frame_step = None;
                // A step back left the pipeline running backward: turn it forward at the frame
                // shown before playing.
                if self.paused
                    && self
                        .current_video
                        .as_ref()
                        .and_then(shown_frame)
                        .is_some_and(|frame| frame.reversed)
                {
                    self.seek_only(self.position, true);
                } else if let Some(position) = self.current_video.as_ref().and_then(query_position)
                {
                    // Pausing stops the ticks, so take the exact position playback stopped at.
                    self.position = position;
                }
                if let Some(video) = &mut self.current_video {
                    self.paused = !self.paused;
                    video.set_paused(self.paused);
                }
                Task::none()
            }
            Message::Seek(position_secs) => {
                self.play_until = None;
                self.seek_to(
                    Duration::from_secs_f64(position_secs.max(0.0) as f64),
                    false,
                )
            }
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
                    video_controls::Message::StepFrame(step) => self.step_frame(step),
                    video_controls::Message::SetSegmentStart => self.capture_segment_start(),
                    video_controls::Message::SetSegmentEnd => self.capture_segment_end(),
                    video_controls::Message::TakeScreenshot => self.capture_screenshot(),
                    video_controls::Message::MarkerKeyPressed => {
                        Task::done(Message::Markers(markers::Message::KeyDown))
                    }
                    video_controls::Message::MarkerKeyReleased => {
                        Task::done(Message::Markers(markers::Message::KeyUp))
                    }
                    video_controls::Message::SetMarkerSpan(guid, start, end) => {
                        Task::done(Message::Markers(markers::Message::SetSpan(
                            guid,
                            secs_to_ms(start),
                            secs_to_ms(end),
                        )))
                    }
                    video_controls::Message::AddRange(start, end) => Task::done(Message::Markers(
                        markers::Message::AddRange(secs_to_ms(start), secs_to_ms(end)),
                    )),
                    video_controls::Message::PlayRange(start, end) => self.play_range(
                        Duration::from_millis(secs_to_ms(start)),
                        Duration::from_millis(secs_to_ms(end)),
                    ),
                    video_controls::Message::DeleteMarker => {
                        Task::done(Message::Markers(markers::Message::DeleteAtPlayhead))
                    }
                    video_controls::Message::PreviousMarker => {
                        Task::done(Message::Markers(markers::Message::Previous))
                    }
                    video_controls::Message::NextMarker => {
                        Task::done(Message::Markers(markers::Message::Next))
                    }
                    video_controls::Message::EditMarker(guid) => {
                        Task::done(Message::Markers(markers::Message::Open(guid)))
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
            Message::SeekExact(ms) => {
                self.play_until = None;
                self.seek_to(Duration::from_millis(ms), true)
            }
            Message::ShowNotice(text) => self.show_notice(text, NOTICE_DURATION),
            Message::ShowLongNotice(text) => self.show_notice(text, LONG_NOTICE_DURATION),
            Message::ClearNotice(number) => {
                if self.notice.as_ref().is_some_and(|(_, n)| *n == number) {
                    self.notice = None;
                }
                Task::none()
            }
            Message::CueListScrolled(y, viewport) => {
                self.cue_scroll_y = y;
                if !self.cue_restore.report(y) {
                    return Task::none();
                }
                // The list was just put back here: if the lit cue is out of the new viewport,
                // bring it in, as the follow does.
                let Some(subtitles) = self.subtitles.as_ref() else {
                    return Task::none();
                };
                let Some(cue) = subtitles.last_started_index(self.display_position()) else {
                    return Task::none();
                };
                let wanted = crate::ui::scroll::keep_row_in_view(
                    y,
                    viewport,
                    cue_offset(subtitles, cue),
                    cue_offset(subtitles, cue + 1) - SPACE_XXS,
                    cue_offset(subtitles, cue.saturating_sub(1)),
                );
                if wanted == y {
                    return Task::none();
                }
                Self::scroll_cue_list_to(wanted)
            }
            Message::RestoreCueScroll(y) => {
                // Only a list on screen reports back; armed otherwise it would fire much later.
                if self.show_cue_list() && self.subtitles.is_some() {
                    self.cue_restore.arm(y);
                }
                Self::scroll_cue_list_to(y)
            }
            Message::SeekToCue(index) => {
                let Some(cue) = self.subtitles.as_ref().and_then(|s| s.cues().get(index)) else {
                    return Task::none();
                };
                // Exact, not keyframe: a keyframe before the cue would land playback on
                // the previous cue, and the highlight would jump back to it.
                let start = cue.start;
                // A click never scrolls the list (§13.2): the row is under the pointer.
                let lit = self
                    .subtitles
                    .as_ref()
                    .and_then(|s| s.last_started_index(start));
                if self.seek_only(start, true) {
                    self.followed_cue.clicked(lit);
                }
                Task::none()
            }
            Message::Unload => {
                // A load still under way belongs to the video being closed: drop it when it
                // lands (`VideoLoaded` above). Until then GStreamer's open on the blocking
                // thread may still hold the file open, so `VideoUnloaded` — the caller's signal
                // that it is safe to rename or resave the file — has to wait for it (#91).
                let load_in_flight = self.loads_in_flight > 0;
                self.load_generation = self.load_generation.wrapping_add(1);
                self.resume_at = None;
                self.frame_step = None;
                self.current_video = None;
                self.current_path = None;
                self.subtitles = None;
                if load_in_flight {
                    // `loading` stays true for this same wait: the view keeps showing its
                    // spinner instead of falling back to "nothing loaded", and `is_active()`
                    // correctly still reports the player as busy in the meantime.
                    log::debug!("Deferring VideoUnloaded: a load is still in flight");
                    self.unloading = true;
                    Task::none()
                } else {
                    self.loading = false;
                    Task::done(Message::VideoUnloaded)
                }
            }
            Message::VideoUnloaded => Task::none(),
            // Intercepted by media_viewer/folder_workspace; no-op here.
            Message::ToggleFullscreen => Task::none(),
            Message::SetAutoplay(autoplay) => {
                self.autoplay = autoplay;
                Task::none()
            }
            Message::ToggleMore => {
                self.more_open = !self.more_open;
                Task::none()
            }
            Message::CloseMore => {
                self.more_open = false;
                Task::none()
            }
            Message::MorePicked(messages) => {
                self.more_open = false;
                messages
                    .into_iter()
                    .map(Task::done)
                    .reduce(Task::chain)
                    .unwrap_or_else(Task::none)
            }
            Message::LoadingTick => {
                self.loading_ticks = self.loading_ticks.wrapping_add(1);
                Task::none()
            }
            Message::FrameStepTick => self.frame_step_tick(),
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
    /// Where the subtitle list is scrolled to.
    pub fn cue_scroll_y(&self) -> f32 {
        self.cue_scroll_y
    }

    /// Whether the subtitle list is open.
    pub fn show_cue_list(&self) -> bool {
        self.overlay == Overlay::Subtitles
    }

    /// Whether the marker list is open.
    pub fn show_marker_list(&self) -> bool {
        self.overlay == Overlay::Markers
    }

    /// Whether **More** is open.
    pub fn more_open(&self) -> bool {
        self.more_open
    }

    /// The file being loaded, by name, while it loads.
    pub fn loading_name(&self) -> Option<String> {
        self.loading
            .then(|| self.current_path.as_ref()?.file_name())
            .flatten()
            .map(|name| name.to_string_lossy().into_owned())
    }

    /// Steps of the spinner clock since the load started.
    pub fn loading_ticks(&self) -> usize {
        self.loading_ticks
    }

    /// Show `text` over the picture for `duration`; a newer note replaces it.
    fn show_notice(&mut self, text: String, duration: Duration) -> Task<Message> {
        self.notice_count += 1;
        let number = self.notice_count;
        self.notice = Some((text, number));
        Task::future(async move {
            tokio::time::sleep(duration).await;
            Message::ClearNotice(number)
        })
    }

    /// The note to show over the picture, if any.
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

    /// Play from `start` to `end` and pause there (a range's band was clicked).
    fn play_range(&mut self, start: Duration, end: Duration) -> Task<Message> {
        if self.current_video.is_none() {
            return Task::none();
        }
        let seek = self.seek_to(start, true);
        self.play_until = Some(RangePlay {
            start,
            end,
            entered: false,
        });
        if !self.paused {
            return seek;
        }
        self.paused = false;
        if let Some(video) = &mut self.current_video {
            video.set_paused(false);
        }
        Task::batch([
            seek,
            Task::done(Message::Controls(video_controls::Message::SetPlaying(true))),
        ])
    }

    /// One frame back or forward. Pauses playback first; the playhead becomes the shown frame's
    /// time once it is on screen (see [`Self::frame_step_tick`]).
    fn step_frame(&mut self, step: video_controls::FrameStep) -> Task<Message> {
        let Some(video) = self.current_video.as_mut() else {
            return Task::none();
        };
        self.play_until = None;
        let paused = if self.paused {
            Task::none()
        } else {
            self.paused = true;
            video.set_paused(true);
            Task::done(Message::Controls(video_controls::Message::SetPlaying(
                false,
            )))
        };
        if self.frame_step.is_some() {
            return paused;
        }
        let Some(frame) = shown_frame(video) else {
            return paused;
        };
        let moved = match frame_step::step_target(frame, step, video.duration()) {
            StepTarget::Stay => return paused,
            StepTarget::NextBuffer => {
                video.step_one_frame();
                Task::none()
            }
            StepTarget::Backward(stop) => {
                seek_backward(video, stop);
                Task::none()
            }
            StepTarget::Forward(target) => self.seek_to(target, true),
        };
        self.frame_step = Some(PendingStep {
            from: frame,
            step,
            since: Instant::now(),
        });
        Task::batch([paused, moved])
    }

    /// While a frame step is on its way: once a new frame is on screen (or the wait is over),
    /// the playhead becomes its time.
    fn frame_step_tick(&mut self) -> Task<Message> {
        let (Some(pending), Some(video)) = (self.frame_step, self.current_video.as_ref()) else {
            self.frame_step = None;
            return Task::none();
        };
        let shown = shown_frame(video).filter(|frame| *frame != pending.from);
        let timed_out = pending.since.elapsed() >= FRAME_STEP_TIMEOUT;
        if shown.is_none() && !timed_out {
            return Task::none();
        }
        self.frame_step = None;
        let Some(frame) = shown else {
            // Nothing new came: the step ran past the first or last frame. Show the frame it
            // started from again, played forward.
            self.seek_only(pending.from.start, true);
            self.position = frame_step::frame_position(pending.from);
            return self.follow_cue(false);
        };
        self.position = frame_step::frame_position(frame);
        let follow = self.follow_cue(false);
        if frame_step::only_found_the_start(pending.from, frame) {
            // The first half of a step back: now the frame before can be found.
            return Task::batch([follow, self.step_frame(pending.step)]);
        }
        follow
    }

    /// Seek and adopt the target as the position right away, so the view shows where
    /// playback is going rather than wherever the pipeline is mid-seek.
    fn seek_to(&mut self, target: Duration, accurate: bool) -> Task<Message> {
        if !self.seek_only(target, accurate) {
            return Task::none();
        }
        self.followed_cue.unpin();
        self.follow_cue(false)
    }

    /// [`Self::seek_to`] without following the cue; false when it did not seek.
    fn seek_only(&mut self, target: Duration, accurate: bool) -> bool {
        let Some(video) = self.current_video.as_mut() else {
            return false;
        };
        if let Err(e) = video.seek(target, accurate) {
            log::error!("Failed to seek: {e}");
            return false;
        }
        self.position = target.min(video.duration());
        true
    }

    /// Scroll the subtitle list so the current cue sits near its top. Only when that cue
    /// changed since the last follow, unless `force`.
    fn follow_cue(&mut self, force: bool) -> Task<Message> {
        let Some(subtitles) = self.subtitles.as_ref() else {
            return Task::none();
        };
        let cue = subtitles.last_started_index(self.display_position());
        if force {
            self.followed_cue.set(cue);
        } else if !self.followed_cue.follow(cue) {
            return Task::none();
        }
        // One cue of context above the current one.
        let rows_above = cue.unwrap_or(0).saturating_sub(1);
        Self::scroll_cue_list_to(cue_offset(subtitles, rows_above))
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
            return if self.loading {
                time::every(SPINNER_TICK).map(|_| Message::LoadingTick)
            } else {
                Subscription::none()
            };
        }
        // The tick exists only to advance the progress bar, so it is pointless while paused:
        // it used to force a full view rebuild 4x/second for as long as a video stayed open.
        // Iced re-evaluates subscriptions after every update, so pausing stops it immediately.
        let frame_tick = if self.paused {
            Subscription::none()
        } else {
            time::every(Duration::from_millis(250)).map(|_| Message::NewFrame)
        };
        let step_tick = if self.frame_step.is_some() {
            time::every(FRAME_STEP_TICK).map(|_| Message::FrameStepTick)
        } else {
            Subscription::none()
        };
        Subscription::batch([
            frame_tick,
            step_tick,
            self.controls.subscription().map(Message::Controls),
        ])
    }
}

/// Seconds on the bar as whole milliseconds.
fn secs_to_ms(secs: f32) -> u64 {
    (secs.max(0.0) as f64 * 1000.0).round() as u64
}

/// Where the pipeline is now; `None` when it cannot say (mid-seek, state change).
/// `Video::position` reports that as 0, which would flash the view back to the start.
fn query_position(video: &Video) -> Option<Duration> {
    video
        .pipeline()
        .query_position::<gst::ClockTime>()
        .map(|t| Duration::from_nanos(t.nseconds()))
}

/// The frame on screen: the player sink's last frame, its timestamp in stream time (what the
/// playhead and markers count in). `None` before the first frame and while a seek flushes.
fn shown_frame(video: &Video) -> Option<ShownFrame> {
    let sink = player_video_sink(&video.pipeline())?;
    let sample = sink.property::<Option<gst::Sample>>("last-sample")?;
    let buffer = sample.buffer()?;
    let pts = buffer.pts()?;
    let segment = sample.segment()?.downcast_ref::<gst::ClockTime>()?.clone();
    let reversed = segment.rate() < 0.0;
    let start = segment
        .to_stream_time_full(pts)
        .and_then(|time| time.positive())
        .unwrap_or(pts);
    Some(ShownFrame {
        start: Duration::from_nanos(start.nseconds()),
        duration: buffer
            .duration()
            .map(|duration| Duration::from_nanos(duration.nseconds())),
        // A forward seek clips the frame it lands in to start at the seek target.
        start_exact: reversed || segment.start() != Some(pts),
        reversed,
    })
}

/// An accurate seek running backward and stopping at `stop`: paused, it shows the frame just
/// before `stop`, whole from its true start. The pipeline keeps running backward until a seek
/// turns it forward (see `TogglePause`).
fn seek_backward(video: &Video, stop: Duration) {
    let result = video.pipeline().seek(
        -1.0,
        gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE,
        gst::SeekType::Set,
        gst::ClockTime::ZERO,
        gst::SeekType::Set,
        gst::ClockTime::from_nseconds(u64::try_from(stop.as_nanos()).unwrap_or(u64::MAX)),
    );
    if let Err(e) = result {
        log::error!("Failed to seek backward: {e}");
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
///
/// `rotation` is how the file says the picture is turned; the player shows it turned.
fn open_video(uri: &url::Url, rotation: Option<Rotation>) -> Result<Video, VideoError> {
    open_video_with(uri, None, flip_direction(rotation)).map_err(|failure| {
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
/// of the default device when one is given, and the picture turned by `videoflip` in `flip`'s
/// direction when one is given.
fn open_video_with(
    uri: &url::Url,
    audio_sink: Option<&str>,
    flip: Option<&str>,
) -> Result<Video, OpenFailure> {
    gst::init().map_err(VideoError::from)?;

    match open_pipeline(uri, false, audio_sink, flip) {
        Err(OpenFailure {
            error: VideoError::Framerate(rate),
            ..
        }) => {
            log::info!(
                "Source reports framerate {rate} (variable); retrying as {VFR_NOMINAL_FRAMERATE}"
            );
            open_pipeline(uri, true, audio_sink, flip)
        }
        other => other,
    }
}

/// The `videoflip` direction that shows a picture stored with `rotation` upright: `None` for
/// one shown as stored, or when GStreamer has no `videoflip` (the video then plays as stored).
///
/// The direction is set from frename's own reading of the file rather than left to `auto`
/// (GStreamer's orientation tag), so a debug build shows a turn it keeps in memory. A mirrored
/// picture is left to `auto`, which knows GStreamer's own naming of mirror and turn.
fn flip_direction(rotation: Option<Rotation>) -> Option<&'static str> {
    let rotation = rotation?;
    let direction = match (rotation.mirrored(), rotation.degrees()) {
        (true, _) => "auto",
        (false, 90) => "90r",
        (false, 180) => "180",
        (false, 270) => "90l",
        (false, _) => return None,
    };
    if gst::ElementFactory::find("videoflip").is_none() {
        log::warn!("GStreamer has no videoflip: the video plays as stored, not turned");
        return None;
    }
    Some(direction)
}

/// Open `uri` exactly as the player does, with the sound going nowhere, and wait until a
/// decoded frame has reached the player's video sink. Used by `--self-test`.
pub(crate) fn check_decodes(uri: &url::Url, timeout: Duration) -> Result<(), String> {
    let video =
        open_video_with(uri, Some("fakesink"), None).map_err(|failure| failure.to_string())?;
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
    flip: Option<&str>,
) -> Result<Video, OpenFailure> {
    let pipeline = gst::parse::launch(&description(uri, relabel_framerate, audio_sink, flip))
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

/// The `playbin` description, optionally rewriting the framerate on the way to the sink,
/// sending the sound to `audio_sink` instead of the default device, and turning the picture
/// with `videoflip` in the `flip` direction.
fn description(
    uri: &url::Url,
    relabel_framerate: bool,
    audio_sink: Option<&str>,
    flip: Option<&str>,
) -> String {
    // videoflip takes raw frames in common formats only: the converter in front of it takes
    // whatever the decoder gives (10-bit, hardware memory). An upright video keeps the chain
    // it always had.
    let flip = flip
        .map(|direction| format!("videoconvert ! videoflip video-direction={direction} ! "))
        .unwrap_or_default();
    // capssetter has to sit behind the NV12 filter, not in front of it: offering its own
    // framerate to a filter that then has to negotiate it upstream collapses the whole
    // graph with "internal data stream error" — including on files that were fine.
    let video_sink = if relabel_framerate {
        format!(
            "{flip}videoscale ! videoconvert ! video/x-raw,format=NV12,pixel-aspect-ratio=1/1 \
             ! capssetter caps=video/x-raw,framerate={VFR_NOMINAL_FRAMERATE} \
             ! appsink name=iced_video drop=true"
        )
    } else {
        format!(
            "{flip}videoscale ! videoconvert ! appsink name=iced_video drop=true \
             caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1"
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_range_stops_at_its_end_only_after_playing_inside_it() {
        let secs = Duration::from_secs;
        let mut range = RangePlay {
            start: secs(10),
            end: secs(20),
            entered: false,
        };
        // A tick from before the seek landed, past the end: not the end of the range.
        assert!(!range.done_at(secs(40)));
        assert!(!range.done_at(secs(12)));
        assert!(range.done_at(secs(20)));
    }

    /// Two turns in a row: the second reopen starts before the first one landed, and both go
    /// back to the moment shown before the first; only the latest load is taken.
    #[test]
    fn reopening_twice_keeps_the_moment_and_takes_only_the_latest_load() {
        let mut player = VideoPlayerState {
            current_path: Some(PathBuf::from("C:/no/such/clip.mp4")),
            position: Duration::from_secs(42),
            ..VideoPlayerState::default()
        };
        let _ = player.reload_video();
        let _ = player.reload_video();
        assert_eq!(player.resume_at, Some(Duration::from_secs(42)));

        let stale = Message::VideoLoaded {
            video: Arc::new(Mutex::new(None)),
            rotation: None,
            generation: player.load_generation - 1,
        };
        let _ = player.update(stale);
        assert!(player.loading, "a stale load does not end the latest one");
        assert!(!player.load_failed);
        assert_eq!(player.resume_at, Some(Duration::from_secs(42)));
    }

    /// Issue #91: `Unload` while a load is still opening the file on a blocking thread must
    /// hold `VideoUnloaded` back until that load lands — GStreamer may still hold the file
    /// open, and a save made right after `VideoUnloaded` (the caller's "safe to rename now"
    /// signal) could hit a sharing violation on Windows.
    #[test]
    fn unload_waits_for_a_load_still_in_flight() {
        let mut player = VideoPlayerState {
            loading: true,
            loads_in_flight: 1,
            load_generation: 3,
            ..VideoPlayerState::default()
        };
        let _ = player.update(Message::Unload);
        assert!(player.unloading, "must wait for the in-flight load");
        assert!(
            player.loading,
            "the view must keep showing its spinner, not \"nothing loaded\", during the wait"
        );

        // The load lands, stale (Unload bumped the generation): only now is it safe to say so.
        let stale = Message::VideoLoaded {
            video: Arc::new(Mutex::new(None)),
            rotation: None,
            generation: 3,
        };
        let _ = player.update(stale);
        assert!(!player.unloading, "the wait ends once the stale load lands");
        assert!(!player.loading, "the wait is over");
        assert_eq!(player.loads_in_flight, 0);
    }

    /// Two loads in flight (reopened twice) when `Unload` arrives: it must wait for both to
    /// land, not just the first stale one to arrive — a plain "is a load closing" flag would
    /// answer `VideoUnloaded` while the other load could still hold the file open.
    #[test]
    fn unload_waits_for_every_load_still_in_flight() {
        let mut player = VideoPlayerState {
            loading: true,
            loads_in_flight: 2,
            load_generation: 5,
            ..VideoPlayerState::default()
        };
        let _ = player.update(Message::Unload);
        assert!(player.unloading);
        assert!(player.loading, "still waiting: the spinner must stay up");

        let stale = |generation| Message::VideoLoaded {
            video: Arc::new(Mutex::new(None)),
            rotation: None,
            generation,
        };
        let _ = player.update(stale(4));
        assert!(
            player.unloading,
            "one of two loads landed: still waiting on the other"
        );
        assert!(player.loading, "still one load left in flight");
        assert_eq!(player.loads_in_flight, 1);

        let _ = player.update(stale(3));
        assert!(!player.unloading, "both loads landed: safe to say so now");
        assert!(!player.loading, "the wait is over");
        assert_eq!(player.loads_in_flight, 0);
    }

    #[test]
    fn only_a_turned_picture_gets_a_flip() {
        assert_eq!(flip_direction(None), None);
        assert_eq!(flip_direction(Some(Rotation::UPRIGHT)), None);
        let url = url::Url::parse("file:///C:/clip.mp4").expect("url");
        assert!(!description(&url, false, None, None).contains("videoflip"));
        let turned = description(&url, true, None, Some("90r"));
        assert!(
            turned.contains("videoconvert ! videoflip video-direction=90r ! videoscale"),
            "{turned}"
        );
    }

    /// The player's sink gets a phone-style portrait clip (stored landscape, flagged as turned)
    /// upright, and every turn frename can set swaps or keeps its sides. Linux and Windows,
    /// like the self-test's decoding tests: both CI jobs have GStreamer.
    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn a_turned_clip_reaches_the_player_turned() {
        let clip =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/folder/rotated-90.mp4");
        let flagged = FileTagger::video_rotation(&clip).expect("rotation");
        assert_eq!(flagged.degrees(), 270, "ffmpeg's 90 is counter-clockwise");
        let url = url::Url::from_file_path(&clip).expect("url");
        let size = |rotation: Rotation| {
            gst::init().expect("gstreamer");
            let flip = flip_direction(Some(rotation));
            let video = open_video_with(&url, Some("fakesink"), flip)
                .map_err(|failure| failure.to_string())
                .expect("open");
            let caps = player_video_sink(&video.pipeline())
                .and_then(|sink| sink.static_pad("sink"))
                .and_then(|pad| pad.current_caps())
                .expect("caps");
            let s = caps.structure(0).expect("structure");
            (
                s.get::<i32>("width").expect("width"),
                s.get::<i32>("height").expect("height"),
            )
        };
        let (width, height) = size(Rotation::UPRIGHT);
        assert!(width > height, "stored landscape: {width}×{height}");
        assert_eq!(size(flagged), (height, width));
        assert_eq!(size(Rotation::UPRIGHT.turned(1)), (height, width));
        assert_eq!(size(Rotation::UPRIGHT.turned(2)), (width, height));
    }

    /// Issue #162 on a real (turned) clip: a step pauses playback, five steps forward visit five
    /// new frames, and five back return to the frame they started from; the playhead is the
    /// shown frame's time each time.
    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn frame_steps_forward_and_back_return_to_the_same_frame() {
        use crate::features::video_controls::FrameStep;
        let clip =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/folder/rotated-90.mp4");
        let url = url::Url::from_file_path(&clip).expect("url");
        gst::init().expect("gstreamer");
        let flip = flip_direction(FileTagger::video_rotation(&clip).ok());
        let video = open_video_with(&url, Some("fakesink"), flip)
            .map_err(|failure| failure.to_string())
            .expect("open");
        let mut player = VideoPlayerState {
            current_video: Some(video),
            paused: false,
            ..VideoPlayerState::default()
        };
        let shown = |player: &VideoPlayerState| {
            let video = player.current_video.as_ref().expect("video");
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                if let Some(frame) = shown_frame(video) {
                    return frame;
                }
                assert!(Instant::now() < deadline, "no frame on screen");
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        let step = |player: &mut VideoPlayerState, step: FrameStep| {
            let _ = player.update(Message::Controls(video_controls::Message::StepFrame(step)));
            assert!(player.paused, "a step pauses playback");
            assert!(player.frame_step.is_some(), "the step is on its way");
            while player.frame_step.is_some() {
                std::thread::sleep(FRAME_STEP_TICK);
                let _ = player.update(Message::FrameStepTick);
            }
            let frame = shown(player);
            assert_eq!(player.position, frame_step::frame_position(frame));
            frame.end().expect("a frame of known length")
        };
        let first = step(&mut player, FrameStep::Forward);
        let mut forward = vec![first];
        for _ in 0..5 {
            forward.push(step(&mut player, FrameStep::Forward));
        }
        assert!(
            forward.windows(2).all(|pair| pair[0] < pair[1]),
            "each step forward shows a later frame: {forward:?}"
        );
        let mut back = Vec::new();
        for _ in 0..5 {
            back.push(step(&mut player, FrameStep::Back));
        }
        back.reverse();
        assert_eq!(back, forward[..5], "back visits the same frames");

        // From a moment a seek landed in mid-frame (a marker): back shows the frame before
        // in one step, and forward returns to the frame the seek showed.
        let _ = player.update(Message::SeekExact(1_234));
        let deadline = Instant::now() + Duration::from_secs(10);
        let marked = loop {
            let frame = shown(&player);
            if frame
                .end()
                .is_some_and(|end| end > Duration::from_millis(1_234))
            {
                break frame.end().expect("an end");
            }
            assert!(Instant::now() < deadline, "the seek did not land");
            std::thread::sleep(Duration::from_millis(10));
        };
        let before = step(&mut player, FrameStep::Back);
        assert!(before < marked);
        assert_eq!(step(&mut player, FrameStep::Forward), marked);

        // A step back leaves the pipeline running backward; playing turns it forward.
        let before = step(&mut player, FrameStep::Back);
        let _ = player.update(Message::TogglePause);
        assert!(!player.paused);
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let frame = shown(&player);
            if !frame.reversed && frame.end().is_some_and(|end| end > before) {
                break;
            }
            assert!(Instant::now() < deadline, "playback did not go on forward");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn the_subtitle_list_offset_is_kept_and_a_load_starts_it_over() {
        let mut state = VideoPlayerState::default();
        let _ = state.update(Message::CueListScrolled(80.0, 300.0));
        assert_eq!(state.cue_scroll_y(), 80.0);
        let _ = state.update(Message::RestoreCueScroll(80.0));
        // Nothing is armed for a list that is not shown.
        assert!(!state.cue_restore.report(80.0));
    }

    fn player_with_cues() -> VideoPlayerState {
        let srt = "1\n00:00:01,000 --> 00:00:02,000\none\n\n2\n00:00:03,000 --> 00:00:04,000\ntwo\n\n3\n00:00:05,000 --> 00:00:06,000\nthree\n";
        VideoPlayerState {
            subtitles: Some(Arc::new(Subtitles::parse(srt))),
            ..VideoPlayerState::default()
        }
    }

    /// Issue #171: after a click on a cue (which seeks), neither the frames before the seek
    /// landed nor the one at the cue scroll the list; playback on its own does.
    #[test]
    fn a_click_on_a_cue_does_not_scroll_the_list_but_playback_to_the_next_does() {
        let mut player = player_with_cues();
        let _ = player.update(Message::CueListScrolled(40.0, 300.0));
        player.followed_cue.clicked(Some(2));
        assert_eq!(player.cue_scroll_y(), 40.0);
        player.position = Duration::from_secs(3);
        assert_eq!(player.follow_cue(false).units(), 0, "the old frame");
        player.position = Duration::from_secs(5);
        assert_eq!(player.follow_cue(false).units(), 0, "the clicked cue");
        player.position = Duration::from_secs(3);
        assert_eq!(player.follow_cue(false).units(), 0, "a late old frame");

        let mut player = player_with_cues();
        player.position = Duration::from_secs(1);
        assert_eq!(player.follow_cue(false).units(), 1);
        player.position = Duration::from_secs(3);
        assert_eq!(player.follow_cue(false).units(), 1, "the next cue: scroll");
    }
}
