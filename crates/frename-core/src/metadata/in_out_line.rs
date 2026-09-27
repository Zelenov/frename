//! In/out points kept in the comment (see [`super::InOutStorage::Comment`]): one line in the
//! comment, in a fixed form frename writes and reads back.
//!
//! ```text
//! Shaky start, good from the second take.
//! In/Out: 00:01:05.250 – 00:02:10.000
//!
//! AI: A guide leads two tourists through a spice market.
//! ```
//!
//! The line starts with `In/Out:`, never with a time, so it is not a marker line
//! (`0:41–0:47 — Lion`) and never becomes a marker. frename writes it as the last line of the
//! editor's part, after the editor's own text and above any AI block: the editor's note stays
//! first (Premiere's Description column shows the first line), and a new AI run replaces only
//! the block and leaves the line alone. It is read from anywhere in the editor's part, never
//! from the AI block. A missing in point is written `start` and a missing out point `end`, the
//! clip's own ends, as the Premiere subclip marker has them.
//!
//! While a file is loaded the line is taken out of its comment into the snapshot's in/out
//! points, so the comment box never shows it and only the in/out controls change it; saving
//! puts it back.

use std::sync::OnceLock;

use regex::Regex;

use super::Segment;

/// What starts the line.
const PREFIX: &str = "In/Out: ";
/// Between the in and the out point.
const DASH: &str = " – ";
/// Written for a missing in point.
const START: &str = "start";
/// Written for a missing out point.
const END: &str = "end";

fn line_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Always `h:mm:ss`, with optional milliseconds.
        let time = r"\d{1,2}:\d{2}:\d{2}(?:\.\d{1,3})?";
        Regex::new(&format!(
            r"(?i)^\s*in/out:\s*(?P<in>{time}|start)\s*(?:–|—|--|-)\s*(?P<out>{time}|end)\s*$"
        ))
        .expect("in/out line regex")
    })
}

/// `HH:MM:SS.mmm`, rounded to the millisecond.
fn format_time(seconds: f32) -> String {
    let ms = (f64::from(seconds.max(0.0)) * 1000.0).round() as u64;
    let total = ms / 1000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        total / 3600,
        total / 60 % 60,
        total % 60,
        ms % 1000
    )
}

/// The points as the line shows them, without its `In/Out: ` prefix:
/// `00:01:05.250 – 00:02:10.000`, with `start` / `end` for a missing point. Empty when neither
/// is set.
pub fn format_in_out_range(segment: Segment) -> String {
    if segment.is_empty() {
        return String::new();
    }
    let start = segment.start.map_or_else(|| START.to_string(), format_time);
    let end = segment.end.map_or_else(|| END.to_string(), format_time);
    format!("{start}{DASH}{end}")
}

/// The line for `segment`: `In/Out: 00:01:05.250 – 00:02:10.000`. `None` when it is empty.
pub(crate) fn format_in_out_line(segment: Segment) -> Option<String> {
    (!segment.is_empty()).then(|| format!("{PREFIX}{}", format_in_out_range(segment)))
}

/// The in and out point of an in/out line. `None` for any other line, and for
/// `In/Out: start – end`, which marks nothing.
pub(crate) fn parse_in_out_line(line: &str) -> Option<Segment> {
    let caps = line_re().captures(line)?;
    let point = |name: &str| -> Option<Option<f32>> {
        let text = &caps[name];
        if text.eq_ignore_ascii_case(START) || text.eq_ignore_ascii_case(END) {
            Some(None)
        } else {
            crate::markers::parse_time(text).map(|ms| Some((ms as f64 / 1000.0) as f32))
        }
    };
    let segment = Segment {
        start: point("in")?,
        end: point("out")?,
    };
    (!segment.is_empty()).then_some(segment)
}

