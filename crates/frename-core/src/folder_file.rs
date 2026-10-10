//! A folder path and an optional file path within that folder. Used for last session, scan target, etc.

use std::path::{Path, PathBuf};

/// A folder path and an optional file in that folder. Session = folder (required) + file (optional).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderAndFile {
    pub folder: PathBuf,
    pub file: Option<PathBuf>,
}

impl FolderAndFile {
    pub fn new(folder: impl Into<PathBuf>, file: Option<impl Into<PathBuf>>) -> Self {
        Self {
            folder: folder.into(),
            file: file.map(Into::into),
        }
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// Decide the (folder, selected file) pair for a path the user gave us (picker, drag-drop, CLI).
    /// A directory opens as itself with no selection; a file opens its parent folder with itself
    /// selected. Returns `None` if the path does not exist (caller logs and ignores it).
    pub fn from_path(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        if path.is_dir() {
            Some(Self::new(path, None::<PathBuf>))
        } else {
            Some(Self::new(path.parent().unwrap_or(path), Some(path)))
        }
    }
}

/// Choose which of several paths dropped at once to open: the first folder, or else the first
/// file (which then opens its folder with that file selected). Paths that do not exist are
/// skipped. Returns `None` when nothing usable was dropped.
pub fn choose_dropped_path(paths: &[PathBuf]) -> Option<&Path> {
    paths
        .iter()
        .find(|path| path.is_dir())
        .or_else(|| paths.iter().find(|path| path.is_file()))
        .map(PathBuf::as_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        crate::test_support::fresh_dir(&format!("folder-file-{name}"))
    }

    #[test]
    fn a_folder_opens_as_itself_with_nothing_selected() {
        let dir = temp_dir("folder");
        let pair = FolderAndFile::from_path(&dir).unwrap();
        assert_eq!(pair.folder(), dir.as_path());
        assert_eq!(pair.file(), None);
    }

    #[test]
    fn a_file_opens_its_folder_with_the_file_selected() {
        let dir = temp_dir("file");
        let file = dir.join("clip.mp4");
        std::fs::write(&file, b"video").unwrap();
        let pair = FolderAndFile::from_path(&file).unwrap();
        assert_eq!(pair.folder(), dir.as_path());
        assert_eq!(pair.file(), Some(file.as_path()));
    }

    #[test]
    fn a_missing_path_opens_nothing() {
        let dir = temp_dir("missing");
        assert_eq!(FolderAndFile::from_path(&dir.join("gone")), None);
        assert_eq!(FolderAndFile::from_path(&dir.join("gone.mp4")), None);
    }

    /// A folder `sub` and two files `a.mp4`, `b.mp4` in a fresh temp folder.
    fn drop_fixture(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let dir = temp_dir(name);
        let sub = dir.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        let a = dir.join("a.mp4");
        let b = dir.join("b.mp4");
        std::fs::write(&a, b"a").unwrap();
        std::fs::write(&b, b"b").unwrap();
        (sub, a, b)
    }

    #[test]
    fn dropping_nothing_opens_nothing() {
        assert_eq!(choose_dropped_path(&[]), None);
    }

    #[test]
    fn dropping_one_file_opens_that_file() {
        let (_, a, _) = drop_fixture("drop-one-file");
        assert_eq!(
            choose_dropped_path(std::slice::from_ref(&a)),
            Some(a.as_path())
        );
    }

    #[test]
    fn dropping_one_folder_opens_that_folder() {
        let (sub, _, _) = drop_fixture("drop-one-folder");
        let dropped = [sub.clone()];
        let chosen = choose_dropped_path(&dropped).unwrap();
        assert_eq!(chosen, sub.as_path());
        let pair = FolderAndFile::from_path(chosen).unwrap();
        assert_eq!(pair.folder(), sub.as_path());
        assert_eq!(pair.file(), None);
    }

    #[test]
    fn dropping_several_files_opens_the_first() {
        let (_, a, b) = drop_fixture("drop-files");
        assert_eq!(choose_dropped_path(&[b.clone(), a]), Some(b.as_path()));
    }

    #[test]
    fn dropping_a_folder_then_files_opens_the_folder() {
        let (sub, a, b) = drop_fixture("drop-folder-first");
        assert_eq!(
            choose_dropped_path(&[sub.clone(), a, b]),
            Some(sub.as_path())
        );
    }

    #[test]
    fn dropping_files_then_a_folder_opens_the_folder() {
        let (sub, a, b) = drop_fixture("drop-folder-last");
        assert_eq!(
            choose_dropped_path(&[a, b, sub.clone()]),
            Some(sub.as_path())
        );
    }

    #[test]
    fn missing_dropped_paths_are_skipped() {
        let (_, a, _) = drop_fixture("drop-missing");
        let gone = a.with_file_name("gone");
        assert_eq!(
            choose_dropped_path(&[gone.clone(), a.clone()]),
            Some(a.as_path())
        );
        assert_eq!(choose_dropped_path(&[gone]), None);
    }
}
