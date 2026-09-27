//! State for the unified media_viewer feature.

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
                self.video
                    .load_video(frename_core::FileTagger::disk_path(file.file_path()))
                    .map(Message::Video)
            }
            FileKind::Other => {
                self.active = ActiveMedia::Unsupported;
                Task::none()
            }
        }
    }

    /// Returns `true` when a video is currently being shown.
    /// Used to guard fullscreen toggle: no point going fullscreen with nothing to show.
    pub fn is_previewable(&self) -> bool {
        matches!(self.active, ActiveMedia::Video)
    }

    /// The playhead of the open video in milliseconds; `None` when no video is shown.
    pub fn video_position_ms(&self) -> Option<u64> {
        matches!(self.active, ActiveMedia::Video).then(|| self.video.position_ms())
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
