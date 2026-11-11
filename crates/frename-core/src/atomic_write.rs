//! Replace a small file whole: write the new contents to a scratch file in the same folder, then
//! rename it over the file. A rename within one folder is atomic on Windows and POSIX alike, so a
//! crash, a kill or a full disk part way through leaves the previous file intact, never a cut-off
//! one (#156).
//!
//! The scratch file is not flushed to disk before the rename: these writes happen on the
//! interface thread, and a power cut is what the recovery journal ([`crate::recovery`]) covers.

use std::path::{Path, PathBuf};

/// Write `contents` to `path`, replacing what is there in one step. A read-only file is refused,
/// as an in-place write refuses it: a rename would replace it on macOS and Linux and drop the lock.
pub(crate) fn write_atomically(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    if std::fs::metadata(path).is_ok_and(|m| m.permissions().readonly()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("{} is read-only", path.display()),
        ));
    }
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

/// `.frename-<hash of the file name>.tmp` next to the file. The length is fixed, so a file whose
/// name is close to the 255-character limit still gets one; the hash keeps two files of one
/// folder written at once (a batch job and the open clip) apart. Never a video, so a folder scan
/// does not list it; the leading dot hides it on macOS and Linux. Not given the hidden attribute
/// on Windows: the rename would carry it over to the file itself.
fn scratch_path(path: &Path) -> PathBuf {
    // FNV-1a: stable across Rust versions, so a scratch file a crash left is written over by the
    // next save of the same file.
    let hash = path
        .file_name()
        .unwrap_or_default()
        .as_encoded_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, &byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        });
    path.with_file_name(format!(".frename-{hash:016x}.tmp"))
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

    /// A clip name near the limit: the comment file fits, so its scratch file must fit too.
    #[test]
    fn a_comment_file_with_a_name_near_the_limit_is_written() {
        let dir = crate::test_support::fresh_dir("atomic-write-long");
        let file = dir.join(format!("{}.mp4.comment.txt", "a".repeat(238)));
        assert_eq!(file.file_name().expect("name").len(), 254);

        write_atomically(&file, "new").expect("write");

        assert_eq!(std::fs::read_to_string(&file).expect("read"), "new");
    }

    /// Two files of one folder get different scratch files.
    #[test]
    fn each_file_has_its_own_scratch_file() {
        let dir = Path::new("shoots");
        assert_ne!(
            scratch_path(&dir.join("a.mp4.comment.txt")),
            scratch_path(&dir.join("b.mp4.comment.txt"))
        );
    }

    /// A read-only file is refused and kept, as `std::fs::write` refused it.
    #[test]
    fn a_read_only_file_is_refused_and_kept() {
        let dir = crate::test_support::fresh_dir("atomic-write-read-only");
        let file = dir.join("clip.mp4.comment.txt");
        std::fs::write(&file, "locked").expect("old");
        let mut permissions = std::fs::metadata(&file).expect("meta").permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&file, permissions.clone()).expect("lock");

        let result = write_atomically(&file, "new");

        assert_eq!(
            result.expect_err("refused").kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert_eq!(std::fs::read_to_string(&file).expect("read"), "locked");
        assert!(!scratch_path(&file).exists());
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(&file, permissions).expect("unlock");
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
