//! Messages for the video player sub-feature.

use frename_core::Subtitles;
use iced_video_player::Video;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::Overlay;
use crate::features::{markers, video_controls};

/// A video handed from the loading thread to the update thread.
///
/// `Message` has to be `Clone` but `Video` is not, so the video travels in a shared
/// slot that the handler empties with `take()`. `None` means the open failed.
pub type LoadedVideo = Arc<Mutex<Option<Video>>>;

/// Messages handled by the video player.
#[derive(Debug, Clone)]
pub enum Message {
    /// Video finished loading; carries the opened video, or `None` when the open failed.
    VideoLoaded(LoadedVideo),
    /// Video became available after loading
    VideoReady { duration_secs: f32 },
    /// New video frame rendered (triggers view refresh for progress bar)
    NewFrame,
    /// End of stream reached
    EndOfStream,
    /// Toggle pause state on the video
    TogglePause,
    /// Seek to position in seconds
    Seek(f32),
    /// Controls message
    Controls(video_controls::Message),
    /// Unload current video (e.g. before switching file). Emits VideoUnloaded when done.
    Unload,
    /// Current video has been unloaded; safe to persist file and load next.
    VideoUnloaded,
    /// User clicked the fullscreen toggle button.
    ToggleFullscreen,
    /// Request to capture the current video position as the segment start marker.
    /// Emits SegmentStartMarked when captured; no-op when no video is loaded.
    CaptureSegmentStart,
    /// Request to capture the current video position as the segment end marker.
    /// Emits SegmentEndMarked when captured; no-op when no video is loaded.
    CaptureSegmentEnd,
    /// Segment start was captured. Bubbles up to FolderWorkspace.
    SegmentStartMarked(f32),
    /// Segment end was captured. Bubbles up to FolderWorkspace.
    SegmentEndMarked(f32),
    /// Frame captured at position (ms) with JPEG bytes. Bubbles up to FolderWorkspace.
    ScreenshotTaken(u64, Vec<u8>),
    /// A marker key or a marker list row. Bubbles up to FolderWorkspace with the playhead.
    Markers(markers::Message),
    /// Show or hide the marker list over the picture (the `◆` button).
    ToggleMarkerList,
    /// Show the marker list (e.g. when `F2 F2` opens a row).
    ShowMarkerList,
    /// A tab of the side list was picked.
    ShowOverlay(Overlay),
    /// Seek exactly to this time (ms), e.g. a marker.
    SeekExact(u64),
    /// Show a short note in the controls bar (e.g. `Frame saved`) for a moment.
    ShowNotice(String),
    /// Hide the note with this number, unless a newer one replaced it (internal).
    ClearNotice(u64),
    /// Subtitle file next to `video_path` was read; `None` when there is none.
    SubtitlesLoaded {
        video_path: PathBuf,
        subtitles: Option<Arc<Subtitles>>,
    },
    /// User picked a cue in the subtitle list: seek to its start.
    SeekToCue(usize),
    /// Show or hide the subtitle list over the picture.
    ToggleCueList,
    /// Autoplay setting changed: whether videos opened from now on start playing.
    SetAutoplay(bool),
}
