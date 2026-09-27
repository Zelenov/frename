//! Access to the folders the user opens, in the Mac App Store build.
//!
//! The App Sandbox lets frename into a folder only when the user chose it (📂, a drop on the
//! window) and only until frename quits. So the Store build keeps a security-scoped bookmark of
//! every folder it opens, and before opening a folder resolves its bookmark to get that access
//! back: the last folder reopens at the next start. A folder without access (a clip picked or
//! dropped alone, whose folder the sandbox does not open) is asked for once, with the folder
//! picker already in it (docs/design/mac-app-store.md). Other builds are not sandboxed: every
//! call here does nothing.

// The bookmark files are plain Rust, built and tested everywhere; only the Store build uses them.
#![cfg_attr(not(all(target_os = "macos", feature = "store")), allow(dead_code))]

use std::path::{Path, PathBuf};

/// How many folders keep their bookmark; the oldest go first.
const KEEP: usize = 50;

/// Get access to `folder` back from an earlier run, and keep the access this run has for the
/// next one. Returns whether frename can read the folder now; always `true` outside the sandbox,
/// where nothing is checked. Blocks briefly (small files, one system call per bookmark).
pub fn prepare(folder: &Path) -> bool {
    #[cfg(all(target_os = "macos", feature = "store"))]
    {
        let store = BookmarkStore::new(frename_core::app_data_dir().join("folder-access"));
        sandbox::prepare(folder, &store);
        std::fs::read_dir(folder).is_ok()
    }
    #[cfg(not(all(target_os = "macos", feature = "store")))]
    {
        let _ = folder;
        true
    }
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

    fn file(&self, folder: &Path) -> PathBuf {
        self.dir
            .join(format!("{:016x}.bookmark", fnv1a(folder.as_os_str())))
    }

    /// The bookmark saved for `folder`, if any.
    pub fn load(&self, folder: &Path) -> Option<Vec<u8>> {
        std::fs::read(self.file(folder)).ok()
    }

    /// Save `bookmark` for `folder`, replacing an older one, and forget all but the [`KEEP`]
    /// newest folders.
    pub fn save(&self, folder: &Path, bookmark: &[u8]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.file(folder), bookmark)?;
        self.prune()
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

#[cfg(all(target_os = "macos", feature = "store"))]
mod sandbox {
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, PoisonError};

    use objc2::runtime::Bool;
    use objc2_foundation::{
        NSData, NSString, NSURLBookmarkCreationOptions, NSURLBookmarkResolutionOptions, NSURL,
    };

    use super::BookmarkStore;

    /// Folders whose saved access this run has started. Never stopped: the user may come back to
    /// a folder, and the access ends with the process.
    static STARTED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

    pub fn prepare(folder: &Path, store: &BookmarkStore) {
        if let Some(bookmark) = store.load(folder) {
            start(folder, &bookmark);
        }
        // A fresh bookmark each time: it replaces a stale one, and it only works while this run
        // has access (chosen now, or started above).
        match bookmark(folder) {
            Ok(bookmark) => {
                if let Err(e) = store.save(folder, &bookmark) {
                    log::warn!("folder access: cannot save for {}: {e}", folder.display());
                }
            }
            Err(e) => log::info!("folder access: none to keep for {}: {e}", folder.display()),
        }
    }

    fn start(folder: &Path, bookmark: &[u8]) {
        let mut started = STARTED.lock().unwrap_or_else(PoisonError::into_inner);
        if started.iter().any(|f| f == folder) {
            return;
        }
        let data = NSData::with_bytes(bookmark);
        let mut stale = Bool::NO;
        // SAFETY: `stale` is a valid pointer for the duration of the call.
        let resolved = unsafe {
            NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error(
                &data,
                NSURLBookmarkResolutionOptions::WithSecurityScope,
                None,
                &mut stale,
            )
        };
        match resolved {
            // SAFETY: a plain message to a valid URL; access is given back when the process ends.
            Ok(url) if unsafe { url.startAccessingSecurityScopedResource() } => {
                log::info!("folder access: reopened {}", folder.display());
                started.push(folder.to_path_buf());
            }
            Ok(_) => log::warn!("folder access: refused for {}", folder.display()),
            Err(e) => log::warn!(
                "folder access: bookmark of {} does not resolve: {}",
                folder.display(),
                e.localizedDescription()
            ),
        }
    }

    fn bookmark(folder: &Path) -> Result<Vec<u8>, String> {
        let path = NSString::from_str(&folder.to_string_lossy());
        let url = NSURL::fileURLWithPath_isDirectory(&path, true);
        url.bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error(
            NSURLBookmarkCreationOptions::WithSecurityScope,
            None,
            None,
        )
        .map(|data| data.to_vec())
        .map_err(|e| e.localizedDescription().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "frename-folder-access-{name}-{}",
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

    #[cfg(not(all(target_os = "macos", feature = "store")))]
    #[test]
    fn outside_the_sandbox_every_folder_is_open() {
        assert!(prepare(Path::new("/no such folder")));
    }
}
