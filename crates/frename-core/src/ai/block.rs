//! The AI block of a comment: the description an AI run writes, kept apart from the editor's
//! own text so a re-run replaces it and never touches what the editor typed.
//!
//! ```text
//! Shaky walk into the market; the guide talks about spices.
//!
//! AI: A guide leads two tourists through a spice market.
//! 0:00–0:14 Walking through the market entrance, crowd, handheld.
//! 0:14–0:41 Close-ups of spice sacks; the guide explains prices.
//! — Claude Haiku 4.5, 2026-09-26 —
//! ```
//!
//! A block starts with a line beginning `AI: ` at the start of the comment or after a blank
//! line, and ends with the first following line `— <model>, <YYYY-MM-DD> —` (`--` also
//! accepted, so hand-typed edits survive). An `AI: ` line with no end line is the editor's
//! text. A comment holds at most one block; text found after it is the editor's too.

use std::ops::Range;

use super::describe::Description;

/// What starts the block's first line, before the summary.
const START: &str = "AI: ";

/// Byte range of the comment's AI block, from the start of its `AI: ` line to the end of its
/// end line (without the line break after it). `None` when the comment has none.
fn find(comment: &str) -> Option<Range<usize>> {
    let mut offset = 0;
    let mut previous_blank = true;
    let mut start = None;
    for line in comment.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        match start {
            None if previous_blank && content.starts_with(START) => start = Some(offset),
            Some(start) if is_end_line(content) => {
                return Some(start..offset + content.len());
            }
            _ => {}
        }
        previous_blank = content.trim().is_empty();
        offset += line.len();
    }
    None
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

/// The text before the block and the text after it, each with its edges trimmed. Both empty
/// when the comment is only a block.
fn editor_parts(comment: &str) -> (&str, &str) {
    match find(comment) {
        Some(range) => (
            comment[..range.start].trim_end(),
            comment[range.end..].trim(),
        ),
        None => (comment.trim_end(), ""),
    }
}

/// The comment without its AI block: the editor's text before the block and any text after
/// it, joined by a blank line.
pub fn editor_comment(comment: &str) -> String {
    match editor_parts(comment) {
        (before, "") => before.to_string(),
        ("", after) => after.to_string(),
        (before, after) => format!("{before}\n\n{after}"),
    }
}

/// Whether the comment holds text of the editor's, not just an AI block. Does not allocate:
/// the folder list asks this for every file.
pub fn has_editor_comment(comment: &str) -> bool {
    let (before, after) = editor_parts(comment);
    !before.trim().is_empty() || !after.is_empty()
}

/// The whole comment: the editor's text, a blank line, then the block. Either may be empty;
/// without a block the editor's text is returned as it is.
pub fn join_comment(editor: &str, block: &str) -> String {
    let block = block.trim();
    if block.is_empty() {
        return editor.to_string();
    }
    let editor = editor.trim_end();
    if editor.is_empty() {
        block.to_string()
    } else {
        format!("{editor}\n\n{block}")
    }
}

/// `comment` with its AI block (if any) replaced by `block`, or removed when `block` is empty.
/// Text found after the old block moves before the new one.
pub fn replace_block(comment: &str, block: &str) -> String {
    join_comment(&editor_comment(comment), block)
}

/// The block's summary (its first line without `AI: `).
pub fn block_summary(block: &str) -> &str {
    let first = block.lines().next().unwrap_or_default();
    first.strip_prefix(START).unwrap_or(first).trim()
}

/// The block's segment lines, between its summary and its end line.
pub fn block_segments(block: &str) -> Vec<&str> {
    let lines: Vec<&str> = block.lines().collect();
    match lines.len() {
        0..=2 => Vec::new(),
        n => lines[1..n - 1].to_vec(),
    }
}

/// Write `description` as a block, signed with the model's name and the day (`YYYY-MM-DD`).
/// Line breaks and runs of white space in the model's text become single spaces, so the
/// summary stays on the first line and no model text can end the block early.
pub fn format_block(description: &Description, model_label: &str, date: &str) -> String {
    let mut block = format!("{START}{}", one_line(&description.summary));
    for segment in &description.segments {
        block.push_str(&format!(
            "\n{}–{} {}",
            format_time(segment.start_s),
            format_time(segment.end_s),
            one_line(&segment.description)
        ));
    }
    block.push_str(&format!("\n— {model_label}, {date} —"));
    block
}

