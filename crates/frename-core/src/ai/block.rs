//! The AI block of a comment: the description an AI run writes, kept apart from the editor's
//! own text so a re-run replaces it and never touches what the editor typed.
//!
//! ```text
//! Shaky walk into the market; the guide talks about spices.
//!
//! AI: A guide leads two tourists through a spice market.
//! 0:00–0:14 Walking through the market entrance, crowd, handheld.
//! 0:14–0:41 Close-ups of spice sacks; the guide explains prices.
//! ```
//!
//! A block starts with a line beginning `AI: ` at the start of the comment or after a blank
//! line, and runs to the end of the comment. When several lines qualify, the last one starts
//! it, so everything above it stays the editor's. Blocks written before the end line was
//! dropped end with a line `— <model>, <YYYY-MM-DD> —` (or with `--`); text after such a line
//! is the editor's too.
//!
//! When the clip has a lead-in or lead-out around the part worth keeping, the block ends with
//! the In and Out the AI suggests (`Suggested In/Out: 00:00:03.200 – 00:00:11.800`, the times of
//! the in/out line of the comment). It is a suggestion only: the in/out points change when the
//! editor applies it. The line starts with a word, so it is never a marker line or an in/out
//! line, and as the block's last line it stays last when markers kept in the comment are
//! written back under the summary.

use std::ops::Range;

use clipscribe::{format_time, Description, MainRange};

use crate::Segment;

/// What starts the block's first line, before the summary.
const START: &str = "AI: ";
/// What the suggested in/out line has before the in/out line it would be (`In/Out: …`).
const SUGGESTED: &str = "Suggested ";

/// Byte range of the comment's AI block, from the start of its last `AI: ` line to the end
/// of the comment (trailing white space left out), or to the end of an old block's end line.
/// `None` when the comment has none.
fn find(comment: &str) -> Option<Range<usize>> {
    let mut offset = 0;
    let mut previous_blank = true;
    let mut block: Option<Range<usize>> = None;
    for line in comment.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        if previous_blank && content.starts_with(START) {
            block = Some(offset..comment.len());
        } else if let Some(open) = block.as_mut().filter(|b| b.end == comment.len()) {
            if is_end_line(content) {
                open.end = offset + content.len();
            }
        }
        previous_blank = content.trim().is_empty();
        offset += line.len();
    }
    block.map(|b| b.start..comment[..b.end].trim_end().len().max(b.start))
}

/// Whether `line` closes a block: `— <model>, <YYYY-MM-DD> —`, or with `--` for the dashes.
fn is_end_line(line: &str) -> bool {
    let line = line.trim();
    let inner = ["—", "--"].iter().find_map(|dash| {
        line.strip_prefix(dash)
            .and_then(|rest| rest.strip_suffix(dash))
    });
    let Some(inner) = inner.map(str::trim) else {
        return false;
    };
    let Some((model, date)) = inner.rsplit_once(", ") else {
        return false;
    };
    !model.trim().is_empty() && is_date(date.trim())
}

/// `YYYY-MM-DD`.
fn is_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(i, b)| match i {
            4 | 7 => *b == b'-',
            _ => b.is_ascii_digit(),
        })
}

/// The comment's AI block, if it has one.
pub fn ai_block(comment: &str) -> Option<&str> {
    find(comment).map(|range| &comment[range])
}

/// Byte range of the comment's AI block (see [`ai_block`]).
pub fn ai_block_range(comment: &str) -> Option<Range<usize>> {
    find(comment)
}

/// Whether the comment holds text of the editor's, not just an AI block. Does not allocate:
/// the folder list asks this for every file.
pub fn has_editor_comment(comment: &str) -> bool {
    match find(comment) {
        Some(range) => {
            !comment[..range.start].trim().is_empty() || !comment[range.end..].trim().is_empty()
        }
        None => !comment.trim().is_empty(),
    }
}

/// `comment` with its AI block replaced by `block`, or with `block` added at the end after a
/// blank line when it has none. Everything outside the old block is kept byte for byte.
pub fn replace_block(comment: &str, block: &str) -> String {
    let block = block.trim();
    if let Some(range) = find(comment) {
        return format!(
            "{}{block}{}",
            &comment[..range.start],
            &comment[range.end..]
        );
    }
    if comment.trim().is_empty() {
        return block.to_string();
    }
    let gap = if comment.ends_with("\n\n") || comment.ends_with("\r\n\r\n") {
        ""
    } else if comment.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    format!("{comment}{gap}{block}")
}

/// Write `description` as a block: `AI: ` and the summary, then one line per segment. Line
/// breaks and runs of white space in the model's text become single spaces, so the summary
/// stays on the first line.
pub fn format_block(description: &Description) -> String {
    let mut block = format!("{START}{}", one_line(&description.summary));
    for segment in &description.segments {
        block.push_str(&format!(
            "\n{}–{} {}",
            format_time(segment.start_s),
            format_time(segment.end_s),
            one_line(&segment.description)
        ));
    }
    push_suggestion(&mut block, description.main);
    block
}

