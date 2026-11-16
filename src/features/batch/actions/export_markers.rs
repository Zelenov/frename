//! "Export markers to text": writes each file's markers as `clip.markers.txt` next to the video,
//! one line per marker in time order, in the line form "Markers ⇄ comment" reads back. A file
//! with no markers gets no file. No options; running it again replaces the earlier text.

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

use frename_core::{markers_text_path, write_markers_text, FileTagger};
use iced::Element;

use super::super::page::{self, Change};
use super::super::{ItemResult, ItemStatus};

/// The log is always English, unlike the UI text `label()` returns.
const LOG_LABEL: &str = "Export markers";

pub fn label() -> String {
    fl!("batch-action-export-markers")
}

pub fn view<'a, M: 'a>() -> Element<'a, M> {
    page::page(
        label(),
        fl!("batch-action-export-markers-hint"),
        &[Change::TextFiles],
        [],
    )
}

/// One run of the action: the text files it has written so far. Two videos of one name
/// (`A001.mov` and `A001.mp4`) have one `A001.markers.txt`; the first keeps it, and the second
/// fails instead of replacing it (#495). Operations compare equal only when they are the same run.
#[derive(Debug, Clone, Default)]
pub struct Run(Arc<Mutex<HashSet<String>>>);

impl PartialEq for Run {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Run {}

/// Write the markers of the file at `path`. A file with none is skipped, and one that cannot hold
/// markers, whose text another clip of the same name wrote in this run, or whose text cannot be
/// written fails with the reason.
pub fn run(job: &Run, path: &Path) -> ItemResult {
    let Some(markers) = FileTagger::load_markers(path) else {
        return ItemResult::failed(fl!("batch-export-markers-cannot-hold"));
    };
    if markers.is_empty() {
        return ItemResult::new(ItemStatus::Skipped, None);
    }
    let target = markers_text_path(path);
    // Case-insensitive, as Generate subtitles decides: clip.MP4 and clip.mov both write
    // clip.markers.txt on Windows.
    let key = target.to_string_lossy().to_lowercase();
    let mut written = job.0.lock().unwrap_or_else(|e| e.into_inner());
    if written.contains(&key) {
        log::warn!("{LOG_LABEL}: {path:?} not written: another clip of the run wrote {target:?}");
        let file = target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        return ItemResult::failed(fl!("batch-export-markers-shared-name", file = file));
    }
    match write_markers_text(&target, &markers) {
        Ok(()) => {
            written.insert(key);
            ItemResult::new(ItemStatus::Done, None)
        }
        Err(error) => {
            log::warn!("{LOG_LABEL}: {target:?} not written: {error}");
            ItemResult::failed(error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use frename_core::{Marker, MarkerColor};

    use super::*;

    fn clip(name: &str, markers: &[Marker]) -> std::path::PathBuf {
        clip_in(
            &frename_core::test_support::fresh_dir(name),
            "clip.mov",
            markers,
        )
    }

    fn clip_in(dir: &Path, file_name: &str, markers: &[Marker]) -> std::path::PathBuf {
        let file = dir.join(file_name);
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("crates/frename-core/tests/fixtures/tiny.mov");
        std::fs::copy(fixture, &file).expect("copy fixture");
        if !markers.is_empty() {
            FileTagger::save_markers(&file, markers, &HashSet::new()).expect("markers");
        }
        file
    }

    fn marker(start_ms: u64, name: &str) -> Marker {
        let mut marker = Marker::new(start_ms);
        marker.name = name.to_string();
        marker
    }

    #[test]
    fn a_clip_with_markers_gets_clip_markers_txt() {
        let mut red = marker(2_000, "Lion");
        red.color = MarkerColor::Red;
        red.comment = "roars".to_string();
        let file = clip("export-markers-done", &[red, marker(1_000, "Start")]);
        let result = run(&Run::default(), &file);
        assert_eq!(result.status, ItemStatus::Done);
        let text = std::fs::read_to_string(file.with_file_name("clip.markers.txt")).expect("text");
        assert_eq!(
            text.replace("\r\n", "\n"),
            "0:01 — Start\n0:02 [red] — Lion — roars\n"
        );
        let _ = std::fs::remove_dir_all(file.parent().expect("folder"));
    }

    #[test]
    fn a_clip_without_markers_is_skipped_and_gets_no_file() {
        let file = clip("export-markers-none", &[]);
        assert_eq!(run(&Run::default(), &file).status, ItemStatus::Skipped);
        assert!(!file.with_file_name("clip.markers.txt").exists());
        let _ = std::fs::remove_dir_all(file.parent().expect("folder"));
    }

    #[test]
    fn running_again_replaces_the_text() {
        let file = clip("export-markers-again", &[marker(1_000, "One")]);
        run(&Run::default(), &file);
        FileTagger::save_markers(
            &file,
            &[marker(1_000, "One"), marker(5_000, "Two")],
            &HashSet::new(),
        )
        .expect("markers");
        assert_eq!(run(&Run::default(), &file).status, ItemStatus::Done);
        let text = std::fs::read_to_string(file.with_file_name("clip.markers.txt")).expect("text");
        assert_eq!(text.lines().count(), 2);
        let _ = std::fs::remove_dir_all(file.parent().expect("folder"));
    }

    /// #495: of two clips of one name in a run, the first keeps `A001.markers.txt` and the
    /// second fails with the reason, instead of replacing it while both say Done.
    #[test]
    fn a_second_clip_of_the_same_name_does_not_replace_the_first_ones_text() {
        let dir = frename_core::test_support::fresh_dir("export-markers-shared-name");
        let mov = clip_in(&dir, "A001.mov", &[marker(1_000, "Original")]);
        let mp4 = clip_in(&dir, "A001.mp4", &[marker(2_000, "Proxy")]);
        let job = Run::default();
        assert_eq!(run(&job, &mov).status, ItemStatus::Done);
        let second = run(&job, &mp4);
        assert_eq!(second.status, ItemStatus::Failed);
        assert!(
            second
                .reason
                .as_deref()
                .is_some_and(|r| r.contains("A001.markers.txt")),
            "{:?}",
            second.reason
        );
        let text = std::fs::read_to_string(dir.join("A001.markers.txt")).expect("text");
        assert_eq!(text.replace("\r\n", "\n"), "0:01 — Original\n");
        // A new run writes it again: running again replaces the text, as before.
        assert_eq!(run(&Run::default(), &mp4).status, ItemStatus::Done);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_file_that_cannot_hold_markers_fails() {
        let dir = frename_core::test_support::fresh_dir("export-markers-cannot");
        let file = dir.join("clip.mp4.txt");
        std::fs::write(&file, "not a video").expect("write");
        assert_eq!(run(&Run::default(), &file).status, ItemStatus::Failed);
        assert!(!dir.join("clip.mp4.markers.txt").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
