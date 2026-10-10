//! Production FileTagger: renames files on disk.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::file_snapshot::FileSnapshot;
use super::file_tagger_backend::FileTaggerBackend;
use super::screenshot::Screenshot;
use super::FolderInfo;
use crate::markers::Marker;
use crate::metadata::{
    self, CommentStorage, InOutStorage, MarkersError, MetadataMove, MetadataStorage, Rotation,
    RotationError, XmpSource,
};

pub struct ProductionFileTagger;

/// Parsing with XMP storage for both reads both homes: the text file and the in/out line
/// first, XMP where they are empty.
const BOTH_HOMES: MetadataStorage = MetadataStorage {
    comment: CommentStorage::InVideo,
    in_out: InOutStorage::InVideo,
};

// ---------------------------------------------------------------------------
// Sidecar path helpers (private)
// ---------------------------------------------------------------------------

pub(crate) fn screenshot_path(file_path: &Path, position_ms: u64) -> PathBuf {
    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let time_str = Screenshot::new(position_ms).format_time();
    let sidecar_name = format!("{}.snap.{}.jpg", file_name, time_str);
    file_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(sidecar_name)
}

/// Positions (ms) of the screenshots saved for the clip at `file_path`: the files named exactly
/// `<clip file name>.snap.<time>.jpg` next to it. A longer clip name that merely starts with
/// this one (`foo.mp4.bak.snap.…`) is not matched.
pub(crate) fn screenshot_positions(file_path: &Path) -> Vec<u64> {
    let Some(name) = file_path.file_name().and_then(|n| n.to_str()) else {
        return Vec::new();
    };
    let prefix = format!("{name}.snap.");
    let dir = file_path.parent().unwrap_or(Path::new("."));
    let dir = if dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        dir
    };
    let mut positions: Vec<u64> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let entry_name = entry.file_name();
            let time = entry_name
                .to_str()?
                .strip_prefix(&prefix)?
                .strip_suffix(".jpg")?
                .to_owned();
            Screenshot::parse_time(&time)
        })
        .collect();
    positions.sort_unstable();
    positions.dedup();
    positions
}

/// Move the screenshots of `old_path` to the names of `new_path`, keeping their times. Never
/// overwrites; failures are logged only, like the other sidecars.
pub(crate) fn rename_screenshots(old_path: &Path, new_path: &Path) {
    for position_ms in screenshot_positions(old_path) {
        let (old, new) = (
            screenshot_path(old_path, position_ms),
            screenshot_path(new_path, position_ms),
        );
        if old == new {
            continue;
        }
        if new.exists() {
            log::warn!("screenshot: not renaming {old:?}: {new:?} already exists");
            continue;
        }
        if let Err(e) = std::fs::rename(&old, &new) {
            log::warn!("screenshot: failed to rename {old:?} → {new:?}: {e}");
        }
    }
}

pub(super) fn is_screenshot_sidecar(name: &str) -> bool {
    if let Some(idx) = name.find(".snap.") {
        let rest = &name[idx + 6..];
        if let Some(time_str) = rest.strip_suffix(".jpg") {
            return Screenshot::parse_time(time_str).is_some();
        }
    }
    false
}

// ---------------------------------------------------------------------------
// FileTaggerBackend impl
// ---------------------------------------------------------------------------

impl ProductionFileTagger {
    /// [`FileTaggerBackend::parse`] with the comment and in/out storage given explicitly.
    fn parse_with(
        &self,
        path: &Path,
        folder_info: &FolderInfo,
        storage: MetadataStorage,
    ) -> FileSnapshot {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let mut snapshot = FileSnapshot::parse(name);
        // Fallback for non-directory parsing paths (e.g. save_and_reparse):
        // if folder_info is empty, collect names once from the parent directory.
        let owned_info;
        let effective_info = if folder_info.file_names().is_empty() {
            let parent = path.parent().unwrap_or(Path::new("."));
            let file_names = std::fs::read_dir(parent)
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
                .collect();
            owned_info = FolderInfo::new(file_names);
            &owned_info
        } else {
            folder_info
        };
        // Look for the comment sidecar in the listing. Probing the disk instead cost a failed
        // file open per file, which a folder scan pays for every file it finds.
        let has_text_file = effective_info.contains(&format!("{name}.comment.txt"));
        // A folder scan does not open files: XMP comes from the tag file's file list or waits.
        let source = match (
            folder_info.defers_comment_loading(),
            folder_info.cached_file(name),
        ) {
            (false, _) => XmpSource::Read,
            (true, Some(cached)) => XmpSource::Cached(cached),
            (true, None) => XmpSource::Deferred,
        };
        metadata::load(path, has_text_file, &mut snapshot, storage, source);
        snapshot
    }

