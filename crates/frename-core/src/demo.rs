//! Demo scenarios: a folder of clips set up in a known state, so the app can take the README
//! screenshots itself (`frename --demo <scenario.toml> --out <png>`).
//!
//! A scenario names clips from a source folder and the name, comment, subtitles and clip
//! markers each copy gets. [`stage`] builds that folder somewhere disposable; [`seed`] points the
//! app's stored state (window, panels, settings, last session) at it, so the app opens it the
//! normal way.

use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Deserialize;

use crate::db::{AppSettings, AppStateStore, WindowGeometry};
use crate::{CommentStorage, FolderAndFile};

/// One demo screenshot: the folder to stage and the app state to show.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemoScenario {
    /// Folder the clips are copied from, relative to the scenario file.
    pub source: PathBuf,
    /// Name of the staged file to open (one of `files[].name`).
    pub open: String,
    /// Where to pause the video, in seconds.
    #[serde(default)]
    pub seek: f32,
    /// Window size in logical pixels: width, height.
    pub window: [u32; 2],
    /// Widths of the video panel and the file list panel.
    pub panels: [f32; 2],
    /// Open the subtitle list over the picture (the CC button).
    #[serde(default)]
    pub subtitle_list: bool,
    /// Open the marker list over the picture (the ◆ button).
    #[serde(default)]
    pub marker_list: bool,
    /// The staged files, oldest first: the file list shows them in this order.
    pub files: Vec<DemoFile>,
}

/// One staged file.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemoFile {
    /// Clip in `source` to copy.
    pub from: String,
    /// Name of the copy, tags and in/out included.
    pub name: String,
    /// Comment, saved as `.comment.txt`.
    #[serde(default)]
    pub comment: Option<String>,
    /// Subtitles, saved as the `.srt` next to the copy.
    #[serde(default)]
    pub subtitles: Option<String>,
    /// Clip markers written into the copy's XMP, one line each as a comment holds them:
    /// `0:06.120 — City lights`, `0:41-0:47 — Lion — roars twice`.
    #[serde(default)]
    pub markers: Vec<String>,
}

/// Why a scenario cannot be used.
#[derive(Debug, PartialEq, Eq)]
pub struct DemoError(pub String);

impl std::fmt::Display for DemoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DemoError {}

impl DemoScenario {
    /// Parse and check a scenario file's text.
    pub fn parse(text: &str) -> Result<Self, DemoError> {
        let scenario: Self = toml::from_str(text).map_err(|e| DemoError(e.to_string()))?;
        scenario.check()?;
        Ok(scenario)
    }

    fn check(&self) -> Result<(), DemoError> {
        let names = self.files.iter().map(|f| f.name.as_str());
        let plain = |name: &str| {
            let mut parts = Path::new(name).components();
            matches!(
                (parts.next(), parts.next()),
                (Some(Component::Normal(_)), None)
            ) && !name.contains(['/', '\\'])
        };
        for file in &self.files {
            for name in [&file.from, &file.name] {
                if !plain(name) {
                    return Err(DemoError(format!("not a plain file name: {name:?}")));
                }
            }
            if let Some(bad) = file
                .markers
                .iter()
                .find(|line| crate::parse_marker_line(line).is_none())
            {
                return Err(DemoError(format!(
                    "{:?}: marker {bad:?} is not `<time> — <name>`",
                    file.name
                )));
            }
        }
        if !names.clone().any(|n| n == self.open) {
            return Err(DemoError(format!(
                "open = {:?} is not one of the staged files",
                self.open
            )));
        }
        // The demo waits for the video to be ready before it takes the screenshot.
        let extension = Path::new(&self.open)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        if crate::FileKind::from_extension(extension) != crate::FileKind::Video {
            return Err(DemoError(format!("open = {:?} is not a video", self.open)));
        }
        let mut sorted: Vec<&str> = names.collect();
        sorted.sort_unstable();
        if let Some(pair) = sorted.windows(2).find(|w| w[0] == w[1]) {
            return Err(DemoError(format!("{:?} is staged twice", pair[0])));
        }
        if self.window.contains(&0) {
            return Err(DemoError("window size must not be zero".into()));
        }
        Ok(())
    }
}

