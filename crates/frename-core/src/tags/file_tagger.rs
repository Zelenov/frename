//! FileTagger facade: routes parse/save to the installed backend.
//!
//! Call `install_file_tagger(backend)` once at startup (in main.rs) before any
//! file operations. If no backend is installed the default is InMemoryFileTagger,
//! which is always correct for tests and debug builds.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::{FileTaggerBackend, Saved};
use super::folder_info::FolderInfo;
use super::folder_tag_store::FolderTagStore;
use super::in_memory_file_tagger::InMemoryFileTagger;
use super::tag_list::TagList;
use crate::markers::Marker;
use crate::metadata::{MarkersError, MetadataMove, MoveOutcome, Rotation, RotationError};

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

    /// [`Self::save`], and what the save did with the in/out points (see [`Saved`]).
    pub fn save_reporting(snapshot: &FileSnapshot, path: &Path) -> Saved {
        backend().save_reporting(snapshot, path)
    }

    /// Bring the tag `tag` in line with the file's comment: added last when the file has a
    /// comment of the editor's (an AI block alone does not count), removed when it has none. Returns the outcome like a move: `NothingToMove` when
    /// the name already matches, `Failed` when the rename did not happen.
    pub fn sync_commented_tag(path: &Path, tag: &str) -> MoveOutcome {
        let mut snapshot = Self::parse(path, &FolderInfo::default());
        // Only the editor's own text counts; an AI description alone does not.
        let commented = crate::ai::has_editor_comment(snapshot.comment());
        let tagged = snapshot.tags().iter().any(|t| super::same_tag_name(t, tag));
        if commented == tagged {
            return MoveOutcome::NothingToMove;
        }
        let mut tags: Vec<String> = snapshot
            .tags()
            .iter()
            .filter(|t| !super::same_tag_name(t, tag))
            .cloned()
            .collect();
        if commented {
            tags.push(tag.to_string());
        }
        snapshot.set_tags(tags);
        Self::renamed(Self::save_reporting(&snapshot, path), path)
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
        Self::renamed(Self::save_reporting(&snapshot, path), path)
    }

    /// Rename the file at `path` to the tag spacing chosen now (see
    /// [`crate::set_space_after_tags`]). Returns the outcome like a move.
    pub fn respace_tags(path: &Path) -> MoveOutcome {
        let mut snapshot = Self::parse(path, &FolderInfo::default());
        // Built in the spacing chosen now, not kept as the file spells it.
        snapshot.set_initial_file_name("");
        let current = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if snapshot.file_name() == current {
            return MoveOutcome::NothingToMove;
        }
        Self::renamed(Self::save_reporting(&snapshot, path), path)
    }

    /// "Comment → markers": turn the lines of the file's comment that start with a time
    /// (`03:24 — shaky`) into clip markers and take them out of the comment (see
    /// [`crate::comment_to_markers`]): a line for a moment that has a marker renames it in
    /// place, and markers the clip already has on one moment are merged. The comment changes
    /// only after the markers were written, so a failed write leaves the comment as it was (the
    /// markers may already be merged). A comment save that does not take (a name taken) is
    /// `Failed` even when the markers were already written. Lines past the end of the clip stay.
    pub fn comment_to_markers(path: &Path) -> Result<MoveOutcome, MarkersError> {
        Self::comment_to_markers_reporting(path).map(|(outcome, _)| outcome)
    }

    /// [`Self::comment_to_markers`], and how many duplicate markers it merged away.
    pub fn comment_to_markers_reporting(path: &Path) -> Result<(MoveOutcome, usize), MarkersError> {
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
        // Markers a line renamed or recolored keep their place and GUID.
        let renamed: Vec<Marker> = markers
            .iter()
            .map(|m| {
                result
                    .updated
                    .iter()
                    .find(|u| u.guid.is_some() && u.guid == m.guid)
                    .unwrap_or(m)
                    .clone()
            })
            .chain(result.added.iter().cloned())
            .collect();
        // Markers on one moment are one marker (see [`crate::merge_duplicate_markers`]).
        let (all, merged_away) = crate::merge_duplicate_markers(&renamed);
        let markers_changed =
            !result.added.is_empty() || !result.updated.is_empty() || merged_away > 0;
        if result.lines_moved == 0 && !markers_changed {
            return Ok((MoveOutcome::NothingToMove, 0));
        }
        if markers_changed {
            Self::save_markers(path, &all, &known_guids(&markers))?;
        }
        if result.lines_moved == 0 {
            // Only AI segments became markers: the comment keeps its block as it is.
            return Ok((MoveOutcome::Moved(path.to_path_buf()), merged_away));
        }
        let was_commented = !snapshot.comment().trim().is_empty();
        snapshot.set_comment(result.comment);
        Self::follow_commented_tag(&mut snapshot, was_commented);
        Ok((Self::saved_outcome(&snapshot, path), merged_away))
    }

    /// "Markers → comment": append a line per clip marker to the file's comment, leaving out
    /// lines it already has (see [`crate::markers_to_comment`]). Markers the clip has twice on
    /// one moment are merged first, in the video as well. The markers stay in the file.
    pub fn markers_to_comment(path: &Path) -> Result<MoveOutcome, MarkersError> {
        Self::markers_to_comment_reporting(path).map(|(outcome, _)| outcome)
    }

    /// [`Self::markers_to_comment`], and how many duplicate markers it merged away.
    pub fn markers_to_comment_reporting(path: &Path) -> Result<(MoveOutcome, usize), MarkersError> {
        let mut snapshot = Self::parse(path, &FolderInfo::default());
        let Some(loaded) = Self::load_markers(path) else {
            return Ok((MoveOutcome::NothingToMove, 0));
        };
        let (markers, merged_away) = crate::merge_duplicate_markers(&loaded);
        if merged_away > 0 {
            Self::save_markers(path, &markers, &known_guids(&loaded))?;
        }
        let (comment, added) = crate::markers::markers_to_comment(snapshot.comment(), &markers);
        if added == 0 {
            let outcome = if merged_away > 0 {
                MoveOutcome::Moved(path.to_path_buf())
            } else {
                MoveOutcome::NothingToMove
            };
            return Ok((outcome, merged_away));
        }
        let was_commented = !snapshot.comment().trim().is_empty();
        snapshot.set_comment(comment);
        Self::follow_commented_tag(&mut snapshot, was_commented);
        Ok((Self::saved_outcome(&snapshot, path), merged_away))
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
            .filter(|t| !super::same_tag_name(t, &tag))
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

    /// Save `snapshot` to `path` and tell whether it took: `Failed` when what was read back
    /// differs from what was wanted (a name taken, a rename or write that failed), so a move
    /// that changed nothing is never reported as done (#503).
    fn saved_outcome(snapshot: &FileSnapshot, path: &Path) -> MoveOutcome {
        let (saved, read_back) = snapshot.save_and_reparse(path);
        if saved.name_too_long {
            MoveOutcome::NameTooLong(saved.path)
        } else if crate::recovery::saved_what_was_wanted(
            snapshot,
            &read_back,
            saved.points_in_comment,
        ) {
            MoveOutcome::Moved(saved.path)
        } else {
            log::warn!("markers: the comment of {path:?} was not saved");
            MoveOutcome::Failed(saved.path)
        }
    }

    /// The outcome of a save that must rename `path`: a save that failed keeps the old path, and
    /// says so apart when a sidecar's name would have been too long (#533).
    fn renamed(saved: Saved, path: &Path) -> MoveOutcome {
        let new_path = saved.path;
        if saved.name_too_long {
            MoveOutcome::NameTooLong(new_path)
        } else if new_path == path {
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

    /// Write the moments of an AI description into the file at `path` as markers in
    /// [`crate::AI_MARKER_COLOR`], replacing the AI markers a previous run wrote (see
    /// [`crate::replace_ai_markers`]). Moments past the end of the clip are left out.
    pub fn save_ai_markers(
        path: &Path,
        segments: &[crate::MarkerLine],
    ) -> Result<(), MarkersError> {
        let existing =
            Self::load_markers(path).ok_or_else(|| crate::metadata::cannot_hold_markers(path))?;
        let clip_length = crate::metadata::clip_length_ms(&Self::disk_path(path));
        let in_clip: Vec<crate::MarkerLine> = segments
            .iter()
            .filter(|s| clip_length.is_none_or(|clip| s.start_ms <= clip))
            .cloned()
            .collect();
        let (markers, dropped) = crate::markers::replace_ai_markers(&existing, &in_clip);
        Self::save_markers(path, &markers, &dropped)
    }

    /// How the video at `path` is turned: the display matrix of its first video track.
    pub fn video_rotation(path: &Path) -> Result<Rotation, RotationError> {
        backend().video_rotation(path)
    }

    /// Turn the video at `path` by `quarter_turns` clockwise (negative: counter-clockwise).
    /// Only the rotation flag of its video tracks changes, in place: the picture is not
    /// re-encoded, and the file keeps its size, times and metadata. Returns the new rotation.
    pub fn rotate_video(path: &Path, quarter_turns: i32) -> Result<Rotation, RotationError> {
        backend().rotate_video(path, quarter_turns)
    }

    /// Save a screenshot image for the given file and position. An error says it was not
    /// written (#424). No image (the player had no frame to give) writes nothing and is an
    /// `InvalidData` error: an empty `.jpg` next to the clip is no frame (#430).
    pub fn save_screenshot(
        file_path: &Path,
        position_ms: u64,
        image_data: &[u8],
    ) -> std::io::Result<()> {
        if image_data.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "no frame was captured",
            ));
        }
        backend().save_screenshot(file_path, position_ms, image_data)
    }

    /// Load the raw image bytes for a screenshot by position.
    #[allow(dead_code)]
    pub fn load_screenshot_image(file_path: &Path, position_ms: u64) -> Option<Vec<u8>> {
        backend().load_screenshot_image(file_path, position_ms)
    }
}

