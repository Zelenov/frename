//! The text styles (`docs/design/design-system.md` §4.2). No other text style exists.

use std::borrow::Cow;

use iced::widget::text::{IntoFragment, LineHeight, Wrapping};
use iced::widget::{rich_text, span, text, Text};
use iced::{Color, Element, Font, Pixels};

use super::tokens::*;

fn styled<'a>(
    content: impl IntoFragment<'a>,
    size: f32,
    line: f32,
    font: Font,
    color: Color,
) -> Text<'a> {
    text(content)
        .size(size)
        .line_height(LineHeight::Absolute(Pixels(line)))
        .font(font)
        .color(color)
}

/// The label of a control that sets its own text color by state (a button): no color of its own.
pub fn label<'a>(content: impl IntoFragment<'a>, font: Font) -> Text<'a> {
    text(content)
        .size(TEXT_BODY)
        .line_height(LineHeight::Absolute(Pixels(LINE_BODY)))
        .font(font)
}

/// A page title; a panel's title; the title of a whole-window empty state.
pub fn heading<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_HEADING, LINE_HEADING, font_strong(), TEXT)
}

/// A section title; a dialog's title; the title of an empty pane.
pub fn title<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_TITLE, LINE_TITLE, font_strong(), TEXT)
}

/// Everything else: labels, values, lines of text.
pub fn body<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_BODY, LINE_BODY, font(), TEXT)
}

/// The value that matters in a line.
pub fn strong<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_BODY, LINE_BODY, font_strong(), TEXT)
}

/// Help, descriptions, meta.
pub fn secondary<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_BODY, LINE_BODY, font(), TEXT_SECONDARY)
}

/// What went wrong, next to where it did.
pub fn error<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_BODY, LINE_BODY, font(), ERROR)
}

/// File names, paths, timecodes, examples of names; in `TEXT_SECONDARY` (a file name in a list
/// takes `.color(TEXT)`).
pub fn mono<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_MONO, LINE_BODY, FONT_MONO, TEXT_SECONDARY)
}

/// Badges, counters, key caps, the second line of a dense row.
pub fn caption<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_CAPTION, LINE_CAPTION, font(), TEXT_SECONDARY)
}

/// A caption that is a fact to notice: a badge.
pub fn caption_strong<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(
        content,
        TEXT_CAPTION,
        LINE_CAPTION,
        font_strong(),
        TEXT_SECONDARY,
    )
}

/// The label of a mini chip in a file list row.
pub fn chip_mini<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_CHIP_MINI, LINE_CHIP_MINI, font(), TAG_TEXT)
}

/// The cue under the windowed video, `scale` times its medium size.
pub fn subtitle<'a>(content: impl IntoFragment<'a>, scale: f32) -> Text<'a> {
    styled(
        content,
        TEXT_SUBTITLE * scale,
        LINE_SUBTITLE * scale,
        font(),
        TEXT,
    )
}

/// The caption over fullscreen video, `scale` times its medium size.
pub fn video_caption<'a>(content: impl IntoFragment<'a>, scale: f32) -> Text<'a> {
    styled(
        content,
        TEXT_VIDEO_CAPTION * scale,
        LINE_VIDEO_CAPTION * scale,
        font(),
        TEXT,
    )
}

/// The text of a tooltip. It breaks between words, and inside a word longer than the line (a file
/// name has no spaces): otherwise it runs past the tooltip's box, which stops at its widest.
pub fn tooltip<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_TOOLTIP, LINE_TOOLTIP, font(), TEXT).wrapping(Wrapping::WordOrGlyph)
}

/// `line` cut with "…" at the end to at most `width` px, at `char_width` px a character: exact for
/// `mono` (`MONO_CHAR_WIDTH`), an estimate otherwise. iced cannot cut text itself; a clipping
/// container behind it keeps an estimate that is short from drawing past its background.
pub fn fit(line: &str, width: f32, char_width: f32) -> Cow<'_, str> {
    let room = (width / char_width).floor().max(0.0) as usize;
    if line.chars().count() <= room {
        return Cow::Borrowed(line);
    }
    let kept: String = line.chars().take(room.saturating_sub(1)).collect();
    Cow::Owned(format!("{}…", kept.trim_end()))
}

/// How a run of a [`body_runs`] line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Run {
    Plain,
    Strong,
    Mono,
}

/// A line of body text that wraps, made of runs in the body, strong and mono styles: release notes.
pub fn body_runs<'a, M: 'a>(runs: impl IntoIterator<Item = (Run, String)>) -> Element<'a, M> {
    let spans: Vec<iced::widget::text::Span<'a, (), Font>> = runs
        .into_iter()
        .map(|(run, content)| match run {
            Run::Plain => span(content),
            Run::Strong => span(content).font(font_strong()),
            Run::Mono => span(content).font(FONT_MONO).size(TEXT_MONO),
        })
        .collect();
    rich_text(spans)
        .size(TEXT_BODY)
        .line_height(LineHeight::Absolute(Pixels(LINE_BODY)))
        .font(font())
        .color(TEXT)
        .into()
}

/// A caption `line` with the byte `ranges` of it marked (a search's hits): the marked parts in the
/// accent color on its tint. Cut with "…" to `width` like [`fit`]; a mark that runs past the cut
/// is cut with it. One line, like every comment line of the list.
pub fn caption_marked<'a, M: 'a>(
    line: &str,
    ranges: &[std::ops::Range<usize>],
    width: f32,
    char_width: f32,
) -> Element<'a, M> {
    let room = (width / char_width).floor().max(0.0) as usize;
    let too_long = line.chars().count() > room;
    let end = if too_long {
        line.char_indices()
            .nth(room.saturating_sub(1))
            .map_or(line.len(), |(i, _)| i)
    } else {
        line.len()
    };
    let shown = &line[..end];
    let mut spans: Vec<iced::widget::text::Span<'a, (), Font>> = Vec::new();
    let mut at = 0;
    for range in ranges {
        let (start, stop) = (range.start.min(end), range.end.min(end));
        if start >= stop {
            continue;
        }
        if start > at {
            spans.push(span(shown[at..start].to_string()));
        }
        spans.push(
            span(shown[start..stop].to_string())
                .color(ACCENT_TEXT)
                .background(ACCENT_TINT),
        );
        at = stop;
    }
    if at < shown.len() {
        spans.push(span(shown[at..].trim_end().to_string()));
    }
    if too_long {
        spans.push(span("…"));
    }
    rich_text(spans)
        .size(TEXT_CAPTION)
        .line_height(LineHeight::Absolute(Pixels(LINE_CAPTION)))
        .font(font())
        .color(TEXT_SECONDARY)
        .wrapping(Wrapping::None)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_longer_than_its_room_ends_in_an_ellipsis() {
        assert_eq!(fit("short", 100.0, 10.0), "short");
        let cut = fit("a very long comment line", 100.0, 10.0);
        assert!(cut.ends_with('…'));
        assert!(cut.chars().count() <= 10, "{cut}");
    }
}