/// Copy the scenario's files from `scenario_dir`/`source` into `into` (created if missing) and
/// write their comments, subtitles and clip markers. Returns the path of the file to open.
/// The source folder is only read.
pub fn stage(
    scenario: &DemoScenario,
    scenario_dir: &Path,
    into: &Path,
) -> std::io::Result<PathBuf> {
    let source = scenario_dir.join(&scenario.source);
    std::fs::create_dir_all(into)?;
    // The list is ordered by modification time: space the copies a minute apart, in order.
    let start = SystemTime::now() - Duration::from_secs(60 * (scenario.files.len() as u64 + 1));
    for (index, file) in scenario.files.iter().enumerate() {
        let target = into.join(&file.name);
        std::fs::copy(source.join(&file.from), &target)?;
        std::fs::File::options()
            .write(true)
            .open(&target)?
            .set_modified(start + Duration::from_secs(60 * index as u64))?;
        if let Some(comment) = &file.comment {
            // Written here, not with `save_comment`, which only logs a failed write.
            std::fs::write(
                crate::comment::comment_path(&target),
                format!("\u{feff}{}", comment.trim()),
            )?;
        }
        if let Some(subtitles) = &file.subtitles {
            std::fs::write(crate::subtitle_path(&target), subtitles)?;
        }
        if !file.markers.is_empty() {
            // Each line was checked by `DemoScenario::check`.
            let markers: Vec<crate::Marker> = file
                .markers
                .iter()
                .filter_map(|line| crate::parse_marker_line(line))
                .map(|line| {
                    let mut marker = crate::Marker::new(line.start_ms);
                    marker.duration_ms = line.duration_ms;
                    marker.name = line.name;
                    marker.comment = line.comment;
                    marker
                })
                .collect();
            crate::metadata::save_markers(&target, &markers, &Default::default())
                .map_err(|e| std::io::Error::other(format!("{}: {e}", file.name)))?;
        }
    }
    Ok(into.join(&scenario.open))
}

