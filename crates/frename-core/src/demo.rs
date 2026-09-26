//! Demo scenarios: a folder of clips set up in a known state, so the app can take the README
//! screenshots itself (`frename --demo <scenario.toml> --out <png>`).
//!
//! A scenario names clips from a source folder and the name, comment, subtitles and screenshot
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
    pub seek: f64,
    /// Show batch mode with every file checked.
    #[serde(default)]
    pub batch: bool,
    /// Window size in logical pixels: width, height.
    pub window: [u32; 2],
    /// Widths of the video panel and the file list panel.
    pub panels: [f32; 2],
    /// A picture in `source` copied for every screenshot marker.
    #[serde(default)]
    pub snap_image: Option<String>,
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
    /// Screenshot marker times, `HH-MM-SS-mmm`.
    #[serde(default)]
    pub snaps: Vec<String>,
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
            if !file.snaps.is_empty() && self.snap_image.is_none() {
                return Err(DemoError(format!(
                    "{:?} has screenshot markers but the scenario has no snap_image",
                    file.name
                )));
            }
        }
        if let Some(image) = &self.snap_image {
            if !plain(image) {
                return Err(DemoError(format!("not a plain file name: {image:?}")));
            }
        }
        if !names.clone().any(|n| n == self.open) {
            return Err(DemoError(format!(
                "open = {:?} is not one of the staged files",
                self.open
            )));
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
/// write their comments, subtitles and screenshot markers. Returns the path of the file to open.
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
            crate::comment::save_comment(&target, comment);
        }
        if let Some(subtitles) = &file.subtitles {
            std::fs::write(crate::subtitle_path(&target), subtitles)?;
        }
        for at in &file.snaps {
            // Checked by `DemoScenario::check`.
            let image = scenario.snap_image.as_deref().unwrap_or_default();
            std::fs::copy(
                source.join(image),
                into.join(format!("{}.snap.{at}.jpg", file.name)),
            )?;
        }
    }
    Ok(into.join(&scenario.open))
}

/// Store the app state the scenario shows: window at the top left with its size and panel
/// widths, videos paused when opened, comments in `.comment.txt` (a staged comment is one), and
/// `file` in `folder` as the last session, so the app opens it on start.
pub fn seed(store: &dyn AppStateStore, scenario: &DemoScenario, folder: &Path, file: &Path) {
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
        assert!(!scenario.batch);
        assert_eq!(scenario.files[0].comment, None);
        assert!(scenario.files[0].snaps.is_empty());
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
    fn markers_need_a_snap_image() {
        let text = format!("{MINIMAL}snaps = [\"00-00-01-000\"]\n");
        assert!(DemoScenario::parse(&text).is_err());
        let text = MINIMAL.replace("[[files]]", "snap_image = \"s.jpg\"\n[[files]]")
            + "snaps = [\"00-00-01-000\"]\n";
        assert!(DemoScenario::parse(&text).is_ok());
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
        std::fs::write(clips.join("b.mp4"), b"video b").unwrap();
        std::fs::write(clips.join("s.jpg"), b"jpeg").unwrap();
        let text = r#"
source = "clips"
open = "wide.b.in_00_00_03.mp4"
window = [1920, 1009]
panels = [800, 400]
snap_image = "s.jpg"
[[files]]
from = "a.mp4"
name = "pick.a.mp4"
[[files]]
from = "b.mp4"
name = "wide.b.in_00_00_03.mp4"
comment = "00-00-01-000: nice"
subtitles = "1\n00:00:00,000 --> 00:00:02,000\nHello\n"
snaps = ["00-00-01-000"]
"#;
        let scenario = DemoScenario::parse(text).unwrap();
        let into = root.join("staged");

        let open = stage(&scenario, &root, &into).unwrap();

        assert_eq!(open, into.join("wide.b.in_00_00_03.mp4"));
        assert_eq!(std::fs::read(&open).unwrap(), b"video b");
        assert_eq!(crate::comment::load_comment(&open), "00-00-01-000: nice");
        assert!(std::fs::read_to_string(into.join("wide.b.in_00_00_03.srt"))
            .unwrap()
            .contains("Hello"));
        assert_eq!(
            std::fs::read(into.join("wide.b.in_00_00_03.mp4.snap.00-00-01-000.jpg")).unwrap(),
            b"jpeg"
        );
        let modified = |name: &str| into.join(name).metadata().unwrap().modified().unwrap();
        assert!(modified("pick.a.mp4") < modified("wide.b.in_00_00_03.mp4"));
        let mut left: Vec<_> = std::fs::read_dir(&clips)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        left.sort();
        assert_eq!(left, ["a.mp4", "b.mp4", "s.jpg"]);
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
        assert_eq!(settings.comment_storage, CommentStorage::TextFile);
        assert_eq!(
            store.get_last_session(),
            Some(FolderAndFile::new("/f", Some("/f/pick.a.mp4")))
        );
    }
}
