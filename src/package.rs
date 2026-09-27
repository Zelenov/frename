//! How this frename was delivered: installed by `frename-win-Setup.exe`, unpacked from the portable
//! zip, or neither (`cargo run`, a bare exe, Linux). Both packages are Velopack packages; they
//! keep frename's own files in the package root, because the folder with the exe (`current\`)
//! is replaced whole on every update.
//!
//! The Microsoft Store build (`--features store`, packed as MSIX) is not a Velopack package: the
//! Store installs and updates it, so it never runs Velopack's hooks or updater, and keeps its
//! files in `%LocalAppData%\frename-store` (docs/design/microsoft-store.md).

use std::path::PathBuf;
use std::sync::OnceLock;

/// Built for the Microsoft Store: the Store installs and updates frename, Velopack does nothing.
pub const STORE_BUILD: bool = cfg!(feature = "store");

/// The package this process runs from, found once at start-up.
static CURRENT: OnceLock<Option<Package>> = OnceLock::new();

/// An installed or portable Velopack package.
#[derive(Debug, Clone)]
pub struct Package {
    /// The package root: `%LocalAppData%\frename` installed, the unzip folder portable.
    pub root: PathBuf,
    /// Unpacked from the portable zip rather than installed.
    pub portable: bool,
    /// The package version, `0.68.0`.
    pub version: String,
}

impl Package {
    /// The package frename runs from, if any. Only Windows has Velopack packages; the Linux
    /// AppImage is built without Velopack, and the Store build is an MSIX package instead.
    #[cfg(windows)]
    pub fn locate() -> Option<Self> {
        if STORE_BUILD {
            return None;
        }
        let locator = velopack::locator::auto_locate_app_manifest(
            velopack::locator::LocationContext::FromCurrentExe,
        )
        .ok()?;
        Some(Self {
            root: locator.get_root_dir(),
            portable: locator.get_is_portable(),
            version: locator.get_manifest_version_full_string(),
        })
    }

    #[cfg(not(windows))]
    pub fn locate() -> Option<Self> {
        None
    }

    /// The folder for frename's own files: the package root.
    pub fn data_dir(&self) -> PathBuf {
        self.root.clone()
    }
}

/// Record the package this process runs from. Called once by `main`; tests never do, so for them
/// frename is not packaged.
pub fn set_current(package: Option<Package>) {
    let _ = CURRENT.set(package);
}

/// The package this process runs from, if any.
pub fn current() -> Option<&'static Package> {
    CURRENT.get().and_then(Option::as_ref)
}

/// Run Velopack's start-up hooks. Must be the first thing `main` does: on install, update and
/// uninstall Velopack starts the exe with hook arguments, and this handles them and exits.
///
/// A downloaded update is not applied at start-up (Velopack's default): it would update without
/// the **Update and restart** button and could close another running frename with unsaved edits.
///
/// Installing (and updating, so an install from before it gets it too) adds "Open in frename" to
/// Explorer's context menu of folders and videos; uninstalling removes it.
///
/// The Store build skips all of it: the Store installs, updates and uninstalls it.
pub fn run_velopack_hooks() {
    if STORE_BUILD {
        return;
    }
    let mut app = velopack::VelopackApp::build().set_auto_apply_on_startup(false);
    #[cfg(windows)]
    {
        app = app
            .on_after_install_fast_callback(|_| explorer_menu::register())
            .on_after_update_fast_callback(|_| explorer_menu::register())
            .on_before_uninstall_fast_callback(|_| explorer_menu::unregister());
    }
    app.run();
}

/// "Open in frename" in Explorer's context menu, for the current user only (`HKCU`, so no admin
/// rights are needed, like the install itself). Explorer runs `frename.exe "<path>"`, which
/// opens the folder, or the video's folder with the video selected.
#[cfg(windows)]
mod explorer_menu {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    /// The menu entries: on folders, and on every file Windows counts as a video.
    const MENU_KEYS: [&str; 2] = [
        r"Software\Classes\Directory\shell\frename",
        r"Software\Classes\SystemFileAssociations\video\shell\frename",
    ];

