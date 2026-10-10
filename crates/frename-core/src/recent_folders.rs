//! The folders opened recently (#63): the list behind the dropdown next to the open button and
//! the empty screen. Pure logic only; the database keeps the list (`db`), the UI shows it.
//!
//! The list is newest first, holds each folder once, and is at most [`MAX_RECENT_FOLDERS`] long.
//! Paths are compared the way Windows compares them: ignoring case and the kind of slash.

use std::path::{Path, PathBuf};

use crate::FolderAndFile;

/// How many folders the list keeps; opening an eleventh drops the one opened longest ago.
pub const MAX_RECENT_FOLDERS: usize = 10;

/// A folder opened recently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentFolder {
    /// The folder, as it was last opened.
    pub folder: PathBuf,
    /// When it was last opened, in milliseconds since the Unix epoch.
    pub opened_at_ms: i64,
    /// The file that was open in it last time, if one was.
    pub last_file: Option<PathBuf>,
}

impl RecentFolder {
    /// The folder's own name (`2026-09 Lisbon`); the whole path for a drive root, which has none.
    pub fn name(&self) -> String {
        self.folder.file_name().map_or_else(
            || self.folder.to_string_lossy().into_owned(),
            |name| name.to_string_lossy().into_owned(),
        )
    }

    /// The path of the folder that holds this one (`D:\Shoots`); `None` for a drive root.
    pub fn parent(&self) -> Option<String> {
        self.folder
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .filter(|parent| !parent.is_empty())
    }

    /// What the folder and `last_file` open as: the folder with its last file selected.
    pub fn to_open(&self) -> FolderAndFile {
        FolderAndFile::new(self.folder.clone(), self.last_file.clone())
    }
}

/// The key two spellings of one folder share: on Windows, where paths ignore case and take either
/// slash, lower case with backslashes and no trailing one; elsewhere the path without its
/// trailing slash.
pub(crate) fn folder_key(folder: &Path) -> String {
    key_of(folder, cfg!(windows))
}

/// [`folder_key`] for the given kind of file system, so both rules can be tested anywhere.
fn key_of(folder: &Path, windows: bool) -> String {
    let text = folder.to_string_lossy();
    if windows {
        text.replace('/', "\\")
            .trim_end_matches('\\')
            .to_lowercase()
    } else {
        text.trim_end_matches('/').to_string()
    }
}

/// Now, in milliseconds since the Unix epoch.
pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
        })
}

/// `session`'s folder was opened at `now_ms`: it moves to the top of `list`, whatever spelling it
/// had there, and the oldest fall off the end. A session with a file remembers that file; one
/// without (the folder was only opened) keeps the file the folder had.
pub fn record(list: &mut Vec<RecentFolder>, session: &FolderAndFile, now_ms: i64) {
    let key = folder_key(session.folder());
    let previous = list
        .iter()
        .position(|entry| folder_key(&entry.folder) == key)
        .map(|at| list.remove(at));
    let last_file = session
        .file()
        .map(Path::to_path_buf)
        .or_else(|| previous.and_then(|entry| entry.last_file));
    list.insert(
        0,
        RecentFolder {
            folder: session.folder().to_path_buf(),
            opened_at_ms: now_ms,
            last_file,
        },
    );
    list.truncate(MAX_RECENT_FOLDERS);
}

/// Take `folder` (in any spelling) off `list`.
pub fn forget(list: &mut Vec<RecentFolder>, folder: &Path) {
    let key = folder_key(folder);
    list.retain(|entry| folder_key(&entry.folder) != key);
}

/// `list` as stored data may have left it: each folder once (its newest entry), at most
/// [`MAX_RECENT_FOLDERS`].
pub fn tidy(list: Vec<RecentFolder>) -> Vec<RecentFolder> {
    let mut seen = std::collections::HashSet::new();
    list.into_iter()
        .filter(|entry| seen.insert(folder_key(&entry.folder)))
        .take(MAX_RECENT_FOLDERS)
        .collect()
}

/// How long ago a folder was opened, in the units the list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Age {
    /// Less than a minute.
    JustNow,
    Minutes(u32),
    Hours(u32),
    Days(u32),
    Weeks(u32),
    Months(u32),
    Years(u32),
}

