//! Searching the file list by what the user typed: words that must each be found in the file's
//! name or its comment, ignoring case. `to_lowercase` folds case for every script (Cyrillic
//! included) but not diacritics: `e` does not find `é`.

use std::ops::Range;

/// The words of a query, lowercased: what [`find_ignore_case`] takes.
pub(crate) fn words(query: &str) -> Vec<String> {
    query.split_whitespace().map(str::to_lowercase).collect()
}

/// The byte range of the first place in `haystack` at or after byte `from` where `needle_lower`
/// (already lowercase) occurs, ignoring case; without allocating a lowercased copy of the
/// haystack, since it runs for every file on every keystroke. An empty needle finds nothing.
pub(crate) fn find_ignore_case(
    haystack: &str,
    needle_lower: &str,
    from: usize,
) -> Option<Range<usize>> {
    if needle_lower.is_empty() {
        return None;
    }
    let tail = haystack.get(from..)?;
    let first = needle_lower.chars().next()?;
    tail.char_indices().find_map(|(start, c)| {
        // Most starts fail on their first character: skip them before setting anything up.
        if c.to_lowercase().next() != Some(first) {
            return None;
        }
        let mut needle = needle_lower.chars();
        for (i, c) in tail[start..].char_indices() {
            for lowered in c.to_lowercase() {
                match needle.next() {
                    None => return Some(from + start..from + start + i + c.len_utf8()),
                    Some(wanted) if wanted != lowered => return None,
                    Some(_) => {}
                }
            }
            if needle.as_str().is_empty() {
                return Some(from + start..from + start + i + c.len_utf8());
            }
        }
        None
    })
}

/// Whether `haystack` has `needle_lower` somewhere, ignoring case; an empty needle matches.
pub(crate) fn contains_ignore_case(haystack: &str, needle_lower: &str) -> bool {
    needle_lower.is_empty() || find_ignore_case(haystack, needle_lower, 0).is_some()
}

/// The part of a comment that shows why a file matched: one line of it, with where the query's
/// words are in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentFragment {
    pub text: String,
    /// Byte ranges of `text` to highlight, in order, not overlapping.
    pub highlights: Vec<Range<usize>>,
}

/// The first line of `comment` that has any of `words`, cut so the first hit is at most
/// `lead_chars` characters from the start (the cut is shown as "…"), with every hit of every
/// word in it marked. `None` when no line has one.
pub(crate) fn fragment(
    comment: &str,
    words: &[String],
    lead_chars: usize,
) -> Option<CommentFragment> {
    let (line, mut hits) = comment.lines().find_map(|line| {
        let hits = hits(line, words);
        (!hits.is_empty()).then_some((line, hits))
    })?;
    let first = hits[0].start;
    let lead = line[..first].chars().count();
    if lead <= lead_chars {
        return Some(CommentFragment {
            text: line.to_string(),
            highlights: hits,
        });
    }
    let cut = line[..first]
        .char_indices()
        .rev()
        .nth(lead_chars.saturating_sub(1))
        .map_or(0, |(i, _)| i);
    const ELLIPSIS: &str = "…";
    for hit in &mut hits {
        *hit = hit.start - cut + ELLIPSIS.len()..hit.end - cut + ELLIPSIS.len();
    }
    Some(CommentFragment {
        text: format!("{ELLIPSIS}{}", &line[cut..]),
        highlights: hits,
    })
}

/// Every place in `line` where a word occurs, in order, overlaps merged.
fn hits(line: &str, words: &[String]) -> Vec<Range<usize>> {
    let mut hits: Vec<Range<usize>> = Vec::new();
    for word in words {
        let mut from = 0;
        while let Some(hit) = find_ignore_case(line, word, from) {
            from = hit.end.max(from + 1);
            while !line.is_char_boundary(from) {
                from += 1;
            }
            hits.push(hit);
        }
    }
    hits.sort_by_key(|h| h.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for hit in hits {
        match merged.last_mut() {
            Some(last) if hit.start <= last.end => last.end = last.end.max(hit.end),
            _ => merged.push(hit),
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_ignoring_case_in_every_script() {
        assert_eq!(find_ignore_case("A Goat here", "goat", 0), Some(2..6));
        assert_eq!(find_ignore_case("Коза на лугу", "коза", 0), Some(0..8));
        assert_eq!(
            find_ignore_case("Коза на лугу", "ЛУГУ".to_lowercase().as_str(), 0),
            Some(14..22)
        );
        assert_eq!(find_ignore_case("goat", "sheep", 0), None);
        assert_eq!(find_ignore_case("goat goat", "goat", 1), Some(5..9));
        assert_eq!(find_ignore_case("goat", "", 0), None);
        assert!(
            !contains_ignore_case("café", "cafe"),
            "diacritics are not folded"
        );
    }

    #[test]
    fn a_word_that_lowercases_to_more_characters_is_found() {
        // 'İ' lowercases to two characters; the range still ends on a character boundary.
        let text = "xİy";
        let hit = find_ignore_case(text, "i\u{307}", 0).expect("found");
        assert!(text.is_char_boundary(hit.start) && text.is_char_boundary(hit.end));
    }

    #[test]
    fn the_fragment_marks_every_word_in_the_first_line_that_has_one() {
        let comment = "Nothing here\nA goat and a Goat with a sheep\nsheep";
        let words = words("goat sheep");
        let f = fragment(comment, &words, 20).expect("a fragment");
        assert_eq!(f.text, "A goat and a Goat with a sheep");
        let marked: Vec<&str> = f.highlights.iter().map(|r| &f.text[r.clone()]).collect();
        assert_eq!(marked, ["goat", "Goat", "sheep"]);
    }

    #[test]
    fn a_hit_far_from_the_start_is_brought_into_view() {
        let comment = "0123456789 0123456789 0123456789 0123456789 goat at the end";
        let words = words("goat");
        let f = fragment(comment, &words, 10).expect("a fragment");
        assert!(f.text.starts_with('…'), "{}", f.text);
        assert_eq!(&f.text[f.highlights[0].clone()], "goat");
        assert!(f.text.chars().take(12).count() == 12);
        assert_eq!(fragment("no match", &words, 10), None);
    }

    #[test]
    fn overlapping_words_are_marked_once() {
        let words = words("goat oat");
        let f = fragment("goat", &words, 10).expect("a fragment");
        assert_eq!(f.highlights, vec![0..4]);
    }
}
