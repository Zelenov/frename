//! Access to the folders the user opens, in the Mac App Store build.
//!
//! The App Sandbox lets frename into a folder only when the user chose it (📂, a drop on the
//! window) and only until frename quits. So the Store build keeps a security-scoped bookmark of
//! every folder it opens (`frename_core::folder_bookmarks`), and before opening a folder resolves
//! its bookmark to get that access back: the last folder reopens at the next start. A folder the
//! sandbox refuses (a clip picked or dropped alone, whose folder it does not open) is asked for,
//! with the folder picker already in it (docs/design/mac-app-store.md). Other builds are not
//! sandboxed: nothing here does anything.

use std::path::Path;

/// Get access to `folder` back from an earlier run, and keep the access this run has for the
/// next one. Returns whether the sandbox refuses the folder now; always `false` outside the
/// Store build. Blocks briefly (small files, one system call per bookmark).
pub fn refused(folder: &Path) -> bool {
    #[cfg(all(target_os = "macos", feature = "store"))]
    {
        sandbox::prepare(
            folder,
            &frename_core::folder_bookmarks::BookmarkStore::in_data_dir(),
        );
        frename_core::folder_bookmarks::access_refused(folder)
    }
    #[cfg(not(all(target_os = "macos", feature = "store")))]
    {
        let _ = folder;
        false
    }
}

#[cfg(all(target_os = "macos", feature = "store"))]
mod sandbox {
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, PoisonError};

    use objc2::runtime::Bool;
    use objc2_foundation::{
        NSData, NSString, NSURLBookmarkCreationOptions, NSURLBookmarkResolutionOptions, NSURL,
    };

    use frename_core::folder_bookmarks::BookmarkStore;

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

// Outside the Store build only: in it `refused` asks the real sandbox.
#[cfg(all(test, not(all(target_os = "macos", feature = "store"))))]
mod tests {
    use super::*;

    #[test]
    fn outside_the_sandbox_no_folder_is_refused() {
        assert!(!refused(Path::new("/no such folder")));
    }
}