    /// [`FileTaggerBackend::save`] with the comment and in/out storage given explicitly.
    fn save_with(&self, snapshot: &FileSnapshot, path: &Path, storage: MetadataStorage) -> PathBuf {
        let new_file_name = snapshot.file_name();
        let new_path = path
            .parent()
            .map(|p| p.join(&new_file_name))
            .unwrap_or_else(|| PathBuf::from(&new_file_name));

        // Checked before any write: a rename onto an existing file (or its comment, subtitle or
        // transcript sidecar) would silently replace it — e.g. two clips that end up with the
        // same tags and base name (duplicated subclips, `clip.mp4` and `clip (1).mp4` renamed
        // alike). See `target_name_taken` for what "taken" excludes (a same-file case change).
        if new_path != path && super::file_tagger::target_name_taken(path, &new_path) {
            log::error!(
                "ProductionFileTagger: not saving {:?}: {:?} (or a sidecar) already exists",
                path,
                new_path
            );
            return path.to_path_buf();
        }

        // XMP goes in before the rename: the metadata then travels with the file.
        let saved = metadata::save_to_xmp(path, snapshot, storage);

        if new_path != path {
            // The video first: when it cannot be renamed (Premiere holds it), its comment and
            // subtitle files must stay with it rather than wait under the new name.
            if let Err(e) = std::fs::rename(path, &new_path) {
                log::error!(
                    "ProductionFileTagger: rename {:?} → {:?} failed: {}",
                    path,
                    new_path,
                    e
                );
                return path.to_path_buf();
            }
            super::file_tagger::rename_sidecars(path, &new_path);
            log::info!("Renamed on disk: {:?} → {:?}", path, new_path);
        }
        // A pending snapshot does not know an XMP comment, so it has no text file to write.
        // In/out points the video did not take are the text's in/out line.
        if !(snapshot.comment_loading() && storage.comment == CommentStorage::InVideo) {
            let text = metadata::stored_comment(snapshot, saved.in_out);
            metadata::save_comment_text_file(&new_path, &text, saved.comment);
        }
        metadata::cache::refresh_after_save(path, &new_path);
        new_path
    }
}

