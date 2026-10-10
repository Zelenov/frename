//! State for the media_viewer feature: video, or a placeholder for anything else.

use std::time::Duration;

use frename_core::{File, FileKind};
use iced::{Subscription, Task};

use super::video::VideoPlayerState;
use super::{video, Message};

/// Which media type is currently active in the left panel.
#[derive(Default)]
pub(super) enum ActiveMedia {
    #[default]
    None,
    Video,
    /// A file is selected but its type is not supported for preview.
    Unsupported,
}

/// Media viewer: opens videos in the video player, shows a placeholder for anything else.
/// FolderWorkspace holds one `MediaViewerState` and calls `open()` for every file.
#[derive(Default)]
pub struct MediaViewerState {
    pub(super) active: ActiveMedia,
    pub(super) video: VideoPlayerState,
}

impl MediaViewerState {
    /// Open a file. Videos go to the video player based on `file.kind()`.
    /// FolderWorkspace must ensure a video unload has completed before calling this for
    /// the next file (when `needs_unload_before_rename()` was true).
    pub fn open(&mut self, file: &File) -> Task<Message> {
        match file.kind() {
            FileKind::Video => {
                self.active = ActiveMedia::Video;
                let in_point = file
                    .snapshot()
                    .segment_start()
                    .filter(|secs| secs.is_finite() && *secs >= 0.0)
                    .map(Duration::from_secs_f32);
                self.video
                    .load_video(
                        frename_core::FileTagger::disk_path(file.file_path()),
                        file.file_path().to_path_buf(),
                        file.id(),
                        in_point,
                    )
                    .map(Message::Video)
            }
            FileKind::Other => {
                // The clip shown until now keeps where it was, and is let go: nothing done while
                // this file shows may move it (#161).
                self.video.close_clip();
                self.active = ActiveMedia::Unsupported;
                Task::none()
            }
        }
    }

    /// Open the shown video again where it was, e.g. after its rotation changed.
    pub fn reload_video(&mut self) -> Task<Message> {
        match self.active {
            ActiveMedia::Video => self.video.reload_video().map(Message::Video),
            ActiveMedia::Unsupported | ActiveMedia::None => Task::none(),
        }
    }

    /// How many times the video player has started loading, for tests of reopens.
    #[cfg(test)]
    pub fn video_loads_started(&self) -> u64 {
        self.video.loads_started()
    }

    /// For tests without a real video: see [`VideoPlayerState::pretend_shown_at`].
    #[cfg(test)]
    pub fn pretend_video_shown_at(&mut self, position: Duration) {
        self.video.pretend_shown_at(position);
    }

    /// For tests: see [`VideoPlayerState::reopens_at`].
    #[cfg(test)]
    pub fn video_reopens_at(&self) -> Option<Duration> {
        self.video.reopens_at()
    }

    /// For tests: see [`VideoPlayerState::pretend_resume_note`].
    #[cfg(test)]
    pub fn pretend_resume_note(&mut self) {
        self.video.pretend_resume_note();
    }

    /// For tests: see [`VideoPlayerState::resume_lookup`].
    #[cfg(test)]
    pub fn video_resume_lookup(&self) -> Option<Option<Duration>> {
        self.video.resume_lookup()
    }

    /// Returns `true` when a video is currently being shown.
    /// Used to guard fullscreen toggle: no point going fullscreen with nothing to show.
    pub fn is_previewable(&self) -> bool {
        matches!(self.active, ActiveMedia::Video)
    }

    /// The video player's short note (e.g. "Frame saved", a save refused), if any. Kept and
    /// shown even once the video that set it is no longer the active pane: a save can be
    /// refused right as the video unloads for the next file or a batch job (issue #84), and the
    /// note must still reach the editor, not disappear behind a hidden video component.
    pub fn notice(&self) -> Option<&str> {
        self.video.notice()
    }

    /// Whether the note over the picture says where the clip continued (#161): `Home` starts
    /// it over then, even from a search field.
    pub fn resume_note_shown(&self) -> bool {
        self.video.notice_starts_over()
    }

