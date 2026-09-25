//! Production FileTagger: renames files on disk.

use std::path::{Path, PathBuf};

use super::file_snapshot::FileSnapshot;
use super::screenshot::Screenshot;
use super::file_tagger_backend::FileTaggerBackend;
use super::FolderInfo;
use crate::metadata::{self, CommentStorage, FileConversion, InOutStorage, MetadataStorage, XmpSource};

pub struct ProductionFileTagger;

// ---------------------------------------------------------------------------
// Sidecar path helpers (private)
// ---------------------------------------------------------------------------

fn screenshot_path(file_path: &Path, position_ms: u64) -> PathBuf {
    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let time_str = Screenshot::new(position_ms).format_time();
    let sidecar_name = format!("{}.snap.{}.jpg", file_name, time_str);
    file_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(sidecar_name)
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

fn load_screenshot_positions(file_path: &Path, folder_info: &FolderInfo) -> Vec<Screenshot> {
    let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) else {
        return Vec::new();
    };
    let prefix = format!("{}.snap.", file_name);
    let mut screenshots: Vec<Screenshot> = folder_info
        .names_starting_with(&prefix)
        .filter_map(|name| {
            let rest = name.strip_prefix(prefix.as_str())?;
            let time_str = rest.strip_suffix(".jpg")?;
            Screenshot::parse_time(time_str).map(Screenshot::new)
        })
        .collect();
    screenshots.sort();
    screenshots
}

// ---------------------------------------------------------------------------
// FileTaggerBackend impl
// ---------------------------------------------------------------------------

impl ProductionFileTagger {
    /// [`FileTaggerBackend::parse`] with the comment and in/out storage given explicitly.
    fn parse_with(&self, path: &Path, folder_info: &FolderInfo, storage: MetadataStorage) -> FileSnapshot {
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
        let source = match (folder_info.defers_comment_loading(), folder_info.cached_file(name)) {
            (false, _) => XmpSource::Read,
            (true, Some(cached)) => XmpSource::Cached(cached),
            (true, None) => XmpSource::Deferred,
        };
        metadata::load(path, has_text_file, &mut snapshot, storage, source);
        snapshot.set_screenshots(load_screenshot_positions(path, effective_info));
        snapshot
    }

