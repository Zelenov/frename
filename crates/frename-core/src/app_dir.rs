//! Where frename keeps its own files: the database and the log.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The data folder of an installed, portable or Store package, set once by the app at start-up.
static PACKAGE_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// The folder for frename's own files: next to the executable, except when frename runs from an
/// AppImage, whose folder is a read-only mount; then `$XDG_DATA_HOME/frename`, by default
/// `~/.local/share/frename`. An installed or portable Windows package keeps them in the folder
/// set with [`set_app_data_dir`], because the executable's folder is replaced on every update; so
/// does the Microsoft Store build, whose executable's folder is read-only.
/// An absolute `FRENAME_DATA_DIR` overrides all of these (demo mode uses it to keep the user's
/// database untouched). The folder may not exist yet.
pub fn app_data_dir() -> PathBuf {
    data_dir_for(
        std::env::current_exe().ok().as_deref(),
        PACKAGE_DATA_DIR.get().map(PathBuf::as_path),
        |name| std::env::var_os(name),
    )
}

/// Set the data folder of the package frename runs from (Velopack or Microsoft Store). Called once
/// by `main`, before anything reads [`app_data_dir`]; a second call is ignored. Tests do not call
/// it.
pub fn set_app_data_dir(dir: PathBuf) {
    if PACKAGE_DATA_DIR.set(dir).is_err() {
        log::warn!("set_app_data_dir called twice; keeping the first folder");
    }
}

/// The log file, in [`app_data_dir`]. Started afresh on each run.
pub fn log_path() -> PathBuf {
    app_data_dir().join("frename_debug.log")
}

/// Environment variable that sets [`app_data_dir`] outright.
pub const DATA_DIR_VAR: &str = "FRENAME_DATA_DIR";

/// [`app_data_dir`] for a given executable path, package data folder and environment.
fn data_dir_for(
    exe: Option<&Path>,
    package: Option<&Path>,
    var: impl Fn(&str) -> Option<OsString>,
) -> PathBuf {
    if let Some(dir) = var(DATA_DIR_VAR)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        return dir;
    }
    if let Some(dir) = package {
        return dir.to_path_buf();
    }
    if runs_from_appimage(exe, &var) {
        let absolute = |name: &str| {
            var(name)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
        };
        // The XDG spec says to ignore a relative XDG_DATA_HOME.
        return absolute("XDG_DATA_HOME")
            .or_else(|| absolute("HOME").map(|home| home.join(".local/share")))
            .map_or_else(
                || std::env::temp_dir().join("frename"),
                |base| base.join("frename"),
            );
    }
    exe.and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir)
}

/// Whether `exe` is inside a mounted AppImage. The AppImage runtime sets `APPIMAGE` and `APPDIR`
/// and passes them on to child processes, so a frename started from another AppImage's terminal
/// also sees them; only an executable inside `APPDIR` is the AppImage's own.
fn runs_from_appimage(exe: Option<&Path>, var: &impl Fn(&str) -> Option<OsString>) -> bool {
    let (Some(exe), Some(_), Some(appdir)) = (exe, var("APPIMAGE"), var("APPDIR")) else {
        return false;
    };
    if appdir.is_empty() {
        return false;
    }
    let appdir = Path::new(&appdir);
    // `APPDIR` is the mount under `TMPDIR`, which may be behind a symlink, while the running
    // executable's path is resolved; compare with the resolved folder too.
    exe.starts_with(appdir)
        || std::fs::canonicalize(appdir).is_ok_and(|resolved| exe.starts_with(resolved))
}

/// The plugin registry of the GStreamer inside the AppImage, or `None` outside one. GStreamer's
/// default, `~/.cache/gstreamer-1.0/registry.x86_64.bin`, is shared with the system's own
/// GStreamer apps and is rebuilt on every start, because the mount path changes each time.
/// frename keeps its own in `$XDG_CACHE_HOME/frename`, by default `~/.cache/frename`.
pub fn appimage_gstreamer_registry() -> Option<PathBuf> {
    gstreamer_registry_for(
        std::env::current_exe().ok().as_deref(),
        PACKAGE_DATA_DIR.get().map(PathBuf::as_path),
        |name| std::env::var_os(name),
    )
}

