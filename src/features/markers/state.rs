//! State of the marker list that is not the markers themselves: the row being edited, the
//! open color picker, and what `F2` did last.

use std::time::{Duration, Instant};

/// A second `F2` within this time after one that added a marker opens that marker's row, so
/// `F2 F2` marks a moment and names it; later, `F2` adds again.
pub const NAME_WINDOW: Duration = Duration::from_millis(1500);

/// `F2` held longer than this makes a range instead of a point; a shorter press stays a point,
/// so `F2 F2` still marks a moment and names it.
pub const RANGE_HOLD: Duration = Duration::from_millis(400);

/// A range shorter than this is a point: dragging a range's ends together makes it one.
pub const MIN_RANGE_MS: u64 = 100;

/// The marker `F2` added and is still held down.
#[derive(Debug)]
pub struct Recording {
    pub guid: String,
    pub start_ms: u64,
    pressed_at: Instant,
}

impl Recording {
    /// The marker's length with the playhead at `position_ms`: up to the playhead once `F2`
    /// has been held for [`RANGE_HOLD`], none before, and none while the playhead is not past
    /// the start (paused and not moved).
    pub fn duration_at(&self, position_ms: u64) -> u64 {
        if self.pressed_at.elapsed() < RANGE_HOLD {
            return 0;
        }
        position_ms.saturating_sub(self.start_ms)
    }
}

/// The row open for editing: its name field is shown.
#[derive(Debug)]
pub struct MarkerEdit {
    pub guid: String,
}

#[derive(Debug, Default)]
pub struct MarkersState {
    /// At most one row is open, so only one editor exists at a time.
    edit: Option<MarkerEdit>,
    /// The marker whose color picker is shown.
    color_picker: Option<String>,
    /// The marker `F2` added last and when; see [`NAME_WINDOW`].
    last_added: Option<(String, Instant)>,
    /// Row the list scrolled to last when following playback.
    followed: Option<usize>,
    /// The marker `F2` is held on; it grows with the playhead.
    recording: Option<Recording>,
}

impl MarkersState {
    pub fn edit(&self) -> Option<&MarkerEdit> {
        self.edit.as_ref()
    }

    pub fn is_editing(&self) -> bool {
        self.edit.is_some()
    }

    /// Open the row of `guid` for renaming.
    pub fn open(&mut self, guid: String) {
        self.edit = Some(MarkerEdit { guid });
        self.color_picker = None;
        self.last_added = None;
    }

    pub fn close(&mut self) {
        self.edit = None;
    }

    pub fn color_picker(&self) -> Option<&str> {
        self.color_picker.as_deref()
    }

    pub fn toggle_color_picker(&mut self, guid: String) {
        self.color_picker = if self.color_picker.as_deref() == Some(guid.as_str()) {
            None
        } else {
            Some(guid)
        };
    }

    pub fn close_color_picker(&mut self) {
        self.color_picker = None;
    }

    pub fn added(&mut self, guid: String) {
        self.last_added = Some((guid, Instant::now()));
    }

    /// The marker `F2` added less than [`NAME_WINDOW`] ago, if nothing else happened since.
    pub fn take_recently_added(&mut self) -> Option<String> {
        let (guid, at) = self.last_added.take()?;
        (at.elapsed() < NAME_WINDOW).then_some(guid)
    }

    /// Something other than `F2` happened: a second `F2` adds again.
    pub fn forget_added(&mut self) {
        self.last_added = None;
    }

    /// `F2` went down and added the marker `guid` at `start_ms`.
    pub fn start_recording(&mut self, guid: String, start_ms: u64) {
        self.recording = Some(Recording {
            guid,
            start_ms,
            pressed_at: Instant::now(),
        });
    }

    pub fn recording(&self) -> Option<&Recording> {
        self.recording.as_ref()
    }

    /// `F2` came up: the held marker, if any.
    pub fn stop_recording(&mut self) -> Option<Recording> {
        self.recording.take()
    }

    /// Pretend `F2` went down `by` earlier (tests of a long hold).
    #[cfg(test)]
    pub fn backdate_recording(&mut self, by: Duration) {
        if let Some(recording) = self.recording.as_mut() {
            recording.pressed_at -= by;
        }
    }

    /// Record the row the list follows; returns whether it changed.
    pub fn follow(&mut self, row: Option<usize>) -> bool {
        std::mem::replace(&mut self.followed, row) != row
    }

    /// Start over for another file.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
