//! Folder-level metadata used during bulk file parsing.

use std::collections::HashMap;

use super::CachedFile;

/// The names of every entry in a folder, held sorted so lookups are a binary search.
///
/// Bulk parsing asks this the same two questions per file — "is there a `X.comment.txt`?" and
/// "which `X.snap.*.jpg` exist?" — so a linear scan per file made a folder scan O(N^2).
/// Sorting once at construction turns each question into O(log N + matches).
#[derive(Debug, Clone, Default)]
pub struct FolderInfo {
    /// Sorted; see [FolderInfo::contains] and [FolderInfo::names_starting_with].
    file_names: Vec<String>,
    /// Set for a folder scan: parsing must not open files, and takes comments stored inside
    /// videos from `cache` when a line still matches, leaving them to load later otherwise.
    scan: Option<ScanInfo>,
}

/// What a folder scan knows about each file without opening it.
#[derive(Debug, Clone, Default)]
struct ScanInfo {
    /// Size and modification time (ms since the Unix epoch) by file name, from the listing.
    stats: HashMap<String, (u64, u64)>,
    /// The tag file's file list, by name.
    cache: HashMap<String, CachedFile>,
}

impl FolderInfo {
    pub fn new(mut file_names: Vec<String>) -> Self {
        file_names.sort();
        Self {
            file_names,
            scan: None,
        }
    }

    /// Folder info for a scan: file sizes and modification times from the listing, and the
    /// tag file's file list. Parsing with it never opens a file.
    pub fn for_scan(
        file_names: Vec<String>,
        stats: HashMap<String, (u64, u64)>,
        cache: Vec<CachedFile>,
    ) -> Self {
        let cache = cache.into_iter().map(|c| (c.name.clone(), c)).collect();
        Self {
            scan: Some(ScanInfo { stats, cache }),
            ..Self::new(file_names)
        }
    }

    /// Whether this is a folder scan, which leaves comments stored inside videos to load later.
    pub fn defers_comment_loading(&self) -> bool {
        self.scan.is_some()
    }

    /// The file list line for `name`, if it still matches the file. A line without a marker
    /// count is from before markers were counted and does not match.
    pub fn cached_file(&self, name: &str) -> Option<&CachedFile> {
        let scan = self.scan.as_ref()?;
        let cached = scan.cache.get(name)?;
        (cached.markers.is_some()
            && scan.stats.get(name) == Some(&(cached.size, cached.modified_ms)))
        .then_some(cached)
    }

    /// All entry names, in sorted order.
    pub fn file_names(&self) -> &[String] {
        &self.file_names
    }

    /// Whether the folder contains an entry with exactly this name.
    pub fn contains(&self, name: &str) -> bool {
        self.file_names
            .binary_search_by(|n| n.as_str().cmp(name))
            .is_ok()
    }

    /// Every entry name starting with `prefix`, in sorted order.
    pub fn names_starting_with<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = &'a str> + 'a {
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
    fn a_cached_line_without_a_marker_count_does_not_match() {
        let line = |markers| CachedFile {
            name: "a.mp4".into(),
            size: 1,
            modified_ms: 2,
            comment: String::new(),
            start: None,
            end: None,
            markers,
        };
        let stats = HashMap::from([("a.mp4".to_string(), (1, 2))]);
        let scan = |markers| {
            FolderInfo::for_scan(vec!["a.mp4".into()], stats.clone(), vec![line(markers)])
        };
        assert!(scan(None).cached_file("a.mp4").is_none());
        assert_eq!(
            scan(Some(2)).cached_file("a.mp4").and_then(|c| c.markers),
            Some(2)
        );
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
        assert_eq!(
            found,
            vec!["a.mp4.snap.00_00_10.jpg", "a.mp4.snap.00_01_00.jpg"]
        );
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
