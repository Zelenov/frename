//! The search over the side list (#170): what the editor typed, and what it does to the list.
//! The text is matched like the file list's search (`frename_core::search_words`), against the
//! cues of the subtitle tab and the names, comments and times of the marker tab.

use std::ops::Range;

use frename_core::{search_hits, search_words, SubtitleCue};

/// Widget id of the bar's text input, for focus.
pub const SIDE_SEARCH_INPUT_ID: &str = "side-list-search-input";

/// What the search bar and its keys ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchMessage {
    /// `Ctrl+F` or the header's button: show the bar and focus it.
    Open,
    /// The text changed.
    Input(String),
    /// The bar's ✕: empty the text.
    Clear,
    /// `Enter` in the bar: the next match, the previous one with `Shift`.
    Submit,
    /// The arrows after the field: `true` is the next match.
    Step(bool),
    /// The header's button while the bar is open.
    Close,
}

/// The bar's state. The text is kept when the tab changes, so both lists can be checked with it.
#[derive(Debug, Clone, Default)]
pub struct SideSearch {
    open: bool,
    query: String,
    words: Vec<String>,
    /// The match the last step went to (its index in the whole list), so that matches that
    /// start at the same time can be stepped through one by one.
    current: Option<usize>,
}

impl SideSearch {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the bar; its text goes with it, since a hidden bar must not hide rows.
    pub fn close(&mut self) {
        self.open = false;
        self.set_query(String::new());
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_query(&mut self, query: String) {
        self.current = None;
        self.words = search_words(&query);
        self.query = query;
    }

    /// The match the last step went to.
    pub fn current(&self) -> Option<usize> {
        self.current
    }

    pub fn set_current(&mut self, current: Option<usize>) {
        self.current = current;
    }

    /// The words of the text, lowercased; none when it is empty.
    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// Whether the lists show only what matches: the bar is open and has words.
    pub fn is_filtering(&self) -> bool {
        self.open && !self.words.is_empty()
    }

    /// The words to mark in `text`: nothing while the lists are not filtered.
    pub fn marks(&self, text: &str) -> Vec<Range<usize>> {
        if self.is_filtering() {
            search_hits(text, &self.words)
        } else {
            Vec::new()
        }
    }
}

/// How far the playhead may be from the start of the match a step went to and still count as
/// being on it: after a seek the first frame shown has its own timestamp (up to a few frames off,
/// more while the clip plays on), so an exact comparison would take the same match again.
const HERE_SLACK_MS: u64 = 250;

/// The cue to go to from `position_ms` among the `matches` (indices of `cues`): the first one
/// that starts after it, the last one that starts before it going back; round the ends.
pub fn step_cue(
    cues: &[SubtitleCue],
    matches: &[usize],
    position_ms: u64,
    forward: bool,
    current: Option<usize>,
) -> Option<usize> {
    let starts: Vec<(usize, u64)> = matches
        .iter()
        .map(|&i| {
            (
                i,
                u64::try_from(cues[i].start.as_millis()).unwrap_or(u64::MAX),
            )
        })
        .collect();
    step(&starts, position_ms, forward, current)
}

/// Like [`step_cue`], over `(index, start in ms)` pairs in time order.
///
/// With the `current` match still under the playhead the step goes to its neighbour in the list,
/// so matches that start together are reached one after the other.
pub fn step(
    starts: &[(usize, u64)],
    position_ms: u64,
    forward: bool,
    current: Option<usize>,
) -> Option<usize> {
    let here = current.and_then(|c| {
        starts
            .iter()
            .position(|&(index, start)| index == c && start.abs_diff(position_ms) <= HERE_SLACK_MS)
    });
    if let Some(at) = here {
        let len = starts.len();
        let next = if forward {
            (at + 1) % len
        } else {
            (at + len - 1) % len
        };
        return Some(starts[next].0);
    }
    if forward {
        starts
            .iter()
            .find(|(_, start)| *start > position_ms)
            .or(starts.first())
    } else {
        starts
            .iter()
            .rfind(|(_, start)| *start < position_ms)
            .or(starts.last())
    }
    .map(|&(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepping_goes_to_the_next_match_and_wraps_round_the_ends() {
        let starts = [(1, 1_000), (4, 4_000), (7, 7_000)];
        assert_eq!(step(&starts, 0, true, None), Some(1));
        assert_eq!(step(&starts, 1_000, true, None), Some(4));
        assert_eq!(
            step(&starts, 7_000, true, None),
            Some(1),
            "after the last: the first"
        );
        assert_eq!(step(&starts, 4_000, false, None), Some(1));
        assert_eq!(
            step(&starts, 1_000, false, None),
            Some(7),
            "before the first: the last"
        );
        assert_eq!(step(&[], 0, true, None), None);
    }

    #[test]
    fn matches_that_start_together_are_stepped_one_by_one() {
        let starts = [(2, 5_000), (3, 5_000), (6, 9_000)];
        assert_eq!(step(&starts, 4_000, true, None), Some(2));
        assert_eq!(step(&starts, 5_000, true, Some(2)), Some(3));
        assert_eq!(step(&starts, 5_000, true, Some(3)), Some(6));
        assert_eq!(step(&starts, 5_000, false, Some(3)), Some(2));
        assert_eq!(
            step(&starts, 5_000, false, Some(2)),
            Some(6),
            "round the end"
        );
        // The playhead moved away from the last step: by time again.
        assert_eq!(step(&starts, 6_000, true, Some(2)), Some(6));
    }

    #[test]
    fn a_playhead_a_few_frames_off_the_match_is_still_on_it() {
        let starts = [(1, 1_000), (4, 5_000), (7, 9_000)];
        // After a seek to 5 000 the frame is stamped 5 005 or 4 967.
        assert_eq!(step(&starts, 5_005, false, Some(4)), Some(1));
        assert_eq!(step(&starts, 4_967, false, Some(4)), Some(1));
        assert_eq!(step(&starts, 5_005, true, Some(4)), Some(7));
        assert_eq!(step(&starts, 4_967, true, Some(4)), Some(7));
        // Far away: by time.
        assert_eq!(step(&starts, 5_400, false, Some(4)), Some(4));
    }

    #[test]
    fn cues_are_stepped_from_a_playhead_off_their_start() {
        use std::time::Duration;
        let cue = |ms: u64| SubtitleCue {
            start: Duration::from_millis(ms),
            end: Duration::from_millis(ms + 500),
            text: String::new(),
        };
        let cues = [cue(1_000), cue(5_000), cue(9_000)];
        let matches = [0, 1, 2];
        assert_eq!(step_cue(&cues, &matches, 5_007, false, Some(1)), Some(0));
        assert_eq!(step_cue(&cues, &matches, 4_960, true, Some(1)), Some(2));
    }

    #[test]
    fn a_closed_bar_filters_and_marks_nothing() {
        let mut search = SideSearch::default();
        search.set_query("goat".into());
        assert!(!search.is_filtering());
        assert!(search.marks("a goat").is_empty());
        search.open();
        assert_eq!(search.marks("a Goat"), vec![2..6]);
        search.close();
        assert_eq!(search.query(), "");
    }
}
