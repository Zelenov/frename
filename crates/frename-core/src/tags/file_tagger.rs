//! FileTagger facade: routes parse/save to the installed backend.
//!
//! Call `install_file_tagger(backend)` once at startup (in main.rs) before any
//! file operations. If no backend is installed the default is InMemoryFileTagger,
//! which is always correct for tests and debug builds.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::FileTaggerBackend;
use super::folder_info::FolderInfo;
use super::folder_tag_store::FolderTagStore;
use super::in_memory_file_tagger::InMemoryFileTagger;
use super::tag_list::TagList;
use crate::markers::Marker;
use crate::metadata::{MarkersError, MetadataMove, MoveOutcome};

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
    /// comment of the editor's (an AI block alone does not count), removed when it has none. Returns the outcome like a move: `NothingToMove` when
    /// the name already matches, `Failed` when the rename did not happen.
    pub fn sync_commented_tag(path: &Path, tag: &str) -> MoveOutcome {
        let mut snapshot = Self::parse(path, &FolderInfo::default());
        // Only the editor's own text counts; an AI description alone does not.
        let commented = crate::ai::has_editor_comment(snapshot.comment());
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

    /// Rename the file at `path` to the tag spacing chosen now (see
    /// [`crate::set_space_after_tags`]). Returns the outcome like a move.
    pub fn respace_tags(path: &Path) -> MoveOutcome {
        let snapshot = Self::parse(path, &FolderInfo::default());
        let current = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if snapshot.file_name() == current {
            return MoveOutcome::NothingToMove;
        }
        Self::renamed(Self::save(&snapshot, path), path)
    }

    /// "Comment → markers": turn the lines of the file's comment that start with a time
    /// (`03:24 — shaky`) into clip markers and take them out of the comment (see
    /// [`crate::comment_to_markers`]). The comment changes only after the markers were written,
    /// so a failed write leaves the file as it was. Lines past the end of the clip stay.
    pub fn comment_to_markers(path: &Path) -> Result<MoveOutcome, MarkersError> {
        let mut snapshot = Self::parse(path, &FolderInfo::default());
        let markers =
            Self::load_markers(path).ok_or_else(|| crate::metadata::cannot_hold_markers(path))?;
        let clip_length = crate::metadata::clip_length_ms(&Self::disk_path(path));
        let result = crate::markers::comment_to_markers(snapshot.comment(), &markers, clip_length);
        if result.past_end > 0 {
            log::info!(
                "markers: {} line(s) of {path:?} are past the end of the clip and stay",
                result.past_end
            );
        }
        if result.lines_moved == 0 {
            return Ok(MoveOutcome::NothingToMove);
        }
        if !result.added.is_empty() {
            let all: Vec<Marker> = markers.into_iter().chain(result.added).collect();
            Self::save_markers(path, &all, &HashSet::new())?;
        }
        let was_commented = !snapshot.comment().trim().is_empty();
        snapshot.set_comment(result.comment);
        Self::follow_commented_tag(&mut snapshot, was_commented);
        Ok(MoveOutcome::Moved(Self::save(&snapshot, path)))
    }

    /// "Markers → comment": append a line per clip marker to the file's comment, leaving out
    /// lines it already has (see [`crate::markers_to_comment`]). The markers stay in the file.
    pub fn markers_to_comment(path: &Path) -> Result<MoveOutcome, MarkersError> {
        let mut snapshot = Self::parse(path, &FolderInfo::default());
        let Some(markers) = Self::load_markers(path) else {
            return Ok(MoveOutcome::NothingToMove);
        };
        let (comment, added) = crate::markers::markers_to_comment(snapshot.comment(), &markers);
        if added == 0 {
            return Ok(MoveOutcome::NothingToMove);
        }
        let was_commented = !snapshot.comment().trim().is_empty();
        snapshot.set_comment(comment);
        Self::follow_commented_tag(&mut snapshot, was_commented);
        Ok(MoveOutcome::Moved(Self::save(&snapshot, path)))
    }

    /// The commented tag follows a comment that appeared or went, as it does when the comment
    /// is typed (see [`crate::active_commented_tag`]).
    fn follow_commented_tag(snapshot: &mut FileSnapshot, was_commented: bool) {
        let commented = !snapshot.comment().trim().is_empty();
        let Some(tag) =
            crate::metadata::active_commented_tag().filter(|_| commented != was_commented)
        else {
            return;
        };
        let mut tags: Vec<String> = snapshot
            .tags()
            .iter()
            .filter(|t| !t.eq_ignore_ascii_case(&tag))
            .cloned()
            .collect();
        if commented {
            tags.push(tag);
        }
        snapshot.set_tags(tags);
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

    /// The clip markers of the file at `path`, in time order; `None` when the file cannot hold
    /// them.
    pub fn load_markers(path: &Path) -> Option<Vec<Marker>> {
        backend().load_markers(path)
    }

    /// Write `markers` into the file at `path` (before it is renamed: they travel with it).
    /// Only markers with a GUID are written; see [`crate::Marker::guid`]. `known` is every GUID
    /// read from or written to the file before: a file marker with one of them that is not in
    /// `markers` was deleted, while other file markers (added elsewhere meanwhile) are kept.
    /// Nothing is written when the file already holds these markers.
    pub fn save_markers(
        path: &Path,
        markers: &[Marker],
        known: &HashSet<String>,
    ) -> Result<(), MarkersError> {
        backend().save_markers(path, markers, known)
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

    /// Both directions of "Markers ⇄ comment", run twice: the second run changes nothing.
    #[test]
    fn comment_lines_and_markers_convert_both_ways_and_reruns_change_nothing() {
        let dir =
            std::env::temp_dir().join(format!("frename-markers-batch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join("clip.mov");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        std::fs::copy(fixture, &file).expect("copy fixture");
        let mut snapshot = FileSnapshot::parse("clip.mov");
        // The fixture is 0.2 s long: the second line is past its end.
        snapshot.set_comment("Intro\n0:00.050 — Start — first frames\n9:00 — too late".into());
        let path = FileTagger::save(&snapshot, &file);

        let MoveOutcome::Moved(path) = FileTagger::comment_to_markers(&path).expect("moved") else {
            panic!("lines must move");
        };
        let markers = FileTagger::load_markers(&path).expect("markers");
        let got: Vec<_> = markers
            .iter()
            .map(|m| (m.start_ms, m.name.as_str(), m.comment.as_str()))
            .collect();
        assert_eq!(got, [(50, "Start", "first frames")]);
        let comment = |path: &Path| {
            FileTagger::parse(path, &FolderInfo::default())
                .comment()
                .to_string()
        };
        assert_eq!(comment(&path), "Intro\n9:00 — too late");
        assert_eq!(
            FileTagger::comment_to_markers(&path),
            Ok(MoveOutcome::NothingToMove)
        );

        let MoveOutcome::Moved(path) = FileTagger::markers_to_comment(&path).expect("copied")
        else {
            panic!("markers must be copied");
        };
        assert_eq!(
            comment(&path),
            "Intro\n9:00 — too late\n0:00.050 — Start — first frames"
        );
        assert_eq!(
            FileTagger::markers_to_comment(&path),
            Ok(MoveOutcome::NothingToMove)
        );
        // Back again: the copied line goes and no duplicate marker is added.
        FileTagger::comment_to_markers(&path).expect("moved");
        assert_eq!(FileTagger::load_markers(&path).expect("markers").len(), 1);
        assert_eq!(comment(&path), "Intro\n9:00 — too late");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_that_cannot_hold_markers_keeps_its_comment_lines() {
        let path = Path::new(r"C:\frename-no-markers\notes.zip");
        let mut snapshot = FileSnapshot::parse("notes.zip");
        snapshot.set_comment("0:01 — x".into());
        let path = FileTagger::save(&snapshot, path);
        assert_eq!(
            FileTagger::comment_to_markers(&path),
            Err(MarkersError::CannotHoldMarkers)
        );
        assert_eq!(
            FileTagger::parse(&path, &FolderInfo::default()).comment(),
            "0:01 — x"
        );
    }

    #[test]
    fn an_ai_description_alone_does_not_get_the_commented_tag() {
        let block = "AI: A walk.\n— Claude Haiku 4.5, 2026-09-26 —";
        let path = Path::new(r"C:\frename-sync-ai-test\Food.clip.mp4");
        let mut described = FileSnapshot::parse("Food.clip.mp4");
        described.set_comment(block.to_string());
        let path = FileTagger::save(&described, path);
        assert_eq!(
            FileTagger::sync_commented_tag(&path, "Commented"),
            MoveOutcome::NothingToMove
        );

        let mut commented = FileTagger::parse(&path, &FolderInfo::default());
        commented.set_comment(format!("Mine\n\n{block}"));
        let path = FileTagger::save(&commented, &path);
        let MoveOutcome::Moved(tagged) = FileTagger::sync_commented_tag(&path, "Commented") else {
            panic!("editor text plus a block counts");
        };

        let mut only_block = FileTagger::parse(&tagged, &FolderInfo::default());
        only_block.set_comment(block.to_string());
        let tagged = FileTagger::save(&only_block, &tagged);
        assert!(
            matches!(
                FileTagger::sync_commented_tag(&tagged, "Commented"),
                MoveOutcome::Moved(_)
            ),
            "removing the editor's text with the block kept removes the tag"
        );
    }
}