impl FileTaggerBackend for ProductionFileTagger {
    fn parse(&self, path: &Path, folder_info: &FolderInfo) -> FileSnapshot {
        self.parse_with(path, folder_info, metadata::metadata_storage())
    }

    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        self.save_with(snapshot, path, metadata::metadata_storage())
    }

    fn load_comments(&self, items: &[(PathBuf, FileSnapshot)]) -> Vec<FileSnapshot> {
        metadata::cache::resolve_batch(items, metadata::metadata_storage())
    }

    fn metadata_move_needed(&self, path: &Path, what: MetadataMove) -> bool {
        metadata::Inspection::of(path).moved_by(what).is_needed()
    }

    fn move_metadata(&self, path: &Path, what: MetadataMove) -> PathBuf {
        let inspection = metadata::Inspection::of(path);
        let moved = inspection.moved_by(what);
        if !moved.is_needed() {
            return path.to_path_buf();
        }
        let storage = inspection.storage_after(what);
        let snapshot = self.parse_with(path, &FolderInfo::default(), BOTH_HOMES);
        let new_path = self.save_with(&snapshot, path, storage);
        metadata::clear_moved_xmp(&new_path, moved, storage);
        log::info!(
            "metadata: moved {:?} → {:?} ({:?}: {:?})",
            path,
            new_path,
            what,
            moved
        );
        new_path
    }

    fn reload_metadata(&self, path: &Path) -> bool {
        metadata::cache::reload_line(path)
    }

    fn save_markers(
        &self,
        path: &Path,
        markers: &[Marker],
        known: &HashSet<String>,
    ) -> Result<(), MarkersError> {
        metadata::save_markers(path, markers, known)
    }

    fn rotate_video(&self, path: &Path, quarter_turns: i32) -> Result<Rotation, RotationError> {
        let rotated = metadata::rotation::rotate(path, quarter_turns);
        match &rotated {
            Ok(rotation) => log::info!(
                "rotation: {path:?} turned {quarter_turns:+} → {}°",
                rotation.degrees()
            ),
            Err(e) => log::warn!("rotation: {path:?} not turned: {e}"),
        }
        rotated
    }

    fn save_screenshot(&self, file_path: &Path, position_ms: u64, image_data: &[u8]) {
        let path = screenshot_path(file_path, position_ms);
        log::info!(
            "save_screenshot: writing {} bytes to {:?}",
            image_data.len(),
            path
        );
        match std::fs::write(&path, image_data) {
            Ok(()) => log::info!("save_screenshot: ok"),
            Err(e) => log::error!("save_screenshot: failed {:?}: {}", path, e),
        }
    }

    #[allow(dead_code)]
    fn load_screenshot_image(&self, file_path: &Path, position_ms: u64) -> Option<Vec<u8>> {
        std::fs::read(screenshot_path(file_path, position_ms)).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADOBE: MetadataStorage = MetadataStorage {
        comment: CommentStorage::InVideo,
        in_out: InOutStorage::InVideo,
    };
    const TEXT: MetadataStorage = MetadataStorage {
        comment: CommentStorage::TextFile,
        in_out: InOutStorage::Comment,
    };

    /// A fresh copy of the tiny QuickTime fixture named `name`, in its own temp folder.
    fn clip_named(test: &str, name: &str) -> PathBuf {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        let dir = crate::test_support::fresh_dir(&format!("tagger-{test}"));
        let file = dir.join(name);
        std::fs::copy(fixture, &file).expect("copy fixture");
        file
    }

    fn file_name(path: &Path) -> &str {
        path.file_name()
            .and_then(|n| n.to_str())
            .expect("file name")
    }

    fn points(snapshot: &FileSnapshot) -> (Option<f32>, Option<f32>) {
        (snapshot.segment_start(), snapshot.segment_end())
    }

    #[test]
    fn in_out_names_of_older_versions_are_kept_as_they_are() {
        let tagger = ProductionFileTagger;
        let path = clip_named("old-name", "goat.in_00_00_01.out_00_00_02.mov");
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        assert_eq!(points(&snapshot), (None, None));
        let new_path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(file_name(&new_path), "goat.in_00_00_01.out_00_00_02.mov");
    }

    #[test]
    fn adobe_storage_keeps_in_out_in_the_video_and_out_of_the_name() {
        let tagger = ProductionFileTagger;
        let path = clip_named("adobe", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_segment_start(Some(0.05));
        snapshot.set_segment_end(Some(0.1));
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(file_name(&path), "goat.mov");
        assert_eq!(
            points(&tagger.parse_with(&path, &FolderInfo::default(), ADOBE)),
            (Some(0.05), Some(0.1))
        );
        assert_eq!(crate::metadata::xmp_comment(&path), "", "no line");
    }

    #[test]
    fn comment_storage_keeps_in_out_as_a_line_of_the_text_file() {
        let tagger = ProductionFileTagger;
        let path = clip_named("text-line", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), TEXT);
        snapshot.set_comment("note".to_string());
        snapshot.set_segment_end(Some(0.1));
        let path = tagger.save_with(&snapshot, &path, TEXT);
        assert_eq!(file_name(&path), "goat.mov");
        assert_eq!(
            crate::comment::load_comment(&path),
            "note\nIn/Out: start – 00:00:00.100"
        );
        let back = tagger.parse_with(&path, &FolderInfo::default(), TEXT);
        assert_eq!(back.comment(), "note");
        assert_eq!(points(&back), (None, Some(0.1)));

        // In/out alone still makes a text file, and clearing them removes it.
        let mut only = back.clone();
        only.set_comment(String::new());
        let path = tagger.save_with(&only, &path, TEXT);
        assert_eq!(
            crate::comment::load_comment(&path),
            "In/Out: start – 00:00:00.100"
        );
        only.set_segment_end(None);
        let path = tagger.save_with(&only, &path, TEXT);
        assert!(!crate::comment::comment_path(&path).exists());
    }

    #[test]
    fn in_out_a_marker_cannot_express_goes_into_the_comment() {
        // The fixture is 0.2 s long, so an in at 1 s with no out is an empty range.
        let tagger = ProductionFileTagger;
        let path = clip_named("past-end", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_segment_start(Some(1.0));
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(file_name(&path), "goat.mov");
        assert_eq!(
            points(&tagger.parse_with(&path, &FolderInfo::default(), ADOBE)),
            (Some(1.0), None)
        );
    }

    #[test]
    fn in_out_goes_into_the_text_file_when_the_file_cannot_hold_xmp() {
        let tagger = ProductionFileTagger;
        let path = clip_named("unsupported", "notes.zip");
        std::fs::write(&path, b"not really a zip").expect("write");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_segment_start(Some(1.0));
        let new_path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(file_name(&new_path), "notes.zip");
        assert_eq!(
            crate::comment::load_comment(&new_path),
            "In/Out: 00:00:01.000 – end"
        );
        assert_eq!(
            points(&tagger.parse_with(&new_path, &FolderInfo::default(), ADOBE)),
            (Some(1.0), None)
        );
    }

    #[test]
    fn comment_storage_setting_picks_the_home_of_the_comment() {
        let tagger = ProductionFileTagger;
        let path = clip_named("comment-setting", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), TEXT);
        snapshot.set_comment("in a text file".to_string());
        let path = tagger.save_with(&snapshot, &path, TEXT);
        assert!(crate::comment::comment_path(&path).exists());

        // Switching to XMP moves the comment into the file and drops the text file.
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        assert_eq!(snapshot.comment(), "in a text file");
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        assert!(!crate::comment::comment_path(&path).exists());
        assert_eq!(
            tagger
                .parse_with(&path, &FolderInfo::default(), ADOBE)
                .comment(),
            "in a text file"
        );
    }

    /// A clip whose text file holds `comment` with an in/out line for an in at 0.05 s.
    fn clip_with_line(test: &str, comment: &str) -> PathBuf {
        let path = clip_named(test, "goat.mov");
        crate::comment::save_comment(&path, &format!("In/Out: 00:00:00.050 – end\n{comment}"));
        path
    }

    #[test]
    fn comments_move_into_xmp_and_back_taking_their_in_out_line_along() {
        let tagger = ProductionFileTagger;
        let path = clip_with_line("move-comments", "old comment");
        let into_video = MetadataMove::Comments(CommentStorage::InVideo);

        assert!(tagger.metadata_move_needed(&path, into_video));
        let path = tagger.move_metadata(&path, into_video);
        assert_eq!(file_name(&path), "goat.mov");
        assert!(!crate::comment::comment_path(&path).exists());
        assert!(!tagger.metadata_move_needed(&path, into_video));
        assert_eq!(
            crate::metadata::xmp_comment(&path),
            "old comment\nIn/Out: 00:00:00.050 – end",
            "the in/out stays in the comment"
        );
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        assert_eq!(snapshot.comment(), "old comment");
        assert_eq!(points(&snapshot), (Some(0.05), None));

        // And back: the XMP copy is removed, so nothing stale is left for Premiere.
        let into_text = MetadataMove::Comments(CommentStorage::TextFile);
        assert!(tagger.metadata_move_needed(&path, into_text));
        let path = tagger.move_metadata(&path, into_text);
        assert_eq!(
            crate::comment::load_comment(&path),
            "old comment\nIn/Out: 00:00:00.050 – end"
        );
        assert!(!tagger.metadata_move_needed(&path, into_text));
        std::fs::remove_file(crate::comment::comment_path(&path)).expect("remove text file");
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        assert_eq!(snapshot.comment(), "");
        assert_eq!(points(&snapshot), (None, None));
    }

    #[test]
    fn in_out_moves_into_xmp_and_back_leaving_the_text_comment_alone() {
        let tagger = ProductionFileTagger;
        let path = clip_with_line("move-in-out", "note");
        let into_video = MetadataMove::InOut(InOutStorage::InVideo);

        assert!(tagger.metadata_move_needed(&path, into_video));
        let path = tagger.move_metadata(&path, into_video);
        assert_eq!(file_name(&path), "goat.mov");
        assert_eq!(
            crate::comment::load_comment(&path),
            "note",
            "the line left the text file"
        );
        assert_eq!(
            points(&tagger.parse_with(&path, &FolderInfo::default(), ADOBE)),
            (Some(0.05), None)
        );
        assert!(!tagger.metadata_move_needed(&path, into_video));

        let into_comment = MetadataMove::InOut(InOutStorage::Comment);
        assert!(tagger.metadata_move_needed(&path, into_comment));
        let path = tagger.move_metadata(&path, into_comment);
        assert_eq!(
            crate::comment::load_comment(&path),
            "note\nIn/Out: 00:00:00.050 – end"
        );
        assert!(!tagger.metadata_move_needed(&path, into_comment));
        // The marker in the video went: without the line nothing is left to move back.
        crate::comment::save_comment(&path, "note");
        assert!(!tagger.metadata_move_needed(&path, into_comment));
        assert_eq!(
            points(&tagger.parse_with(&path, &FolderInfo::default(), ADOBE)),
            (None, None)
        );
    }

    #[test]
    fn moving_comments_to_text_keeps_an_xmp_comment_that_a_text_file_would_hide() {
        let tagger = ProductionFileTagger;
        let path = clip_named("no-loss", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_comment("in xmp".to_string());
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        crate::comment::save_comment(&path, "in text");

        // The text file wins, so there is nothing to move, and the XMP text is not removed.
        let into_text = MetadataMove::Comments(CommentStorage::TextFile);
        assert!(!tagger.metadata_move_needed(&path, into_text));
        let path = tagger.move_metadata(&path, into_text);
        std::fs::remove_file(crate::comment::comment_path(&path)).expect("remove text file");
        assert_eq!(
            tagger
                .parse_with(&path, &FolderInfo::default(), ADOBE)
                .comment(),
            "in xmp"
        );
    }

    #[test]
    fn files_already_in_the_destination_have_nothing_to_move() {
        let tagger = ProductionFileTagger;
        let path = clip_with_line("nothing", "text");
        let into_text = MetadataMove::Comments(CommentStorage::TextFile);
        let into_comment = MetadataMove::InOut(InOutStorage::Comment);
        assert!(!tagger.metadata_move_needed(&path, into_text));
        assert!(!tagger.metadata_move_needed(&path, into_comment));
        assert_eq!(tagger.move_metadata(&path, into_text), path);
    }

    #[test]
    fn nothing_moves_into_a_file_that_cannot_hold_xmp() {
        let tagger = ProductionFileTagger;
        let path = clip_named("move-unsupported", "notes.zip");
        std::fs::write(&path, b"not really a zip").expect("write");
        crate::comment::save_comment(&path, "In/Out: 00:00:01 – end\ntext");
        assert!(
            !tagger.metadata_move_needed(&path, MetadataMove::Comments(CommentStorage::InVideo))
        );
        assert!(!tagger.metadata_move_needed(&path, MetadataMove::InOut(InOutStorage::InVideo)));
    }

    #[test]
    fn a_save_that_renames_moves_the_same_sidecars_as_undo_does() {
        let tagger = ProductionFileTagger;
        let path = clip_named("rename-sidecars", "goat.mov");
        crate::comment::save_comment(&path, "note");
        let mut old_sidecars = crate::subtitles::subtitle_candidates(&path);
        old_sidecars.push(crate::subtitles::transcript_path(&path));
        old_sidecars.push(crate::comment::comment_path(&path));
        old_sidecars.push(screenshot_path(&path, 10_936));
        old_sidecars.push(screenshot_path(&path, 0));
        for p in &old_sidecars {
            std::fs::write(p, "x").expect("sidecar");
        }
        let bak = path.with_file_name("goat.mov.bak.snap.00-00-01-000.jpg");
        std::fs::write(&bak, "other").expect("other clip's screenshot");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), TEXT);
        snapshot.set_tags(["Food"]);
        let new_path = tagger.save_with(&snapshot, &path, TEXT);
        assert_ne!(new_path, path);

        let mut expected = crate::subtitles::subtitle_candidates(&new_path);
        expected.push(crate::subtitles::transcript_path(&new_path));
        expected.push(crate::comment::comment_path(&new_path));
        expected.push(screenshot_path(&new_path, 10_936));
        expected.push(screenshot_path(&new_path, 0));
        for p in &expected {
            assert!(p.exists(), "{p:?} should have moved with the clip");
        }
        for p in &old_sidecars {
            assert!(!p.exists(), "{p:?} should have left the old name");
        }
        assert!(bak.exists());
    }

    #[test]
    fn a_failed_rename_keeps_the_comment_with_the_video() {
        let tagger = ProductionFileTagger;
        let path = clip_named("rename-fails", "goat.mov");
        crate::comment::save_comment(&path, "note");
        // A folder under the new name makes the rename fail, as a clip Premiere holds would.
        std::fs::create_dir(path.with_file_name("Food.goat.mov")).expect("folder");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), TEXT);
        snapshot.set_tags(["Food"]);
        assert_eq!(tagger.save_with(&snapshot, &path, TEXT), path);
        assert_eq!(crate::comment::load_comment(&path), "note");
        assert!(!path.with_file_name("Food.goat.mov.comment.txt").exists());
    }

    #[test]
    fn a_text_file_with_only_the_in_out_line_never_replaces_the_videos_comment() {
        // The Premiere description is in the video; comments are then kept in text files and
        // an in point is set, which writes a text file holding only the line.
        let tagger = ProductionFileTagger;
        let path = clip_named("line-only-text", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_comment("Premiere note".to_string());
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), TEXT);
        snapshot.set_segment_start(Some(0.05));
        let path = tagger.save_with(&snapshot, &path, TEXT);
        assert_eq!(
            crate::comment::load_comment(&path),
            "In/Out: 00:00:00.050 – end"
        );

        // Read with comments in the video, the video's comment is still the comment.
        let in_video = MetadataStorage {
            comment: CommentStorage::InVideo,
            in_out: InOutStorage::Comment,
        };
        let back = tagger.parse_with(&path, &FolderInfo::default(), in_video);
        assert_eq!(back.comment(), "Premiere note");
        assert_eq!(points(&back), (Some(0.05), None));

        // Moving comments into the video merges the line in; the description stays.
        let into_video = MetadataMove::Comments(CommentStorage::InVideo);
        assert!(tagger.metadata_move_needed(&path, into_video));
        let path = tagger.move_metadata(&path, into_video);
        assert_eq!(
            crate::metadata::xmp_comment(&path),
            "Premiere note\nIn/Out: 00:00:00.050 – end"
        );
        assert!(!crate::comment::comment_path(&path).exists());
    }

    #[test]
    fn moving_comments_to_text_files_takes_the_videos_comment_past_a_line_only_text_file() {
        let tagger = ProductionFileTagger;
        let path = clip_named("line-only-text-out", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_comment("Premiere note".to_string());
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        crate::comment::save_comment(&path, "In/Out: 00:00:00.050 – end");

        let into_text = MetadataMove::Comments(CommentStorage::TextFile);
        assert!(tagger.metadata_move_needed(&path, into_text));
        let path = tagger.move_metadata(&path, into_text);
        assert_eq!(
            crate::comment::load_comment(&path),
            "Premiere note\nIn/Out: 00:00:00.050 – end"
        );
        assert_eq!(crate::metadata::xmp_comment(&path), "");
    }

    /// Issue #84: a tag change that renames a clip onto a name another clip (or its comment
    /// sidecar) already has must not silently replace it. This is the main save path
    /// (`save_with`, reached on every tag change), not the "move in/out out of name" action.
    #[test]
    fn saving_a_name_another_clip_has_does_not_overwrite_it_or_its_comment() {
        let tagger = ProductionFileTagger;
        let path = clip_named("save-name-taken", "old.mov");
        crate::comment::save_comment(&path, "mine");

        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_tags(["Goat"]);
        let target_name = snapshot.file_name();
        let other = path.with_file_name(&target_name);
        std::fs::write(&other, b"another clip").expect("write");
        crate::comment::save_comment(&other, "not mine");

        let result = tagger.save_with(&snapshot, &path, ADOBE);

        assert_eq!(result, path, "refused: the clip is left where it was");
        assert!(path.exists(), "the clip itself is not lost");
        assert_eq!(
            crate::comment::load_comment(&path),
            "mine",
            "nor its comment"
        );
        assert_eq!(
            std::fs::read(&other).expect("read"),
            b"another clip",
            "the other clip is untouched"
        );
        assert_eq!(crate::comment::load_comment(&other), "not mine");
    }

    /// Folder info as a scan builds it: the listing's sizes and times, and the tag file's list.
    fn scan_info(folder: &Path) -> FolderInfo {
        let mut names = Vec::new();
        let mut stats = std::collections::HashMap::new();
        for entry in std::fs::read_dir(folder).expect("folder").flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let metadata = entry.metadata().expect("metadata");
            if metadata.is_file() {
                let modified =
                    crate::metadata::cache::modified_ms(metadata.modified().expect("time"));
                stats.insert(name.clone(), (metadata.len(), modified));
            }
            names.push(name);
        }
        FolderInfo::for_scan(names, stats, crate::FolderTagStore::read_file_cache(folder))
    }

    /// A clip whose XMP holds `comment`, written as the app would.
    fn commented_clip(test: &str, comment: &str) -> PathBuf {
        let tagger = ProductionFileTagger;
        let path = clip_named(test, "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_comment(comment.to_string());
        tagger.save_with(&snapshot, &path, ADOBE)
    }

    #[test]
    fn a_scan_takes_xmp_from_the_file_list_and_defers_the_rest() {
        let tagger = ProductionFileTagger;
        let path = commented_clip("scan-cache", "goat");
        let folder = path.parent().expect("folder").to_path_buf();

        // Saving recorded the file in the list, so the scan knows its comment unopened.
        let hit = tagger.parse_with(&path, &scan_info(&folder), ADOBE);
        assert!(!hit.comment_loading());
        assert_eq!(hit.comment(), "goat");

        // Without a line for it, the scan leaves the file for later rather than opening it.
        crate::FolderTagStore::update_file_cache(
            &folder,
            &[file_name(&path).to_string()],
            Vec::new(),
        );
        let miss = tagger.parse_with(&path, &scan_info(&folder), ADOBE);
        assert!(miss.comment_loading());
        assert_eq!(miss.comment(), "");

        // Reading it later gives the comment, and records it for the next scan.
        let items = [(path.clone(), miss)];
        let resolved = tagger.load_comments(&items);
        assert_eq!(resolved[0].comment(), "goat");
        assert!(!resolved[0].comment_loading());
        assert_eq!(
            tagger
                .parse_with(&path, &scan_info(&folder), ADOBE)
                .comment(),
            "goat"
        );
    }

    #[test]
    fn a_line_that_no_longer_matches_the_file_is_not_trusted() {
        let tagger = ProductionFileTagger;
        let path = commented_clip("scan-stale", "goat");
        let folder = path.parent().expect("folder").to_path_buf();
        let mut line = crate::FolderTagStore::read_file_cache(&folder)
            .pop()
            .expect("line");
        line.size += 1;
        line.comment = "stale".into();
        crate::FolderTagStore::update_file_cache(&folder, &[], vec![line]);

        let snapshot = tagger.parse_with(&path, &scan_info(&folder), ADOBE);
        assert!(snapshot.comment_loading(), "a changed file is read again");
    }

    #[test]
    fn reloading_replaces_a_stale_line_and_leaves_a_good_one() {
        let tagger = ProductionFileTagger;
        let path = commented_clip("reload", "goat");
        let folder = path.parent().expect("folder").to_path_buf();
        assert!(
            !tagger.reload_metadata(&path),
            "saving wrote a matching line"
        );

        let mut line = crate::FolderTagStore::read_file_cache(&folder)
            .pop()
            .expect("line");
        line.comment = "stale".into();
        crate::FolderTagStore::update_file_cache(&folder, &[], vec![line]);
        assert!(tagger.reload_metadata(&path));
        let line = crate::FolderTagStore::read_file_cache(&folder)
            .pop()
            .expect("line");
        assert_eq!(line.comment, "goat", "read again from the video");
    }

    #[test]
    fn a_comment_edited_in_the_file_list_is_shown_and_saved_into_the_video() {
        let tagger = ProductionFileTagger;
        let path = commented_clip("scan-ai-edit", "goat");
        let folder = path.parent().expect("folder").to_path_buf();
        let mut line = crate::FolderTagStore::read_file_cache(&folder)
            .pop()
            .expect("line");
        line.comment = "edited by hand".into();
        crate::FolderTagStore::update_file_cache(&folder, &[], vec![line]);

        let snapshot = tagger.parse_with(&path, &scan_info(&folder), ADOBE);
        assert_eq!(snapshot.comment(), "edited by hand");
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(
            tagger
                .parse_with(&path, &FolderInfo::default(), ADOBE)
                .comment(),
            "edited by hand"
        );
    }

    #[test]
    fn saving_a_file_whose_xmp_was_not_read_keeps_its_comment_and_tag() {
        let tagger = ProductionFileTagger;
        let path = commented_clip("scan-pending-save", "goat");
        let path = {
            let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
            snapshot.set_tags(["Commented"]);
            tagger.save_with(&snapshot, &path, ADOBE)
        };
        let folder = path.parent().expect("folder").to_path_buf();
        crate::FolderTagStore::update_file_cache(
            &folder,
            &[file_name(&path).to_string()],
            Vec::new(),
        );

        let pending = tagger.parse_with(&path, &scan_info(&folder), ADOBE);
        assert!(pending.comment_loading());
        let path = tagger.save_with(&pending, &path, ADOBE);
        assert_eq!(file_name(&path), "Commented.goat.mov", "tag kept");
        assert_eq!(
            tagger
                .parse_with(&path, &FolderInfo::default(), ADOBE)
                .comment(),
            "goat",
            "comment kept"
        );
    }

    /// Issue #125 asked whether a cleared comment is really gone from the video, the text file
    /// and the file list. It is: a guard for that path (scan, edit, save, scan again), not a
    /// reproduction of the bug, which was in the workspace (see the folder workspace tests).
    #[test]
    fn a_cleared_comment_stays_cleared_in_the_video_and_the_file_list() {
        let tagger = ProductionFileTagger;
        let cases = ["tiny.mov", "wide.mov", "wide.mp4"]
            .into_iter()
            .flat_map(|clip| {
                [(ADOBE, false), (TEXT, false), (ADOBE, true), (TEXT, true)]
                    .map(|(storage, tag)| (clip, storage, tag))
            });
        for (clip, storage, tag) in cases {
            let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(clip);
            let path = clip_named("cleared", &format!("goat.{}", &clip[clip.len() - 3..]));
            std::fs::copy(fixture, &path).expect("copy fixture");
            let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), storage);
            snapshot.set_comment("goat".to_string());
            if tag {
                snapshot.set_tags(["Commented"]);
            }
            let path = tagger.save_with(&snapshot, &path, storage);
            let folder = path.parent().expect("folder").to_path_buf();

            // The user opens it from a scan's list, clears the comment and leaves.
            let mut open = tagger.parse_with(&path, &scan_info(&folder), storage);
            assert_eq!(open.comment(), "goat");
            open.set_comment(String::new());
            if tag {
                // Clearing the comment unchecks the commented tag, which renames the clip.
                open.set_tags(Vec::<String>::new());
            }
            let path = tagger.save_with(&open, &path, storage);

            let folder = path.parent().expect("folder").to_path_buf();
            assert_eq!(
                tagger
                    .parse_with(&path, &scan_info(&folder), storage)
                    .comment(),
                "",
                "{clip} {storage:?} {tag}: the next scan reads it empty"
            );
            assert_eq!(
                tagger
                    .parse_with(&path, &FolderInfo::default(), storage)
                    .comment(),
                "",
                "{clip} {storage:?} {tag}: the file itself is empty"
            );
            assert!(
                !crate::comment::comment_path(&path).exists(),
                "{clip} {storage:?} {tag}"
            );
            if storage.comment == CommentStorage::InVideo {
                assert_eq!(metadata::xmp_comment(&path), "");
            }
        }
    }
}
