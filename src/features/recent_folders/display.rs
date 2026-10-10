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

/// `path` cut from the front to at most `room` characters: what is left starts with "…" and, if
/// it can, at a separator, so it reads as the end of a path (`…\2026\Lisbon`). The end of a
/// path is the part that tells one folder from the next.
pub fn shorten_path(path: &str, room: usize) -> String {
    let count = path.chars().count();
    if count <= room {
        return path.to_string();
    }
    let keep = room.saturating_sub(1);
    let tail: String = path.chars().skip(count - keep).collect();
    // Start at the next separator if one is near, so no folder name is cut in two.
    let at_separator = tail
        .char_indices()
        .find(|(_, c)| matches!(c, '/' | '\\'))
        .filter(|(at, _)| *at < keep / 2)
        .map_or(tail.as_str(), |(at, _)| &tail[at..]);
    format!("…{at_separator}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_that_fits_is_left_alone() {
        assert_eq!(shorten_path(r"D:\Shoots", 20), r"D:\Shoots");
        assert_eq!(shorten_path("abcde", 5), "abcde");
    }

    #[test]
    fn a_long_path_keeps_its_end_from_a_separator() {
        let cut = shorten_path(r"D:\Shoots\2026\Clients\Lisbon", 20);
        assert_eq!(cut, r"…\Clients\Lisbon");
        assert!(cut.chars().count() <= 20);
    }

    #[test]
    fn a_name_longer_than_the_room_is_cut_inside_a_name_when_no_separator_is_near() {
        let cut = shorten_path("/a-very-long-single-folder-name", 10);
        assert_eq!(cut.chars().count(), 10);
        assert!(cut.starts_with('…'));
        assert!(cut.ends_with("name"));
    }

    #[test]
    fn no_room_gives_just_the_ellipsis() {
        assert_eq!(shorten_path("/shoots", 1), "…");
        assert_eq!(shorten_path("/shoots", 0), "…");
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