/// `comment` without its in/out line, and the points the line holds (empty when it has
/// none). The line is looked for anywhere outside the AI block: the block is the AI's text.
/// When several lines qualify the first one counts, and all of them go.
pub(crate) fn split_in_out_line(comment: &str) -> (String, Segment) {
    let block = crate::ai::block::ai_block_range(comment);
    let mut segment: Option<Segment> = None;
    let mut kept = String::with_capacity(comment.len());
    let mut first_line_taken = false;
    let mut offset = 0;
    for raw in comment.split_inclusive('\n') {
        let in_block = block.as_ref().is_some_and(|b| b.contains(&offset));
        let at_start = offset == 0;
        offset += raw.len();
        let parsed = (!in_block)
            .then(|| parse_in_out_line(raw.trim_end_matches(['\n', '\r'])))
            .flatten();
        match parsed {
            Some(points) => {
                segment.get_or_insert(points);
                first_line_taken |= at_start;
            }
            None => kept.push_str(raw),
        }
    }
    let Some(segment) = segment else {
        return (comment.to_string(), Segment::default());
    };
    // With no text of the editor's, [`with_in_out_line`] puts the line first and a blank line
    // between it and the AI block, which the block needs to be one; without the line that gap
    // goes too.
    if first_line_taken {
        for gap in ["\r\n", "\n"] {
            if let Some(rest) = kept.strip_prefix(gap) {
                if crate::ai::block::ai_block_range(rest).is_some_and(|b| b.start == 0) {
                    kept = rest.to_string();
                }
                break;
            }
        }
    }
    let kept = kept.trim_end_matches(['\n', '\r']).to_string();
    (kept, segment)
}