    /// The playhead of the open video in milliseconds; `None` when no video is shown.
    pub fn video_position_ms(&self) -> Option<u64> {
        matches!(self.active, ActiveMedia::Video).then(|| self.video.position_ms())
    }

    /// Where the subtitle list is scrolled to.
    pub fn cue_scroll_y(&self) -> f32 {
        self.video.cue_scroll_y()
    }

    /// Whether the marker list is open over the video.
    pub fn marker_list_shown(&self) -> bool {
        matches!(self.active, ActiveMedia::Video) && self.video.show_marker_list()
    }

    /// Returns `true` when a GStreamer video is active and must be unloaded before the
    /// previous file can be renamed.
    pub fn needs_unload_before_rename(&self) -> bool {
        matches!(self.active, ActiveMedia::Video) && self.video.is_active()
    }

    /// Handle media viewer messages.
    /// Note: FolderWorkspace intercepts `Message::Unloaded` before calling this.
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            // Video pipeline finished teardown — translate to Unloaded so FolderWorkspace can proceed.
            Message::Video(video::Message::VideoUnloaded) => {
                self.active = ActiveMedia::None;
                Task::done(Message::Unloaded)
            }
            // Bubble ToggleFullscreen up so FolderWorkspace can intercept it.
            Message::Video(video::Message::ToggleFullscreen) => {
                Task::done(Message::ToggleFullscreen)
            }
            // Bubble segment markers up so FolderWorkspace can intercept them.
            Message::Video(video::Message::SegmentStartMarked(secs)) => {
                Task::done(Message::SegmentStartMarked(secs))
            }
            Message::Video(video::Message::SegmentEndMarked(secs)) => {
                Task::done(Message::SegmentEndMarked(secs))
            }
            Message::Video(video::Message::ScreenshotTaken(position_ms, jpeg)) => {
                Task::done(Message::ScreenshotTaken(position_ms, jpeg))
            }
            Message::Video(vm) => self.video.update(vm).map(Message::Video),

            Message::Unload => match self.active {
                ActiveMedia::Video => {
                    // Async teardown — VideoUnloaded will follow → translated to Unloaded above.
                    self.video
                        .update(video::Message::Unload)
                        .map(Message::Video)
                }
                // Unsupported and idle state unload synchronously.
                ActiveMedia::Unsupported | ActiveMedia::None => {
                    self.active = ActiveMedia::None;
                    Task::done(Message::Unloaded)
                }
            },

            // Intercepted by FolderWorkspace; no-op here if it ever reaches update().
            Message::Unloaded => Task::none(),
            // Intercepted by FolderWorkspace; no-op here if it ever reaches update().
            Message::ToggleFullscreen => Task::none(),
            // Intercepted by FolderWorkspace; no-op here if they ever reach update().
            Message::SegmentStartMarked(_) | Message::SegmentEndMarked(_) => Task::none(),
            Message::ScreenshotTaken(_, _) => Task::none(),
        }
    }

    /// Subscriptions: only the video player needs periodic ticks and keyboard shortcuts.
    pub fn subscription(&self) -> Subscription<Message> {
        match self.active {
            ActiveMedia::Video => self.video.subscription().map(Message::Video),
            ActiveMedia::Unsupported | ActiveMedia::None => Subscription::none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Issue #84: a save can be refused right as the video unloads for the next file or a batch
    /// job, so the notice must still be readable once the video is no longer the active pane —
    /// it lives in `video`'s own state, not gated behind `active`.
    #[test]
    fn the_notice_survives_the_video_becoming_inactive() {
        let mut state = MediaViewerState::default();
        let _ = state.update(Message::Video(video::Message::ShowNotice(
            "Not saved: a file with that name already exists".to_string(),
        )));
        assert_eq!(
            state.notice(),
            Some("Not saved: a file with that name already exists")
        );

        let _ = state.update(Message::Video(video::Message::VideoUnloaded));

        assert!(!state.is_previewable(), "no video is active any more");
        assert_eq!(
            state.notice(),
            Some("Not saved: a file with that name already exists"),
            "the notice must still be readable once the video became inactive"
        );
    }
}