/// Whether `new_path`, or one of the sidecars that travel with it (comment, subtitle,
/// transcript, CAPT, screenshot at a time the old clip has one), already exists — checked before any write a rename would otherwise make, since
/// renaming onto an existing file (issue #84) or its sidecar would silently replace it.
///
/// A `new_path` that differs from `old_path` only by case is never "taken": on Windows' case-
/// insensitive, case-preserving filesystem it is the same file being renamed, not another one,
/// and a rename that only fixes case (`clip.MOV` → `clip.mov`) must still go through.
pub(crate) fn target_name_taken(old_path: &Path, new_path: &Path) -> bool {
    let same_file_other_case = old_path
        .file_name()
        .zip(new_path.file_name())
        .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b));
    if same_file_other_case {
        return false;
    }
    [
        new_path.to_path_buf(),
        crate::comment::comment_path(new_path),
        crate::subtitles::transcript_path(new_path),
        crate::subtitles::capt_path(new_path),
    ]
    .iter()
    .chain(crate::subtitles::subtitle_candidates(new_path).iter())
    .any(|p| p.exists())
        || super::production_file_tagger::screenshot_positions(old_path)
            .into_iter()
            .any(|ms| super::production_file_tagger::screenshot_path(new_path, ms).exists())
}

/// Whether the folder of `path` would refuse its file name as too long. Asked of the file
/// system, not guessed from the operating system: ext4 and APFS count 255 bytes of UTF-8, but an
/// exFAT card or an NTFS drive on Linux or macOS counts 255 UTF-16 units (a Cyrillic name fits
/// there that ext4 refuses). Looking a name up has no side effect and fails with
/// `ENAMETOOLONG` (`InvalidFilename`) only where the name really is too long. On Windows every
/// volume (NTFS, exFAT, FAT32) takes 255 UTF-16 units, so the name is also counted there.
fn name_too_long(path: &Path) -> bool {
    #[cfg(windows)]
    if path
        .file_name()
        .is_some_and(|name| std::os::windows::ffi::OsStrExt::encode_wide(name).count() > 255)
    {
        return true;
    }
    matches!(
        std::fs::symlink_metadata(path),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidFilename
    )
}

