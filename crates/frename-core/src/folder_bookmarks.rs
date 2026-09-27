//! Security-scoped bookmarks of the folders the user opened, kept as files: the Mac App Store
//! build keeps one per folder so the sandbox lets frename back into it after a restart
//! (the app's `folder_access` makes and resolves them; docs/design/mac-app-store.md). Plain file
//! storage here, the same on every OS.

use std::path::{Path, PathBuf};

use crate::app_dir::app_data_dir;

/// How many folders keep their bookmark; the oldest go first.
const KEEP: usize = 50;

/// Whether the system refused to list `folder` (the sandbox keeps it closed). A folder that does
/// not exist, or fails otherwise, is not refused: opening it fails as it does in every build.
pub fn access_refused(folder: &Path) -> bool {
    matches!(
        std::fs::read_dir(folder),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied
    )
}

/// Saved bookmarks: one file per folder, named after a hash of the folder's path (the bookmark
/// itself holds the path).
pub struct BookmarkStore {
    dir: PathBuf,
}

impl BookmarkStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The store in frename's data folder.
    pub fn in_data_dir() -> Self {
        Self::new(app_data_dir().join("folder-access"))
    }

    fn file(&self, folder: &Path) -> PathBuf {
        self.dir
            .join(format!("{:016x}.bookmark", fnv1a(folder.as_os_str())))
    }

    /// The bookmark saved for `folder`, if any.
    pub fn load(&self, folder: &Path) -> Option<Vec<u8>> {
        std::fs::read(self.file(folder)).ok()
    }

    /// Save `bookmark` for `folder`, replacing an older one, and, when the folder is new, forget
    /// all but the [`KEEP`] newest folders.
    pub fn save(&self, folder: &Path, bookmark: &[u8]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let file = self.file(folder);
        let new = !file.exists();
        std::fs::write(file, bookmark)?;
        if new {
            self.prune()?;
        }
        Ok(())
    }

    fn prune(&self) -> std::io::Result<()> {
        let mut files: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(&self.dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "bookmark"))
            .filter_map(|path| {
                let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok()?;
                Some((modified, path))
            })
            .collect();
        files.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
        for (_, old) in files.into_iter().skip(KEEP) {
            std::fs::remove_file(old)?;
        }
        Ok(())
    }
}

/// FNV-1a, 64 bits: a file name that stays the same across Rust versions (`DefaultHasher` may
/// not).
fn fnv1a(text: &std::ffi::OsStr) -> u64 {
    text.as_encoded_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "frename-folder-bookmarks-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_saved_bookmark_is_found_by_its_folder_only() {
        let dir = temp_folder("load");
        let store = BookmarkStore::new(dir.clone());
        assert_eq!(store.load(Path::new("/Movies/day 1")), None);

        store
            .save(Path::new("/Movies/day 1"), b"one")
            .expect("save");
        store
            .save(Path::new("/Movies/day 2"), b"two")
            .expect("save");
        store
            .save(Path::new("/Movies/day 1"), b"one, again")
            .expect("save");

        assert_eq!(
            store.load(Path::new("/Movies/day 1")).as_deref(),
            Some(&b"one, again"[..])
        );
        assert_eq!(
            store.load(Path::new("/Movies/day 2")).as_deref(),
            Some(&b"two"[..])
        );
        assert_eq!(store.load(Path::new("/Movies/day 3")), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_the_newest_folders_keep_their_bookmark() {
        let dir = temp_folder("prune");
        let store = BookmarkStore::new(dir.clone());
        for n in 0..KEEP + 5 {
            store
                .save(&PathBuf::from(format!("/Movies/{n}")), b"x")
                .expect("save");
            // Distinct modification times, oldest first.
            let file = store.file(&PathBuf::from(format!("/Movies/{n}")));
            let time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000 + n as u64);
            std::fs::File::options()
                .write(true)
                .open(&file)
                .and_then(|f| f.set_modified(time))
                .expect("set time");
        }
        store.save(Path::new("/Movies/last"), b"x").expect("save");

        let kept = std::fs::read_dir(&dir).expect("dir").count();
        assert_eq!(kept, KEEP);
        assert!(store.load(Path::new("/Movies/last")).is_some());
        assert!(store.load(Path::new("/Movies/0")).is_none());
        assert!(store
            .load(&PathBuf::from(format!("/Movies/{}", KEEP + 4)))
            .is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_file_name_is_stable() {
        // FNV-1a test vectors.
        assert_eq!(fnv1a("".as_ref()), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a("a".as_ref()), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn a_missing_folder_is_not_refused() {
        assert!(!access_refused(Path::new("/no such folder")));
        assert!(!access_refused(&std::env::temp_dir()));
    }
}
