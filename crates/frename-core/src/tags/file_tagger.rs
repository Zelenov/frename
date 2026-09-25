//! FileTagger facade: routes parse/save to the installed backend.
//!
//! Call `install_file_tagger(backend)` once at startup (in main.rs) before any
//! file operations. If no backend is installed the default is InMemoryFileTagger,
//! which is always correct for tests and debug builds.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::FileTaggerBackend;
use super::folder_info::FolderInfo;
use super::folder_tag_store::FolderTagStore;
use super::in_memory_file_tagger::InMemoryFileTagger;
use super::tag_list::TagList;
use crate::metadata::{MetadataMove, MoveOutcome};

static BACKEND: OnceLock<Box<dyn FileTaggerBackend>> = OnceLock::new();

/// Install the file tagger backend. Must be called before any file operations.
/// Subsequent calls are ignored (OnceLock semantics).
pub fn install_file_tagger(backend: Box<dyn FileTaggerBackend>) {
    let _ = BACKEND.set(backend);
}

fn backend() -> &'static dyn FileTaggerBackend {
    BACKEND
        .get_or_init(|| Box::new(InMemoryFileTagger::default()))
        .as_ref()
}

/// Thin facade over the installed `FileTaggerBackend`.
pub struct FileTagger;

impl FileTagger {
    pub fn parse(path: &Path, folder_info: &FolderInfo) -> FileSnapshot {
        backend().parse(path, folder_info)
    }

    /// Returns the new path (may differ from `path` after a disk rename).
    pub fn save(snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        backend().save(snapshot, path)
    }

    /// Bring the tag `tag` in line with the file's comment: added last when the file has a
    /// comment, removed when it has none. Returns the outcome like a move: `NothingToMove` when
    /// the name already matches, `Failed` when the rename did not happen.
    pub fn sync_commented_tag(path: &Path, tag: &str) -> MoveOutcome {
        let mut snapshot = Self::parse(path, &FolderInfo::default());
        let commented = !snapshot.comment().trim().is_empty();
        let tagged = snapshot.tags().iter().any(|t| t.eq_ignore_ascii_case(tag));
        if commented == tagged {
            return MoveOutcome::NothingToMove;
        }
        let mut tags: Vec<String> = snapshot
            .tags()
            .iter()
            .filter(|t| !t.eq_ignore_ascii_case(tag))
            .cloned()
            .collect();
        if commented {
            tags.push(tag.to_string());
        }
        snapshot.set_tags(tags);
        Self::renamed(Self::save(&snapshot, path), path)
    }

    /// Put the tags in the file's name in the folder's tag order (the tag panel's order), as the
    /// tag panel's "sync down" does for the open file. Returns the outcome like a move.
    pub fn sort_tags_by_folder_order(path: &Path) -> MoveOutcome {
        let Some(folder) = path.parent() else {
            return MoveOutcome::NothingToMove;
        };
        let mut snapshot = Self::parse(path, &FolderInfo::default());
        let mut list = TagList::new(FolderTagStore::for_folder(folder), snapshot.clone());
        list.sync_display_to_selected();
        let sorted = list.file_snapshot().tags().to_vec();
        if sorted == snapshot.tags() {
            return MoveOutcome::NothingToMove;
        }
        snapshot.set_tags(sorted);
        Self::renamed(Self::save(&snapshot, path), path)
    }

    /// Read the comment and in/out points of the file at `path` from the file again, replacing
    /// what the folder's file list cached for it. Returns whether the cached values were missing
    /// or stale.
    pub fn reload_metadata(path: &Path) -> bool {
        backend().reload_metadata(path)
    }

    /// The outcome of a save that must rename `path`: a save that failed keeps the old path.
    fn renamed(new_path: PathBuf, path: &Path) -> MoveOutcome {
        if new_path == path {
            MoveOutcome::Failed(new_path)
        } else {
            MoveOutcome::Moved(new_path)
        }
    }

    /// Where the file at `path` is on disk; see [`FileTaggerBackend::disk_path`].
    pub fn disk_path(path: &Path) -> PathBuf {
        backend().disk_path(path)
    }

    /// Returns true if `path` is a sidecar file that should be hidden from the file list.
    pub fn is_sidecar_file(path: &Path) -> bool {
        backend().is_sidecar_file(path)
    }

