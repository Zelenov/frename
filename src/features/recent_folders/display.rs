//! The words and the cut paths of the list: pure functions of what a row shows.

use frename_core::recent_folders::Age;

/// How long ago, in words: "just now", "5 min ago", "2 days ago".
pub fn age_text(age: Age) -> String {
    match age {
        Age::JustNow => fl!("recent-folders-just-now"),
        Age::Minutes(count) => fl!("recent-folders-minutes", count = i64::from(count)),
        Age::Hours(count) => fl!("recent-folders-hours", count = i64::from(count)),
        Age::Days(count) => fl!("recent-folders-days", count = i64::from(count)),
        Age::Weeks(count) => fl!("recent-folders-weeks", count = i64::from(count)),
        Age::Months(count) => fl!("recent-folders-months", count = i64::from(count)),
        Age::Years(count) => fl!("recent-folders-years", count = i64::from(count)),
    }
}

/// `path` cut from the front to at most `room` wide, each character `width(c)` wide: what is
/// left starts with "…" and, if it can, at a separator, so it reads as the end of a path
/// (`…\2026\Lisbon`). The end of a path is the part that tells one folder from the next. The
/// view measures characters in caption text, so Cyrillic or CJK is cut where it really ends (#408).
pub fn shorten_path(path: &str, room: f32, width: impl Fn(char) -> f32) -> String {
    let total: f32 = path.chars().map(&width).sum();
    if total <= room {
        return path.to_string();
    }
    let keep = room - width('…');
    // The longest tail of `path` that fits in `keep`.
    let mut used = 0.0;
    let mut start = path.len();
    for (at, c) in path.char_indices().rev() {
        used += width(c);
        if used > keep {
            break;
        }
        start = at;
    }
    let tail = &path[start..];
    // Start at the next separator if one is near, so no folder name is cut in two.
    let near = tail.len() / 2;
    let at_separator = tail
        .char_indices()
        .find(|(_, c)| matches!(c, '/' | '\\'))
        .filter(|(at, _)| *at < near)
        .map_or(tail, |(at, _)| &tail[at..]);
    format!("…{at_separator}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every character one unit wide, so a room counts characters.
    fn one(_: char) -> f32 {
        1.0
    }

    /// Issue #408: wide characters are cut by their width, so the cut path fits its room.
    #[test]
    fn a_path_of_wide_characters_is_cut_by_their_width() {
        let wide = |c: char| if c.is_ascii() { 1.0 } else { 2.0 };
        let cut = shorten_path("/撮影/2026年夏/北海道", 10.0, wide);
        assert!(cut.starts_with('…'), "{cut}");
        assert!(cut.chars().map(wide).sum::<f32>() <= 10.0, "{cut}");
        assert!(cut.ends_with("北海道"), "{cut}");
    }

    #[test]
    fn a_path_that_fits_is_left_alone() {
        assert_eq!(shorten_path(r"D:\Shoots", 20.0, one), r"D:\Shoots");
        assert_eq!(shorten_path("abcde", 5.0, one), "abcde");
    }

    #[test]
    fn a_long_path_keeps_its_end_from_a_separator() {
        let cut = shorten_path(r"D:\Shoots\2026\Clients\Lisbon", 20.0, one);
        assert_eq!(cut, r"…\Clients\Lisbon");
        assert!(cut.chars().count() <= 20);
    }

    #[test]
    fn a_name_longer_than_the_room_is_cut_inside_a_name_when_no_separator_is_near() {
        let cut = shorten_path("/a-very-long-single-folder-name", 10.0, one);
        assert_eq!(cut.chars().count(), 10);
        assert!(cut.starts_with('…'));
        assert!(cut.ends_with("name"));
    }

    #[test]
    fn no_room_gives_just_the_ellipsis() {
        assert_eq!(shorten_path("/shoots", 1.0, one), "…");
        assert_eq!(shorten_path("/shoots", 0.0, one), "…");
    }

    #[test]
    fn ages_are_worded() {
        assert_eq!(age_text(Age::JustNow), "Just now");
        assert_eq!(age_text(Age::Minutes(5)), "5 min ago");
        assert_eq!(age_text(Age::Hours(3)), "3 h ago");
        assert_eq!(age_text(Age::Days(1)), "1 day ago");
        assert_eq!(age_text(Age::Days(2)), "2 days ago");
        assert_eq!(age_text(Age::Weeks(1)), "1 week ago");
        assert_eq!(age_text(Age::Months(3)), "3 months ago");
        assert_eq!(age_text(Age::Years(1)), "1 year ago");
    }
}