/// The first sidecar of `old_path` (a screenshot, a subtitle file, the CAPT file or the saved
/// transcript) whose name under `new_path` would be longer than a folder takes, or `None` when
/// every one fits. Such a sidecar could not follow the clip: it would stay under the old name,
/// lost to the clip (#526), so the clip keeps its name instead. Only sidecars that are there
/// count. The comment file is not among them: a comment that cannot be written has its own
/// notice (#416).
pub(crate) fn sidecar_name_too_long(old_path: &Path, new_path: &Path) -> Option<PathBuf> {
    if old_path == new_path {
        return None;
    }
    let screenshots = super::production_file_tagger::screenshot_positions(old_path)
        .into_iter()
        .map(|ms| super::production_file_tagger::screenshot_path(new_path, ms));
    let companions = crate::subtitles::companion_paths(old_path, new_path)
        .filter(|(old, _)| old.is_file())
        .map(|(_, new)| new);
    screenshots.chain(companions).find(|p| name_too_long(p))
}

/// Move the sidecars that travel with a renamed clip (comment, subtitles, transcript,
/// screenshots) from the
/// names of `old_path` to those of `new_path`. The one list for the save that renames a clip and
/// for its undo and redo, so they cannot drift apart; it matches the set `target_name_taken`
/// and `sidecar_name_too_long` guard.
///
/// Every file that can move is moved; the result says which did and the first one that could not
/// (another program holds it open, #622), so the caller can move them back or try again later.
pub(crate) fn rename_sidecars(old_path: &Path, new_path: &Path) -> SidecarMove {
    let mut moves = SidecarMove::default();
    super::production_file_tagger::rename_screenshots(old_path, new_path, &mut moves);
    crate::comment::rename_comment_file(old_path, new_path, &mut moves);
    crate::subtitles::rename_subtitle_file(old_path, new_path, &mut moves);
    moves
}