    /// Load the comment a folder scan deferred; see [`FileSnapshot::comment_loading`].
    pub fn load_comment(path: &Path, snapshot: &FileSnapshot) -> FileSnapshot {
        let items = [(path.to_path_buf(), snapshot.clone())];
        backend()
            .load_comments(&items)
            .pop()
            .unwrap_or_else(|| snapshot.clone())
    }

    /// [`Self::load_comment`] for many files at once. Returns the snapshots in the same order.
    pub fn load_comments(items: &[(PathBuf, FileSnapshot)]) -> Vec<FileSnapshot> {
        backend().load_comments(items)
    }

    /// Move the file's comment or in/out points as `what` says, leaving the rest where it is.
    pub fn move_metadata(path: &Path, what: MetadataMove) -> MoveOutcome {
        if !backend().metadata_move_needed(path, what) {
            return MoveOutcome::NothingToMove;
        }
        let new_path = backend().move_metadata(path, what);
        if backend().metadata_move_needed(&new_path, what) {
            MoveOutcome::Failed(new_path)
        } else {
            MoveOutcome::Moved(new_path)
        }
    }

    /// Save a screenshot image for the given file and position.
    pub fn save_screenshot(file_path: &Path, position_ms: u64, image_data: &[u8]) {
        backend().save_screenshot(file_path, position_ms, image_data);
    }

    /// Load the raw image bytes for a screenshot by position.
    #[allow(dead_code)]
    pub fn load_screenshot_image(file_path: &Path, position_ms: u64) -> Option<Vec<u8>> {
        backend().load_screenshot_image(file_path, position_ms)
    }
}

/// Extension trait: save this snapshot then re-parse from the new path.
/// Returns `(new_path, new_snapshot)`.
pub trait SaveAndReparse {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, FileSnapshot);
}

impl SaveAndReparse for FileSnapshot {
    fn save_and_reparse(&self, path: &Path) -> (PathBuf, FileSnapshot) {
        let new_path = FileTagger::save(self, path);
        let folder_info = FolderInfo::default();
        let snapshot = FileTagger::parse(&new_path, &folder_info);
        (new_path, snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StoredTag, StoredTagStore};

    /// Tags follow the folder's order; a file already in order is left alone.
    #[test]
    fn tags_are_sorted_in_the_folder_order() {
        let folder = std::env::temp_dir().join(format!("frename-sort-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("temp dir");
        let mut store = FolderTagStore::for_folder(&folder);
        // Orders after the built-in tags', Zeta first.
        store
            .save_tag(
                StoredTag::with_all(uuid::Uuid::new_v4(), "Zeta", 1_000_000, false),
                0,
            )
            .expect("save");
        store
            .save_tag(
                StoredTag::with_all(uuid::Uuid::new_v4(), "Alpha", 2_000_000, false),
                1,
            )
            .expect("save");

        let path = folder.join("Alpha.Zeta.clip.mp4");
        let MoveOutcome::Moved(sorted) = FileTagger::sort_tags_by_folder_order(&path) else {
            panic!("the tags must be reordered");
        };
        assert_eq!(sorted, folder.join("Zeta.Alpha.clip.mp4"));
        assert_eq!(
            FileTagger::sort_tags_by_folder_order(&sorted),
            MoveOutcome::NothingToMove
        );
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// Core tests run with the default in-memory backend, so paths need not exist.
    #[test]
    fn the_commented_tag_follows_the_comment_of_the_file() {
        let path = Path::new(r"C:\frename-sync-test\Food.clip.mp4");
        let mut commented = FileSnapshot::parse("Food.clip.mp4");
        commented.set_comment("goat".to_string());
        let path = FileTagger::save(&commented, path);

        let MoveOutcome::Moved(tagged) = FileTagger::sync_commented_tag(&path, "Commented") else {
            panic!("the tag must be added");
        };
        assert!(
            tagged.ends_with("Food.Commented.clip.mp4"),
            "added last: {tagged:?}"
        );
        assert_eq!(
            FileTagger::sync_commented_tag(&tagged, "commented"),
            MoveOutcome::NothingToMove
        );

        let mut cleared = FileTagger::parse(&tagged, &FolderInfo::default());
        cleared.set_comment(String::new());
        let tagged = FileTagger::save(&cleared, &tagged);
        let MoveOutcome::Moved(untagged) = FileTagger::sync_commented_tag(&tagged, "Commented")
        else {
            panic!("the tag must be removed");
        };
        assert!(untagged.ends_with("Food.clip.mp4"), "removed: {untagged:?}");
    }
}
