//! Folder-level metadata used during bulk file parsing.

/// The names of every entry in a folder, held sorted so lookups are a binary search.
///
/// Bulk parsing asks this the same two questions per file — "is there a `X.comment.txt`?" and
/// "which `X.snap.*.jpg` exist?" — so a linear scan per file made a folder scan O(N^2).
/// Sorting once at construction turns each question into O(log N + matches).
#[derive(Debug, Clone, Default)]
pub struct FolderInfo {
    /// Sorted; see [FolderInfo::contains] and [FolderInfo::names_starting_with].
    file_names: Vec<String>,
}

impl FolderInfo {
    pub fn new(mut file_names: Vec<String>) -> Self {
        file_names.sort();
        Self { file_names }
    }

    /// All entry names, in sorted order.
    pub fn file_names(&self) -> &[String] {
        &self.file_names
    }

    /// Whether the folder contains an entry with exactly this name.
    pub fn contains(&self, name: &str) -> bool {
        self.file_names.binary_search_by(|n| n.as_str().cmp(name)).is_ok()
    }

    /// Every entry name starting with `prefix`, in sorted order.
    pub fn names_starting_with<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        let start = self.file_names.partition_point(|n| n.as_str() < prefix);
        self.file_names[start..]
            .iter()
            .map(String::as_str)
            .take_while(move |n| n.starts_with(prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info() -> FolderInfo {
        FolderInfo::new(vec![
            "b.mp4".into(),
            "a.mp4.snap.00_00_10.jpg".into(),
            "a.mp4".into(),
            "a.mp4.comment.txt".into(),
            "a.mp4.snap.00_01_00.jpg".into(),
            "ab.mp4".into(),
        ])
    }

    #[test]
    fn contains_finds_exact_names_only() {
        let i = info();
        assert!(i.contains("a.mp4.comment.txt"));
        assert!(!i.contains("b.mp4.comment.txt"));
        assert!(!i.contains("a.mp4.snap."));
    }

    #[test]
    fn names_starting_with_returns_only_the_prefix_range() {
        let i = info();
        let found: Vec<&str> = i.names_starting_with("a.mp4.snap.").collect();
        assert_eq!(found, vec!["a.mp4.snap.00_00_10.jpg", "a.mp4.snap.00_01_00.jpg"]);
    }

    #[test]
    fn names_starting_with_does_not_bleed_into_the_next_name() {
        // "ab.mp4" sorts right after the "a.mp4..." entries and must not match the "a.mp4" prefix.
        let i = info();
        let found: Vec<&str> = i.names_starting_with("ab.").collect();
        assert_eq!(found, vec!["ab.mp4"]);
    }

    #[test]
    fn names_starting_with_is_empty_when_nothing_matches() {
        assert_eq!(info().names_starting_with("zz").count(), 0);
    }
}
