//! Reading text files another program saved: comment files and subtitle files may be UTF-8,
//! UTF-16 with its byte order mark, or an 8-bit "ANSI" code page (#458, #467).

/// The text of a text file's `bytes` (a comment file, a subtitle file): UTF-8 (with or without
/// its byte order mark), UTF-16 with its byte order mark, or else an 8-bit "ANSI" code page. A
/// UTF-8 file with a few broken bytes (cut at its end, joined from two files) stays UTF-8, with
/// "�" for those: read as "ANSI", every letter outside English would be wrong.
pub(crate) fn decode(bytes: &[u8]) -> String {
    if let Some((encoding, bom_length)) = encoding_rs::Encoding::for_bom(bytes) {
        let (text, _) = encoding.decode_without_bom_handling(&bytes[bom_length..]);
        return text.into_owned();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }
    if mostly_utf8(bytes) {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let encoding = ansi_code_page()
        .and_then(code_page_encoding)
        .unwrap_or_else(|| guess_encoding(bytes, locale_country().as_deref()));
    let (text, _) = encoding.decode_without_bom_handling(bytes);
    text.into_owned()
}

/// Whether `bytes`, which are not valid UTF-8, are still UTF-8 with a few broken sequences: it
/// has letters encoded as UTF-8, many more of them than broken places. An 8-bit "ANSI" text
/// almost never forms a valid UTF-8 sequence, so read as UTF-8 it is nearly all broken places.
fn mostly_utf8(bytes: &[u8]) -> bool {
    let text = String::from_utf8_lossy(bytes);
    let broken = text.matches(char::REPLACEMENT_CHARACTER).count();
    let letters = text
        .chars()
        .filter(|c| !c.is_ascii() && *c != char::REPLACEMENT_CHARACTER)
        .count();
    letters >= 4 * broken
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

/// A sentence in the encoding "ANSI" text files are read in on this system, for tests:
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

#[cfg(test)]
mod tests {
    use super::*;

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

    /// #467: a UTF-8 file with one broken byte (here cut in the middle of its last letter)
    /// keeps its letters, with "�" for that byte only, instead of being read as "ANSI".
    #[test]
    fn utf8_with_a_broken_byte_stays_utf8() {
        let mut bytes = "Привет, вода в кадре номер 1\n".repeat(20).into_bytes();
        bytes.push(0xD0);
        let text = decode(&bytes);
        assert!(text.starts_with("Привет, вода в кадре номер 1"), "{text}");
        assert!(text.ends_with('\u{fffd}'));
        let (ansi, _, _) = encoding_rs::WINDOWS_1251.encode("Привет, вода в кадре");
        assert!(!mostly_utf8(&ansi), "an ANSI text is not mostly UTF-8");
    }
}