    /// Add the entries, pointing at this exe (in the package's `current` folder, whose path
    /// stays the same across updates). Failures are ignored: the menu is a convenience and must
    /// not fail the install.
    pub fn register() {
        let Ok(exe) = std::env::current_exe() else {
            return;
        };
        let exe = exe.display().to_string();
        let classes = RegKey::predef(HKEY_CURRENT_USER);
        for key in MENU_KEYS {
            let _ = add_entry(&classes, key, &exe);
        }
    }

    fn add_entry(root: &RegKey, key: &str, exe: &str) -> std::io::Result<()> {
        let (entry, _) = root.create_subkey(key)?;
        entry.set_value("", &"Open in frename")?;
        entry.set_value("Icon", &exe)?;
        let (command, _) = entry.create_subkey("command")?;
        command.set_value("", &format!("\"{exe}\" \"%1\""))
    }

    /// Remove the entries (and their `command` subkeys).
    pub fn unregister() {
        let root = RegKey::predef(HKEY_CURRENT_USER);
        for key in MENU_KEYS {
            let _ = root.delete_subkey_all(key);
        }
    }
}

/// The data folder of the Store build: `%LocalAppData%\frename-store`. `None` for other builds,
/// and when Windows has no local app data folder. Its own folder, not the installed version's
/// `%LocalAppData%\frename`: the two builds may run side by side, at different versions of the
/// database, and Windows puts a packaged app's new files in the package's own storage, where the
/// other build cannot see them. The Store removes it with the app.
pub fn store_data_dir() -> Option<PathBuf> {
    local_app_data_folder(STORE_DATA_FOLDER)
}

/// The installed version's data folder, `%LocalAppData%\frename`, whose settings the Store build
/// takes on its first start. `None` for other builds.
pub fn installed_data_dir() -> Option<PathBuf> {
    local_app_data_folder(INSTALLED_DATA_FOLDER)
}

const STORE_DATA_FOLDER: &str = "frename-store";

/// The installed version's package root in `%LocalAppData%`: Velopack names it after the pack id,
/// `vpk pack -u frename` in the workflows. Renaming one without the other would make the Store
/// build's first start miss the installed version's settings.
const INSTALLED_DATA_FOLDER: &str = "frename";

/// `%LocalAppData%\<name>` in the Store build; `None` otherwise.
fn local_app_data_folder(name: &str) -> Option<PathBuf> {
    STORE_BUILD
        .then(dirs::data_local_dir)
        .flatten()
        .map(|local| local.join(name))
}

/// The log as other apps see it. The Store build's data folder is new in `%LocalAppData%`, so
/// Windows keeps it in the package's own storage, which only frename's package sees at
/// `%LocalAppData%\frename-store`; Explorer and the editor it opens the log in run outside the
/// package and find it at `%LocalAppData%\Packages\<family name>\LocalCache\Local\frename-store`.
/// Every other build, and a Store build run outside its package, returns [`frename_core::log_path`].
pub fn log_path_for_other_apps() -> PathBuf {
    let log = frename_core::log_path();
    let packaged = STORE_BUILD
        .then(|| {
            let exe = std::env::current_exe().ok()?;
            let family = package_family_name(exe.parent()?.file_name()?.to_str()?)?;
            let local = dirs::data_local_dir()?;
            let relative = log.strip_prefix(&local).ok()?;
            Some(
                local
                    .join("Packages")
                    .join(family)
                    .join("LocalCache")
                    .join("Local")
                    .join(relative),
            )
        })
        .flatten();
    packaged.filter(|path| path.exists()).unwrap_or(log)
}

/// The package family name (`Name_PublisherId`) from the folder an MSIX package is installed in,
/// its full name `Name_Version_Architecture_ResourceId_PublisherId`
/// (`frename.dev_0.74.0.0_x64__mjf909wv0ft0r`). Package names cannot contain `_`.
fn package_family_name(install_folder: &str) -> Option<String> {
    let parts: Vec<&str> = install_folder.split('_').collect();
    match parts.as_slice() {
        [name, _version, _arch, _resource, publisher_id]
            if !name.is_empty() && !publisher_id.is_empty() =>
        {
            Some(format!("{name}_{publisher_id}"))
        }
        _ => None,
    }
}

