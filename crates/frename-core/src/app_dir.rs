//! Where frename keeps its own files: the database and the log.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The folder for frename's own files: next to the executable, except when frename runs as an
/// AppImage, whose folder is a read-only mount; then `$XDG_DATA_HOME/frename`, by default
/// `~/.local/share/frename`. The folder may not exist yet.
pub fn app_data_dir() -> PathBuf {
    data_dir_for(std::env::current_exe().ok().as_deref(), |name| {
        std::env::var_os(name)
    })
}

/// [`app_data_dir`] for a given executable path and environment.
fn data_dir_for(exe: Option<&Path>, var: impl Fn(&str) -> Option<OsString>) -> PathBuf {
    // The AppImage runtime sets APPIMAGE to the path of the image it runs.
    if var("APPIMAGE").is_some() {
        let non_empty = |name: &str| var(name).filter(|value| !value.is_empty());
        let base = non_empty("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| non_empty("HOME").map(|home| Path::new(&home).join(".local/share")));
        if let Some(base) = base {
            return base.join("frename");
        }
    }
    exe.and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir)
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

    #[test]
    fn next_to_the_executable_normally() {
        assert_eq!(
            data_dir_for(Some(Path::new(EXE)), env(&[("HOME", "/home/ed")])),
            PathBuf::from("/opt/frename")
        );
    }

    #[test]
    fn in_the_xdg_data_folder_inside_an_appimage() {
        let appimage = ("APPIMAGE", "/home/ed/frename.AppImage");
        assert_eq!(
            data_dir_for(
                Some(Path::new(EXE)),
                env(&[appimage, ("XDG_DATA_HOME", "/data"), ("HOME", "/home/ed")])
            ),
            PathBuf::from("/data/frename")
        );
        assert_eq!(
            data_dir_for(
                Some(Path::new(EXE)),
                env(&[appimage, ("XDG_DATA_HOME", ""), ("HOME", "/home/ed")])
            ),
            PathBuf::from("/home/ed/.local/share/frename")
        );
    }

    #[test]
    fn an_appimage_without_a_home_falls_back_to_the_executable_folder() {
        assert_eq!(
            data_dir_for(Some(Path::new(EXE)), env(&[("APPIMAGE", "/x.AppImage")])),
            PathBuf::from("/opt/frename")
        );
    }

    #[test]
    fn the_temp_folder_when_the_executable_is_unknown() {
        assert_eq!(data_dir_for(None, env(&[])), std::env::temp_dir());
    }
}
