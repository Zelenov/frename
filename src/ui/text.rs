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

/// `line` cut with "…" at the end to at most `width` px, at `char_width` px a character: for
/// `mono` text (`MONO_CHAR_WIDTH`), where it is exact. Inter text uses [`fit_body`] or
/// [`fit_caption`], which measure the font. iced cannot cut text itself; a clipping
/// container behind it keeps an estimate that is short from drawing past its background.
pub fn fit(line: &str, width: f32, char_width: f32) -> Cow<'_, str> {
    let room = (width / char_width).floor().max(0.0) as usize;
    if line.chars().count() <= room {
        return Cow::Borrowed(line);
    }
    let kept: String = line.chars().take(room.saturating_sub(1)).collect();
    Cow::Owned(format!("{}…", kept.trim_end()))
}

/// The bundled Inter Regular, parsed once.
fn inter() -> Option<&'static ttf_parser::Face<'static>> {
    use std::sync::OnceLock;
    static INTER: OnceLock<Option<ttf_parser::Face<'static>>> = OnceLock::new();
    INTER
        .get_or_init(|| ttf_parser::Face::parse(FONT_FILES[0], 0).ok())
        .as_ref()
}

/// How wide `c` is in Inter at `size` px; a character Inter does not have (CJK, Hangul, kana)
/// counts as one em, as the CJK fonts draw it.
fn char_width(face: Option<&ttf_parser::Face<'_>>, c: char, size: f32) -> f32 {
    face.and_then(|face| {
        let advance = face.glyph_hor_advance(face.glyph_index(c)?)?;
        Some(f32::from(advance) * size / f32::from(face.units_per_em()))
    })
    .unwrap_or(size)
}

/// How wide `line` is in Inter at `size` px, from the font's own advances (see [`char_width`]).
pub fn text_width(line: &str, size: f32) -> f32 {
    let face = inter();
    line.chars().map(|c| char_width(face, c, size)).sum()
}

/// `line` in body text cut with "…" so it is at most `width` px wide, measured with
/// [`text_width`] (unlike [`fit`], which takes an average width a character and can run past
/// its room in Cyrillic or CJK, #391).
pub fn fit_body(line: &str, width: f32) -> Cow<'_, str> {
    fit_measured(line, width, TEXT_BODY)
}

/// `line` in caption text cut with "…" so it is at most `width` px wide, as [`fit_body`] (#399).
pub fn fit_caption(line: &str, width: f32) -> Cow<'_, str> {
    fit_measured(line, width, TEXT_CAPTION)
}

/// One pass that stops as soon as the line is known not to fit, so a long comment costs what
/// its row shows, not its whole length (it runs for every row on every view).
fn fit_measured(line: &str, width: f32, size: f32) -> Cow<'_, str> {
    let face = inter();
    let room = width - char_width(face, '…', size);
    let mut used = 0.0;
    // Where the text stops if it gets a "…": the last char that fits in `room`.
    let mut cut = None;
    for (at, c) in line.char_indices() {
        used += char_width(face, c, size);
        if cut.is_none() && used > room {
            cut = Some(at);
        }
        if used > width {
            let kept = &line[..cut.unwrap_or(at)];
            return Cow::Owned(format!("{}…", kept.trim_end()));
        }
    }
    Cow::Borrowed(line)
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

/// A body-size `content` that wraps, with the byte `ranges` of it marked (a search's hits) in the
/// accent color on its tint; the rest in `color`. Like [`caption_marked`], for a list's rows.
pub fn body_marked<'a, M: 'a>(
    content: &str,
    ranges: &[std::ops::Range<usize>],
    color: Color,
) -> Element<'a, M> {
    let mut spans: Vec<iced::widget::text::Span<'a, (), Font>> = Vec::new();
    let mut at = 0;
    for range in ranges {
        let (start, stop) = (range.start.min(content.len()), range.end.min(content.len()));
        if start >= stop || start < at {
            continue;
        }
        if start > at {
            spans.push(span(content[at..start].to_string()));
        }
        spans.push(
            span(content[start..stop].to_string())
                .color(ACCENT_TEXT)
                .background(ACCENT_TINT),
        );
        at = stop;
    }
    if at < content.len() {
        spans.push(span(content[at..].to_string()));
    }
    rich_text(spans)
        .size(TEXT_BODY)
        .line_height(LineHeight::Absolute(Pixels(LINE_BODY)))
        .font(font())
        .color(color)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Issue #391: a measured cut is never wider than its room, in Latin, Cyrillic or CJK, and
    /// a line that fits is left whole.
    #[test]
    fn a_measured_cut_stays_within_its_room_in_every_script() {
        for line in [
            "Reordering below reorders the folder",
            "Порядок нижче — для всієї теки",
            "下での並べ替えはフォルダー全体に適用されます",
            "아래에서 순서를 바꾸면 폴더 전체의 순서가 바뀝니다",
        ] {
            assert_eq!(
                fit_body(line, 10_000.0),
                line,
                "a line that fits stays whole"
            );
            for room in [60.0, 120.0, 190.0] {
                let cut = fit_body(line, room);
                assert!(cut.ends_with('…'), "{cut}");
                assert!(
                    text_width(&cut, TEXT_BODY) <= room,
                    "{cut}: {} > {room}",
                    text_width(&cut, TEXT_BODY)
                );
            }
        }
        // CJK takes about an em a character, Latin about half.
        assert!(text_width("下での", TEXT_BODY) > 2.5 * TEXT_BODY);
        assert!(text_width("abc", TEXT_BODY) < 2.0 * TEXT_BODY);
    }

    /// Issue #399: a measured caption cut is never wider than its room either.
    #[test]
    fn a_measured_caption_cut_stays_within_its_room() {
        for line in [
            "Съёмка на набережной, закат",
            "夕方の海岸で撮影した素材",
            "Wide shot at dusk, harbour",
        ] {
            assert_eq!(fit_caption(line, 10_000.0), line);
            for room in [40.0, 90.0] {
                let cut = fit_caption(line, room);
                assert!(cut.ends_with('…'), "{cut}");
                assert!(text_width(&cut, TEXT_CAPTION) <= room, "{cut}");
            }
        }
        assert!(text_width("abc", TEXT_CAPTION) < text_width("abc", TEXT_BODY));
    }

    #[test]
    fn a_line_longer_than_its_room_ends_in_an_ellipsis() {
        assert_eq!(fit("short", 100.0, 10.0), "short");
        let cut = fit("a very long comment line", 100.0, 10.0);
        assert!(cut.ends_with('…'));
        assert!(cut.chars().count() <= 10, "{cut}");
    }
}