/// Store the app state the scenario shows: window at the top left with its size and panel
/// widths, videos paused when opened, comments in `.comment.txt` (a staged comment is one), and
/// `file` in `folder` as the last session, so the app opens it on start. `monochrome_tags` turns
/// on that setting.
pub fn seed(
    store: &dyn AppStateStore,
    scenario: &DemoScenario,
    folder: &Path,
    file: &Path,
    monochrome_tags: bool,
) {
    let [width, height] = scenario.window;
    store.set_window_state(WindowGeometry {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
        is_maximized: false,
        monitor_width: 0.0,
        monitor_height: 0.0,
        left_panel_width: scenario.panels[0],
        folder_panel_width: scenario.panels[1],
    });
    store.set_app_settings(AppSettings {
        autoplay_video: false,
        monochrome_tags,
        comment_storage: CommentStorage::TextFile,
        ..AppSettings::default()
    });
    store.set_last_folder_and_file(&FolderAndFile::new(folder, Some(file)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    const MINIMAL: &str = r#"
source = "clips"
open = "pick.a.mp4"
window = [1920, 1009]
panels = [800, 400]
[[files]]
from = "a.mp4"
name = "pick.a.mp4"
"#;

    #[test]
    fn a_minimal_scenario_gets_defaults() {
        let scenario = DemoScenario::parse(MINIMAL).unwrap();
        assert_eq!(scenario.seek, 0.0);
        assert_eq!(scenario.files[0].comment, None);
        assert!(scenario.files[0].markers.is_empty());
    }

    #[test]
    fn unknown_keys_are_refused() {
        let text = MINIMAL.replace("[[files]]", "zoom = 2\n[[files]]");
        assert!(DemoScenario::parse(&text).is_err());
    }

    #[test]
    fn open_must_be_a_staged_file() {
        let text = MINIMAL.replace("open = \"pick.a.mp4\"", "open = \"b.mp4\"");
        assert!(DemoScenario::parse(&text).unwrap_err().0.contains("b.mp4"));
    }

    #[test]
    fn names_must_stay_inside_the_folder() {
        for bad in ["../x.mp4", "sub/x.mp4", "sub\\\\x.mp4", ".."] {
            let text = MINIMAL.replace("from = \"a.mp4\"", &format!("from = \"{bad}\""));
            assert!(DemoScenario::parse(&text).is_err(), "{bad}");
        }
    }

    #[test]
    fn markers_must_be_marker_lines() {
        let with_markers = |markers: &str| format!("{MINIMAL}markers = [{markers}]\n");
        assert!(DemoScenario::parse(&with_markers("\"0:06.120 — City\"")).is_ok());
        for bad in ["\"0:06\"", "\"City\"", "\"6 — x\""] {
            assert!(DemoScenario::parse(&with_markers(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn only_a_video_can_be_opened() {
        let text = MINIMAL.replace("pick.a.mp4", "pick.a.jpg");
        assert!(DemoScenario::parse(&text)
            .unwrap_err()
            .0
            .contains("not a video"));
    }

    #[test]
    fn a_file_is_staged_once() {
        let text = format!("{MINIMAL}[[files]]\nfrom = \"a.mp4\"\nname = \"pick.a.mp4\"\n");
        assert!(DemoScenario::parse(&text).is_err());
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("frename-demo-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn staging_copies_renames_and_writes_sidecars_without_touching_the_source() {
        let root = temp_dir("stage");
        let clips = root.join("clips");
        std::fs::create_dir_all(&clips).unwrap();
        std::fs::write(clips.join("a.mp4"), b"video a").unwrap();
        let tiny = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        std::fs::copy(&tiny, clips.join("b.mov")).unwrap();
        let text = r#"
source = "clips"
open = "wide.b.in_00_00_03.mov"
window = [1920, 1009]
panels = [800, 400]
[[files]]
from = "a.mp4"
name = "pick.a.mp4"
[[files]]
from = "b.mov"
name = "wide.b.in_00_00_03.mov"
comment = "nice"
subtitles = "1\n00:00:00,000 --> 00:00:02,000\nHello\n"
markers = ["0:00.100 — Start — first frames"]
"#;
        let scenario = DemoScenario::parse(text).unwrap();
        let into = root.join("staged");

        let open = stage(&scenario, &root, &into).unwrap();

        assert_eq!(open, into.join("wide.b.in_00_00_03.mov"));
        assert_eq!(crate::comment::load_comment(&open), "nice");
        assert!(std::fs::read_to_string(into.join("wide.b.in_00_00_03.srt"))
            .unwrap()
            .contains("Hello"));
        let markers = crate::metadata::load_markers(&open).unwrap();
        assert_eq!(markers.len(), 1);
        assert_eq!(
            (markers[0].start_ms, markers[0].name.as_str()),
            (100, "Start")
        );
        let modified = |name: &str| into.join(name).metadata().unwrap().modified().unwrap();
        assert!(modified("pick.a.mp4") < modified("wide.b.in_00_00_03.mov"));
        let mut left: Vec<_> = std::fs::read_dir(&clips)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        left.sort();
        assert_eq!(left, ["a.mp4", "b.mov"]);
        assert_eq!(
            std::fs::read(clips.join("b.mov")).unwrap(),
            std::fs::read(&tiny).unwrap()
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[derive(Default)]
    struct Recorder {
        window: Mutex<Option<WindowGeometry>>,
        settings: Mutex<Option<AppSettings>>,
        session: Mutex<Option<FolderAndFile>>,
    }

    impl AppStateStore for Recorder {
        fn get_last_session(&self) -> Option<FolderAndFile> {
            self.session.lock().unwrap().clone()
        }
        fn set_last_folder_and_file(&self, value: &FolderAndFile) {
            *self.session.lock().unwrap() = Some(value.clone());
        }
        fn set_window_state(&self, geometry: WindowGeometry) {
            *self.window.lock().unwrap() = Some(geometry);
        }
        fn set_app_settings(&self, settings: AppSettings) {
            *self.settings.lock().unwrap() = Some(settings);
        }
    }

    #[test]
    fn seeding_stores_the_window_settings_and_session() {
        let scenario = DemoScenario::parse(MINIMAL).unwrap();
        let store = Recorder::default();

        seed(
            &store,
            &scenario,
            Path::new("/f"),
            Path::new("/f/pick.a.mp4"),
            true,
        );

        let window = store.window.lock().unwrap().unwrap();
        assert_eq!(
            (window.x, window.y, window.width, window.height),
            (0.0, 0.0, 1920.0, 1009.0)
        );
        assert_eq!(
            (window.left_panel_width, window.folder_panel_width),
            (800.0, 400.0)
        );
        assert!(!window.is_maximized);
        let settings = store.settings.lock().unwrap().clone().unwrap();
        assert!(!settings.autoplay_video);
        assert!(settings.monochrome_tags);
        assert_eq!(settings.comment_storage, CommentStorage::TextFile);
        assert_eq!(
            store.get_last_session(),
            Some(FolderAndFile::new("/f", Some("/f/pick.a.mp4")))
        );
    }
}