/// The block with the summary only, for when the moments go into the video as markers. The
/// suggested in/out line stays.
pub fn format_summary_block(description: &Description) -> String {
    let mut block = format!("{START}{}", one_line(&description.summary));
    push_suggestion(&mut block, description.main);
    block
}

/// Add the suggested in/out line for `main`, if there is one, as the block's last line.
fn push_suggestion(block: &mut String, main: Option<MainRange>) {
    let Some(main) = main else {
        return;
    };
    let segment = Segment {
        start: Some(main.start_s as f32),
        end: Some(main.end_s as f32),
    };
    block.push('\n');
    block.push_str(SUGGESTED);
    block.push_str(&crate::metadata::format_in_out_line(segment).unwrap_or_default());
}

/// The In and Out the comment's AI block suggests, as `[` and `]` would set them: the in
/// rounded down to a whole second, the out up, so the part worth keeping stays whole. `None`
/// when the block has no suggestion, or the comment no block.
pub fn suggested_in_out(comment: &str) -> Option<Segment> {
    let segment = ai_block(comment)?.lines().find_map(|line| {
        line.trim()
            .strip_prefix(SUGGESTED)
            .and_then(crate::metadata::parse_in_out_line)
    })?;
    Some(Segment {
        start: Some(segment.start?.floor()),
        end: Some(segment.end?.ceil()),
    })
}

/// The description's segments as markers hold them: whole milliseconds, one-line names.
pub fn segment_lines(description: &Description) -> Vec<crate::MarkerLine> {
    let ms = |seconds: f64| (seconds.max(0.0) * 1000.0).round() as u64;
    description
        .segments
        .iter()
        .map(|segment| {
            let start_ms = ms(segment.start_s);
            crate::MarkerLine {
                start_ms,
                duration_ms: ms(segment.end_s).saturating_sub(start_ms),
                name: one_line(&segment.description),
                comment: String::new(),
                color: crate::MarkerColor::Green,
            }
        })
        .collect()
}

