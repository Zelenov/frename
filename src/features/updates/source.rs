//! The update source: frename's GitHub releases, read by Velopack. Every call blocks (network),
//! so the state runs them in `spawn_blocking`.

use std::sync::mpsc::Sender;
use std::sync::Arc;

use velopack::sources::GithubSource;
use velopack::{UpdateCheck, UpdateManager};

use super::Release;

/// Where releases are published. Drafts (branch builds) and prereleases are not offered.
const REPOSITORY: &str = "https://github.com/Zelenov/frename";

fn manager() -> Result<UpdateManager, velopack::Error> {
    UpdateManager::new(GithubSource::new(REPOSITORY, None, false), None, None)
}

/// Look for a release newer than the running one.
pub fn check() -> Result<Option<Release>, String> {
    let manager = manager().map_err(|e| describe(&e))?;
    match manager.check_for_updates().map_err(|e| describe(&e))? {
        UpdateCheck::UpdateAvailable(info) => Ok(Some(Release(Arc::from(info)))),
        UpdateCheck::NoUpdateAvailable | UpdateCheck::RemoteIsEmpty => Ok(None),
    }
}

/// Download `release` into the package, reporting progress in percent.
pub fn download(release: &Release, progress: Sender<i16>) -> Result<(), String> {
    let manager = manager().map_err(|e| describe(&e))?;
    if other_frename_running(&manager) {
        return Err("Close the other frename window first".to_string());
    }
    manager
        .download_updates(&release.0, Some(progress))
        .map_err(|e| describe(&e))
}

/// Start Velopack's updater, which waits for this process to exit, applies `release` and starts
/// the new version. Call right before exiting.
pub fn apply_on_exit(release: &Release) -> Result<(), String> {
    let manager = manager().map_err(|e| describe(&e))?;
    manager
        .wait_exit_then_apply_updates(&*release.0, false, true, Vec::<String>::new())
        .map_err(|e| describe(&e))
}

/// A short reason for the UI: `Could not check for updates: <reason>`.
fn describe(error: &velopack::Error) -> String {
    match error {
        velopack::Error::Network(network) => match network.as_ref() {
            velopack::NetworkError::Http(ureq::Error::StatusCode(_)) => {
                "GitHub did not respond, try later".to_string()
            }
            _ => "no connection".to_string(),
        },
        velopack::Error::Io(io) if io.kind() == std::io::ErrorKind::PermissionDenied => {
            "the folder is read-only".to_string()
        }
        velopack::Error::NotInstalled(_) => "updates work in the installed version".to_string(),
        other => other.to_string(),
    }
}

/// Whether another frename from the same package is running: applying the update would close it
/// with its unsaved edits.
#[cfg(windows)]
fn other_frename_running(manager: &UpdateManager) -> bool {
    let _ = manager;
    let Some(root) = crate::package::current().map(|p| p.root.clone()) else {
        return false;
    };
    windows_processes::other_process_under(&root)
}

#[cfg(not(windows))]
fn other_frename_running(_manager: &UpdateManager) -> bool {
    false
}

#[cfg(windows)]
mod windows_processes {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::{Path, PathBuf};

    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    /// Whether a process other than this one runs an exe inside `root`.
    pub fn other_process_under(root: &Path) -> bool {
        let own = std::process::id();
        process_ids()
            .into_iter()
            .filter(|pid| *pid != own)
            .filter_map(image_path)
            .any(|path| path.starts_with(root) && is_frename(&path))
    }

    fn is_frename(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("frename.exe"))
    }

    fn process_ids() -> Vec<u32> {
        let mut ids = Vec::new();
        // SAFETY: the snapshot handle is checked and closed; the entry is sized as the API needs.
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return ids;
            }
            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            let mut more = Process32FirstW(snapshot, &mut entry) != 0;
            while more {
                ids.push(entry.th32ProcessID);
                more = Process32NextW(snapshot, &mut entry) != 0;
            }
            CloseHandle(snapshot);
        }
        ids
    }

    fn image_path(pid: u32) -> Option<PathBuf> {
        // SAFETY: the process handle is checked and closed; the buffer length is passed in.
        unsafe {
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if process.is_null() {
                return None;
            }
            let mut buffer = [0u16; 32768];
            let mut length = buffer.len() as u32;
            let ok = QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                buffer.as_mut_ptr(),
                &mut length,
            ) != 0;
            CloseHandle(process);
            ok.then(|| PathBuf::from(OsString::from_wide(&buffer[..length as usize])))
        }
    }
}
