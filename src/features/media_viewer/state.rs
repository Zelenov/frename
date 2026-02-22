//! State for the unified media_viewer feature.

use frename_core::{File, FileKind};
use iced::{Subscription, Task};

use super::image::ImageViewerState;
use super::video::VideoPlayerState;
use super::{video, Message};

/// Which media type is currently active in the left panel.
#[derive(Default)]
pub(super) enum ActiveMedia {
    #[default]
    None,
    Video,
    Image,
    /// A file is selected but its type is not supported for preview.
    Unsupported,
}

/// Unified media viewer: routes video and image files to the correct sub-feature.
/// FolderWorkspace holds one `MediaViewerState` and calls `open()` for every file.
#[derive(Default)]
pub struct MediaViewerState {
    pub(super) active: ActiveMedia,
    pub(super) video: VideoPlayerState,
    pub(super) image: ImageViewerState,
}

impl MediaViewerState {
    /// Open a file. Routes internally to the video player or image viewer based on `file.kind()`.
    /// FolderWorkspace must ensure a video unload has completed before calling this for
    /// the next file (when `needs_unload_before_rename()` was true).
    pub fn open(&mut self, file: &File) -> Task<Message> {
        match file.kind() {
            FileKind::Video => {
                self.active = ActiveMedia::Video;
                self.image.unload();
                self.video
                    .load_video(file.file_path().to_path_buf())
                    .map(Message::Video)
            }
            FileKind::Image => {
                self.active = ActiveMedia::Image;
                // video is already idle — caller ensured this if needed
                self.image
                    .load_image(file.file_path().to_path_buf())
                    .map(Message::Image)
            }
            FileKind::Other => {
                self.active = ActiveMedia::Unsupported;
                self.image.unload();
                Task::none()
            }
        }
    }

    /// Returns `true` when a GStreamer video is active and must be unloaded before the
    /// previous file can be renamed. Images hold no file handle so no unload is needed.
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
            Message::Video(vm) => self.video.update(vm).map(Message::Video),
            Message::Image(im) => self.image.update(im).map(Message::Image),

            Message::Unload => match self.active {
                ActiveMedia::Video => {
                    // Async teardown — VideoUnloaded will follow → translated to Unloaded above.
                    self.video
                        .update(video::Message::Unload)
                        .map(Message::Video)
                }
                // Images, unsupported, and idle state unload synchronously.
                ActiveMedia::Image | ActiveMedia::Unsupported | ActiveMedia::None => {
                    self.image.unload();
                    self.active = ActiveMedia::None;
                    Task::done(Message::Unloaded)
                }
            },

            // Intercepted by FolderWorkspace; no-op here if it ever reaches update().
            Message::Unloaded => Task::none(),
        }
    }

    /// Subscriptions: only the video player needs periodic ticks and keyboard shortcuts.
    /// When an image is displayed subscriptions are suppressed automatically.
    pub fn subscription(&self) -> Subscription<Message> {
        match self.active {
            ActiveMedia::Video => self.video.subscription().map(Message::Video),
            ActiveMedia::Image | ActiveMedia::Unsupported | ActiveMedia::None => Subscription::none(),
        }
    }
}