/// `comment` (which holds no in/out line) with the line for `segment` added as the last line
/// of the editor's part, above the AI block if there is one; `comment` as it is when `segment`
/// is empty. A blank line keeps separating the AI block from the text above it.
pub(crate) fn with_in_out_line(comment: &str, segment: Segment) -> String {
    let Some(line) = format_in_out_line(segment) else {
        return comment.to_string();
    };
    let (editor, block) = match crate::ai::block::ai_block_range(comment) {
        Some(range) => comment.split_at(range.start),
        None => (comment, ""),
    };
    let editor = editor.trim_end();
    let mut result = if editor.is_empty() {
        line
    } else {
        format!("{editor}\n{line}")
    };
    if !block.is_empty() {
        result.push_str("\n\n");
        result.push_str(block);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(start: Option<f32>, end: Option<f32>) -> Segment {
        Segment { start, end }
    }

    fn line(start: Option<f32>, end: Option<f32>) -> Option<String> {
        format_in_out_line(segment(start, end))
    }

    #[test]
    fn the_line_has_fixed_width_times_and_words_for_open_ends() {
        assert_eq!(
            line(Some(65.25), Some(130.0)).as_deref(),
            Some("In/Out: 00:01:05.250 – 00:02:10.000")
        );
        assert_eq!(
            line(Some(3_723.5), None).as_deref(),
            Some("In/Out: 01:02:03.500 – end")
        );
        assert_eq!(
            line(None, Some(7.0)).as_deref(),
            Some("In/Out: start – 00:00:07.000")
        );
        assert_eq!(line(None, None), None);
        assert_eq!(
            format_in_out_range(segment(Some(1.0), None)),
            "00:00:01.000 – end"
        );
        assert_eq!(format_in_out_range(Segment::default()), "");
    }

    #[test]
    fn a_line_reads_back_as_the_same_points() {
        for (start, end) in [
            (Some(65.25), Some(130.0)),
            (Some(0.0), None),
            (None, Some(0.1)),
            (Some(36_061.005), Some(36_062.0)),
        ] {
            let text = line(start, end).expect("line");
            let back = parse_in_out_line(&text).expect("parses");
            let close = |a: Option<f32>, b: Option<f32>| match (a, b) {
                (Some(a), Some(b)) => (a - b).abs() < 0.001,
                (a, b) => a == b,
            };
            assert!(close(back.start, start) && close(back.end, end), "{text}");
        }
    }

    #[test]
    fn hand_typed_variants_parse_and_other_lines_do_not() {
        assert_eq!(
            parse_in_out_line("  in/out: 0:01:05 - 0:02:10.5 "),
            Some(segment(Some(65.0), Some(130.5)))
        );
        assert_eq!(
            parse_in_out_line("In/Out: START — 00:00:07"),
            Some(segment(None, Some(7.0)))
        );
        assert_eq!(parse_in_out_line("In/Out: start – end"), None);
        assert_eq!(parse_in_out_line("In/Out: 00:75:00 – end"), None);
        assert_eq!(
            parse_in_out_line("In/Out: 1:05 – end"),
            None,
            "h:mm:ss only"
        );
        assert_eq!(parse_in_out_line("0:41–0:47 — Lion"), None);
        assert_eq!(parse_in_out_line("Note: In/Out: 00:00:01 – end"), None);
    }

    #[test]
    fn the_line_is_no_marker_line() {
        let text = line(Some(1.0), Some(2.0)).expect("line");
        assert_eq!(crate::markers::parse_marker_line(&text), None);
        assert_eq!(crate::markers::parse_ai_line(&text), None);
    }

    #[test]
    fn the_line_ends_the_editors_part_and_comes_out_again_byte_for_byte() {
        let points = segment(Some(65.25), Some(130.0));
        let text = "In/Out: 00:01:05.250 – 00:02:10.000";
        for (comment, expected) in [
            ("", text.to_string()),
            ("Good take.", format!("Good take.\n{text}")),
            (
                "Good take.\nSecond line\n",
                format!("Good take.\nSecond line\n{text}"),
            ),
            (
                "Mine\n0:02 — Take 3\n\nAI: A walk.\n0:00–0:14 Street.",
                format!("Mine\n0:02 — Take 3\n{text}\n\nAI: A walk.\n0:00–0:14 Street."),
            ),
            (
                "AI: A walk.\n0:00–0:14 Street.",
                format!("{text}\n\nAI: A walk.\n0:00–0:14 Street."),
            ),
            ("Mine\r\nline two", format!("Mine\r\nline two\n{text}")),
        ] {
            let with = with_in_out_line(comment, points);
            assert_eq!(with, expected);
            let (back, got) = split_in_out_line(&with);
            assert_eq!(back, comment.trim_end_matches(['\n', '\r']), "{with:?}");
            assert_eq!(got, points);
        }
        assert_eq!(with_in_out_line("note", Segment::default()), "note");
    }

    #[test]
    fn the_line_is_found_anywhere_in_the_editors_part() {
        assert_eq!(
            split_in_out_line("In/Out: 00:00:01 – end\nMine"),
            ("Mine".to_string(), segment(Some(1.0), None))
        );
        assert_eq!(
            split_in_out_line("Mine\nIn/Out: 00:00:01 – end\nmore"),
            ("Mine\nmore".to_string(), segment(Some(1.0), None))
        );
    }

    #[test]
    fn the_ai_block_under_the_line_is_still_a_block() {
        let with = with_in_out_line("AI: A walk.\n0:00–0:14 Street.", segment(Some(1.0), None));
        assert_eq!(
            crate::ai::block::ai_block(&with),
            Some("AI: A walk.\n0:00–0:14 Street.")
        );
        // A new AI run replaces the block and keeps the line.
        let replaced = crate::ai::block::replace_block(&with, "AI: A run.");
        assert_eq!(replaced, "In/Out: 00:00:01.000 – end\n\nAI: A run.");
        assert_eq!(split_in_out_line(&replaced).0, "AI: A run.");

        let mine = with_in_out_line("Mine\n\nAI: A walk.", segment(Some(1.0), None));
        assert_eq!(crate::ai::block::ai_block(&mine), Some("AI: A walk."));
        assert!(crate::ai::block::has_editor_comment(&mine));
    }

    #[test]
    fn marker_lines_and_the_ai_block_survive_next_to_the_line() {
        let comment = "Mine\n0:02 — Take 3\n\nAI: A walk.\n0:00–0:14 Street.";
        let with = with_in_out_line(comment, segment(Some(1.0), Some(3.0)));
        let (text, markers) = crate::markers::markers_from_comment(&with);
        assert_eq!(
            text,
            "Mine\nIn/Out: 00:00:01.000 – 00:00:03.000\n\nAI: A walk."
        );
        assert_eq!(markers.len(), 2, "the editor's marker and the AI segment");
        let back = crate::markers::markers_into_comment(&text, &markers);
        assert_eq!(split_in_out_line(&back), split_in_out_line(&with));
    }

    #[test]
    fn a_line_inside_the_ai_block_is_the_ais_text() {
        let comment = "Mine\n\nAI: A walk.\nIn/Out: 00:00:01 – end";
        assert_eq!(
            split_in_out_line(comment),
            (comment.to_string(), Segment::default())
        );
    }

    #[test]
    fn the_first_of_several_lines_counts_and_all_go() {
        let comment = "In/Out: 00:00:01 – end\nMine\nIn/Out: 00:00:05 – end";
        assert_eq!(
            split_in_out_line(comment),
            ("Mine".to_string(), segment(Some(1.0), None))
        );
    }
}
