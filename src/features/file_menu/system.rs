//! What the file menu's actions hand to the system: the text put on the clipboard (by the
//! workspace, which keeps the clipboard open), and showing a file in Explorer or the Linux file
//! manager. The text is built by plain functions, tested
//! without a clipboard, Explorer or a D-Bus session.

use std::path::{Path, PathBuf};

use super::FileAction;

/// The text a copy action puts on the clipboard for the file at `path`: the full path as the
/// system shows it (`D:\footage\clip.mp4`), or the name with its extension (`clip.mp4`).
/// `None` for an action that copies nothing.
pub fn clipboard_text(action: FileAction, path: &Path) -> Option<String> {
    match action {
        FileAction::CopyPath => Some(path_text(path)),
        FileAction::CopyName => Some(name_text(path)),
        FileAction::ShowInFileManager => None,
    }
}

/// The full path as Explorer shows it: without the `\\?\` prefix Windows uses for long paths
/// (`\\?\UNC\server\share` becomes `\\server\share`).
pub fn path_text(path: &Path) -> String {
    let text = path.display().to_string();
    if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{unc}");
    }
    match text.strip_prefix(r"\\?\") {
        Some(plain) => plain.to_string(),
        None => text,
    }
}

/// The file's name with its extension.
pub fn name_text(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path_text(path))
}

/// The argument that makes Explorer open the file's folder with the file selected. Quoted as a
/// whole path, since Explorer reads its command line itself and a comma or space in the path
/// would otherwise end it (a Windows path cannot contain `"`). Tested on every system, used on
/// Windows.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn explorer_select_arg(path: &Path) -> String {
    format!("/select,\"{}\"", path_text(path))
}

/// The `file://` URI the freedesktop file manager interface takes for `path`.
pub fn file_uri(path: &Path) -> Option<String> {
    url::Url::from_file_path(path).ok().map(String::from)
}

/// Show the file at `path` in the system's file manager: its folder opens with it selected.
/// Windows: `explorer /select,<path>`. Linux: the file manager's
/// `org.freedesktop.FileManager1.ShowItems` over D-Bus; when that fails (no session bus, no
/// file manager implementing it), the folder opens with `xdg-open` instead, without the file
/// selected. The error says why nothing could be opened.
pub async fn show_in_file_manager(path: PathBuf) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Explorer returns 1 even when it worked: its exit code says nothing, so it is not
        // waited for.
        std::process::Command::new("explorer")
            .raw_arg(explorer_select_arg(&path))
            .spawn()
            .map(drop)
            .map_err(|e| format!("explorer: {e}"))
    }
    #[cfg(target_os = "linux")]
    {
        match show_items_over_dbus(&path).await {
            Ok(()) => Ok(()),
            Err(e) => {
                log::info!("file manager over D-Bus failed ({e}), opening the folder instead");
                open_folder(&path)
            }
        }
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        open_folder(&path)
    }
}

/// How long the file manager gets to answer before the folder is opened another way.
#[cfg(target_os = "linux")]
const DBUS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Ask the file manager to show `path`: `ShowItems(uris, startup_id)` of
/// `org.freedesktop.FileManager1` on the session bus (Nautilus, Dolphin, Nemo, Caja, Thunar…).
#[cfg(target_os = "linux")]
async fn show_items_over_dbus(path: &Path) -> Result<(), String> {
    let uri = file_uri(path).ok_or_else(|| format!("not an absolute path: {path:?}"))?;
    let call = async {
        let connection = zbus::Connection::session().await?;
        connection
            .call_method(
                Some("org.freedesktop.FileManager1"),
                "/org/freedesktop/FileManager1",
                Some("org.freedesktop.FileManager1"),
                "ShowItems",
                &(vec![uri.as_str()], ""),
            )
            .await
            .map(drop)
    };
    match tokio::time::timeout(DBUS_TIMEOUT, call).await {
        Ok(result) => result.map_err(|e| e.to_string()),
        Err(_) => Err("no answer".to_string()),
    }
}

/// Open the folder holding `path` with the desktop's default app for folders.
#[cfg(not(windows))]
fn open_folder(path: &Path) -> Result<(), String> {
    let folder = path.parent().unwrap_or(path);
    std::process::Command::new("xdg-open")
        .arg(folder)
        .spawn()
        .map(drop)
        .map_err(|e| format!("xdg-open: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An absolute path on the system the tests run on: `D:\footage\clip.mp4` on Windows.
    fn clip() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"D:\footage\clip.mp4")
        } else {
            PathBuf::from("/footage/clip.mp4")
        }
    }

    #[test]
    fn copy_path_gives_the_full_path_and_copy_name_the_name_with_its_extension() {
        let path = clip();
        let expected = if cfg!(windows) {
            r"D:\footage\clip.mp4"
        } else {
            "/footage/clip.mp4"
        };
        assert_eq!(
            clipboard_text(FileAction::CopyPath, &path).as_deref(),
            Some(expected)
        );
        assert_eq!(
            clipboard_text(FileAction::CopyName, &path).as_deref(),
            Some("clip.mp4")
        );
        assert_eq!(clipboard_text(FileAction::ShowInFileManager, &path), None);
    }

    #[test]
    fn the_name_keeps_tags_spaces_and_letters_of_any_alphabet() {
        let path = clip().with_file_name("b-roll.Рыбалка.закат 2.MP4");
        assert_eq!(name_text(&path), "b-roll.Рыбалка.закат 2.MP4");
        assert!(path_text(&path).ends_with("b-roll.Рыбалка.закат 2.MP4"));
    }

    #[test]
    fn the_long_path_prefix_is_not_copied() {
        assert_eq!(
            path_text(Path::new(r"\\?\D:\footage\clip.mp4")),
            r"D:\footage\clip.mp4"
        );
        assert_eq!(
            path_text(Path::new(r"\\?\UNC\nas\footage\clip.mp4")),
            r"\\nas\footage\clip.mp4"
        );
        assert_eq!(
            path_text(Path::new(r"\\nas\footage\clip.mp4")),
            r"\\nas\footage\clip.mp4"
        );
    }

    #[test]
    fn explorer_gets_the_whole_path_quoted_after_select() {
        assert_eq!(
            explorer_select_arg(Path::new(r"D:\my footage, day 1\clip.mp4")),
            r#"/select,"D:\my footage, day 1\clip.mp4""#
        );
        assert_eq!(
            explorer_select_arg(Path::new(r"\\?\D:\footage\clip.mp4")),
            r#"/select,"D:\footage\clip.mp4""#
        );
    }

    #[test]
    fn the_file_manager_gets_a_percent_encoded_file_uri() {
        let path = clip().with_file_name("day 1 #2.mp4");
        let uri = file_uri(&path).expect("an absolute path");
        let expected = if cfg!(windows) {
            "file:///D:/footage/day%201%20%232.mp4"
        } else {
            "file:///footage/day%201%20%232.mp4"
        };
        assert_eq!(uri, expected);
        assert_eq!(file_uri(Path::new("relative/clip.mp4")), None);
    }
}