    /// [`FileTaggerBackend::save`] with the comment and in/out storage given explicitly.
    fn save_with(&self, snapshot: &FileSnapshot, path: &Path, storage: MetadataStorage) -> PathBuf {
        // XMP goes in before the rename: whether the in/out made it in decides the new name,
        // and the metadata then travels with the file.
        let saved = metadata::save_to_xmp(path, snapshot, storage);
        let new_file_name = if saved.in_out {
            let mut without_in_out = snapshot.clone();
            without_in_out.set_segment_start(None);
            without_in_out.set_segment_end(None);
            without_in_out.file_name()
        } else {
            snapshot.file_name()
        };
        let new_path = path
            .parent()
            .map(|p| p.join(&new_file_name))
            .unwrap_or_else(|| PathBuf::from(&new_file_name));

        if new_path != path {
            crate::comment::rename_comment_file(path, &new_path);
            crate::subtitles::rename_subtitle_file(path, &new_path);
            log::info!(
                "Renaming {} screenshot(s) for {:?} → {:?}",
                snapshot.screenshots().len(), path, new_path
            );
            for s in snapshot.screenshots() {
                let old_shot = screenshot_path(path, s.position_ms);
                let new_shot = screenshot_path(&new_path, s.position_ms);
                log::info!("  screenshot: {:?} exists={} → {:?}", old_shot, old_shot.exists(), new_shot);
                if old_shot.exists() {
                    match std::fs::rename(&old_shot, &new_shot) {
                        Ok(()) => log::info!("  screenshot renamed ok"),
                        Err(e) => log::error!("  screenshot rename failed: {}", e),
                    }
                }
            }
            if let Err(e) = std::fs::rename(path, &new_path) {
                log::error!("ProductionFileTagger: rename {:?} → {:?} failed: {}", path, new_path, e);
                return path.to_path_buf();
            }
            log::info!("Renamed on disk: {:?} → {:?}", path, new_path);
        }
        // A pending snapshot does not know an XMP comment, so it has no text file to write.
        if !(snapshot.comment_loading() && storage.comment == CommentStorage::InVideo) {
            metadata::save_comment_text_file(&new_path, snapshot.comment(), saved.comment);
        }
        metadata::cache::refresh_after_save(path, &new_path, storage);
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

    fn metadata_conversion(&self, path: &Path, storage: MetadataStorage) -> FileConversion {
        metadata::Inspection::of(path).needed(storage, metadata::commented_tag().as_deref())
    }

    fn convert_metadata(&self, path: &Path, storage: MetadataStorage) -> PathBuf {
        let tag = metadata::commented_tag();
        let moved = metadata::Inspection::of(path).needed(storage, tag.as_deref());
        if !moved.is_needed() {
            return path.to_path_buf();
        }
        // Parsing with XMP storage for both reads both homes: the text file and the name
        // first, XMP where they are empty.
        let both_homes = MetadataStorage { comment: CommentStorage::InVideo, in_out: InOutStorage::InVideo };
        let snapshot = self.parse_with(path, &FolderInfo::default(), both_homes);
        let snapshot = metadata::with_commented_tag(&snapshot, storage, tag.as_deref()).unwrap_or(snapshot);
        let new_path = self.save_with(&snapshot, path, storage);
        metadata::clear_moved_xmp(&new_path, moved, storage);
        log::info!("metadata: converted {:?} → {:?} ({:?})", path, new_path, moved);
        new_path
    }

    fn save_screenshot(&self, file_path: &Path, position_ms: u64, image_data: &[u8]) {
        let path = screenshot_path(file_path, position_ms);
        log::info!("save_screenshot: writing {} bytes to {:?}", image_data.len(), path);
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

    const ADOBE: MetadataStorage =
        MetadataStorage { comment: CommentStorage::InVideo, in_out: InOutStorage::InVideo };
    const FILE_NAME: MetadataStorage =
        MetadataStorage { comment: CommentStorage::TextFile, in_out: InOutStorage::FileName };

    /// A fresh copy of the tiny QuickTime fixture named `name`, in its own temp folder.
    fn clip_named(test: &str, name: &str) -> PathBuf {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        let dir = std::env::temp_dir().join(format!("frename-tagger-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join(name);
        std::fs::copy(fixture, &file).expect("copy fixture");
        file
    }

    fn file_name(path: &Path) -> &str {
        path.file_name().and_then(|n| n.to_str()).expect("file name")
    }

    #[test]
    fn adobe_storage_moves_in_out_from_the_name_into_xmp() {
        let tagger = ProductionFileTagger;
        let path = clip_named("adobe", "goat.in_00_00_01.out_00_00_02.mov");
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        let new_path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(file_name(&new_path), "goat.mov");

        let back = tagger.parse_with(&new_path, &FolderInfo::default(), ADOBE);
        assert_eq!((back.segment_start(), back.segment_end()), (Some(1.0), Some(2.0)));
    }

    #[test]
    fn file_name_storage_moves_in_out_back_into_the_name() {
        let tagger = ProductionFileTagger;
        let path = clip_named("file-name", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_segment_end(Some(0.1));
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(file_name(&path), "goat.mov");

        // Switching to file-name storage: the points were loaded from XMP under the old
        // setting and are still on the snapshot, so the next save puts them in the name.
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        let new_path = tagger.save_with(&snapshot, &path, FILE_NAME);
        assert_eq!(file_name(&new_path), "goat.out_00_00_00.mov");
    }

    #[test]
    fn in_out_a_marker_cannot_express_stays_in_the_name() {
        // The fixture is 0.2 s long, so an in at 1 s with no out is an empty range.
        let tagger = ProductionFileTagger;
        let path = clip_named("past-end", "goat.in_00_00_01.mov");
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        let new_path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(file_name(&new_path), "goat.in_00_00_01.mov");
    }

    #[test]
    fn in_out_stays_in_the_name_when_the_file_cannot_hold_xmp() {
        let tagger = ProductionFileTagger;
        let path = clip_named("unsupported", "notes.in_00_00_01.zip");
        std::fs::write(&path, b"not really a zip").expect("write");
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        let new_path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(file_name(&new_path), "notes.in_00_00_01.zip");
    }

    #[test]
    fn comment_storage_setting_picks_the_home_of_the_comment() {
        let tagger = ProductionFileTagger;
        let path = clip_named("comment-setting", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), FILE_NAME);
        snapshot.set_comment("in a text file".to_string());
        let path = tagger.save_with(&snapshot, &path, FILE_NAME);
        assert!(crate::comment::comment_path(&path).exists());

        // Switching to XMP moves the comment into the file and drops the text file.
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        assert_eq!(snapshot.comment(), "in a text file");
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        assert!(!crate::comment::comment_path(&path).exists());
        assert_eq!(tagger.parse_with(&path, &FolderInfo::default(), ADOBE).comment(), "in a text file");
    }

    #[test]
    fn conversion_moves_everything_into_xmp_and_back() {
        let tagger = ProductionFileTagger;
        let path = clip_named("convert", "goat.in_00_00_00.mov");
        crate::comment::save_comment(&path, "old comment");

        // Comments in XMP also tag the name (the default commented tag).
        let needed = tagger.metadata_conversion(&path, ADOBE);
        assert_eq!(needed, FileConversion { comment: true, in_out: true, commented_tag: true });
        let path = tagger.convert_metadata(&path, ADOBE);
        assert_eq!(file_name(&path), "Commented.goat.mov");
        assert!(!crate::comment::comment_path(&path).exists());
        assert!(!tagger.metadata_conversion(&path, ADOBE).is_needed());
        let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        assert_eq!(snapshot.comment(), "old comment");
        assert_eq!(snapshot.segment_start(), Some(0.0));

        // And back: the old copies leave the XMP, so nothing stale is left for Premiere.
        // The tag is only managed while comments are in XMP, so it stays.
        let needed = tagger.metadata_conversion(&path, FILE_NAME);
        assert_eq!(needed, FileConversion { comment: true, in_out: true, commented_tag: false });
        let path = tagger.convert_metadata(&path, FILE_NAME);
        assert_eq!(file_name(&path), "Commented.goat.in_00_00_00.mov");
        assert_eq!(crate::comment::load_comment(&path), "old comment");
        assert!(!tagger.metadata_conversion(&path, FILE_NAME).is_needed());

        // With the text file and the name tokens gone, an XMP copy would show up as work to do.
        std::fs::remove_file(crate::comment::comment_path(&path)).expect("remove text file");
        let bare = path.with_file_name("goat.mov");
        std::fs::rename(&path, &bare).expect("rename");
        assert_eq!(tagger.metadata_conversion(&bare, FILE_NAME), FileConversion::default());
    }

    #[test]
    fn converting_to_text_keeps_an_xmp_comment_that_a_text_file_would_hide() {
        let tagger = ProductionFileTagger;
        let path = clip_named("no-loss", "goat.mov");
        let mut snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
        snapshot.set_comment("in xmp".to_string());
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        crate::comment::save_comment(&path, "in text");

        // The text file wins, so there is nothing to move, and the XMP text is not removed.
        assert!(!tagger.metadata_conversion(&path, FILE_NAME).is_needed());
        let path = tagger.convert_metadata(&path, FILE_NAME);
        std::fs::remove_file(crate::comment::comment_path(&path)).expect("remove text file");
        assert_eq!(tagger.parse_with(&path, &FolderInfo::default(), ADOBE).comment(), "in xmp");
    }

    #[test]
    fn files_already_in_the_chosen_storage_need_no_conversion() {
        let tagger = ProductionFileTagger;
        let path = clip_named("nothing", "goat.in_00_00_00.mov");
        crate::comment::save_comment(&path, "text");
        assert!(!tagger.metadata_conversion(&path, FILE_NAME).is_needed());
        assert_eq!(tagger.convert_metadata(&path, FILE_NAME), path);
    }

    /// Folder info as a scan builds it: the listing's sizes and times, and the tag file's list.
    fn scan_info(folder: &Path) -> FolderInfo {
        let mut names = Vec::new();
        let mut stats = std::collections::HashMap::new();
        for entry in std::fs::read_dir(folder).expect("folder").flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let metadata = entry.metadata().expect("metadata");
            if metadata.is_file() {
                let modified = crate::metadata::cache::modified_ms(metadata.modified().expect("time"));
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
        crate::FolderTagStore::update_file_cache(&folder, &[file_name(&path).to_string()], Vec::new());
        let miss = tagger.parse_with(&path, &scan_info(&folder), ADOBE);
        assert!(miss.comment_loading());
        assert_eq!(miss.comment(), "");

        // Reading it later gives the comment, and records it for the next scan.
        let items = [(path.clone(), miss)];
        let resolved = tagger.load_comments(&items);
        assert_eq!(resolved[0].comment(), "goat");
        assert!(!resolved[0].comment_loading());
        assert_eq!(tagger.parse_with(&path, &scan_info(&folder), ADOBE).comment(), "goat");
    }

    #[test]
    fn a_line_that_no_longer_matches_the_file_is_not_trusted() {
        let tagger = ProductionFileTagger;
        let path = commented_clip("scan-stale", "goat");
        let folder = path.parent().expect("folder").to_path_buf();
        let mut line = crate::FolderTagStore::read_file_cache(&folder).pop().expect("line");
        line.size += 1;
        line.comment = "stale".into();
        crate::FolderTagStore::update_file_cache(&folder, &[], vec![line]);

        let snapshot = tagger.parse_with(&path, &scan_info(&folder), ADOBE);
        assert!(snapshot.comment_loading(), "a changed file is read again");
    }

    #[test]
    fn a_comment_edited_in_the_file_list_is_shown_and_saved_into_the_video() {
        let tagger = ProductionFileTagger;
        let path = commented_clip("scan-ai-edit", "goat");
        let folder = path.parent().expect("folder").to_path_buf();
        let mut line = crate::FolderTagStore::read_file_cache(&folder).pop().expect("line");
        line.comment = "edited by hand".into();
        crate::FolderTagStore::update_file_cache(&folder, &[], vec![line]);

        let snapshot = tagger.parse_with(&path, &scan_info(&folder), ADOBE);
        assert_eq!(snapshot.comment(), "edited by hand");
        let path = tagger.save_with(&snapshot, &path, ADOBE);
        assert_eq!(tagger.parse_with(&path, &FolderInfo::default(), ADOBE).comment(), "edited by hand");
    }

    #[test]
    fn saving_a_file_whose_xmp_was_not_read_keeps_its_comment_and_tag() {
        let tagger = ProductionFileTagger;
        let path = commented_clip("scan-pending-save", "goat");
        let path = {
            // The default commented tag is in the name now.
            let snapshot = tagger.parse_with(&path, &FolderInfo::default(), ADOBE);
            let snapshot = crate::metadata::with_commented_tag(&snapshot, ADOBE, Some("Commented")).unwrap_or(snapshot);
            tagger.save_with(&snapshot, &path, ADOBE)
        };
        let folder = path.parent().expect("folder").to_path_buf();
        crate::FolderTagStore::update_file_cache(&folder, &[file_name(&path).to_string()], Vec::new());

        let pending = tagger.parse_with(&path, &scan_info(&folder), ADOBE);
        assert!(pending.comment_loading());
        assert!(crate::metadata::with_commented_tag(&pending, ADOBE, Some("Commented")).is_none());
        let path = tagger.save_with(&pending, &path, ADOBE);
        assert_eq!(file_name(&path), "Commented.goat.mov", "tag kept");
        assert_eq!(tagger.parse_with(&path, &FolderInfo::default(), ADOBE).comment(), "goat", "comment kept");
    }
}