fn gstreamer_registry_for(
    exe: Option<&Path>,
    package: Option<&Path>,
    var: impl Fn(&str) -> Option<OsString>,
) -> Option<PathBuf> {
    if package.is_some() || !runs_from_appimage(exe, &var) {
        return None;
    }
    let absolute = |name: &str| {
        var(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    // The XDG spec says to ignore a relative XDG_CACHE_HOME.
    let folder = absolute("XDG_CACHE_HOME")
        .or_else(|| absolute("HOME").map(|home| home.join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("frename");
    Some(folder.join("gst-registry.bin"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `path` made absolute on this platform: Windows needs a drive for a path to be absolute.
    fn abs(path: &str) -> String {
        if cfg!(windows) {
            format!("C:{path}")
        } else {
            path.to_string()
        }
    }

    /// An environment holding exactly `pairs`.
    fn env(pairs: Vec<(&'static str, String)>) -> impl Fn(&str) -> Option<OsString> {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        }
    }

    fn exe() -> PathBuf {
        PathBuf::from(abs("/opt/frename/frename"))
    }
    fn mounted_exe() -> PathBuf {
        PathBuf::from(abs("/tmp/.mount_frenam1/usr/bin/frename"))
    }
    fn appimage() -> (&'static str, String) {
        ("APPIMAGE", abs("/home/ed/frename.AppImage"))
    }
    fn appdir() -> (&'static str, String) {
        ("APPDIR", abs("/tmp/.mount_frenam1"))
    }
    fn home() -> (&'static str, String) {
        ("HOME", abs("/home/ed"))
    }

    #[test]
    fn next_to_the_executable_normally() {
        assert_eq!(
            data_dir_for(Some(&exe()), None, env(vec![home()])),
            PathBuf::from(abs("/opt/frename"))
        );
    }

    #[test]
    fn in_the_xdg_data_folder_inside_an_appimage() {
        assert_eq!(
            data_dir_for(
                Some(&mounted_exe()),
                None,
                env(vec![
                    appimage(),
                    appdir(),
                    ("XDG_DATA_HOME", abs("/data")),
                    home()
                ])
            ),
            PathBuf::from(abs("/data")).join("frename")
        );
        for xdg in ["", "relative/data"] {
            assert_eq!(
                data_dir_for(
                    Some(&mounted_exe()),
                    None,
                    env(vec![
                        appimage(),
                        appdir(),
                        ("XDG_DATA_HOME", xdg.to_string()),
                        home()
                    ])
                ),
                PathBuf::from(abs("/home/ed")).join(".local/share/frename"),
                "XDG_DATA_HOME={xdg:?}"
            );
        }
    }

    #[test]
    fn an_appimage_without_a_home_uses_its_own_temp_subfolder_not_shared_temp() {
        assert_eq!(
            data_dir_for(Some(&mounted_exe()), None, env(vec![appimage(), appdir()])),
            std::env::temp_dir().join("frename")
        );
    }

    #[test]
    fn the_appimage_keeps_its_gstreamer_registry_in_its_own_cache_folder() {
        let registry = |pairs| gstreamer_registry_for(Some(&mounted_exe()), None, env(pairs));
        assert_eq!(
            registry(vec![
                appimage(),
                appdir(),
                ("XDG_CACHE_HOME", abs("/cache")),
                home()
            ]),
            Some(PathBuf::from(abs("/cache")).join("frename/gst-registry.bin"))
        );
        for xdg in ["", "relative/cache"] {
            assert_eq!(
                registry(vec![
                    appimage(),
                    appdir(),
                    ("XDG_CACHE_HOME", xdg.to_string()),
                    home()
                ]),
                Some(PathBuf::from(abs("/home/ed")).join(".cache/frename/gst-registry.bin")),
                "XDG_CACHE_HOME={xdg:?}"
            );
        }
        assert_eq!(
            registry(vec![appimage(), appdir()]),
            Some(std::env::temp_dir().join("frename/gst-registry.bin"))
        );
    }

    #[test]
    fn no_gstreamer_registry_outside_an_appimage() {
        let vars = vec![appimage(), appdir(), home()];
        assert_eq!(
            gstreamer_registry_for(Some(&exe()), None, env(vars.clone())),
            None
        );
        assert_eq!(
            gstreamer_registry_for(Some(&mounted_exe()), None, env(vec![home()])),
            None
        );
        assert_eq!(
            gstreamer_registry_for(Some(&mounted_exe()), Some(Path::new("/pkg")), env(vars)),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_appdir_behind_a_symlink_still_finds_its_executable() {
        let base = std::env::temp_dir().join(format!("frename-appdir-link-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let real = base.join("real/.mount_frenam1");
        std::fs::create_dir_all(real.join("usr/bin")).expect("mount");
        std::os::unix::fs::symlink(base.join("real"), base.join("link")).expect("symlink");
        let appdir = base.join("link/.mount_frenam1");
        let exe = std::fs::canonicalize(&real)
            .expect("resolved")
            .join("usr/bin/frename");
        let vars = vec![
            appimage(),
            ("APPDIR", appdir.to_string_lossy().into_owned()),
            home(),
        ];
        assert_eq!(
            data_dir_for(Some(&exe), None, env(vars)),
            PathBuf::from(abs("/home/ed")).join(".local/share/frename")
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn appimage_variables_inherited_from_another_appimage_are_ignored() {
        assert_eq!(
            data_dir_for(Some(&exe()), None, env(vec![appimage(), appdir(), home()])),
            PathBuf::from(abs("/opt/frename"))
        );
    }

    #[test]
    fn an_empty_appdir_is_no_appimage() {
        // An empty path is a prefix of every path; without the guard every exe would qualify.
        assert_eq!(
            data_dir_for(
                Some(&exe()),
                None,
                env(vec![appimage(), ("APPDIR", String::new()), home()])
            ),
            PathBuf::from(abs("/opt/frename"))
        );
    }

    #[test]
    fn an_absolute_frename_data_dir_wins_even_inside_an_appimage() {
        let data = ("FRENAME_DATA_DIR", abs("/tmp/demo/data"));
        assert_eq!(
            data_dir_for(Some(&exe()), None, env(vec![data.clone(), home()])),
            PathBuf::from(abs("/tmp/demo/data"))
        );
        assert_eq!(
            data_dir_for(
                Some(&mounted_exe()),
                None,
                env(vec![data, appimage(), appdir(), home()])
            ),
            PathBuf::from(abs("/tmp/demo/data"))
        );
    }

    #[test]
    fn a_relative_or_empty_frename_data_dir_is_ignored() {
        for value in ["", "data"] {
            assert_eq!(
                data_dir_for(
                    Some(&exe()),
                    None,
                    env(vec![("FRENAME_DATA_DIR", value.to_string()), home()])
                ),
                PathBuf::from(abs("/opt/frename")),
                "{value:?}"
            );
        }
    }

    #[test]
    fn a_package_keeps_its_data_in_its_own_folder_unless_frename_data_dir_is_set() {
        let package = PathBuf::from(abs("/Users/ed/AppData/Local/frename"));
        assert_eq!(
            data_dir_for(Some(&exe()), Some(&package), env(vec![home()])),
            package
        );
        assert_eq!(
            data_dir_for(
                Some(&exe()),
                Some(&package),
                env(vec![("FRENAME_DATA_DIR", abs("/tmp/demo/data"))])
            ),
            PathBuf::from(abs("/tmp/demo/data"))
        );
    }

    #[test]
    fn the_temp_folder_when_the_executable_is_unknown() {
        assert_eq!(data_dir_for(None, None, env(vec![])), std::env::temp_dir());
    }
}
