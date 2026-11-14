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
        .unwrap_or_else(|| guess_encoding(bytes, locale_country().as_deref()));
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

/// Other systems have no "ANSI" code page; the encoding is detected (see [`guess_encoding`]).
#[cfg(not(windows))]
fn ansi_code_page() -> Option<u32> {
    None
}

/// The encoding of the Windows code pages Windows uses as "ANSI". `None` for any other (65001,
/// UTF-8, among them): those bytes fall back to detection.
fn code_page_encoding(code_page: u32) -> Option<&'static encoding_rs::Encoding> {
    use encoding_rs::*;
    Some(match code_page {
        874 => WINDOWS_874,
        932 => SHIFT_JIS,
        936 => GBK,
        949 => EUC_KR,
        950 => BIG5,
        1250 => WINDOWS_1250,
        1251 => WINDOWS_1251,
        1252 => WINDOWS_1252,
        1253 => WINDOWS_1253,
        1254 => WINDOWS_1254,
        1255 => WINDOWS_1255,
        1256 => WINDOWS_1256,
        1257 => WINDOWS_1257,
        1258 => WINDOWS_1258,
        _ => return None,
    })
}

/// The country of the system's locale (`ru` for `ru_RU.UTF-8` in `LC_ALL`, `LC_CTYPE` or
/// `LANG`), lower-case; `None` without one (`C`, `POSIX`, `C.UTF-8`). On Windows these are
/// usually unset (a Unix convention), so there detection, reached only for a code page without
/// an encoding (65001), gets no hint.
fn locale_country() -> Option<String> {
    let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()))?;
    country_of_locale(&locale)
}

/// A sentence in the encoding "ANSI" comment files are read in on this system, for tests:
/// Windows-1251 where detection decides (when this locale's hint lets it find 1251: a Greek or
/// Hebrew one may not), the system's code page on Windows when it is 1251 or 1252 (CI's),
/// `None` otherwise.
#[cfg(test)]
pub(crate) fn ansi_sample() -> Option<(&'static str, &'static encoding_rs::Encoding)> {
    const CYRILLIC: &str = "Привет, вода в кадре";
    match ansi_code_page().map(code_page_encoding) {
        None => {
            let (ansi, _, _) = encoding_rs::WINDOWS_1251.encode(CYRILLIC);
            let found = guess_encoding(&ansi, locale_country().as_deref());
            (found == encoding_rs::WINDOWS_1251).then_some((CYRILLIC, found))
        }
        Some(Some(e)) if e == encoding_rs::WINDOWS_1251 => Some((CYRILLIC, e)),
        Some(Some(e)) if e == encoding_rs::WINDOWS_1252 => Some(("Café crème, à revoir", e)),
        Some(_) => None,
    }
}

/// The country part of a locale name (`uk` → none, `uk_UA.UTF-8` → `ua`).
fn country_of_locale(locale: &str) -> Option<String> {
    let (_, rest) = locale.split_once('_')?;
    let country: String = rest.chars().take_while(char::is_ascii_alphabetic).collect();
    (country.len() == 2).then(|| country.to_ascii_lowercase())
}

/// The 8-bit encoding `bytes` most likely are in, where the system has no "ANSI" code page (a
/// file made on Windows, read on macOS or Linux). `country` is the detector's hint: its top
/// level domain (`ru`). Detection is unsure on a word or two: a short comment may read in
/// another script.
fn guess_encoding(bytes: &[u8], country: Option<&str>) -> &'static encoding_rs::Encoding {
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    detector.guess(country.map(str::as_bytes), true)
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

    /// #458: a comment Notepad saved as "Unicode" (UTF-16 LE with its byte order mark) is read,
    /// not shown as empty and then deleted by the next save; UTF-8 reads as before.
    #[test]
    fn utf16_and_utf8_comments_are_read() {
        let dir = crate::test_support::fresh_dir("comment-encodings");
        let clip = dir.join("goat.mov");
        let text = "Привет, вода в кадре";
        let mut utf16 = vec![0xFF, 0xFE];
        utf16.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
        std::fs::write(comment_path(&clip), utf16).expect("write");
        assert_eq!(load_comment(&clip), text);
        std::fs::write(comment_path(&clip), format!("{UTF8_BOM}{text}")).expect("write");
        assert_eq!(load_comment(&clip), text);
        std::fs::write(comment_path(&clip), text).expect("write");
        assert_eq!(load_comment(&clip), text);
    }

    /// #458: an "ANSI" comment (Windows-1251, Windows-1252) is read on a system without an
    /// ANSI code page, by detection: a sentence in either, and a word or two with the locale's
    /// country as the hint.
    #[cfg(not(windows))]
    #[test]
    fn ansi_comments_are_detected() {
        for (text, encoding) in [
            ("Привет, вода в кадре", encoding_rs::WINDOWS_1251),
            ("Café crème, à revoir", encoding_rs::WINDOWS_1252),
        ] {
            let (ansi, _, _) = encoding.encode(text);
            assert_eq!(guess_encoding(&ansi, None), encoding, "{text}");
        }
        for word in ["брак", "фокус", "ок", "нет звука"] {
            let (ansi, _, _) = encoding_rs::WINDOWS_1251.encode(word);
            assert_eq!(
                guess_encoding(&ansi, Some("ru")),
                encoding_rs::WINDOWS_1251,
                "{word}"
            );
        }
    }

    /// #458: the country of a locale is the detector's hint (`uk` is Ukrainian, not the UK).
    #[test]
    fn the_hint_is_the_locales_country() {
        assert_eq!(country_of_locale("ru_RU.UTF-8").as_deref(), Some("ru"));
        assert_eq!(country_of_locale("uk_UA").as_deref(), Some("ua"));
        assert_eq!(country_of_locale("pt_BR.utf8@x").as_deref(), Some("br"));
        assert_eq!(country_of_locale("C.UTF-8"), None);
        assert_eq!(country_of_locale("POSIX"), None);
        assert_eq!(country_of_locale("en"), None);
    }

    /// #458: the code pages Windows uses as "ANSI" have an encoding; others fall back to
    /// detection.
    #[test]
    fn windows_code_pages_map_to_encodings() {
        for code_page in [874, 932, 936, 949, 950, 1250, 1251, 1252, 1258] {
            assert!(code_page_encoding(code_page).is_some(), "{code_page}");
        }
        assert_eq!(code_page_encoding(1251), Some(encoding_rs::WINDOWS_1251));
        assert!(code_page_encoding(65001).is_none());
        assert!(code_page_encoding(437).is_none());
    }

    /// #458, on Windows: a short "ANSI" comment is read in the system's code page, whatever
    /// detection would guess. Runs where that code page is 1251 or 1252 (CI's is 1252).
    #[cfg(windows)]
    #[test]
    fn a_short_ansi_comment_reads_in_the_system_code_page() {
        let Some((_, encoding)) = ansi_sample() else {
            eprintln!("system code page {:?}: nothing to check", ansi_code_page());
            return;
        };
        let word = if encoding == encoding_rs::WINDOWS_1251 {
            "брак"
        } else {
            "naïve"
        };
        let dir = crate::test_support::fresh_dir("comment-acp");
        let clip = dir.join("goat.mov");
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
