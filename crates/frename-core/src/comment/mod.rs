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
    crate::text_encoding::decode(&bytes).trim().to_string()
}

/// Save (or delete) the comment file for a file.
/// If `comment` is empty/whitespace, removes the comment file (if any). Saves use
/// [`try_save_comment`], which says whether the write took.
#[cfg(test)]
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

    /// #458, on Windows: a short "ANSI" comment is read in the system's code page, whatever
    /// detection would guess. Runs where that code page is 1251 or 1252 (CI's is 1252).
    #[cfg(windows)]
    #[test]
    fn a_short_ansi_comment_reads_in_the_system_code_page() {
        let Some((_, encoding)) = crate::text_encoding::ansi_sample() else {
            eprintln!("system code page: nothing to check");
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
