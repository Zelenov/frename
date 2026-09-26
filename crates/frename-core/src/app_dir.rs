//! Where frename keeps its own files: the database and the log.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The folder for frename's own files: next to the executable, except when frename runs from an
/// AppImage, whose folder is a read-only mount; then `$XDG_DATA_HOME/frename`, by default
/// `~/.local/share/frename`. The folder may not exist yet.
pub fn app_data_dir() -> PathBuf {
    data_dir_for(std::env::current_exe().ok().as_deref(), |name| {
        std::env::var_os(name)
    })
}

/// [`app_data_dir`] for a given executable path and environment.
fn data_dir_for(exe: Option<&Path>, var: impl Fn(&str) -> Option<OsString>) -> PathBuf {
    if runs_from_appimage(exe, &var) {
        let absolute = |name: &str| {
            var(name)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
        };
        // The XDG spec says to ignore a relative XDG_DATA_HOME.
        return absolute("XDG_DATA_HOME")
            .or_else(|| absolute("HOME").map(|home| home.join(".local/share")))
            .map_or_else(std::env::temp_dir, |base| base.join("frename"));
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
    !appdir.is_empty() && exe.starts_with(Path::new(&appdir))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<OsString> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        }
    }

    const EXE: &str = "/opt/frename/frename";
    const MOUNTED_EXE: &str = "/tmp/.mount_frenam1/usr/bin/frename";
    const APPIMAGE: (&str, &str) = ("APPIMAGE", "/home/ed/frename.AppImage");
    const APPDIR: (&str, &str) = ("APPDIR", "/tmp/.mount_frenam1");

    #[test]
    fn next_to_the_executable_normally() {
        assert_eq!(
            data_dir_for(Some(Path::new(EXE)), env(&[("HOME", "/home/ed")])),
            PathBuf::from("/opt/frename")
        );
    }

    #[test]
    fn in_the_xdg_data_folder_inside_an_appimage() {
        let exe = Some(Path::new(MOUNTED_EXE));
        assert_eq!(
            data_dir_for(
                exe,
                env(&[
                    APPIMAGE,
                    APPDIR,
                    ("XDG_DATA_HOME", "/data"),
                    ("HOME", "/home/ed")
                ])
            ),
            PathBuf::from("/data/frename")
        );
        for xdg in ["", "relative/data"] {
            assert_eq!(
                data_dir_for(
                    exe,
                    env(&[
                        APPIMAGE,
                        APPDIR,
                        ("XDG_DATA_HOME", xdg),
                        ("HOME", "/home/ed")
                    ])
                ),
                PathBuf::from("/home/ed/.local/share/frename"),
                "XDG_DATA_HOME={xdg:?}"
            );
        }
    }

    #[test]
    fn an_appimage_without_a_home_uses_the_temp_folder_not_the_read_only_mount() {
        assert_eq!(
            data_dir_for(Some(Path::new(MOUNTED_EXE)), env(&[APPIMAGE, APPDIR])),
            std::env::temp_dir()
        );
    }

    #[test]
    fn appimage_variables_inherited_from_another_appimage_are_ignored() {
        assert_eq!(
            data_dir_for(
                Some(Path::new(EXE)),
                env(&[APPIMAGE, APPDIR, ("HOME", "/home/ed")])
            ),
            PathBuf::from("/opt/frename")
        );
    }

    #[test]
    fn the_temp_folder_when_the_executable_is_unknown() {
        assert_eq!(data_dir_for(None, env(&[])), std::env::temp_dir());
    }
}
