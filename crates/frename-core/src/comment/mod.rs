//! Comment text files.
//!
//! A file's comment may be stored in `{filename}.comment.txt` in the same directory; see
//! [`crate::metadata`] for when this is used instead of storing it inside the video.
//! An empty or absent file means no comment.

use std::path::{Path, PathBuf};

/// Returns the path of the comment file for a given file path.
/// E.g. `/dir/video.mp4` → `/dir/video.mp4.comment.txt`
pub fn comment_path(file_path: &Path) -> PathBuf {
    let file_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let comment_name = format!("{}.comment.txt", file_name);
    file_path
        .parent()
        .unwrap_or(Path::new(""))
        .join(comment_name)
}

/// Returns true if the path looks like a comment file (ends with `.comment.txt`).
pub fn is_comment_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".comment.txt"))
        .unwrap_or(false)
}

/// UTF-8 byte order mark. Written at the start of comment files so Windows editors
/// don't guess the ANSI codepage (1251/1252) for BOM-less UTF-8.
const UTF8_BOM: char = '\u{feff}';

/// Load the comment file for a file. Returns empty string if absent or empty.
///
/// A file another editor saved in another encoding is read too (#458): Notepad's "Unicode"
/// (UTF-16 with its byte order mark) and "ANSI" (the system's code page, such as Windows-1251).
/// Read as nothing, its text would be gone at the next save, which writes the comment shown.
pub fn load_comment(file_path: &Path) -> String {
    let bytes = std::fs::read(comment_path(file_path)).unwrap_or_default();
    decode_comment(&bytes).trim().to_string()
}

/// The text of a comment file's `bytes`: UTF-8 (with or without its byte order mark), UTF-16
/// with its byte order mark, or else an 8-bit "ANSI" code page.
fn decode_comment(bytes: &[u8]) -> String {
    if let Some((encoding, bom_length)) = encoding_rs::Encoding::for_bom(bytes) {
        let (text, _) = encoding.decode_without_bom_handling(&bytes[bom_length..]);
        return text.into_owned();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }
    let encoding = ansi_code_page()
        .and_then(code_page_encoding)
        .unwrap_or_else(|| guess_encoding(bytes));
    let (text, _) = encoding.decode_without_bom_handling(bytes);
    text.into_owned()
}

/// The system's "ANSI" code page, which Notepad saves "ANSI" files in (Windows-1251 on a
/// Russian system).
#[cfg(windows)]
fn ansi_code_page() -> Option<u32> {
    // SAFETY: GetACP takes no arguments and only reads a system setting.
    Some(unsafe { windows_sys::Win32::Globalization::GetACP() })
}

/// Other systems have no "ANSI" code page: the bytes are guessed from (see [`guess_encoding`]).
#[cfg(not(windows))]
fn ansi_code_page() -> Option<u32> {
    None
}

/// The encoding of a Windows code page, for the ones Windows uses as "ANSI". `None` for any
/// other (65001, UTF-8, never reaches here: the bytes were not UTF-8).
fn code_page_encoding(code_page: u32) -> Option<&'static encoding_rs::Encoding> {
    let label = match code_page {
        874 => "windows-874".to_string(),
        932 => "shift_jis".to_string(),
        936 => "gbk".to_string(),
        949 => "euc-kr".to_string(),
        950 => "big5".to_string(),
        1250..=1258 => format!("windows-{code_page}"),
        _ => return None,
    };
    encoding_rs::Encoding::for_label(label.as_bytes())
}

/// The 8-bit encoding `bytes` most likely are in, where the system does not say (a file made on
/// Windows read on macOS or Linux). Detection is unsure on a word or two, so the language of the
/// system's locale (`LANG`, e.g. `ru_RU.UTF-8`) is its hint.
fn guess_encoding(bytes: &[u8]) -> &'static encoding_rs::Encoding {
    let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()))
        .unwrap_or_default();
    let language: String = locale
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_lowercase();
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let hint = (language.len() == 2).then_some(language.as_bytes());
    detector.guess(hint, true)
}

/// Save (or delete) the comment file for a file.
/// If `comment` is empty/whitespace, removes the comment file (if any).
pub fn save_comment(file_path: &Path, comment: &str) {
    if let Err(e) = try_save_comment(file_path, comment) {
        log::error!(
            "comment: failed to write comment file for {:?}: {}",
            file_path,
            e
        );
    }
}

/// Like [`save_comment`], but says when the file could not be written or removed.
pub fn try_save_comment(file_path: &Path, comment: &str) -> std::io::Result<()> {
    let trimmed = comment.trim();
    let path = comment_path(file_path);
    if trimmed.is_empty() {
        match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    } else {
        crate::write_atomically(&path, format!("{UTF8_BOM}{trimmed}"))
    }
}

/// Remove the comment file for a file, if there is one.
pub fn remove_comment_file(file_path: &Path) {
    let path = comment_path(file_path);
    if let Err(e) = std::fs::remove_file(&path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            log::warn!("comment: failed to remove {:?}: {}", path, e);
        }
    }
}

