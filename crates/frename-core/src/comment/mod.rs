//! Comment text files.
//!
//! A file's comment may be stored in `{filename}.comment.txt` in the same directory; see
//! [`crate::metadata`] for when this is used instead of the file's XMP.
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
pub fn load_comment(file_path: &Path) -> String {
    let text = std::fs::read_to_string(comment_path(file_path)).unwrap_or_default();
    text.strip_prefix(UTF8_BOM).unwrap_or(&text).trim().to_string()
}

/// Save (or delete) the comment file for a file.
/// If `comment` is empty/whitespace, removes the comment file (if any).
pub fn save_comment(file_path: &Path, comment: &str) {
    let trimmed = comment.trim();
    if trimmed.is_empty() {
        remove_comment_file(file_path);
    } else if let Err(e) = std::fs::write(comment_path(file_path), format!("{UTF8_BOM}{trimmed}")) {
        log::error!("comment: failed to write comment file for {:?}: {}", file_path, e);
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
/// If no comment file exists, does nothing.
pub fn rename_comment_file(old_file_path: &Path, new_file_path: &Path) {
    let old_comment = comment_path(old_file_path);
    let new_comment = comment_path(new_file_path);
    if old_comment.exists() {
        if let Err(e) = std::fs::rename(&old_comment, &new_comment) {
            log::warn!(
                "comment: failed to rename {:?} → {:?}: {}",
                old_comment, new_comment, e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("frename-comment-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
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
