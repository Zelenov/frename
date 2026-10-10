//! Helpers for tests, here and in the root crate (which turns on the `test-support` feature in its
//! dev-dependencies, so release builds do not contain this module).

use std::path::PathBuf;

/// An empty folder of this test's own: `frename-<name>-<process id>` under the temp folder.
///
/// Windows reuses process ids, so a folder an earlier run left under the same name could leak its
/// files into this run. The folder is removed first and created again, so the test always starts
/// from nothing. The name stays predictable (unlike one with a counter or a random part), which
/// keeps a failed test's files easy to find, and tests that need a folder of their own pass
/// different names. Call it once per test: calling it again for the same name empties the folder.
pub fn fresh_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("frename-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir).or_else(|_| std::fs::remove_file(&dir));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Issue #226: a folder an earlier run with the same process id left behind, with a file in
    /// it, is gone when the next test asks for its folder.
    #[test]
    fn a_stale_folder_is_gone_after_fresh_dir() {
        let stale =
            std::env::temp_dir().join(format!("frename-fresh-dir-stale-{}", std::process::id()));
        std::fs::create_dir_all(stale.join("inner")).expect("stale folder");
        std::fs::write(stale.join("old.txt"), "left over").expect("stale file");
        std::fs::write(stale.join("inner").join("deep.txt"), "left over").expect("stale file");

        let dir = fresh_dir("fresh-dir-stale");

        assert_eq!(dir, stale);
        assert!(dir.is_dir());
        assert_eq!(std::fs::read_dir(&dir).expect("read").count(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
