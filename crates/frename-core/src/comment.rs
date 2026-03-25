//! Comment file utilities.
//!
//! Each file may have an associated comment stored in `{filename}.comment.txt`
//! in the same directory. An empty or absent file means no comment.

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

/// Load the comment for a file from disk. Returns empty string if absent or empty.
pub fn load_comment(file_path: &Path) -> String {
    std::fs::read_to_string(comment_path(file_path))
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Save (or delete) the comment file for a file.
/// If `comment` is empty/whitespace, removes the comment file (if any).
pub fn save_comment(file_path: &Path, comment: &str) {
    let path = comment_path(file_path);
    let trimmed = comment.trim();
    if trimmed.is_empty() {
        let _ = std::fs::remove_file(&path);
    } else {
        let _ = std::fs::write(&path, trimmed);
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
