//! First start of an installed or portable frename: offer the settings of a zip version found in
//! the places a downloaded zip gets unpacked. Nothing found, nothing asked.

use std::path::{Path, PathBuf};

use frename_core::old_settings;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

/// Run the import scheduled from Settings, or on a first start offer one found by searching.
/// Call before the database is opened.
pub fn run(data_dir: &Path) {
    match old_settings::run_scheduled_import(data_dir) {
        Ok(Some(folder)) => log::info!("imported settings from {}", folder.display()),
        Ok(None) => {}
        Err(e) => log::error!("importing the scheduled settings failed: {e}"),
    }
    if !old_settings::import_offer_due(data_dir) {
        return;
    }
    let found: Vec<PathBuf> =
        old_settings::find_old_frename_folders(&search_roots(), old_settings::SEARCH_DEPTH)
            .into_iter()
            // A portable frename's root looks like an old one; this one's own is not.
            .filter(|folder| !same_folder(folder, data_dir))
            .collect();
    let Some(folder) = found.first() else {
        return;
    };
    log::info!("found settings of an older frename in {}", folder.display());
    let answer = MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title("frename")
        .set_description(format!(
            "Import your settings and recent folders from {}?\n\n\
             Yes: import them.\nNo: start fresh.\nCancel: ask again next time.",
            folder.display()
        ))
        .set_buttons(MessageButtons::YesNoCancel)
        .show();
    let ask_again = match answer {
        MessageDialogResult::Yes => {
            match old_settings::import_into_data_dir(data_dir, folder) {
                Ok(()) => log::info!("imported settings from {}", folder.display()),
                Err(e) => log::error!("importing settings from {} failed: {e}", folder.display()),
            }
            false
        }
        MessageDialogResult::No => false,
        _ => true,
    };
    if let Err(e) = old_settings::set_import_offer_again(data_dir, ask_again) {
        log::warn!("could not record the answer to the settings import: {e}");
    }
}

/// Downloads, Desktop and Documents, where Windows keeps them (they may be moved to OneDrive).
fn search_roots() -> Vec<PathBuf> {
    [
        dirs::download_dir(),
        dirs::desktop_dir(),
        dirs::document_dir(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn same_folder(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}
