//! Messages for the video player sub-feature.

use crate::features::video_controls;

/// Messages handled by the video player.
#[derive(Debug, Clone)]
pub enum Message {
    /// Video finished loading (success flag)
    VideoLoaded(bool),
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
    /// Screenshot captured at position (ms) with JPEG bytes. Bubbles up to FolderWorkspace.
    ScreenshotTaken(u64, Vec<u8>),
}
