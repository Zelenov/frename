//! State of the recent folders list: the entries, whether the dropdown is open, which row the
//! pointer or the arrow keys are on, and the missing folder the user is being asked about. Pure
//! data and rules; the workspace does what an [`Effect`] asks (reading and writing the database,
//! opening a folder).

use std::path::{Path, PathBuf};

use frename_core::recent_folders::RecentFolder;
use frename_core::FolderAndFile;

use super::Message;

/// Whether a listed folder is there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    /// Not looked at yet: shown as usable, since the check runs off the interface's thread (a
    /// disconnected network drive can take seconds to answer).
    Unknown,
    Found,
    Missing,
}

/// One row of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub folder: RecentFolder,
    pub presence: Presence,
}

/// What the workspace does after a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    None,
    /// Read the list again and check which folders exist (the dropdown was opened).
    Reload,
    /// Open this folder, with this file selected if there is one.
    Open(FolderAndFile),
    /// Take this folder off the stored list.
    Forget(PathBuf),
    /// Empty the stored list.
    Clear,
}

#[derive(Debug, Default)]
pub struct RecentFoldersState {
    entries: Vec<Entry>,
    open: bool,
    highlight: Option<usize>,
    /// The missing folder the user clicked: asked whether to remove it from the list.
    asking: Option<PathBuf>,
    /// When the list was read: the ages are counted from it.
    now_ms: i64,
}

