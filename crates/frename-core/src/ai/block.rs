//! The AI block: the part of a comment written by an AI run, kept apart from the editor's text.
//!
//! ```text
//! Shaky walk into the market; the guide talks about spices.
//!
//! AI: A guide leads two tourists through a spice market and they taste saffron.
//! 0:00–0:14 Walking through the market entrance.
//! 0:14–0:41 The guide explains spice prices.
//! — Claude Opus 5, 2026-09-26 —
//! ```
//!
//! The block starts with a line beginning `AI: ` at the start of the comment or after a blank
//! line, and ends with the first following line of the form `— <model>, <YYYY-MM-DD> —` (`--`
//! also accepted, so hand-typed edits survive). An `AI: ` line without such an end line is the
//! editor's text and is never removed. The summary is the block's first line, so a clip without
//! text of the editor's shows the summary wherever only a comment's first line is shown.

use std::time::Duration;

/// The block's opening: the first line starts with it.
const START: &str = "AI: ";

/// One time range of a summary, in seconds from the start of the clip.
#[derive(Debug, Clone, PartialEq)]
pub struct AiSegment {
    pub start: Duration,
    pub end: Duration,
    pub description: String,
}

/// What an AI run says about a clip.
#[derive(Debug, Clone, PartialEq)]
pub struct AiSummary {
    pub summary: String,
    pub segments: Vec<AiSegment>,
}

/// Byte range of the AI block inside a comment: its first line to the end of its end line.
fn find_block(comment: &str) -> Option<(usize, usize)> {
    let mut offset = 0;
    let mut previous_blank = true;
    let mut start = None;
    for line in comment.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        match start {
            None if previous_blank && content.starts_with(START) => start = Some(offset),
            Some(begin) if is_end_line(content) => return Some((begin, offset + line.len())),
            // A blank line inside the block: the block was never closed there, so a later
            // `AI: ` line may still start the real one.
            Some(_) if content.trim().is_empty() => start = None,
            _ => {}
        }
        previous_blank = content.trim().is_empty();
        offset += line.len();
    }
    None
}

/// `— Claude Opus 5, 2026-09-26 —` or the same with `--`.
fn is_end_line(line: &str) -> bool {
    let line = line.trim();
    let inner = ["—", "--"].iter().find_map(|dash| {
        line.strip_prefix(dash)
            .and_then(|rest| rest.strip_suffix(dash))
    });
    let Some(inner) = inner else {
        return false;
    };
    let Some((model, date)) = inner.rsplit_once(',') else {
        return false;
    };
    !model.trim().is_empty() && is_date(date.trim())
}

/// `YYYY-MM-DD`.
fn is_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(i, b)| match i {
            4 | 7 => *b == b'-',
            _ => b.is_ascii_digit(),
        })
}

/// Whether the comment holds an AI block.
pub fn has_ai_block(comment: &str) -> bool {
    find_block(comment).is_some()
}

/// The comment without its AI block: the editor's text before it and any text after it (hand
/// edits), joined by a blank line. Trailing whitespace is trimmed.
pub fn editor_comment(comment: &str) -> String {
    let Some((start, end)) = find_block(comment) else {
        return comment.trim_end().to_string();
    };
    let before = comment[..start].trim_end();
    let after = comment[end..].trim();
    match (before.is_empty(), after.is_empty()) {
        (_, true) => before.to_string(),
        (true, false) => after.to_string(),
        (false, false) => format!("{before}\n\n{after}"),
    }
}

/// Whether the comment has text of the editor's, not only an AI block. This is what
/// "commented" means everywhere (the commented tag and filter). Does not allocate: it runs for
/// every file of the list while it is drawn.
pub fn has_editor_comment(comment: &str) -> bool {
    match find_block(comment) {
        None => !comment.trim().is_empty(),
        Some((start, end)) => {
            !comment[..start].trim().is_empty() || !comment[end..].trim().is_empty()
        }
    }
}

/// The comment with `block` as its AI block: the old block (if any) is removed, the new one is
/// appended after the editor's text with one blank line between them. An empty `block` only
/// removes the old one.
pub fn replace_ai_block(comment: &str, block: &str) -> String {
    let editor = editor_comment(comment);
    match (editor.is_empty(), block.is_empty()) {
        (_, true) => editor,
        (true, false) => block.to_string(),
        (false, false) => format!("{editor}\n\n{block}"),
    }
}

/// The comment's AI block as text, `None` without one.
pub fn ai_block(comment: &str) -> Option<&str> {
    find_block(comment).map(|(start, end)| comment[start..end].trim_end())
}

