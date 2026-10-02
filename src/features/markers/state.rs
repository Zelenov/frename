//! State of the marker list that is not the markers themselves: the row being edited, the
//! open color picker, and what `F2` did last.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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

/// The row open for editing: its name field is shown, wrapping a long name.
#[derive(Debug)]
pub struct MarkerEdit {
    pub guid: String,
    /// The name field's text and cursor.
    pub name: iced::widget::text_editor::Content,
    /// The marker's name when the row opened, to record the editing as one undo step.
    pub original_name: String,
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
    followed: crate::ui::scroll::FollowedRow,
    /// The marker `F2` is held on; it grows with the playhead.
    recording: Option<Recording>,
    /// Where the list is scrolled to (from the top). The list's own offset is lost when the
    /// view builds it somewhere else in the tree (fullscreen on or off), so it is put back.
    scroll_y: f32,
    /// The list's viewport height as last reported; 0 until it is.
    viewport: f32,
    /// The list being put back after fullscreen; its end report carries the new viewport's
    /// height, to see whether the lit marker is still shown.
    restore: crate::ui::scroll::ScrollRestore,
    /// Markers being described with AI: each one's request (its number, which its answer
    /// carries) and the flag that stops it.
    describing: HashMap<String, (u64, Arc<AtomicBool>)>,
    /// The number the next request gets. Never reset, so the answer of a request stopped
    /// earlier never passes for a later one's.
    next_request: u64,
}

impl MarkersState {
    pub fn edit(&self) -> Option<&MarkerEdit> {
        self.edit.as_ref()
    }

    pub fn is_editing(&self) -> bool {
        self.edit.is_some()
    }

    /// Open the row of `guid` for renaming; its field starts with `name`, the cursor at the end.
    pub fn open(&mut self, guid: String, name: &str) {
        let mut content = iced::widget::text_editor::Content::with_text(name);
        content.perform(iced::widget::text_editor::Action::Move(
            iced::widget::text_editor::Motion::DocumentEnd,
        ));
        self.edit = Some(MarkerEdit {
            guid,
            name: content,
            original_name: name.to_string(),
        });
        self.color_picker = None;
        self.last_added = None;
    }

    pub fn close(&mut self) {
        self.edit = None;
    }

    /// Apply an edit to the open row's name field; returns the open marker and its new name,
    /// on one line.
    pub fn edit_name(
        &mut self,
        action: iced::widget::text_editor::Action,
    ) -> Option<(String, String)> {
        let edit = self.edit.as_mut()?;
        edit.name.perform(action);
        let name = edit.name.text().replace(['\r', '\n'], " ");
        Some((edit.guid.clone(), name.trim_end_matches(' ').to_string()))
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
        self.followed.follow(row)
    }

    /// A click on a row jumped to `row`: the list stays where it is (§13.2).
    pub fn clicked(&mut self, row: Option<usize>) {
        self.followed.clicked(row);
    }

    pub fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    /// The list's viewport height as last reported by a scroll; 0 when none was yet.
    pub fn viewport(&self) -> f32 {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: f32) {
        self.viewport = viewport;
    }

    /// Record where the list is scrolled to; true when this is the report of a restore (see
    /// [`Self::restored`]), which is then done.
    pub fn set_scroll_y(&mut self, y: f32) -> bool {
        self.scroll_y = y;
        self.restore.report(y)
    }

    /// The list is being put back at `y`.
    pub fn restored(&mut self, y: f32) {
        self.restore.arm(y);
    }

    /// Mark `guid` as being described: the new request's number and the flag that stops it, or
    /// `None` when one is on its way already.
    pub fn start_describing(&mut self, guid: &str) -> Option<(u64, Arc<AtomicBool>)> {
        if self.describing.contains_key(guid) {
            return None;
        }
        self.next_request += 1;
        let request = (self.next_request, Arc::new(AtomicBool::new(false)));
        self.describing.insert(guid.to_string(), request.clone());
        Some(request)
    }

    /// The answer of request `request` for `guid` came: whether it is the one still waited
    /// for (then it is not any more). An answer of a stopped request, or of an earlier one, is
    /// not.
    pub fn finish_describing(&mut self, guid: &str, request: u64) -> bool {
        if self
            .describing
            .get(guid)
            .is_some_and(|(id, _)| *id == request)
        {
            self.describing.remove(guid);
            true
        } else {
            false
        }
    }

    /// Stop the request for `guid`; its answer is not used.
    pub fn stop_describing(&mut self, guid: &str) {
        if let Some((_, cancel)) = self.describing.remove(guid) {
            cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Stop every request (a batch job closes the clip).
    pub fn stop_all_describing(&mut self) {
        for (_, cancel) in self.describing.values() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.describing.clear();
    }

    /// The number of the request for `guid` on its way (tests hand its answer in).
    #[cfg(test)]
    pub fn request_of(&self, guid: &str) -> Option<u64> {
        self.describing.get(guid).map(|(id, _)| *id)
    }

    pub fn is_describing(&self, guid: &str) -> bool {
        self.describing.contains_key(guid)
    }

    /// Whether any marker is being described (the spinner turns).
    pub fn any_describing(&self) -> bool {
        !self.describing.is_empty()
    }

    /// Start over for another file. Its requests stop: their answers would not find the markers.
    pub fn reset(&mut self) {
        self.stop_all_describing();
        let next_request = self.next_request;
        *self = Self {
            next_request,
            ..Self::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_marker_is_described_once_at_a_time_and_leaving_stops_it() {
        let mut state = MarkersState::default();
        let (first, cancel) = state.start_describing("a").expect("started");
        assert!(state.start_describing("a").is_none(), "already on its way");
        assert!(state.is_describing("a") && state.any_describing());
        let (other, other_cancel) = state.start_describing("b").expect("another marker");
        state.stop_describing("b");
        assert!(other_cancel.load(Ordering::Relaxed), "stopped");
        assert!(
            !state.finish_describing("b", other),
            "a stopped answer is not used"
        );
        state.reset();
        assert!(cancel.load(Ordering::Relaxed), "leaving the clip stops it");
        assert!(!state.any_describing());
        let (again, _) = state.start_describing("a").expect("started again");
        assert!(again > first, "numbers go on after a reset");
        assert!(
            !state.finish_describing("a", first),
            "the old answer is not this one's"
        );
        assert!(state.is_describing("a"));
        assert!(state.finish_describing("a", again));
    }

    #[test]
    fn a_stopped_request_answering_late_does_not_end_the_next_one() {
        let mut state = MarkersState::default();
        let (first, _) = state.start_describing("a").unwrap();
        state.stop_describing("a");
        let (second, _) = state.start_describing("a").unwrap();
        assert!(!state.finish_describing("a", first));
        assert!(state.is_describing("a"), "the second is still waited for");
        assert!(state.finish_describing("a", second));
        let (_, cancel) = state.start_describing("b").unwrap();
        state.stop_all_describing();
        assert!(cancel.load(Ordering::Relaxed) && !state.any_describing());
    }

    /// Issue #171: a click on a marker jumps there without scrolling the list.
    #[test]
    fn a_click_on_a_marker_does_not_scroll_the_list() {
        let mut state = MarkersState::default();
        assert!(state.follow(Some(0)));
        state.clicked(Some(4));
        assert!(!state.follow(Some(4)), "the seek's own follow");
        assert!(!state.follow(Some(3)), "a frame before the jump landed");
        assert!(!state.follow(Some(4)), "the landed one");
    }
}
