//! Replace a small file whole: write the new contents to a scratch file in the same folder, then
//! rename it over the file. A rename within one folder is atomic on Windows and POSIX alike, so a
//! crash, a kill or a full disk part way through leaves the previous file intact, never a cut-off
//! one (#156).
//!
//! The scratch file is not flushed to disk before the rename: these writes happen on the
//! interface thread, and a power cut is what the recovery journal ([`crate::recovery`]) covers.

use std::path::{Path, PathBuf};

/// Write `contents` to `path`, replacing what is there in one step.
pub fn write_atomically(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    replace_through(path, &scratch_path(path), contents.as_ref())
}

/// Write `contents` to `scratch`, then rename it over `path`. A failed rename removes the scratch
/// file, so nothing is left next to the user's footage.
pub(crate) fn replace_through(path: &Path, scratch: &Path, contents: &[u8]) -> std::io::Result<()> {
    if let Err(e) = std::fs::write(scratch, contents) {
        let _ = std::fs::remove_file(scratch);
        return Err(e);
    }
    std::fs::rename(scratch, path).inspect_err(|_| {
        let _ = std::fs::remove_file(scratch);
    })
}

/// `.clip.mp4.comment.txt.frename-tmp` next to `clip.mp4.comment.txt`: hidden on macOS and Linux,
/// and never a video, so a folder scan does not list it.
fn scratch_path(path: &Path) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!(".{name}.frename-tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_new_contents_replace_the_old_and_no_scratch_file_is_left() {
        let dir = crate::test_support::fresh_dir("atomic-write-replace");
        let file = dir.join("clip.mp4.comment.txt");
        std::fs::write(&file, "an old comment that is longer than the new one").expect("old");

        write_atomically(&file, "new").expect("write");

        assert_eq!(std::fs::read_to_string(&file).expect("read"), "new");
        let names: Vec<_> = std::fs::read_dir(&dir)
            .expect("list")
            .map(|e| e.expect("entry").file_name())
            .collect();
        assert_eq!(
            names,
            vec![std::ffi::OsString::from("clip.mp4.comment.txt")]
        );
    }

    #[test]
    fn a_missing_file_is_created() {
        let dir = crate::test_support::fresh_dir("atomic-write-create");
        let file = dir.join("clip.markers.txt");

        write_atomically(&file, "0:01 Start").expect("write");

        assert_eq!(std::fs::read_to_string(&file).expect("read"), "0:01 Start");
    }

    /// A scratch file an earlier crash left behind is written over, not appended to.
    #[test]
    fn a_stale_scratch_file_does_not_leak_into_the_new_contents() {
        let dir = crate::test_support::fresh_dir("atomic-write-stale");
        let file = dir.join("clip.mp4.comment.txt");
        std::fs::write(scratch_path(&file), "half of an older comm").expect("stale");

        write_atomically(&file, "new").expect("write");

        assert_eq!(std::fs::read_to_string(&file).expect("read"), "new");
        assert!(!scratch_path(&file).exists());
    }

    /// When the rename cannot happen (here the target is a folder), the call fails, the target is
    /// untouched and the scratch file is removed.
    #[test]
    fn a_failed_replace_leaves_the_target_and_no_scratch_file() {
        let dir = crate::test_support::fresh_dir("atomic-write-fail");
        let target = dir.join("clip.mp4.comment.txt");
        std::fs::create_dir(&target).expect("folder in the way");

        assert!(write_atomically(&target, "new").is_err());

        assert!(target.is_dir());
        assert!(!scratch_path(&target).exists());
    }

    /// The file is replaced by a new one, not rewritten in place: an in-place write truncates the
    /// file first, so a crash in between would leave it empty or cut off.
    #[cfg(unix)]
    #[test]
    fn the_file_is_replaced_not_rewritten_in_place() {
        use std::os::unix::fs::MetadataExt;
        let dir = crate::test_support::fresh_dir("atomic-write-inode");
        let file = dir.join("clip.mp4.comment.txt");
        std::fs::write(&file, "old").expect("old");
        let before = std::fs::metadata(&file).expect("meta").ino();

        write_atomically(&file, "new").expect("write");

        assert_ne!(std::fs::metadata(&file).expect("meta").ino(), before);
    }
}