impl RecentFoldersState {
    /// The list as read from the database at `now_ms`. Folders that were listed keep what is
    /// known of them; the highlight and the question stay while their folder is listed.
    pub fn set_entries(&mut self, list: Vec<RecentFolder>, now_ms: i64) {
        let previous = std::mem::take(&mut self.entries);
        self.entries = list
            .into_iter()
            .map(|folder| {
                let presence = previous
                    .iter()
                    .find(|entry| entry.folder.folder == folder.folder)
                    .map_or(Presence::Unknown, |entry| entry.presence);
                Entry { folder, presence }
            })
            .collect();
        self.now_ms = now_ms;
        self.highlight = self.highlight.filter(|&at| at < self.entries.len());
        if let Some(path) = &self.asking {
            if !self.entries.iter().any(|e| &e.folder.folder == path) {
                self.asking = None;
            }
        }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The dropdown is open (and takes the arrow keys, Enter and Esc).
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn highlight(&self) -> Option<usize> {
        self.highlight
    }

    /// The row to scroll to, as a fraction of the way down the list: 0 for the first, 1 for the
    /// last. A viewport at that fraction of the content always shows the whole row.
    pub fn highlight_fraction(&self) -> Option<f32> {
        let at = self.highlight?;
        Some(if self.entries.len() < 2 {
            0.0
        } else {
            at as f32 / (self.entries.len() - 1) as f32
        })
    }

    /// Whether the user is being asked about this folder.
    pub fn is_asking(&self, folder: &Path) -> bool {
        self.asking.as_deref() == Some(folder)
    }

    pub fn now_ms(&self) -> i64 {
        self.now_ms
    }

    /// The folders whose presence is worth checking.
    pub fn folders(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .map(|entry| entry.folder.folder.clone())
            .collect()
    }

    fn close(&mut self) {
        self.open = false;
        self.highlight = None;
        self.asking = None;
    }

    fn path_at(&self, at: usize) -> Option<PathBuf> {
        self.entries.get(at).map(|e| e.folder.folder.clone())
    }

    fn choose(&mut self, folder: &Path) -> Effect {
        let Some(entry) = self.entries.iter().find(|e| e.folder.folder == folder) else {
            return Effect::None;
        };
        if entry.presence == Presence::Missing {
            // Never removed silently: the drive may come back.
            self.asking = Some(folder.to_path_buf());
            return Effect::None;
        }
        let open = entry.folder.to_open();
        self.close();
        Effect::Open(open)
    }

    fn remove(&mut self, folder: &Path) -> Effect {
        self.entries.retain(|entry| entry.folder.folder != folder);
        if self.asking.as_deref() == Some(folder) {
            self.asking = None;
        }
        self.highlight = self.highlight.filter(|&at| at < self.entries.len());
        Effect::Forget(folder.to_path_buf())
    }

    pub fn update(&mut self, message: Message) -> Effect {
        match message {
            Message::Toggle if self.open => {
                self.close();
                Effect::None
            }
            Message::Toggle => {
                self.open = true;
                self.highlight = None;
                self.asking = None;
                Effect::Reload
            }
            Message::Close => {
                self.close();
                Effect::None
            }
            Message::Escape => {
                if self.asking.take().is_none() {
                    self.close();
                }
                Effect::None
            }
            Message::Highlight(at) => {
                if at < self.entries.len() {
                    self.highlight = Some(at);
                }
                Effect::None
            }
            Message::Move(by) => {
                let count = self.entries.len() as i32;
                if count > 0 {
                    self.asking = None;
                    self.highlight = Some(match self.highlight {
                        None if by >= 0 => 0,
                        None => count as usize - 1,
                        Some(at) => (at as i32 + by).rem_euclid(count) as usize,
                    });
                }
                Effect::None
            }
            Message::Choose(folder) => self.choose(&folder),
            Message::ChooseHighlighted => {
                let Some(folder) = self.highlight.and_then(|at| self.path_at(at)) else {
                    return Effect::None;
                };
                // Enter on the row being asked about confirms: the user has seen the question.
                if self.is_asking(&folder) {
                    return self.remove(&folder);
                }
                self.choose(&folder)
            }
            Message::Remove(folder) => self.remove(&folder),
            Message::Keep => {
                self.asking = None;
                Effect::None
            }
            Message::Clear => {
                self.entries.clear();
                self.close();
                Effect::Clear
            }
            Message::Checked(results) => {
                for (folder, found) in results {
                    if let Some(entry) = self.entries.iter_mut().find(|e| e.folder.folder == folder)
                    {
                        entry.presence = if found {
                            Presence::Found
                        } else {
                            Presence::Missing
                        };
                    }
                }
                Effect::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str, file: Option<&str>) -> RecentFolder {
        RecentFolder {
            folder: PathBuf::from(format!("/shoots/{name}")),
            opened_at_ms: 0,
            last_file: file.map(|f| PathBuf::from(format!("/shoots/{name}/{f}"))),
        }
    }

    fn state(names: &[&str]) -> RecentFoldersState {
        let mut state = RecentFoldersState::default();
        state.set_entries(
            names.iter().map(|n| folder(n, Some("a.mp4"))).collect(),
            1_000,
        );
        state
    }

    fn missing(state: &mut RecentFoldersState, name: &str) {
        let _ = state.update(Message::Checked(vec![(
            PathBuf::from(format!("/shoots/{name}")),
            false,
        )]));
    }

    #[test]
    fn opening_the_dropdown_reloads_the_list_and_toggling_closes_it() {
        let mut state = state(&["a"]);
        assert_eq!(state.update(Message::Toggle), Effect::Reload);
        assert!(state.is_open());
        assert_eq!(state.update(Message::Toggle), Effect::None);
        assert!(!state.is_open());
    }

    #[test]
    fn the_arrows_walk_the_rows_and_wrap_round() {
        let mut state = state(&["a", "b", "c"]);
        let _ = state.update(Message::Toggle);
        let _ = state.update(Message::Move(1));
        assert_eq!(state.highlight(), Some(0), "down from nothing is the first");
        let _ = state.update(Message::Move(1));
        let _ = state.update(Message::Move(1));
        assert_eq!(state.highlight(), Some(2));
        let _ = state.update(Message::Move(1));
        assert_eq!(state.highlight(), Some(0), "past the last is the first");
        let _ = state.update(Message::Move(-1));
        assert_eq!(state.highlight(), Some(2), "above the first is the last");
    }

    #[test]
    fn up_from_nothing_is_the_last_row() {
        let mut state = state(&["a", "b", "c"]);
        let _ = state.update(Message::Move(-1));
        assert_eq!(state.highlight(), Some(2));
    }

    #[test]
    fn enter_opens_the_highlighted_folder_with_its_last_file_and_closes() {
        let mut state = state(&["a", "b"]);
        let _ = state.update(Message::Toggle);
        let _ = state.update(Message::Move(1));
        let _ = state.update(Message::Move(1));
        let effect = state.update(Message::ChooseHighlighted);
        assert_eq!(
            effect,
            Effect::Open(FolderAndFile::new("/shoots/b", Some("/shoots/b/a.mp4")))
        );
        assert!(!state.is_open());
    }

    #[test]
    fn enter_with_nothing_highlighted_does_nothing() {
        let mut state = state(&["a"]);
        assert_eq!(state.update(Message::ChooseHighlighted), Effect::None);
    }

    #[test]
    fn a_click_opens_the_folder() {
        let mut state = state(&["a", "b"]);
        let effect = state.update(Message::Choose(PathBuf::from("/shoots/a")));
        assert_eq!(
            effect,
            Effect::Open(FolderAndFile::new("/shoots/a", Some("/shoots/a/a.mp4")))
        );
    }

    #[test]
    fn a_folder_not_checked_yet_can_be_opened() {
        let state = state(&["a"]);
        assert_eq!(state.entries()[0].presence, Presence::Unknown);
        let mut state = state;
        assert!(matches!(
            state.update(Message::Choose(PathBuf::from("/shoots/a"))),
            Effect::Open(_)
        ));
    }

    #[test]
    fn a_missing_folder_is_asked_about_not_opened_and_not_removed() {
        let mut state = state(&["a", "b"]);
        missing(&mut state, "a");
        let effect = state.update(Message::Choose(PathBuf::from("/shoots/a")));
        assert_eq!(effect, Effect::None);
        assert!(state.is_asking(Path::new("/shoots/a")));
        assert_eq!(state.entries().len(), 2, "still listed");
    }

    #[test]
    fn keeping_or_escaping_leaves_a_missing_folder_listed() {
        let mut state = state(&["a"]);
        let _ = state.update(Message::Toggle);
        missing(&mut state, "a");
        let _ = state.update(Message::Choose(PathBuf::from("/shoots/a")));
        let _ = state.update(Message::Keep);
        assert!(!state.is_asking(Path::new("/shoots/a")));
        assert_eq!(state.entries().len(), 1);

        let _ = state.update(Message::Choose(PathBuf::from("/shoots/a")));
        let _ = state.update(Message::Escape);
        assert!(state.is_open(), "the first Esc only ends the question");
        assert!(!state.is_asking(Path::new("/shoots/a")));
        let _ = state.update(Message::Escape);
        assert!(!state.is_open(), "the second closes the dropdown");
        assert_eq!(state.entries().len(), 1);
    }

    #[test]
    fn removing_a_missing_folder_after_the_question_forgets_it() {
        let mut state = state(&["a", "b"]);
        missing(&mut state, "a");
        let _ = state.update(Message::Choose(PathBuf::from("/shoots/a")));
        let effect = state.update(Message::Remove(PathBuf::from("/shoots/a")));
        assert_eq!(effect, Effect::Forget(PathBuf::from("/shoots/a")));
        assert_eq!(state.entries().len(), 1);
        assert!(!state.is_asking(Path::new("/shoots/a")));
    }

    #[test]
    fn enter_on_the_row_being_asked_about_removes_it() {
        let mut state = state(&["a", "b"]);
        let _ = state.update(Message::Toggle);
        missing(&mut state, "a");
        let _ = state.update(Message::Move(1));
        assert_eq!(state.update(Message::ChooseHighlighted), Effect::None);
        assert!(state.is_asking(Path::new("/shoots/a")));
        assert_eq!(
            state.update(Message::ChooseHighlighted),
            Effect::Forget(PathBuf::from("/shoots/a"))
        );
    }

    #[test]
    fn moving_to_another_row_drops_the_question() {
        let mut state = state(&["a", "b"]);
        missing(&mut state, "a");
        let _ = state.update(Message::Choose(PathBuf::from("/shoots/a")));
        let _ = state.update(Message::Move(1));
        assert!(!state.is_asking(Path::new("/shoots/a")));
    }

    #[test]
    fn the_cross_removes_one_entry_and_clear_removes_all() {
        let mut state = state(&["a", "b", "c"]);
        let _ = state.update(Message::Toggle);
        assert_eq!(
            state.update(Message::Remove(PathBuf::from("/shoots/b"))),
            Effect::Forget(PathBuf::from("/shoots/b"))
        );
        assert_eq!(state.entries().len(), 2);
        assert_eq!(state.update(Message::Clear), Effect::Clear);
        assert!(state.is_empty());
        assert!(!state.is_open(), "an empty list has nothing to show");
    }

    #[test]
    fn a_reload_keeps_what_is_known_about_the_folders_that_are_still_listed() {
        let mut state = state(&["a", "b"]);
        missing(&mut state, "a");
        state.set_entries(vec![folder("c", None), folder("a", None)], 9_000);
        assert_eq!(state.entries()[0].presence, Presence::Unknown);
        assert_eq!(state.entries()[1].presence, Presence::Missing);
        assert_eq!(state.now_ms(), 9_000);
    }

    #[test]
    fn the_highlight_stays_inside_a_shorter_list() {
        let mut state = state(&["a", "b", "c"]);
        let _ = state.update(Message::Highlight(2));
        state.set_entries(vec![folder("a", None)], 1);
        assert_eq!(state.highlight(), None);
    }

    #[test]
    fn the_scroll_fraction_runs_from_the_first_row_to_the_last() {
        let mut state = state(&["a", "b", "c"]);
        assert_eq!(state.highlight_fraction(), None);
        let _ = state.update(Message::Highlight(0));
        assert_eq!(state.highlight_fraction(), Some(0.0));
        let _ = state.update(Message::Highlight(1));
        assert_eq!(state.highlight_fraction(), Some(0.5));
        let _ = state.update(Message::Highlight(2));
        assert_eq!(state.highlight_fraction(), Some(1.0));
    }
}
