//! Clip markers: moments of a video with a color, a name and a comment, kept in the video's
//! XMP where Premiere Pro shows them on the clip after import (see `metadata::markers_xmp`).
//!
//! Also the one-line text form of a marker that the batch action "Markers ⇄ comment" writes
//! into comments and reads back: `<time>[–<time>] — <name>[ — <comment>]`.

use std::sync::OnceLock;

use regex::Regex;

/// How close two markers are to count as the same moment: `F2` opens the marker under the
/// playhead instead of adding one this close to it, and the batch skips a duplicate this close.
pub const MARKER_SNAP_MS: u64 = 500;

/// One clip marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    /// Premiere's `xmpDM:guid`. `None` for markers other tools wrote without one: frename shows
    /// those but never changes them, since it could not find them again in the file.
    pub guid: Option<String>,
    pub start_ms: u64,
    /// 0 for a point marker.
    pub duration_ms: u64,
    /// One line.
    pub name: String,
    /// Line breaks as `\n`.
    pub comment: String,
    pub color: MarkerColor,
}

impl Marker {
    /// A new point marker at `start_ms` with a fresh GUID, green like Premiere's default.
    pub fn new(start_ms: u64) -> Self {
        Self {
            guid: Some(uuid::Uuid::new_v4().to_string()),
            start_ms,
            duration_ms: 0,
            name: String::new(),
            comment: String::new(),
            color: MarkerColor::Green,
        }
    }

    /// Whether frename may change or delete it.
    pub fn is_editable(&self) -> bool {
        self.guid.is_some()
    }

    pub fn end_ms(&self) -> u64 {
        self.start_ms.saturating_add(self.duration_ms)
    }

    /// Whether `guid` is this marker's GUID.
    pub fn has_guid(&self, guid: &str) -> bool {
        self.guid.as_deref() == Some(guid)
    }
}

/// Sort markers by time, the order every list shows them in.
pub fn sort_markers(markers: &mut [Marker]) {
    markers.sort_by(|a, b| {
        a.start_ms
            .cmp(&b.start_ms)
            .then_with(|| a.duration_ms.cmp(&b.duration_ms))
    });
}

/// A marker color as Premiere stores it: a `keywordExtDVAv1_…` cue point parameter holding
/// `{"color":<0xAABBGGRR>}`, or no parameter for the default green.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkerColor {
    Green,
    Red,
    Orange,
    Yellow,
    White,
    Blue,
    Cyan,
    Lavender,
    Magenta,
    /// A value frename does not know, kept as it is until the user picks a color.
    Other(u32),
}

impl MarkerColor {
    /// The colors the picker offers, in Premiere's order.
    pub const ALL: [MarkerColor; 9] = [
        MarkerColor::Green,
        MarkerColor::Red,
        MarkerColor::Orange,
        MarkerColor::Yellow,
        MarkerColor::White,
        MarkerColor::Blue,
        MarkerColor::Cyan,
        MarkerColor::Lavender,
        MarkerColor::Magenta,
    ];

    /// The stored value; `None` for green, which Premiere writes as no value at all.
    pub fn value(self) -> Option<u32> {
        match self {
            Self::Green => None,
            Self::Red => Some(4_281_740_498),
            Self::Orange => Some(4_280_578_025),
            Self::Yellow => Some(4_281_049_552),
            Self::White => Some(4_294_967_295),
            Self::Blue => Some(4_294_741_314),
            Self::Cyan => Some(4_292_277_273),
            Self::Lavender => Some(4_289_825_711),
            Self::Magenta => Some(4_294_902_015),
            Self::Other(value) => Some(value),
        }
    }

    /// The color a stored value stands for.
    pub fn from_value(value: Option<u32>) -> Self {
        let Some(value) = value else {
            return Self::Green;
        };
        Self::ALL
            .into_iter()
            .find(|c| c.value() == Some(value))
            .unwrap_or(Self::Other(value))
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Green => "Green",
            Self::Red => "Red",
            Self::Orange => "Orange",
            Self::Yellow => "Yellow",
            Self::White => "White",
            Self::Blue => "Blue",
            Self::Cyan => "Cyan",
            Self::Lavender => "Lavender",
            Self::Magenta => "Magenta",
            Self::Other(_) => "Other",
        }
    }
}

// ---------------------------------------------------------------------------
// Times and the one-line text form
// ---------------------------------------------------------------------------

/// `m:ss` below an hour, `h:mm:ss` from an hour, with `.mmm` when the milliseconds are not zero.
pub fn format_marker_time(ms: u64) -> String {
    let total = ms / 1000;
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    let whole = if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    };
    match ms % 1000 {
        0 => whole,
        millis => format!("{whole}.{millis:03}"),
    }
}

