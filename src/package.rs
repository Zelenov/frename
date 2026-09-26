//! How this frename was delivered: installed by `frename-win-Setup.exe`, unpacked from the portable
//! zip, or neither (`cargo run`, a bare exe, Linux). Both packages are Velopack packages; they
//! keep frename's own files in the package root, because the folder with the exe (`current\`)
//! is replaced whole on every update.

use std::path::PathBuf;
use std::sync::OnceLock;

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
    /// The package frename runs from, if any.
    pub fn locate() -> Option<Self> {
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
pub fn run_velopack_hooks() {
    velopack::VelopackApp::build()
        .set_auto_apply_on_startup(false)
        .run();
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
    fn a_package_keeps_its_data_in_its_root() {
        let package = Package {
            root: PathBuf::from("root"),
            portable: true,
            version: "0.70.0".to_string(),
        };
        assert_eq!(package.data_dir(), PathBuf::from("root"));
    }
}