/// `text` with every run of white space (line breaks included) turned into one space.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipscribe::Segment;

    const BLOCK: &str = "AI: A guide leads tourists.\n0:00–0:14 Entrance.";

    fn description() -> Description {
        Description {
            summary: "A guide\nleads   tourists.".to_string(),
            segments: vec![Segment {
                start_s: 0.0,
                end_s: 14.2,
                description: "Entrance.".to_string(),
            }],
            main: None,
        }
    }

    #[test]
    fn format_writes_the_summary_first_and_collapses_line_breaks() {
        assert_eq!(format_block(&description()), BLOCK);
    }

    #[test]
    fn the_segments_become_marker_lines_and_the_summary_a_block_of_its_own() {
        assert_eq!(
            format_summary_block(&description()),
            "AI: A guide leads tourists."
        );
        assert_eq!(
            segment_lines(&description()),
            [crate::MarkerLine {
                start_ms: 0,
                duration_ms: 14_200,
                name: "Entrance.".to_string(),
                comment: String::new(),
                color: crate::MarkerColor::Green,
            }]
        );
    }

    #[test]
    fn no_segments_leaves_just_the_summary_and_no_markers() {
        let no_segments = Description {
            summary: description().summary,
            segments: vec![],
            main: None,
        };
        assert_eq!(format_block(&no_segments), "AI: A guide leads tourists.");
        assert_eq!(
            format_summary_block(&no_segments),
            "AI: A guide leads tourists."
        );
        assert_eq!(segment_lines(&no_segments), []);
    }

    #[test]
    fn a_new_run_keeps_everything_above_the_block_byte_for_byte() {
        let editor = "Шаткий проход 🎥\n  indented line  \n\n";
        let comment = format!("{editor}{BLOCK}");
        assert_eq!(ai_block(&comment), Some(BLOCK));
        let new_block = BLOCK.replace("tourists", "visitors");
        assert_eq!(
            replace_block(&comment, &new_block),
            format!("{editor}{new_block}")
        );
    }

    #[test]
    fn a_comment_without_a_block_gets_one_after_a_blank_line() {
        assert_eq!(replace_block("", BLOCK), BLOCK);
        assert_eq!(replace_block("Mine", BLOCK), format!("Mine\n\n{BLOCK}"));
        assert_eq!(replace_block("Mine\n", BLOCK), format!("Mine\n\n{BLOCK}"));
        assert_eq!(replace_block("Mine\n\n", BLOCK), format!("Mine\n\n{BLOCK}"));
    }

    #[test]
    fn the_block_runs_to_the_end_of_the_comment() {
        let comment = format!("Mine\n\n{BLOCK}\n0:14–0:20 Added by hand.\n");
        assert_eq!(
            ai_block(&comment),
            Some(format!("{BLOCK}\n0:14–0:20 Added by hand.").as_str())
        );
        assert_eq!(replace_block(&comment, BLOCK), format!("Mine\n\n{BLOCK}\n"));
    }

    #[test]
    fn the_last_ai_paragraph_starts_the_block() {
        let editor = "AI: check the audio later\nsecond line\n\n";
        let comment = format!("{editor}{BLOCK}");
        assert_eq!(ai_block(&comment), Some(BLOCK));
        assert_eq!(
            replace_block(&comment, "AI: New"),
            format!("{editor}AI: New")
        );
        assert!(has_editor_comment(&comment));
    }

    #[test]
    fn an_ai_line_inside_a_paragraph_does_not_start_a_block() {
        let comment = "Note\nAI: not a block\n0:00–0:01 x";
        assert_eq!(ai_block(comment), None);
        assert_eq!(
            replace_block(comment, BLOCK),
            format!("{comment}\n\n{BLOCK}")
        );
    }

    #[test]
    fn an_old_block_ends_at_its_end_line_and_text_after_it_is_kept() {
        let old = "AI: Summary\n0:00–0:14 Entrance.\n— Claude Haiku 4.5, 2026-09-26 —";
        let comment = format!("Before\n\n{old}\n\nAfter");
        assert_eq!(ai_block(&comment), Some(old));
        assert_eq!(
            replace_block(&comment, BLOCK),
            format!("Before\n\n{BLOCK}\n\nAfter")
        );
        let dashes = "Mine\n\nAI: Summary\n-- Claude Haiku 4.5, 2026-09-26 --";
        assert_eq!(
            ai_block(dashes),
            Some("AI: Summary\n-- Claude Haiku 4.5, 2026-09-26 --")
        );
    }

    #[test]
    fn a_block_alone_is_not_an_editor_comment() {
        assert!(!has_editor_comment(BLOCK));
        assert!(!has_editor_comment(&format!("  \n\n{BLOCK}\n ")));
        assert!(has_editor_comment(&format!("x\n\n{BLOCK}")));
        assert!(!has_editor_comment(""));
        assert!(has_editor_comment("x"));
    }

    fn with_main(start_s: f64, end_s: f64) -> Description {
        Description {
            main: Some(MainRange { start_s, end_s }),
            ..description()
        }
    }

    const SUGGESTION: &str = "Suggested In/Out: 00:00:03.200 – 00:00:11.800";

    #[test]
    fn a_main_range_is_written_as_the_blocks_last_line() {
        let described = with_main(3.2, 11.8);
        assert_eq!(format_block(&described), format!("{BLOCK}\n{SUGGESTION}"));
        assert_eq!(
            format_summary_block(&described),
            format!("AI: A guide leads tourists.\n{SUGGESTION}")
        );
        assert_eq!(
            segment_lines(&described).len(),
            1,
            "the suggestion is no segment"
        );
    }

    #[test]
    fn the_suggestion_is_read_as_brackets_would_set_it() {
        let comment = format!("Mine\n\n{}", format_block(&with_main(3.2, 11.8)));
        assert_eq!(
            suggested_in_out(&comment),
            Some(crate::Segment {
                start: Some(3.0),
                end: Some(12.0),
            }),
            "in rounded down, out up: the part worth keeping stays whole"
        );
        assert_eq!(suggested_in_out(&format_block(&description())), None);
        assert_eq!(suggested_in_out(""), None);
        assert_eq!(
            suggested_in_out(&format!("{SUGGESTION}\n\nAI: Summary")),
            None,
            "only the AI block suggests"
        );
        assert_eq!(
            suggested_in_out(&format!("{BLOCK}\r\n{SUGGESTION}\r\n")),
            Some(crate::Segment {
                start: Some(3.0),
                end: Some(12.0),
            })
        );
    }

    #[test]
    fn the_suggestion_is_no_marker_and_no_in_out_point() {
        let comment = format!("Mine\n\n{}", format_block(&with_main(3.2, 11.8)));
        assert_eq!(crate::parse_marker_line(SUGGESTION), None);
        assert_eq!(crate::markers::parse_ai_line(SUGGESTION), None);
        let (kept, segment) = crate::metadata::split_in_out_line(&comment);
        assert_eq!(
            (kept.as_str(), segment),
            (comment.as_str(), crate::Segment::default())
        );
    }

    #[test]
    fn markers_kept_in_the_comment_leave_the_suggestion_last_and_in_place() {
        let comment = format!("Mine\n\n{}", format_block(&with_main(3.2, 11.8)));
        let (text, markers) = crate::markers_from_comment(&comment);
        assert_eq!(
            markers.len(),
            1,
            "the segment is a marker, the suggestion is not"
        );
        assert!(text.ends_with(SUGGESTION));
        assert_eq!(crate::markers_into_comment(&text, &markers), comment);
    }

    #[test]
    fn crlf_comments_are_parsed() {
        let comment = "Mine\r\n\r\nAI: S\r\n0:00–0:01 x\r\n";
        assert_eq!(ai_block(comment), Some("AI: S\r\n0:00–0:01 x"));
        assert_eq!(replace_block(comment, "AI: T"), "Mine\r\n\r\nAI: T\r\n");
    }
}