/// Whether frename keeps its data away from the exe (a Velopack package or the Store build), so
/// settings of an older zip version next to some exe can be imported.
pub fn keeps_data_away_from_exe(package: Option<&Package>) -> bool {
    package.is_some() || STORE_BUILD
}

/// The running version as the UI shows it: `0.68`, not Velopack's `0.68.0`. Packaged builds know
/// it from the package, release builds from `APP_VERSION` at build time; otherwise `dev`.
pub fn display_version(package: Option<&Package>) -> String {
    package
        .map(|p| p.version.clone())
        .or_else(|| option_env!("APP_VERSION").map(str::to_string))
        .map_or_else(|| "dev".to_string(), |v| short_version(&v))
}

/// A version as `version.md` writes it: a trailing `.0` of a 3-part version is dropped
/// (`0.68.0` -> `0.68`, `0.68.1` stays).
pub fn short_version(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() == 3 && parts[2] == "0" {
        format!("{}.{}", parts[0], parts[1])
    } else {
        version.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trailing_zero_patch_is_dropped() {
        assert_eq!(short_version("0.68.0"), "0.68");
        assert_eq!(short_version("0.68.1"), "0.68.1");
        assert_eq!(short_version("1.0"), "1.0");
        assert_eq!(short_version("dev"), "dev");
    }

    #[test]
    fn a_package_version_wins_over_the_build_version() {
        let package = Package {
            root: PathBuf::from("root"),
            portable: false,
            version: "0.70.0".to_string(),
        };
        assert_eq!(display_version(Some(&package)), "0.70");
    }

    #[test]
    fn the_store_build_keeps_its_own_folder_next_to_the_installed_versions() {
        assert_eq!(STORE_DATA_FOLDER, "frename-store");
        assert_eq!(INSTALLED_DATA_FOLDER, "frename");
        let local = dirs::data_local_dir().filter(|_| STORE_BUILD);
        assert_eq!(
            store_data_dir(),
            local.as_ref().map(|l| l.join("frename-store"))
        );
        assert_eq!(installed_data_dir(), local.map(|l| l.join("frename")));
    }

    #[test]
    fn the_package_family_name_comes_from_the_install_folder() {
        assert_eq!(
            package_family_name("frename.dev_0.0.1.0_x64__mjf909wv0ft0r").as_deref(),
            Some("frename.dev_mjf909wv0ft0r")
        );
        assert_eq!(
            package_family_name("12345EugeneZelenov.frename_0.74.0.0_x64__8wekyb3d8bbwe")
                .as_deref(),
            Some("12345EugeneZelenov.frename_8wekyb3d8bbwe")
        );
        // Not a package folder: a Velopack `current`, a cargo target folder.
        assert_eq!(package_family_name("current"), None);
        assert_eq!(package_family_name("debug"), None);
    }

    #[test]
    fn outside_the_store_package_the_log_is_where_frename_writes_it() {
        if !STORE_BUILD {
            assert_eq!(log_path_for_other_apps(), frename_core::log_path());
        }
    }

    #[test]
    fn a_package_or_the_store_build_keeps_its_data_away_from_the_exe() {
        let package = Package {
            root: PathBuf::from("root"),
            portable: false,
            version: "0.70.0".to_string(),
        };
        assert!(keeps_data_away_from_exe(Some(&package)));
        assert_eq!(keeps_data_away_from_exe(None), STORE_BUILD);
    }

    #[test]
    fn a_package_keeps_its_data_in_its_root() {
        let package = Package {
            root: PathBuf::from("root"),
            portable: true,
            version: "0.70.0".to_string(),
        };
        assert_eq!(package.data_dir(), PathBuf::from("root"));
    }
}