/// The marker as one comment line: `<time>[–<time>] — <name>[ — <comment>]`. Line breaks of
/// the marker's comment become spaces.
pub fn format_marker_line(marker: &Marker) -> String {
    let mut line = format_marker_time(marker.start_ms);
    if marker.duration_ms > 0 {
        line.push('–');
        line.push_str(&format_marker_time(marker.end_ms()));
    }
    line.push_str(" — ");
    line.push_str(marker.name.trim());
    let comment = marker
        .comment
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if !comment.is_empty() {
        line.push_str(" — ");
        line.push_str(&comment);
    }
    line.trim_end().to_string()
}

/// A comment line that starts with a time: the marker it stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerLine {
    pub start_ms: u64,
    pub duration_ms: u64,
    pub name: String,
    pub comment: String,
}

fn line_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // A time is `h:mm:ss`, `m:ss` or `mm:ss` with optional `.mmm`, or the old screenshot
        // form `HH-MM-SS-mmm`. A range is two times joined by a dash with no spaces. The time
        // must be followed by a separator, so `12:30 call the client back` is not a marker.
        let time = r"\d{2}-\d{2}-\d{2}-\d{3}|(?:\d{1,2}:)?\d{1,2}:\d{2}(?:\.\d{1,3})?";
        Regex::new(&format!(
            r"^\s*(?P<start>{time})(?:[–-](?P<end>{time}))?\s*(?:—|–|--|-|:)(?P<rest>.*)$"
        ))
        .expect("marker line regex")
    })
}

/// Milliseconds of one time as [`line_re`] matches it; `None` when a field is out of range.
fn parse_time(text: &str) -> Option<u64> {
    let fields: Vec<&str> = text.split('-').collect();
    if let [h, m, s, ms] = fields.as_slice() {
        let (h, m, s, ms): (u64, u64, u64, u64) = (
            h.parse().ok()?,
            m.parse().ok()?,
            s.parse().ok()?,
            ms.parse().ok()?,
        );
        return (m < 60 && s < 60).then_some(((h * 60 + m) * 60 + s) * 1000 + ms);
    }
    let (clock, millis) = match text.split_once('.') {
        Some((clock, millis)) => {
            // `.5` is half a second: pad to three digits.
            let padded = format!("{millis:0<3}");
            (clock, padded.parse::<u64>().ok()?)
        }
        None => (text, 0),
    };
    let parts: Vec<u64> = clock
        .split(':')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    let (h, m, s) = match parts.as_slice() {
        [m, s] => (0, *m, *s),
        [h, m, s] if *m < 60 => (*h, *m, *s),
        _ => return None,
    };
    (s < 60).then_some(((h * 60 + m) * 60 + s) * 1000 + millis)
}

/// Read a comment line written by [`format_marker_line`] or by hand (`03:24 — shaky`).
/// `None` for a line that does not start with a time and a separator.
///
/// The rest of the line is split on the first ` — ` or ` -- `: the name before it, the comment
/// after it. A plain ` - ` does not split, since names often contain one.
pub fn parse_marker_line(line: &str) -> Option<MarkerLine> {
    let caps = line_re().captures(line)?;
    let start_ms = parse_time(&caps["start"])?;
    let duration_ms = match caps.name("end") {
        Some(end) => parse_time(end.as_str())?.checked_sub(start_ms)?,
        None => 0,
    };
    let rest = caps["rest"].trim();
    let (name, comment) =
        if let Some(comment) = rest.strip_prefix("— ").or_else(|| rest.strip_prefix("-- ")) {
            ("", comment)
        } else {
            [" — ", " -- "]
                .iter()
                .filter_map(|sep| rest.find(sep).map(|at| (at, sep.len())))
                .min()
                .map_or((rest, ""), |(at, len)| (&rest[..at], &rest[at + len..]))
        };
    Some(MarkerLine {
        start_ms,
        duration_ms,
        name: name.trim().to_string(),
        comment: comment.trim().to_string(),
    })
}

/// What "comment → markers" does to one file: the markers to add and the comment left over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentToMarkers {
    /// New markers, not yet in `existing`.
    pub added: Vec<Marker>,
    /// The comment without the lines that became markers.
    pub comment: String,
    /// Lines that became markers, duplicates of existing markers included.
    pub lines_moved: usize,
    /// Lines left in the comment because their time is past the end of the clip.
    pub past_end: usize,
}

