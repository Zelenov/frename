//! The Windows file drag: the shell's own data object for the files (what Explorer hands out when
//! its files are dragged: `CF_HDROP`, `Shell IDList Array`, a drag image) and `SHDoDragDrop`
//! with the shell's default drop source. OLE is initialised on the window's thread by winit,
//! which registers its drop target there.

use std::path::PathBuf;

use iced::window::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::core::HSTRING;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{IBindCtx, IDataObject};
use windows::Win32::System::Ole::{IDropSource, DROPEFFECT_COPY, DROPEFFECT_LINK, DROPEFFECT_NONE};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::{
    BHID_DataObject, ILFree, IShellItemArray, SHCreateShellItemArrayFromIDLists, SHDoDragDrop,
    SHParseDisplayName,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SWAPBUTTON};

use super::Outcome;

/// Run the drag loop for `paths` from `window`.
pub fn start(window: &dyn HasWindowHandle, paths: &[PathBuf]) -> Outcome {
    // The drop source drops when the button is up: started after a release, the files would
    // land wherever the pointer is.
    if !primary_button_down() {
        log::info!("drag out: the button is already up, no drag");
        return Outcome::NotStarted;
    }
    let hwnd = match window.window_handle().map(|handle| handle.as_raw()) {
        Ok(RawWindowHandle::Win32(handle)) => Some(HWND(handle.hwnd.get() as *mut _)),
        _ => None,
    };
    let data = match data_object(paths) {
        Ok(data) => data,
        Err(error) => {
            log::error!("drag out: no data object for {paths:?}: {error}");
            return Outcome::Failed(error.to_string());
        }
    };
    log::info!("drag out: {paths:?}");
    // Copy and link only: Explorer copies, never moves the files away.
    // SAFETY: called on the window's thread, where winit initialised OLE; `data` is a live
    // COM object; a missing drop source makes the shell supply its default one.
    let result = unsafe {
        SHDoDragDrop(
            hwnd,
            &data,
            None::<&IDropSource>,
            DROPEFFECT_COPY | DROPEFFECT_LINK,
        )
    };
    match result {
        Ok(effect) if effect != DROPEFFECT_NONE => {
            log::info!("drag out: dropped ({:#x})", effect.0);
            Outcome::Dropped
        }
        Ok(_) => {
            log::info!("drag out: cancelled");
            Outcome::Cancelled
        }
        Err(error) => {
            log::error!("drag out: drag loop failed: {error}");
            Outcome::Failed(error.to_string())
        }
    }
}

/// Whether the primary mouse button is held. `GetAsyncKeyState` reads the physical buttons, so
/// with swapped buttons the primary one is the right one.
fn primary_button_down() -> bool {
    // SAFETY: plain Win32 queries without pointers.
    unsafe {
        let key = if GetSystemMetrics(SM_SWAPBUTTON) != 0 {
            VK_RBUTTON
        } else {
            VK_LBUTTON
        };
        GetAsyncKeyState(i32::from(key.0)) < 0
    }
}

/// Item ID lists freed when dropped.
struct IdLists(Vec<*mut ITEMIDLIST>);

impl Drop for IdLists {
    fn drop(&mut self) {
        for pidl in self.0.drain(..) {
            // SAFETY: each pointer came from `SHParseDisplayName` and is freed once.
            unsafe { ILFree(Some(pidl.cast_const())) };
        }
    }
}

/// The shell's data object for `paths` (files of one folder), as Explorer builds it.
pub(super) fn data_object(paths: &[PathBuf]) -> windows::core::Result<IDataObject> {
    let mut ids = IdLists(Vec::with_capacity(paths.len()));
    for path in paths {
        let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
        // SAFETY: `pidl` receives an ID list owned by `ids` from here on.
        unsafe {
            SHParseDisplayName(
                &HSTRING::from(path.as_os_str()),
                None::<&IBindCtx>,
                &mut pidl,
                0,
                None,
            )?
        };
        ids.0.push(pidl);
    }
    let lists: Vec<*const ITEMIDLIST> = ids.0.iter().map(|pidl| pidl.cast_const()).collect();
    // SAFETY: the ID lists stay alive until after the data object is made; the shell copies them.
    unsafe {
        let items: IShellItemArray = SHCreateShellItemArrayFromIDLists(&lists)?;
        items.BindToHandler(None::<&IBindCtx>, &BHID_DataObject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Com::{DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL};
    use windows::Win32::System::Ole::{OleInitialize, ReleaseStgMedium, CF_HDROP};
    use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};

    /// The data Premiere and Explorer read: `CF_HDROP` of the shell's data object holds exactly
    /// the dragged paths.
    #[test]
    fn the_data_object_carries_the_paths_as_an_hdrop() {
        let dir = std::env::temp_dir().join(format!("frename-drag-out-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let paths: Vec<PathBuf> = ["Goat.Food.clip01.mp4", "clip 02 — ёж.mov"]
            .iter()
            .map(|name| dir.join(name))
            .collect();
        for path in &paths {
            std::fs::write(path, b"x").expect("file");
        }

        // SAFETY: OLE on this test thread (harmless when already on); the medium is released.
        let dropped = unsafe {
            let _ = OleInitialize(None);
            let data = data_object(&paths).expect("data object");
            let format = FORMATETC {
                cfFormat: CF_HDROP.0,
                ptd: std::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT.0,
                lindex: -1,
                tymed: TYMED_HGLOBAL.0 as u32,
            };
            let mut medium = data.GetData(&format).expect("CF_HDROP");
            let drop = HDROP(medium.u.hGlobal.0);
            let count = DragQueryFileW(drop, u32::MAX, None);
            let mut dropped = Vec::new();
            for index in 0..count {
                let len = DragQueryFileW(drop, index, None) as usize;
                let mut buffer = vec![0u16; len + 1];
                let written = DragQueryFileW(drop, index, Some(&mut buffer)) as usize;
                dropped.push(PathBuf::from(String::from_utf16_lossy(&buffer[..written])));
            }
            ReleaseStgMedium(&mut medium);
            dropped
        };
        // The shell spells the paths out in full: a temp folder given by its 8.3 short name
        // (`RUNNER~1` on CI) comes back long, so the files are compared, not the spellings.
        let same_files = |list: &[PathBuf]| -> Vec<PathBuf> {
            list.iter()
                .map(|path| std::fs::canonicalize(path).expect("file exists"))
                .collect()
        };
        let (dropped, paths) = (same_files(&dropped), same_files(&paths));
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(dropped, paths);
    }
}