/// Rename the comment file when its parent file is renamed.
/// If no comment file exists, does nothing. Does not overwrite a comment under the new name,
/// except one spelled the same in other case (on Windows and macOS that is this file).
pub fn rename_comment_file(old_file_path: &Path, new_file_path: &Path) {
    let old_comment = comment_path(old_file_path);
    let new_comment = comment_path(new_file_path);
    if !old_comment.exists() {
        return;
    }
    // Never over another clip's comment. A change of case only is the same file on Windows and
    // macOS, where the new name "exists" already.
    let same_name_other_case = old_comment
        .file_name()
        .zip(new_comment.file_name())
        .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b));
    if new_comment.exists() && !same_name_other_case {
        log::warn!("comment: not renaming {old_comment:?}: {new_comment:?} already exists");
        return;
    }
    if let Err(e) = std::fs::rename(&old_comment, &new_comment) {
        log::warn!("comment: failed to rename {old_comment:?} → {new_comment:?}: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_folder(name: &str) -> PathBuf {
        crate::test_support::fresh_dir(&format!("comment-{name}"))
    }

    /// #458: a comment Notepad saved as "ANSI" (Windows-1251 here) or "Unicode" (UTF-16 LE with
    /// its byte order mark) is read, not shown as empty and then deleted by the next save.
    #[test]
    fn comments_in_other_encodings_are_read() {
        let dir = crate::test_support::fresh_dir("comment-encodings");
        let clip = dir.join("goat.mov");
        let text = "Привет, вода в кадре";
        let (ansi, _, _) = encoding_rs::WINDOWS_1251.encode(text);
        std::fs::write(comment_path(&clip), &*ansi).expect("write");
        assert_eq!(load_comment(&clip), text);
        let mut utf16 = vec![0xFF, 0xFE];
        utf16.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
        std::fs::write(comment_path(&clip), utf16).expect("write");
        assert_eq!(load_comment(&clip), text);
        let latin = "Café crème, à revoir";
        let (ansi, _, _) = encoding_rs::WINDOWS_1252.encode(latin);
        std::fs::write(comment_path(&clip), &*ansi).expect("write");
        assert_eq!(load_comment(&clip), latin);
        // UTF-8, with and without its byte order mark, as before.
        std::fs::write(comment_path(&clip), format!("{UTF8_BOM}{text}")).expect("write");
        assert_eq!(load_comment(&clip), text);
        std::fs::write(comment_path(&clip), text).expect("write");
        assert_eq!(load_comment(&clip), text);
    }

    /// #458: Notepad's "ANSI" is the system's code page: decoded with it, a word or two in it
    /// (most comments) reads right, where detection alone guesses another script.
    #[test]
    fn short_ansi_comments_read_in_the_system_code_page() {
        let cyrillic = code_page_encoding(1251).expect("windows-1251");
        for word in ["брак", "шум", "ок", "Да", "звук", "фокус"] {
            let (ansi, _, _) = encoding_rs::WINDOWS_1251.encode(word);
            let (text, _) = cyrillic.decode_without_bom_handling(&ansi);
            assert_eq!(text, word);
        }
        let latin = code_page_encoding(1252).expect("windows-1252");
        let (ansi, _, _) = encoding_rs::WINDOWS_1252.encode("naïve");
        assert_eq!(latin.decode_without_bom_handling(&ansi).0, "naïve");
        for code_page in [874, 932, 936, 949, 950, 1250, 1258] {
            assert!(code_page_encoding(code_page).is_some(), "{code_page}");
        }
        assert!(code_page_encoding(65001).is_none());
        assert!(code_page_encoding(437).is_none());
    }

    /// #458, on Windows: the system's code page is what a short "ANSI" comment is read in.
    #[cfg(windows)]
    #[test]
    fn a_short_ansi_comment_reads_in_the_system_code_page() {
        let Some(encoding) = ansi_code_page().and_then(code_page_encoding) else {
            return;
        };
        let dir = crate::test_support::fresh_dir("comment-acp");
        let clip = dir.join("goat.mov");
        let word = if encoding == encoding_rs::WINDOWS_1251 {
            "брак"
        } else {
            "naïve"
        };
        let (ansi, _, _) = encoding.encode(word);
        std::fs::write(comment_path(&clip), &*ansi).expect("write");
        assert_eq!(load_comment(&clip), word);
    }

    #[test]
    fn saved_comment_has_bom_and_loads_without_it() {
        let file = temp_folder("bom").join("clip.mp4");
        save_comment(&file, "Café 5G");
        let bytes = std::fs::read(comment_path(&file)).expect("comment file");
        assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        assert_eq!(load_comment(&file), "Café 5G");
    }

    #[test]
    fn loads_comment_without_bom() {
        let file = temp_folder("plain").join("clip.mp4");
        std::fs::write(comment_path(&file), "  plain  ").expect("write");
        assert_eq!(load_comment(&file), "plain");
    }

    #[test]
    fn empty_comment_removes_the_file() {
        let file = temp_folder("remove").join("clip.mp4");
        save_comment(&file, "x");
        save_comment(&file, "  ");
        assert!(!comment_path(&file).exists());
    }
}