/// `text` with every run of white space (line breaks included) turned into one space.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `m:ss` below an hour, `h:mm:ss` from an hour on. Seconds are rounded down.
pub fn format_time(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    let (h, m, s) = (total / 3600, total / 60 % 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// Today's date (UTC) as `YYYY-MM-DD`, for the block's end line.
pub fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    date_from_unix_days((secs / 86_400) as i64)
}

/// The civil date of a day counted from 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn date_from_unix_days(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::describe::Segment;

    const BLOCK: &str =
        "AI: A guide leads tourists.\n0:00–0:14 Entrance.\n— Claude Haiku 4.5, 2026-09-26 —";

    fn description() -> Description {
        Description {
            summary: "A guide\nleads   tourists.".to_string(),
            segments: vec![Segment {
                start_s: 0.0,
                end_s: 14.2,
                description: "Entrance.".to_string(),
            }],
        }
    }

    #[test]
    fn format_writes_the_summary_first_and_collapses_line_breaks() {
        let block = format_block(&description(), "Claude Haiku 4.5", "2026-09-26");
        assert_eq!(block, BLOCK);
        assert_eq!(block_summary(&block), "A guide leads tourists.");
        assert_eq!(block_segments(&block), vec!["0:00–0:14 Entrance."]);
    }

    #[test]
    fn model_text_that_looks_like_an_end_line_cannot_end_the_block() {
        let mut d = description();
        d.segments[0].description = "x\n— Evil, 2026-01-01 —\nmore".to_string();
        let block = format_block(&d, "Claude Haiku 4.5", "2026-09-26");
        let comment = join_comment("Mine", &block);
        assert_eq!(ai_block(&comment), Some(block.as_str()));
    }

    #[test]
    fn replace_keeps_the_editors_text_byte_for_byte() {
        let editor = "Шаткий проход 🎥\n  indented line";
        let comment = join_comment(editor, BLOCK);
        assert_eq!(editor_comment(&comment), editor);
        let new_block = BLOCK.replace("tourists", "visitors");
        let replaced = replace_block(&comment, &new_block);
        assert_eq!(replaced, format!("{editor}\n\n{new_block}"));
        assert_eq!(replaced.matches("AI: ").count(), 1);
    }

    #[test]
    fn a_comment_without_a_block_gets_one_appended_and_an_empty_one_is_just_the_block() {
        assert_eq!(replace_block("", BLOCK), BLOCK);
        assert_eq!(replace_block("Mine", BLOCK), format!("Mine\n\n{BLOCK}"));
        assert_eq!(replace_block(&format!("Mine\n\n{BLOCK}"), ""), "Mine");
    }

    #[test]
    fn an_unterminated_block_is_the_editors_text() {
        let comment = "AI: my own note\nno end line";
        assert_eq!(ai_block(comment), None);
        assert_eq!(editor_comment(comment), comment);
        assert!(has_editor_comment(comment));
        assert_eq!(
            replace_block(comment, BLOCK),
            format!("{comment}\n\n{BLOCK}")
        );
    }

    #[test]
    fn an_ai_line_inside_a_paragraph_does_not_start_a_block() {
        let comment = "Note\nAI: not a block\n0:00–0:01 x\n— M, 2026-01-01 —";
        assert_eq!(ai_block(comment), None);
    }

    #[test]
    fn hand_typed_double_dashes_end_a_block() {
        let comment = "Mine\n\nAI: Summary\n-- Claude Haiku 4.5, 2026-09-26 --";
        assert_eq!(block_summary(ai_block(comment).expect("block")), "Summary");
        assert_eq!(editor_comment(comment), "Mine");
    }

    #[test]
    fn text_after_a_block_is_kept_and_moves_before_it() {
        let comment = format!("Before\n\n{BLOCK}\n\nAfter");
        assert_eq!(editor_comment(&comment), "Before\n\nAfter");
        assert_eq!(
            replace_block(&comment, BLOCK),
            format!("Before\n\nAfter\n\n{BLOCK}")
        );
        let only_after = format!("{BLOCK}\nAfter");
        assert_eq!(editor_comment(&only_after), "After");
    }

    #[test]
    fn a_block_alone_is_not_an_editor_comment() {
        assert!(!has_editor_comment(BLOCK));
        assert!(!has_editor_comment(&format!("  \n\n{BLOCK}\n ")));
        assert!(has_editor_comment(&format!("x\n\n{BLOCK}")));
        assert!(!has_editor_comment(""));
        assert!(has_editor_comment("x"));
    }

    #[test]
    fn crlf_comments_are_parsed() {
        let comment = "Mine\r\n\r\nAI: S\r\n— M, 2026-09-26 —\r\n";
        assert_eq!(ai_block(comment), Some("AI: S\r\n— M, 2026-09-26 —"));
        assert_eq!(editor_comment(comment), "Mine");
    }

    #[test]
    fn times_are_minutes_below_an_hour_and_hours_above() {
        assert_eq!(format_time(0.0), "0:00");
        assert_eq!(format_time(62.9), "1:02");
        assert_eq!(format_time(3599.0), "59:59");
        assert_eq!(format_time(3600.0), "1:00:00");
        assert_eq!(format_time(3725.0), "1:02:05");
    }

    #[test]
    fn dates_are_civil_dates() {
        assert_eq!(date_from_unix_days(0), "1970-01-01");
        assert_eq!(date_from_unix_days(20_722), "2026-09-26");
        assert_eq!(date_from_unix_days(11_016), "2000-02-29");
    }
}