/// Format `summary` as an AI block signed with `model` and `date` (`YYYY-MM-DD`). Line breaks
/// and runs of whitespace inside the model's text become single spaces, so the summary stays on
/// the first line and no model text can form an end line.
pub fn format_ai_block(summary: &AiSummary, model: &str, date: &str) -> String {
    let mut block = format!("{START}{}", one_line(&summary.summary));
    for segment in &summary.segments {
        block.push_str(&format!(
            "\n{}–{} {}",
            format_time(segment.start),
            format_time(segment.end),
            one_line(&segment.description)
        ));
    }
    block.push_str(&format!("\n— {model}, {date} —"));
    block
}

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `m:ss` below an hour, `h:mm:ss` from an hour.
pub fn format_time(time: Duration) -> String {
    let total = time.as_secs();
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// Today's date in UTC as `YYYY-MM-DD`, for the block's end line.
pub fn today() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 86_400);
    civil_date(days)
}

/// The date `days` after 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn civil_date(days: u64) -> String {
    let z = days as i64 + 719_468;
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

    const BLOCK: &str =
        "AI: A cooking lesson.\n0:00–0:14 Chopping beets.\n— Claude Opus 5, 2026-09-26 —";

    fn summary() -> AiSummary {
        AiSummary {
            summary: "A guide\nshows   the market.".to_string(),
            segments: vec![AiSegment {
                start: Duration::from_secs(14),
                end: Duration::from_secs(3_725),
                description: "Tasting\r\n— Claude, 2026-01-01 —".to_string(),
            }],
        }
    }

    #[test]
    fn formats_summary_first_with_segments_and_an_end_line() {
        let block = format_ai_block(&summary(), "Claude Opus 5", "2026-09-26");
        assert_eq!(
            block,
            "AI: A guide shows the market.\n\
             0:14–1:02:05 Tasting — Claude, 2026-01-01 —\n\
             — Claude Opus 5, 2026-09-26 —"
        );
        assert_eq!(ai_block(&block), Some(block.as_str()));
        assert_eq!(editor_comment(&block), "");
    }

    #[test]
    fn replacing_keeps_the_editors_text_byte_for_byte() {
        let editor = "Shaky walk — Café 🎥\n  indented line";
        let once = replace_ai_block(editor, BLOCK);
        assert_eq!(once, format!("{editor}\n\n{BLOCK}"));
        let new_block = format_ai_block(&summary(), "Claude Haiku 4.5", "2026-09-27");
        let twice = replace_ai_block(&once, &new_block);
        assert_eq!(twice, format!("{editor}\n\n{new_block}"));
        assert_eq!(editor_comment(&twice), editor);
    }

    #[test]
    fn an_empty_comment_gets_only_the_block_and_removing_it_leaves_nothing() {
        assert_eq!(replace_ai_block("", BLOCK), BLOCK);
        assert_eq!(replace_ai_block(BLOCK, ""), "");
    }

    #[test]
    fn an_unterminated_block_is_the_editors_text() {
        let comment = "AI: my own note\nstill mine";
        assert!(!has_ai_block(comment));
        assert_eq!(
            replace_ai_block(comment, BLOCK),
            format!("{comment}\n\n{BLOCK}")
        );
    }

    #[test]
    fn an_ai_line_inside_a_paragraph_does_not_start_a_block() {
        let comment = "Note\nAI: not a block\n— x, 2026-01-01 —";
        assert!(!has_ai_block(comment));
    }

    #[test]
    fn hand_typed_double_dashes_close_a_block() {
        let comment = "Mine\n\nAI: Summary.\n-- Claude Opus 5, 2026-09-26 --";
        assert!(has_ai_block(comment));
        assert_eq!(editor_comment(comment), "Mine");
    }

    #[test]
    fn text_after_a_block_is_kept_and_moves_before_the_new_one() {
        let comment = format!("Before\n\n{BLOCK}\n\nAfter\r\n");
        assert_eq!(editor_comment(&comment), "Before\n\nAfter");
        assert_eq!(
            replace_ai_block(&comment, BLOCK),
            format!("Before\n\nAfter\n\n{BLOCK}")
        );
    }

    #[test]
    fn crlf_comments_are_parsed() {
        let comment = "Mine\r\n\r\nAI: Summary.\r\n— Claude Opus 5, 2026-09-26 —\r\n";
        assert_eq!(editor_comment(comment), "Mine");
    }

    #[test]
    fn only_editor_text_counts_as_commented() {
        assert!(!has_editor_comment(""));
        assert!(!has_editor_comment(BLOCK));
        assert!(!has_editor_comment(&format!("  \n\n{BLOCK}\n ")));
        assert!(has_editor_comment(&format!("Mine\n\n{BLOCK}")));
        assert!(has_editor_comment(&format!("{BLOCK}\n\nafter")));
        assert!(has_editor_comment("AI: unterminated"));
    }

    #[test]
    fn times_switch_to_hours_from_an_hour() {
        assert_eq!(format_time(Duration::from_secs(59)), "0:59");
        assert_eq!(format_time(Duration::from_secs(600)), "10:00");
        assert_eq!(format_time(Duration::from_secs(3_600)), "1:00:00");
    }

    #[test]
    fn civil_dates() {
        assert_eq!(civil_date(0), "1970-01-01");
        assert_eq!(civil_date(11_016), "2000-02-29");
        assert_eq!(civil_date(20_722), "2026-09-26");
        assert_eq!(today().len(), 10);
    }
}