/// The age of something opened at `opened_at_ms`, seen at `now_ms`. A time in the future (a clock
/// set back) counts as just now.
pub fn age(opened_at_ms: i64, now_ms: i64) -> Age {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    let seconds = (now_ms - opened_at_ms).max(0) / 1000;
    let count = |units: i64| u32::try_from(units).unwrap_or(u32::MAX);
    match seconds {
        s if s < MINUTE => Age::JustNow,
        s if s < HOUR => Age::Minutes(count(s / MINUTE)),
        s if s < DAY => Age::Hours(count(s / HOUR)),
        s if s < 7 * DAY => Age::Days(count(s / DAY)),
        s if s < 30 * DAY => Age::Weeks(count(s / (7 * DAY))),
        s if s < 365 * DAY => Age::Months(count(s / (30 * DAY))),
        s => Age::Years(count(s / (365 * DAY))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(list: &mut Vec<RecentFolder>, folder: &str, file: Option<&str>, at: i64) {
        record(list, &FolderAndFile::new(folder, file), at);
    }

    fn folders(list: &[RecentFolder]) -> Vec<String> {
        list.iter()
            .map(|e| e.folder.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn the_newest_folder_comes_first() {
        let mut list = Vec::new();
        open(&mut list, "/shoots/a", None::<&str>, 1);
        open(&mut list, "/shoots/b", None::<&str>, 2);
        assert_eq!(folders(&list), ["/shoots/b", "/shoots/a"]);
        assert_eq!(list[0].opened_at_ms, 2);
    }

    #[test]
    fn opening_a_folder_again_moves_it_to_the_top_without_a_duplicate() {
        let mut list = Vec::new();
        open(&mut list, "/shoots/a", None::<&str>, 1);
        open(&mut list, "/shoots/b", None::<&str>, 2);
        open(&mut list, "/shoots/a", None::<&str>, 3);
        assert_eq!(folders(&list), ["/shoots/a", "/shoots/b"]);
        assert_eq!(list[0].opened_at_ms, 3);
    }

    #[test]
    fn only_the_ten_newest_are_kept() {
        let mut list = Vec::new();
        for n in 0..12 {
            open(&mut list, &format!("/shoots/{n}"), None::<&str>, n);
        }
        assert_eq!(list.len(), MAX_RECENT_FOLDERS);
        assert_eq!(list[0].folder, Path::new("/shoots/11"));
        assert_eq!(list[9].folder, Path::new("/shoots/2"));
    }

    #[test]
    fn a_folder_opened_again_is_not_pushed_out_by_its_own_duplicate() {
        let mut list = Vec::new();
        for n in 0..MAX_RECENT_FOLDERS as i64 {
            open(&mut list, &format!("/shoots/{n}"), None::<&str>, n);
        }
        open(&mut list, "/shoots/0", None::<&str>, 99);
        assert_eq!(list.len(), MAX_RECENT_FOLDERS);
        assert_eq!(list[0].folder, Path::new("/shoots/0"));
        assert_eq!(list[9].folder, Path::new("/shoots/1"));
    }

    #[test]
    fn a_folder_remembers_the_file_that_was_open_in_it() {
        let mut list = Vec::new();
        open(&mut list, "/shoots/a", Some("/shoots/a/pick.mp4"), 1);
        assert_eq!(
            list[0].last_file.as_deref(),
            Some(Path::new("/shoots/a/pick.mp4"))
        );
        open(&mut list, "/shoots/a", Some("/shoots/a/skip.mp4"), 2);
        assert_eq!(
            list[0].last_file.as_deref(),
            Some(Path::new("/shoots/a/skip.mp4"))
        );
    }

    #[test]
    fn opening_a_folder_without_a_file_keeps_the_file_it_had() {
        let mut list = Vec::new();
        open(&mut list, "/shoots/a", Some("/shoots/a/pick.mp4"), 1);
        open(&mut list, "/shoots/b", None::<&str>, 2);
        open(&mut list, "/shoots/a", None::<&str>, 3);
        assert_eq!(list[0].folder, Path::new("/shoots/a"));
        assert_eq!(
            list[0].last_file.as_deref(),
            Some(Path::new("/shoots/a/pick.mp4"))
        );
        assert_eq!(list[1].last_file, None);
    }

    #[test]
    fn windows_ignores_case_and_the_kind_of_slash() {
        let key = |text: &str| key_of(Path::new(text), true);
        assert_eq!(key(r"D:\Shoots\Lisbon"), key("d:/shoots/LISBON/"));
        assert_eq!(key(r"D:\Shoots\Lisbon\"), key(r"d:\shoots\lisbon"));
        assert_ne!(key(r"D:\Shoots\Lisbon"), key(r"D:\Shoots\Porto"));
    }

    #[test]
    fn other_systems_keep_case_but_not_a_trailing_slash() {
        let key = |text: &str| key_of(Path::new(text), false);
        assert_eq!(key("/shoots/Lisbon/"), key("/shoots/Lisbon"));
        assert_ne!(key("/shoots/Lisbon"), key("/shoots/lisbon"));
    }

    #[cfg(windows)]
    #[test]
    fn another_spelling_of_a_folder_is_the_same_folder() {
        let mut list = Vec::new();
        open(&mut list, r"D:\Shoots\Lisbon", None::<&str>, 1);
        open(&mut list, r"D:\Shoots\Porto", None::<&str>, 2);
        open(&mut list, "d:/shoots/lisbon/", None::<&str>, 3);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].folder, Path::new("d:/shoots/lisbon/"));
    }

    #[test]
    fn forgetting_a_folder_takes_only_that_one_off() {
        let mut list = Vec::new();
        open(&mut list, "/shoots/a", None::<&str>, 1);
        open(&mut list, "/shoots/b", None::<&str>, 2);
        forget(&mut list, Path::new("/shoots/a/"));
        assert_eq!(folders(&list), ["/shoots/b"]);
        forget(&mut list, Path::new("/shoots/never"));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn stored_rows_are_tidied_to_each_folder_once_and_at_most_ten() {
        let entry = |folder: &str, at| RecentFolder {
            folder: folder.into(),
            opened_at_ms: at,
            last_file: None,
        };
        let mut stored: Vec<RecentFolder> = (0..12)
            .map(|n| entry(&format!("/shoots/{n}"), 100 - n))
            .collect();
        stored.insert(1, entry("/shoots/0/", 1));
        let kept = tidy(stored);
        assert_eq!(kept.len(), MAX_RECENT_FOLDERS);
        assert_eq!(kept[0].opened_at_ms, 100, "the newest of a duplicate stays");
        assert_eq!(kept[1].folder, Path::new("/shoots/1"));
    }

    #[test]
    fn a_folder_shows_its_name_and_where_it_is() {
        let entry = RecentFolder {
            folder: PathBuf::from("/shoots/2026/Lisbon"),
            opened_at_ms: 0,
            last_file: None,
        };
        assert_eq!(entry.name(), "Lisbon");
        assert_eq!(entry.parent().as_deref(), Some("/shoots/2026"));
        let root = RecentFolder {
            folder: PathBuf::from("/"),
            opened_at_ms: 0,
            last_file: None,
        };
        assert_eq!(root.name(), "/");
        assert_eq!(root.parent(), None);
    }

    #[test]
    fn it_opens_the_folder_with_its_last_file() {
        let entry = RecentFolder {
            folder: PathBuf::from("/shoots/a"),
            opened_at_ms: 0,
            last_file: Some(PathBuf::from("/shoots/a/pick.mp4")),
        };
        let open = entry.to_open();
        assert_eq!(open.folder(), Path::new("/shoots/a"));
        assert_eq!(open.file(), Some(Path::new("/shoots/a/pick.mp4")));
    }

    #[test]
    fn ages_count_in_the_largest_unit_that_fits() {
        let now = 10_000_000_000_i64;
        let ago = |seconds: i64| age(now - seconds * 1000, now);
        assert_eq!(ago(0), Age::JustNow);
        assert_eq!(ago(59), Age::JustNow);
        assert_eq!(ago(60), Age::Minutes(1));
        assert_eq!(ago(59 * 60 + 59), Age::Minutes(59));
        assert_eq!(ago(3600), Age::Hours(1));
        assert_eq!(ago(23 * 3600), Age::Hours(23));
        assert_eq!(ago(24 * 3600), Age::Days(1));
        assert_eq!(ago(6 * 86_400), Age::Days(6));
        assert_eq!(ago(7 * 86_400), Age::Weeks(1));
        assert_eq!(ago(29 * 86_400), Age::Weeks(4));
        assert_eq!(ago(30 * 86_400), Age::Months(1));
        assert_eq!(ago(364 * 86_400), Age::Months(12));
        assert_eq!(ago(365 * 86_400), Age::Years(1));
        assert_eq!(ago(800 * 86_400), Age::Years(2));
    }

    #[test]
    fn a_time_in_the_future_is_just_now() {
        assert_eq!(age(5_000, 1_000), Age::JustNow);
    }
}