/// What moving a clip's side files did: the files moved (old path, new path) in order, and the
/// first move the system refused.
#[derive(Debug, Default)]
pub(crate) struct SidecarMove {
    moved: Vec<(PathBuf, PathBuf)>,
    failed: Option<std::io::Error>,
}

impl SidecarMove {
    /// Move one side file, recording what happened. `what` names it in the log.
    pub(crate) fn rename(&mut self, what: &str, old: &Path, new: &Path) {
        match rename_file(old, new) {
            Ok(()) => self.moved.push((old.to_path_buf(), new.to_path_buf())),
            Err(e) => {
                log::warn!("{what}: failed to rename {old:?} → {new:?}: {e}");
                self.failed.get_or_insert(e);
            }
        }
    }

    /// The first move the system refused, if any.
    pub(crate) fn into_result(self) -> Result<(), (Vec<(PathBuf, PathBuf)>, std::io::Error)> {
        match self.failed {
            None => Ok(()),
            Some(e) => Err((self.moved, e)),
        }
    }
}

#[cfg(test)]
thread_local! {
    static HELD: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Tests: on this thread, a file at `path` cannot be moved, as when another program holds it
/// open on Windows (#622).
#[cfg(test)]
pub(crate) fn hold_on_this_thread(path: &Path) {
    HELD.with(|held| held.borrow_mut().push(path.to_path_buf()));
}

/// `std::fs::rename`, which in tests refuses a file held with [`hold_on_this_thread`].
pub(crate) fn rename_file(old: &Path, new: &Path) -> std::io::Result<()> {
    #[cfg(test)]
    if HELD.with(|held| held.borrow().iter().any(|h| h == old)) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "held open by another program (test)",
        ));
    }
    std::fs::rename(old, new)
}

/// Move the side files in `moved` back to where they were, last first. False when one of them
/// could not go back (it stays under the new name).
pub(crate) fn move_back(moved: Vec<(PathBuf, PathBuf)>) -> bool {
    let mut all = true;
    for (old, new) in moved.into_iter().rev() {
        if let Err(e) = rename_file(&new, &old) {
            log::error!("could not move {new:?} back to {old:?}: {e}");
            all = false;
        }
    }
    all
}

/// Extension trait: save this snapshot then re-parse from the new path.
pub trait SaveAndReparse {
    /// What the save did (its path after, see [`Saved`]) and the snapshot read back from there.
    fn save_and_reparse(&self, path: &Path) -> (Saved, FileSnapshot);
}

impl SaveAndReparse for FileSnapshot {
    fn save_and_reparse(&self, path: &Path) -> (Saved, FileSnapshot) {
        let saved = FileTagger::save_reporting(self, path);
        let snapshot = FileTagger::parse(&saved.path, &FolderInfo::default());
        (saved, snapshot)
    }
}