/// Turn the timecoded lines of `comment` into markers. A line whose marker already exists
/// (same name within [`MARKER_SNAP_MS`]) adds nothing but still leaves the comment. With the
/// clip length known, a line past its end stays in the comment.
pub fn comment_to_markers(
    comment: &str,
    existing: &[Marker],
    clip_duration_ms: Option<u64>,
) -> CommentToMarkers {
    let mut added: Vec<Marker> = Vec::new();
    let mut kept: Vec<&str> = Vec::new();
    let mut lines_moved = 0;
    let mut past_end = 0;
    for line in comment.lines() {
        let Some(parsed) = parse_marker_line(line) else {
            kept.push(line);
            continue;
        };
        if clip_duration_ms.is_some_and(|clip| parsed.start_ms > clip) {
            past_end += 1;
            kept.push(line);
            continue;
        }
        lines_moved += 1;
        let duplicate = existing.iter().chain(added.iter()).any(|m| {
            m.name.trim() == parsed.name && m.start_ms.abs_diff(parsed.start_ms) <= MARKER_SNAP_MS
        });
        if duplicate {
            continue;
        }
        let mut marker = Marker::new(parsed.start_ms);
        marker.duration_ms = parsed.duration_ms;
        marker.name = parsed.name;
        marker.comment = parsed.comment;
        added.push(marker);
    }
    CommentToMarkers {
        added,
        comment: kept.join("\n").trim().to_string(),
        lines_moved,
        past_end,
    }
}

