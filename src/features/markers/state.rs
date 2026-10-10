//! State of the marker list that is not the markers themselves: the row being edited, the
//! open color picker, and what `F2` did last.

use crate::ui::tokens::MARKER_ROW_HEIGHT;
use frename_core::Marker;
use std::collections::{HashMap, VecDeque};
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

/// "Describe all unnamed" keeps at most this many requests on their way at once (the others wait
/// their turn): each reads the clip's frames and is paid for, and the API has rate limits.
pub const MAX_DESCRIBING_AT_ONCE: usize = 3;

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
    /// Each row's height as laid out, by index; the list is scrolled to a row by them. Kept when
    /// another file is opened: a row's widget reports only when its height changes.
    row_heights: Vec<f32>,
    /// The list being put back after fullscreen; its end report carries the new viewport's
    /// height, to see whether the lit marker is still shown.
    restore: crate::ui::scroll::ScrollRestore,
    /// Markers being described with AI: each one's request (its number, which its answer
    /// carries) and the flag that stops it.
    describing: HashMap<String, (u64, Arc<AtomicBool>)>,
    /// Markers "Describe all unnamed" has not sent yet, in the list's order: they show as waiting,
    /// and each goes out when a request comes back (see [`MAX_DESCRIBING_AT_ONCE`]).
    waiting: VecDeque<String>,
    /// The number the next request gets. Never reset, so the answer of a request stopped
    /// earlier never passes for a later one's.
    next_request: u64,
    /// A request failed and said so: the others of the run that fail too stay quiet.
    failed_said: bool,
    /// "Describe N unnamed" is asking whether to send `N` markers, with the price shown.
    confirming_describe_all: Option<usize>,
    /// A request found no key and said so: the others that find it out too stay quiet.
    no_key_said: bool,
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

    /// The heights the rows reported, by index (a row not reported yet is missing).
    pub fn row_heights(&self) -> &[f32] {
        &self.row_heights
    }

    /// Row `index` was laid out `height` tall.
    pub fn set_row_height(&mut self, index: usize, height: f32) {
        if self.row_heights.len() <= index {
            self.row_heights.resize(index + 1, MARKER_ROW_HEIGHT);
        }
        self.row_heights[index] = height;
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
        if self.is_describing(guid) {
            return None;
        }
        self.next_request += 1;
        self.no_key_said = false;
        self.failed_said = false;
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

    /// A request found no key: whether this is the first since the last request was sent, so
    /// that it is said once. Takes the turn: the next call gets false.
    pub fn take_first_no_key(&mut self) -> bool {
        !std::mem::replace(&mut self.no_key_said, true)
    }

    /// Whether the missing key was said since the last request was sent.
    #[cfg(test)]
    pub fn no_key_said(&self) -> bool {
        self.no_key_said
    }

    /// A request failed: whether that was said already (since the last request was sent), so
    /// several that fail in one run give one notice.
    pub fn failed_said(&mut self) -> bool {
        std::mem::replace(&mut self.failed_said, true)
    }

    /// Stop the request for `guid`, or take it out of the queue; its answer is not used.
    pub fn stop_describing(&mut self, guid: &str) {
        self.waiting.retain(|waiting| waiting != guid);
        if let Some((_, cancel)) = self.describing.remove(guid) {
            cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Stop every request (a batch job closes the clip).
    pub fn stop_all_describing(&mut self) {
        self.waiting.clear();
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

    /// Whether `guid` is being described or waits for its turn.
    pub fn is_describing(&self, guid: &str) -> bool {
        self.describing.contains_key(guid) || self.is_waiting(guid)
    }

    /// Whether `guid` waits for its turn: asked for, not sent yet.
    pub fn is_waiting(&self, guid: &str) -> bool {
        self.waiting.iter().any(|waiting| waiting == guid)
    }

    /// Whether any marker is being described or waits (the spinner turns).
    pub fn any_describing(&self) -> bool {
        !self.describing.is_empty() || !self.waiting.is_empty()
    }

    /// How many markers wait for their turn.
    pub fn waiting_count(&self) -> usize {
        self.waiting.len()
    }

    /// How many requests are on their way.
    pub fn in_flight(&self) -> usize {
        self.describing.len()
    }

    /// Put `guids` in the queue, leaving out the ones already described or waiting.
    pub fn queue_describing(&mut self, guids: impl IntoIterator<Item = String>) {
        for guid in guids {
            if !self.is_describing(&guid) {
                self.waiting.push_back(guid);
            }
        }
    }

    /// Start the request of the first marker in the queue that `wanted` still says is to be
    /// described (one named or removed meanwhile is dropped), unless `alive` requests are on
    /// their way already: the number really alive, stopped ones that have not come back
    /// included, since they still read the clip. Pops and starts together.
    pub fn start_next_waiting(
        &mut self,
        alive: usize,
        wanted: impl Fn(&str) -> bool,
    ) -> Option<(String, u64, Arc<AtomicBool>)> {
        if alive >= MAX_DESCRIBING_AT_ONCE {
            return None;
        }
        while let Some(guid) = self.waiting.pop_front() {
            if !wanted(&guid) {
                continue;
            }
            if let Some((request, cancel)) = self.start_describing(&guid) {
                return Some((guid, request, cancel));
            }
        }
        None
    }

    /// The editable markers of `markers` without a name that are not on their way or waiting,
    /// in the list's order: what "Describe N unnamed" sends and counts.
    pub fn unnamed_guids(&self, markers: &[Marker]) -> Vec<String> {
        markers
            .iter()
            .filter(|m| m.is_unnamed())
            .filter_map(|m| m.guid.clone())
            .filter(|guid| !self.is_describing(guid))
            .collect()
    }

    /// The number of markers "Describe N unnamed" is asking about, while it asks.
    pub fn describe_all_asked(&self) -> Option<usize> {
        self.confirming_describe_all
    }

    pub fn set_describe_all_asked(&mut self, asked: Option<usize>) {
        self.confirming_describe_all = asked;
    }

    /// Nothing waits any more (the requests on their way go on).
    pub fn clear_waiting(&mut self) {
        self.waiting.clear();
    }

    /// Start over for another file. Its requests stop: their answers would not find the markers.
    pub fn reset(&mut self) {
        self.stop_all_describing();
        let next_request = self.next_request;
        let row_heights = std::mem::take(&mut self.row_heights);
        *self = Self {
            next_request,
            row_heights,
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
    fn markers_wait_in_order_and_a_stop_takes_one_out_of_the_queue() {
        let mut state = MarkersState::default();
        state.queue_describing(["a", "b", "c"].map(String::from));
        assert!(state.is_waiting("b") && state.is_describing("b"));
        assert_eq!((state.waiting_count(), state.in_flight()), (3, 0));
        assert!(
            state.start_describing("b").is_none(),
            "waiting counts as on its way"
        );
        state.queue_describing(["a", "d"].map(String::from));
        assert_eq!(state.waiting_count(), 4, "a is queued once");
        state.stop_describing("b");
        let (first, _, cancel) = state.start_next_waiting(0, |_| true).unwrap();
        assert_eq!(first, "a");
        assert!(state.is_describing("a") && !state.is_waiting("a"));
        assert_eq!(state.in_flight(), 1);
        state.stop_all_describing();
        assert!(cancel.load(Ordering::Relaxed));
        assert!(!state.any_describing() && state.start_next_waiting(0, |_| true).is_none());
        state.queue_describing(["x".to_string()]);
        state.reset();
        assert!(
            !state.any_describing(),
            "leaving the clip empties the queue"
        );
    }

    #[test]
    fn the_queue_starts_one_at_a_time_up_to_the_cap_and_skips_the_unwanted() {
        let mut state = MarkersState::default();
        state.queue_describing(["a", "b", "c", "d"].map(String::from));
        let named = |guid: &str| guid != "a";
        let (guid, ..) = state.start_next_waiting(0, named).unwrap();
        assert_eq!(guid, "b", "a was named meanwhile: dropped");
        assert!(!state.is_describing("a"));
        assert!(
            state
                .start_next_waiting(MAX_DESCRIBING_AT_ONCE, |_| true)
                .is_none(),
            "no slot: alive requests count, not the ones in the map"
        );
        assert_eq!(state.waiting_count(), 2);
        assert!(state
            .start_next_waiting(MAX_DESCRIBING_AT_ONCE - 1, |_| true)
            .is_some());
    }

    #[test]
    fn unnamed_markers_are_the_editable_ones_without_a_name_not_on_their_way() {
        let mut named = Marker::new(1_000);
        named.name = "Lion".into();
        let mut spaces = Marker::new(2_000);
        spaces.name = "  ".into();
        let mut read_only = Marker::new(3_000);
        read_only.guid = None;
        let busy = Marker::new(4_000);
        let mut state = MarkersState::default();
        state.start_describing(busy.guid.as_deref().unwrap());
        let markers = [named, spaces.clone(), read_only, busy, Marker::new(5_000)];
        let guids = state.unnamed_guids(&markers);
        assert_eq!(guids.len(), 2);
        assert_eq!(guids[0].as_str(), spaces.guid.as_deref().unwrap());
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

    #[test]
    fn reported_row_heights_are_kept_by_index_and_survive_another_file() {
        let mut state = MarkersState::default();
        state.set_row_height(2, 90.0);
        assert_eq!(
            state.row_heights(),
            [MARKER_ROW_HEIGHT, MARKER_ROW_HEIGHT, 90.0]
        );
        state.set_row_height(0, 40.0);
        state.reset();
        assert_eq!(state.row_heights()[0], 40.0);
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