/// The GUIDs of `markers`, which a save may replace or drop: a marker left out of the list
/// leaves the file only when its GUID is among them.
fn known_guids(markers: &[Marker]) -> HashSet<String> {
    markers.iter().filter_map(|m| m.guid.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StoredTag, StoredTagStore};

    /// Issue #84: a target that differs from the old path only by case must never count as
    /// "taken" (it is the same file on Windows' case-insensitive filesystem, and a rename that
    /// only fixes case must still go through). The check short-circuits on the name comparison
    /// before ever consulting the filesystem, so this holds the same way on every platform,
    /// including a case-sensitive one where the two names would otherwise be unrelated files.
    #[test]
    fn a_same_file_case_change_is_never_taken() {
        let dir = std::env::temp_dir().join(format!("frename-case-only-{}", std::process::id()));
        let old_path = dir.join("goat.mov");
        let new_path = dir.join("goat.MOV");
        assert!(!target_name_taken(&old_path, &new_path));
    }

    /// A `.capt` already standing at the new name blocks the rename, like the other sidecars.
    #[test]
    fn an_existing_capt_file_makes_the_new_name_taken() {
        let dir = crate::test_support::fresh_dir("capt-taken");
        let (old_path, new_path) = (dir.join("goat.mov"), dir.join("tag.goat.mov"));
        assert!(!target_name_taken(&old_path, &new_path));
        std::fs::write(crate::subtitles::capt_path(&new_path), "capt").expect("capt");
        assert!(target_name_taken(&old_path, &new_path));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #526: only a sidecar that is there counts, and only when its new name is too long for
    /// the folder (ext4 here: 255 bytes). A missing one, or one whose new name fits, does not.
    #[cfg(unix)]
    #[test]
    fn only_an_existing_sidecar_whose_new_name_is_too_long_counts() {
        let dir = crate::test_support::fresh_dir("sidecar-too-long");
        // 240 bytes: `.soniox.json` makes 252 of the old name, 257 with `.pick`.
        let stem = "я".repeat(120);
        let old_path = dir.join(format!("{stem}.mov"));
        let new_path = dir.join(format!("{stem}.pick.mov"));
        let short = dir.join("clip.pick.mov");
        assert_eq!(
            sidecar_name_too_long(&old_path, &new_path),
            None,
            "none there"
        );

        std::fs::write(crate::subtitles::transcript_path(&old_path), "{}").expect("transcript");
        assert_eq!(
            sidecar_name_too_long(&old_path, &new_path),
            Some(crate::subtitles::transcript_path(&new_path))
        );
        assert_eq!(sidecar_name_too_long(&old_path, &short), None, "fits");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #430: no captured frame writes no file and says why.
    #[test]
    fn an_empty_frame_is_not_saved() {
        let dir = crate::test_support::fresh_dir("empty-frame");
        let clip = dir.join("clip.mov");
        std::fs::write(&clip, b"clip").expect("clip");
        let err = FileTagger::save_screenshot(&clip, 1_000, &[]).expect_err("refused");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(
            std::fs::read_dir(&dir).expect("dir").count(),
            1,
            "nothing written"
        );
    }

    /// Tags follow the folder's order; a file already in order is left alone.
    #[test]
    fn tags_are_sorted_in_the_folder_order() {
        let folder = crate::test_support::fresh_dir("sort-test");
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

    /// #642: a save of a clip nobody changed keeps its spelling; the batch action that
    /// respaces names still renames it. (The space setting is off in tests.)
    #[test]
    fn only_respacing_renames_a_name_that_says_the_same() {
        let path = Path::new("/frename-respace-test/Food. Goat. clip.mp4");
        let kept = FileTagger::save(&FileSnapshot::parse("Food. Goat. clip.mp4"), path);
        assert_eq!(kept, path);

        let MoveOutcome::Moved(respaced) = FileTagger::respace_tags(path) else {
            panic!("the batch action renames it");
        };
        assert!(respaced.ends_with("Food.Goat.clip.mp4"), "{respaced:?}");
        assert!(matches!(
            FileTagger::respace_tags(&respaced),
            MoveOutcome::NothingToMove
        ));
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

        // #494: in any alphabet, not only A–Z.
        let path = Path::new(r"C:\frename-sync-test\Лес.clip.mp4");
        let mut cyrillic = FileSnapshot::parse("Лес.clip.mp4");
        cyrillic.set_comment("коза".to_string());
        let path = FileTagger::save(&cyrillic, path);
        let MoveOutcome::Moved(tagged_ru) = FileTagger::sync_commented_tag(&path, "Готово")
        else {
            panic!("the tag must be added");
        };
        assert_eq!(
            FileTagger::sync_commented_tag(&tagged_ru, "готово"),
            MoveOutcome::NothingToMove,
            "the tag is there in other capitals"
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
        let dir = crate::test_support::fresh_dir("markers-batch");
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

    /// The segments of an AI block become White markers in the video's XMP, ranges with their
    /// length, and the block stays in the comment; copying the markers back adds no line for
    /// them.
    #[test]
    fn ai_segments_become_white_ranged_markers_in_the_xmp() {
        let dir = crate::test_support::fresh_dir("markers-ai");
        let file = dir.join("clip.mov");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        std::fs::copy(fixture, &file).expect("copy fixture");
        let comment = "Mine

AI: A walk.
0:00–0:14 Street, handheld.
0:00 Flash.";
        let mut snapshot = FileSnapshot::parse("clip.mov");
        snapshot.set_comment(comment.into());
        let path = FileTagger::save(&snapshot, &file);

        let MoveOutcome::Moved(path) = FileTagger::comment_to_markers(&path).expect("written")
        else {
            panic!("AI segments must become markers");
        };
        let markers = FileTagger::load_markers(&path).expect("markers");
        let mut got: Vec<_> = markers
            .iter()
            .map(|m| (m.start_ms, m.duration_ms, m.name.as_str(), m.color))
            .collect();
        got.sort_by_key(|m| m.1);
        assert_eq!(
            got,
            [
                (0, 0, "Flash.", crate::AI_MARKER_COLOR),
                (0, 14_000, "Street, handheld.", crate::AI_MARKER_COLOR),
            ]
        );
        let read_comment = |path: &Path| {
            FileTagger::parse(path, &FolderInfo::default())
                .comment()
                .to_string()
        };
        assert_eq!(read_comment(&path), comment);
        assert_eq!(
            FileTagger::comment_to_markers(&path),
            Ok(MoveOutcome::NothingToMove)
        );
        assert_eq!(
            FileTagger::markers_to_comment(&path),
            Ok(MoveOutcome::NothingToMove)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A second AI run replaces the markers of the first, and leaves the editor's alone.
    #[test]
    fn ai_markers_written_again_replace_the_previous_run() {
        let dir = crate::test_support::fresh_dir("ai-markers");
        let file = dir.join("clip.mov");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        std::fs::copy(fixture, &file).expect("copy fixture");
        let segment = |name: &str| crate::MarkerLine {
            start_ms: 0,
            duration_ms: 150,
            name: name.to_string(),
            comment: String::new(),
            color: crate::MarkerColor::Green,
        };
        let mut mine = Marker::new(100);
        mine.name = "Mine".to_string();
        FileTagger::save_markers(&file, &[mine], &HashSet::new()).expect("mine");
        FileTagger::save_ai_markers(&file, &[segment("First run")]).expect("first");
        FileTagger::save_ai_markers(&file, &[segment("Second run")]).expect("second");
        let names: Vec<_> = FileTagger::load_markers(&file)
            .expect("markers")
            .into_iter()
            .map(|m| (m.name, m.duration_ms, m.color))
            .collect();
        assert_eq!(
            names,
            [
                ("Second run".to_string(), 150, crate::AI_MARKER_COLOR),
                ("Mine".to_string(), 0, crate::MarkerColor::Green),
            ]
        );
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
        // Its own clip name: on Linux the backslashes are no folders, so the in-memory store keys
        // the clip by its name alone, and another test's "Food.clip.mp4" would be this one (#436).
        let path = Path::new(r"C:\frename-sync-ai-test\Walk.clip.mp4");
        let mut described = FileSnapshot::parse("Walk.clip.mp4");
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

    /// #533: a batch rename that would leave the saved transcript behind under a name too long
    /// for the folder keeps the clip's name and says why, apart from any other refusal.
    #[test]
    fn a_rename_a_sidecar_name_would_not_fit_in_says_so() {
        let dir = crate::test_support::fresh_dir("sync-name-too-long");
        let name = format!("{}.mov", "A".repeat(240));
        let path = dir.join(&name);
        std::fs::write(&path, b"clip").expect("clip");
        std::fs::write(crate::subtitles::transcript_path(&path), "{}").expect("transcript");
        let mut commented = FileSnapshot::parse(&name);
        commented.set_comment("goat".to_string());
        let path = FileTagger::save(&commented, &path);

        assert_eq!(
            FileTagger::sync_commented_tag(&path, "Commented"),
            MoveOutcome::NameTooLong(path.clone())
        );
        assert!(
            crate::subtitles::transcript_path(&path).exists(),
            "kept with it"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A move whose save is refused (the commented tag's new name is another clip's) changed
    /// nothing, and says so: never `Moved`, which the batch list shows as Done (#503).
    #[test]
    fn a_move_whose_save_is_refused_is_failed_not_done() {
        let _storage = crate::use_storage_on_this_thread(
            crate::MetadataStorage {
                comment: crate::CommentStorage::InVideo,
                in_out: crate::InOutStorage::InVideo,
            },
            crate::MarkerStorage::InVideo,
        );
        let tag = crate::metadata::active_commented_tag().expect("commented tag on");
        let file = clip_with_markers("refused-to-comment", &[named(100, "Lion")]);
        let taken = file.with_file_name(format!("{tag}.clip.mov"));
        std::fs::write(&taken, b"another clip").expect("taken name");

        let outcome = FileTagger::markers_to_comment(&file).expect("move");
        assert_eq!(outcome, MoveOutcome::Failed(file.clone()));
        let comment = FileTagger::parse(&file, &FolderInfo::default())
            .comment()
            .to_string();
        assert_eq!(comment, "", "nothing was written");

        // The other way: a comment line the save cannot take off the clip.
        let mut snapshot = FileTagger::parse(&file, &FolderInfo::default());
        snapshot.set_comment("0:00.050 — Start".into());
        snapshot.set_tags([tag.as_str()]);
        std::fs::remove_file(&taken).expect("free the name");
        let tagged = FileTagger::save(&snapshot, &file);
        assert_ne!(tagged, file, "commented clip renamed");
        std::fs::write(&file, b"another clip").expect("taken name");
        let outcome = FileTagger::comment_to_markers(&tagged).expect("move");
        assert_eq!(outcome, MoveOutcome::Failed(tagged.clone()));
        let _ = std::fs::remove_dir_all(file.parent().expect("dir"));
    }

    fn clip_with_markers(name: &str, markers: &[Marker]) -> PathBuf {
        let dir = crate::test_support::fresh_dir(name);
        let file = dir.join("clip.mov");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        std::fs::copy(fixture, &file).expect("copy fixture");
        FileTagger::save_markers(&file, markers, &HashSet::new()).expect("markers");
        file
    }

    fn named(start_ms: u64, name: &str) -> Marker {
        let mut m = Marker::new(start_ms);
        m.name = name.to_string();
        m
    }

    /// Issue #145: the owner's two lines on one frame become one marker in either direction, and
    /// no number of rounds adds another.
    #[test]
    fn two_markers_on_one_frame_are_merged_by_both_move_actions() {
        let both = [
            named(63_558, "Субтитр: нет субтитра «Цельное дерево»"),
            named(63_558, "нет субтитра «Цельное дерево»"),
        ];
        let file = clip_with_markers("dup-to-comment", &both);
        FileTagger::markers_to_comment(&file).expect("move");
        let markers = FileTagger::load_markers(&file).expect("markers");
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].name, "Субтитр: нет субтитра «Цельное дерево»");
        let _ = std::fs::remove_dir_all(file.parent().unwrap());

        let file = clip_with_markers("dup-from-comment", &both);
        FileTagger::comment_to_markers(&file).expect("move");
        assert_eq!(FileTagger::load_markers(&file).expect("markers").len(), 1);
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }
}