/// "Markers → comment": `comment` with a line per marker appended in time order, leaving out
/// lines already in it. Returns the new comment and how many lines were added.
pub fn markers_to_comment(comment: &str, markers: &[Marker]) -> (String, usize) {
    let mut sorted = markers.to_vec();
    sort_markers(&mut sorted);
    let present: Vec<&str> = comment.lines().map(str::trim).collect();
    let new_lines: Vec<String> = sorted
        .iter()
        .map(format_marker_line)
        .filter(|line| !present.contains(&line.as_str()))
        .fold(Vec::new(), |mut lines, line| {
            if !lines.contains(&line) {
                lines.push(line);
            }
            lines
        });
    if new_lines.is_empty() {
        return (comment.to_string(), 0);
    }
    let added = new_lines.len();
    let mut result = comment.trim_end().to_string();
    for line in new_lines {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&line);
    }
    (result, added)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(start_ms: u64, duration_ms: u64, name: &str, comment: &str) -> MarkerLine {
        MarkerLine {
            start_ms,
            duration_ms,
            name: name.to_string(),
            comment: comment.to_string(),
        }
    }

    fn marker(start_ms: u64, duration_ms: u64, name: &str, comment: &str) -> Marker {
        let mut m = Marker::new(start_ms);
        m.duration_ms = duration_ms;
        m.name = name.to_string();
        m.comment = comment.to_string();
        m
    }

    #[test]
    fn times_are_short_below_an_hour_and_show_milliseconds_only_when_there_are_some() {
        assert_eq!(format_marker_time(0), "0:00");
        assert_eq!(format_marker_time(204_000), "3:24");
        assert_eq!(format_marker_time(204_250), "3:24.250");
        assert_eq!(format_marker_time(3_600_000), "1:00:00");
        assert_eq!(format_marker_time(36_061_005), "10:01:01.005");
    }

    #[test]
    fn every_accepted_time_form_parses() {
        assert_eq!(
            parse_marker_line("03:24 — x"),
            Some(line(204_000, 0, "x", ""))
        );
        assert_eq!(
            parse_marker_line("3:24 — x"),
            Some(line(204_000, 0, "x", ""))
        );
        assert_eq!(
            parse_marker_line("1:02:03 — x"),
            Some(line(3_723_000, 0, "x", ""))
        );
        assert_eq!(
            parse_marker_line("0:01.5 - x"),
            Some(line(1_500, 0, "x", ""))
        );
        assert_eq!(
            parse_marker_line("0:01.250: x"),
            Some(line(1_250, 0, "x", ""))
        );
        assert_eq!(
            parse_marker_line("  0:05 – x"),
            Some(line(5_000, 0, "x", ""))
        );
        assert_eq!(
            parse_marker_line("00-00-02-440: old screenshot"),
            Some(line(2_440, 0, "old screenshot", ""))
        );
    }

    #[test]
    fn a_time_needs_a_separator_after_it() {
        assert_eq!(parse_marker_line("12:30 call the client back"), None);
        assert_eq!(parse_marker_line("no time here"), None);
        assert_eq!(parse_marker_line("see 0:12 — later"), None);
        assert_eq!(parse_marker_line("0:75 — bad seconds"), None);
    }

    #[test]
    fn an_unspaced_dash_makes_a_range_and_a_spaced_one_a_separator() {
        assert_eq!(
            parse_marker_line("0:41-0:47 — Lion"),
            Some(line(41_000, 6_000, "Lion", ""))
        );
        assert_eq!(
            parse_marker_line("0:41–0:47 — Lion"),
            Some(line(41_000, 6_000, "Lion", ""))
        );
        assert_eq!(
            parse_marker_line("0:41 - 0:47 is the best part"),
            Some(line(41_000, 0, "0:47 is the best part", ""))
        );
    }

    #[test]
    fn name_and_comment_split_on_the_first_long_dash_or_double_hyphen() {
        assert_eq!(
            parse_marker_line("3:24 — shaky - fix — stabilize — later"),
            Some(line(204_000, 0, "shaky - fix", "stabilize — later"))
        );
        assert_eq!(
            parse_marker_line("3:24 - Take 3 -- nice light"),
            Some(line(204_000, 0, "Take 3", "nice light"))
        );
        assert_eq!(
            parse_marker_line("3:24 — — only a comment"),
            Some(line(204_000, 0, "", "only a comment"))
        );
    }

    #[test]
    fn a_marker_line_reads_back_as_the_same_marker() {
        for m in [
            marker(204_000, 0, "Take 3", ""),
            marker(41_000, 6_000, "Lion", "roars twice"),
            marker(3_723_456, 1_000, "", "no name"),
            marker(1_500, 0, "", ""),
        ] {
            let parsed = parse_marker_line(&format_marker_line(&m)).expect("parses");
            assert_eq!(
                parsed,
                line(m.start_ms, m.duration_ms, &m.name, &m.comment),
                "{}",
                format_marker_line(&m)
            );
        }
        let multi = marker(0, 0, "Name", "line one\nline two");
        assert_eq!(
            format_marker_line(&multi),
            "0:00 — Name — line one line two"
        );
    }

    #[test]
    fn comment_lines_become_markers_and_leave_the_comment() {
        let comment = "Good take.\n0:12 — Take 3 — nice light\n12:30 call back\n0:41-0:47 — Lion";
        let result = comment_to_markers(comment, &[], None);
        assert_eq!(result.comment, "Good take.\n12:30 call back");
        assert_eq!(result.lines_moved, 2);
        let got: Vec<_> = result
            .added
            .iter()
            .map(|m| {
                (
                    m.start_ms,
                    m.duration_ms,
                    m.name.as_str(),
                    m.comment.as_str(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                (12_000, 0, "Take 3", "nice light"),
                (41_000, 6_000, "Lion", "")
            ]
        );
        assert!(result.added.iter().all(|m| m.guid.is_some()));
    }

    #[test]
    fn a_line_past_the_end_of_the_clip_stays_and_an_unknown_length_accepts_all() {
        let comment = "0:01 — in\n9:00 — out";
        let known = comment_to_markers(comment, &[], Some(60_000));
        assert_eq!(known.added.len(), 1);
        assert_eq!(known.past_end, 1);
        assert_eq!(known.comment, "9:00 — out");
        assert_eq!(comment_to_markers(comment, &[], None).added.len(), 2);
    }

    #[test]
    fn an_existing_marker_is_not_added_twice_but_its_line_goes() {
        let existing = [marker(12_300, 0, "Take 3", "")];
        let result = comment_to_markers("0:12 — Take 3", &existing, None);
        assert!(result.added.is_empty());
        assert_eq!(result.lines_moved, 1);
        assert_eq!(result.comment, "");
    }

    #[test]
    fn markers_to_comment_appends_in_time_order_and_reruns_add_nothing() {
        let markers = [
            marker(41_000, 6_000, "Lion", ""),
            marker(12_000, 0, "Take 3", "a\nb"),
        ];
        let (comment, added) = markers_to_comment("Good take.", &markers);
        assert_eq!(comment, "Good take.\n0:12 — Take 3 — a b\n0:41–0:47 — Lion");
        assert_eq!(added, 2);
        assert_eq!(markers_to_comment(&comment, &markers), (comment.clone(), 0));
    }

    #[test]
    fn markers_to_comment_to_markers_round_trips_and_is_stable() {
        let markers = [
            marker(41_000, 6_000, "Lion", "one line"),
            marker(12_500, 0, "Take", ""),
        ];
        let (comment, _) = markers_to_comment("", &markers);
        let back = comment_to_markers(&comment, &[], None);
        assert_eq!(back.comment, "");
        let mut got: Vec<_> = back
            .added
            .iter()
            .map(|m| (m.start_ms, m.duration_ms, m.name.clone(), m.comment.clone()))
            .collect();
        got.sort();
        assert_eq!(
            got,
            [
                (12_500, 0, "Take".to_string(), String::new()),
                (41_000, 6_000, "Lion".to_string(), "one line".to_string())
            ]
        );
        // Comment → markers → comment gives the same lines back.
        let (again, _) = markers_to_comment("", &back.added);
        assert_eq!(again, comment);
    }

    #[test]
    fn colors_round_trip_through_their_stored_value() {
        for color in MarkerColor::ALL {
            assert_eq!(MarkerColor::from_value(color.value()), color);
        }
        assert_eq!(MarkerColor::from_value(Some(7)), MarkerColor::Other(7));
        assert_eq!(MarkerColor::Green.value(), None);
    }
}
