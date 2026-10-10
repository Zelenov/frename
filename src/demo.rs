//! Demo mode: `frename --demo <scenario.toml> --out <png>` opens a staged folder in a known state,
//! saves a screenshot of the main window and exits. `docs/screenshots/render.sh` uses it for the
//! README screenshots.
//!
//! Everything runs in a throwaway work folder: the staged clips and the app's own database and
//! log (`FRENAME_DATA_DIR`), so the user's files and settings are never touched.

use std::path::{Path, PathBuf};
use std::time::Duration;

use frename_core::ai::key::{ApiKey, KeyState};
use frename_core::demo::DemoScenario;
use iced::{window, Task};

use crate::features::{
    batch, folder, folder_workspace, media_viewer, media_viewer::video, settings,
};

/// Time from the video being ready to the screenshot: covers the seek, the subtitles, and the
/// comments that load in the background. A fixed wait, not a signal: the open file's comment is
/// loaded before it is shown, and the seek and the list's comment batch take well under a second
/// with the scenario's few files, so 3 s leaves a wide margin even on a slow runner.
const SETTLE: Duration = Duration::from_secs(3);

/// [`SETTLE`] for a scenario that turns the clip: short enough to catch the turn's note.
const ROTATE_SETTLE: Duration = Duration::from_millis(1300);

/// A demo that has not produced its screenshot by then has failed.
const TIMEOUT: Duration = Duration::from_secs(90);

/// Demo mode messages.
#[derive(Debug, Clone)]
pub enum Message {
    /// A window's screenshot, and the size in pixels it must have (at scale 1) to be saved.
    Captured(window::Screenshot, (u32, u32)),
    /// The demo took too long.
    TimedOut,
}

/// A demo in progress.
#[derive(Debug, Clone)]
pub struct DemoRun {
    scenario: DemoScenario,
    out: PathBuf,
    work: PathBuf,
    /// Show batch mode with every file checked.
    batch: bool,
    /// Show the AI description: its segments, or in batch mode the "Describe with AI" action.
    ai: bool,
    /// Capture the settings window, on this page, instead of the main one.
    settings: Option<settings::Page>,
    video_ready: bool,
}

impl DemoRun {
    /// `work` is the throwaway folder; `main` removes it after the app has exited.
    pub fn new(scenario: DemoScenario, out: PathBuf, work: PathBuf, args: &DemoArgs) -> Self {
        Self {
            scenario,
            out,
            work,
            batch: args.batch,
            ai: args.ai,
            settings: args.settings,
            video_ready: false,
        }
    }

    /// The settings page the demo captures, when it captures the settings window (the app opens
    /// it when the video is ready) instead of the main one.
    pub fn settings_page(&self) -> Option<settings::Page> {
        self.settings
    }

    /// What a demo reads instead of the renderer's own API keys: Anthropic saved, Soniox missing,
    /// so one screenshot has both states and no server is asked for languages. `request`
    /// numbers each answer the way a real read would be numbered.
    pub fn key_states(
        mut request: impl FnMut(ApiKey) -> u64,
    ) -> Vec<(ApiKey, settings::KeyMessage)> {
        [
            (ApiKey::Anthropic, KeyState::Saved),
            (ApiKey::Soniox, KeyState::Missing),
        ]
        .into_iter()
        .map(|(which, state)| {
            let message = settings::KeyMessage::State {
                request: request(which),
                result: Ok(state),
            };
            (which, message)
        })
        .collect()
    }

    /// Start the watchdog; call once the main window is open.
    pub fn start() -> Task<Message> {
        Task::future(async {
            tokio::time::sleep(TIMEOUT).await;
            Message::TimedOut
        })
    }

    /// The steps to take when the video is ready: the first time, set up the scenario's state;
    /// afterwards nothing. The app then opens the window to capture and calls [`Self::capture`].
    pub fn video_ready(&mut self) -> Option<Vec<folder_workspace::Message>> {
        if std::mem::replace(&mut self.video_ready, true) {
            return None;
        }
        Some(steps(&self.scenario, self.batch, self.ai))
    }

    /// Schedule the screenshot of `window`. `size` is its size in logical pixels, checked at
    /// scale 1; `None` for the main window, which has the scenario's size.
    pub fn capture(&self, window: window::Id, size: Option<(u32, u32)>) -> Task<Message> {
        let [width, height] = self.scenario.window;
        let expected = size.unwrap_or((width, height));
        // The screenshot re-renders what was last drawn. A message would rebuild the UI first,
        // and a text editor's rebuilt text is not in the last drawing, so the comment box would
        // come out empty: chain the screenshot straight onto the wait, with no message between.
        // A turn shows a note over the picture for 2 s: take the shot while it is there (the
        // reopened clip of a demo is small and ready well before).
        let settle = if self.scenario.rotate != 0 {
            ROTATE_SETTLE
        } else {
            SETTLE
        };
        Task::future(async move { tokio::time::sleep(settle).await })
            .then(move |()| window::screenshot(window))
            .map(move |shot| Message::Captured(shot, expected))
    }

    /// Handle a demo message.
    pub fn update(&self, message: Message) -> Task<Message> {
        match message {
            Message::Captured(shot, expected) => match save_png(&shot, expected, &self.out) {
                Ok(()) => {
                    log::info!("demo: saved {}", self.out.display());
                    iced::exit()
                }
                Err(e) => self.fail(&e),
            },
            Message::TimedOut => self.fail("the video did not get ready in time"),
        }
    }

    fn fail(&self, reason: &str) -> ! {
        log::error!(
            "demo: {reason}; the work folder is kept: {}",
            self.work.display()
        );
        std::process::exit(1);
    }
}

/// What to do once the video is ready: pause at the scenario's time, open the subtitle or marker
/// list when the scenario asks for it, and turn on batch mode with every file checked when asked to.
/// In batch mode, select the scenario's `batch_action`; `ai` stands for "Describe with AI" (the
/// open file's AI description is in its comment box already).
fn steps(scenario: &DemoScenario, batch: bool, ai: bool) -> Vec<folder_workspace::Message> {
    let video =
        |message| folder_workspace::Message::MediaViewer(media_viewer::Message::Video(message));
    // A clip opened where playback stopped is already there (#161): a seek would move it on.
    let mut steps = if scenario.resume.is_some() {
        Vec::new()
    } else {
        vec![video(video::Message::Seek(scenario.seek))]
    };
    if scenario.subtitle_list {
        steps.push(video(video::Message::ToggleCueList));
    }
    if scenario.marker_list {
        steps.push(video(video::Message::ShowMarkerList));
    }
    if scenario.more {
        steps.push(video(video::Message::ToggleMore));
    }
    if scenario.slow_step {
        steps.push(video(video::Message::ShowSlowStep));
    }
    if scenario.recent_menu {
        steps.push(folder_workspace::Message::Folder(
            folder::Message::ToggleRecentFolders,
        ));
    }
    if let Some(name) = &scenario.describing {
        steps.push(folder_workspace::Message::ShowDescribing(name.clone()));
    }
    if scenario.describing_unnamed {
        steps.push(folder_workspace::Message::ShowDescribingUnnamed);
    }
    if scenario.rotate != 0 {
        steps.push(folder_workspace::Message::RotateVideo(scenario.rotate));
    }
    // Multi-select (issue #60): reproduce Ctrl/Shift+click the way a real click does — hold
    // the modifier, then select — independently of `--batch`, since either click turns batch
    // mode on by itself.
    let index_of = |name: &str| scenario.files.iter().position(|f| f.name == name);
    for name in &scenario.ctrl_click {
        if let Some(index) = index_of(name) {
            steps.push(folder_workspace::Message::ModifiersChanged(
                iced::keyboard::Modifiers::COMMAND,
            ));
            steps.push(folder_workspace::Message::Folder(
                folder::Message::SelectFile(index),
            ));
        }
    }
    if let Some(index) = scenario.shift_click.as_deref().and_then(index_of) {
        steps.push(folder_workspace::Message::ModifiersChanged(
            iced::keyboard::Modifiers::SHIFT,
        ));
        steps.push(folder_workspace::Message::Folder(
            folder::Message::SelectFile(index),
        ));
    }
    if !scenario.ctrl_click.is_empty() || scenario.shift_click.is_some() {
        steps.push(folder_workspace::Message::ModifiersChanged(
            iced::keyboard::Modifiers::empty(),
        ));
    }
    if let Some(query) = scenario.search.clone() {
        steps.push(folder_workspace::Message::Folder(
            folder::Message::SetNameFilter(query),
        ));
    }
    if batch {
        steps.push(folder_workspace::Message::Folder(
            folder::Message::SetBatchMode(true),
        ));
        steps.push(folder_workspace::Message::Folder(
            folder::Message::ToggleAllChecked,
        ));
        let name = if ai {
            Some(batch::Action::DescribeAi.log_id())
        } else {
            scenario.batch_action.as_deref()
        };
        let action = name.and_then(|name| {
            let found = batch::Action::ALL
                .into_iter()
                .find(|action| action.log_id() == name);
            if found.is_none() {
                log::warn!("demo: no batch action is named {name:?}");
            }
            found
        });
        if let Some(action) = action {
            steps.push(folder_workspace::Message::Batch(
                batch::Message::SelectAction(action),
            ));
        }
    }
    if let Some(seconds) = scenario.ai_retry_wait_s.filter(|_| batch) {
        steps.push(folder_workspace::Message::ShowBatchRetry(seconds));
    }
    steps
}

/// Save `shot` as a PNG at `path`, when it is `expected` pixels in size: the README templates
/// place their labels for exactly that size.
fn save_png(shot: &window::Screenshot, expected: (u32, u32), path: &Path) -> Result<(), String> {
    let size = (shot.size.width, shot.size.height);
    if size != expected {
        return Err(format!(
            "the window is {}x{} pixels, not {}x{} (the display must fit it, at scale 1)",
            size.0, size.1, expected.0, expected.1
        ));
    }
    let image = image::RgbaImage::from_raw(size.0, size.1, shot.rgba.to_vec())
        .ok_or("the screenshot has the wrong number of bytes")?;
    image
        .save(path)
        .map_err(|e| format!("cannot save {}: {e}", path.display()))
}

/// What `--demo <scenario> --out <png> [--batch] [--mono] [--ai] [--settings [page]]
/// [--lang <code>]` asks for.
#[derive(Debug, Clone, PartialEq)]
pub struct DemoArgs {
    pub scenario: PathBuf,
    pub out: PathBuf,
    /// Show batch mode with every file checked: one scenario gives both README screenshots.
    pub batch: bool,
    /// Turn on the monochrome tags setting.
    pub mono: bool,
    /// Show the AI description (see [`DemoRun`]).
    pub ai: bool,
    /// Capture the settings window, on this page (`--settings ai`; the first page when none is
    /// named), instead of the main one.
    pub settings: Option<settings::Page>,
    /// The UI language setting: `en` unless given, so screenshots never follow the renderer's
    /// OS language; `--lang ""` follows it (System).
    pub lang: String,
}

/// The demo the command line asks for; `None` for a normal start.
pub fn demo_args(args: &[String]) -> Option<Result<DemoArgs, String>> {
    let at = args.iter().position(|a| a == "--demo")?;
    let value = |flag: &str, index: Option<usize>| {
        index
            .and_then(|i| args.get(i + 1))
            .filter(|v| !v.starts_with("--"))
            .map(PathBuf::from)
            .ok_or(format!("{flag} needs a value"))
    };
    let out_at = args.iter().position(|a| a == "--out");
    let lang_at = args.iter().position(|a| a == "--lang");
    let settings_at = args.iter().position(|a| a == "--settings");
    Some(value("--demo", Some(at)).and_then(|scenario| {
        let out =
            value("--out", out_at).map_err(|_| "--demo needs --out <file.png>".to_string())?;
        let lang = match lang_at {
            None => "en".to_string(),
            Some(_) => value("--lang", lang_at)?.to_string_lossy().into_owned(),
        };
        if settings_at.is_some() && args.iter().any(|a| a == "--batch") {
            // The batch panel reads the real keys, which a settings screenshot must not show.
            return Err("--settings and --batch go in separate demos".to_string());
        }
        let settings = match settings_at {
            None => None,
            Some(at) => match args.get(at + 1).filter(|v| !v.starts_with("--")) {
                None => Some(settings::Page::default()),
                Some(name) => Some(
                    settings::Page::from_name(name)
                        .ok_or(format!("--settings: no page called {name}"))?,
                ),
            },
        };
        Ok(DemoArgs {
            scenario,
            out,
            batch: args.iter().any(|a| a == "--batch"),
            mono: args.iter().any(|a| a == "--mono"),
            ai: args.iter().any(|a| a == "--ai"),
            settings,
            lang,
        })
    }))
}

/// The demo's throwaway folder (`<temp>/frename-demo-<pid>`), removed when dropped.
pub struct WorkDir(PathBuf);

impl WorkDir {
    /// A fresh, empty work folder path for this process.
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!("frename-demo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Stage the scenario in `work`, point the app's stored state at it, and return the run.
/// The app database must already be set up (`FRENAME_DATA_DIR` pointing into `work`).
pub fn prepare(args: &DemoArgs, work: &Path) -> Result<DemoRun, String> {
    let scenario_path = &args.scenario;
    let text = std::fs::read_to_string(scenario_path)
        .map_err(|e| format!("cannot read {}: {e}", scenario_path.display()))?;
    let scenario = DemoScenario::parse(&text).map_err(|e| e.to_string())?;
    let scenario_dir = scenario_path.parent().unwrap_or(Path::new("."));
    let folder = work.join(scenario.folder_name.as_deref().unwrap_or("folder"));
    let file = frename_core::demo::stage(&scenario, scenario_dir, &folder)
        .map_err(|e| format!("cannot stage the demo folder: {e}"))?;
    let db = frename_core::AppDatabase::new();
    frename_core::demo::seed(&db, &scenario, &folder, &file, args.mono, &args.lang);
    Ok(DemoRun::new(
        scenario,
        args.out.clone(),
        work.to_path_buf(),
        args,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn no_demo_flag_means_a_normal_start() {
        assert_eq!(demo_args(&args(&["frename", "--out", "x.png"])), None);
    }

    #[test]
    fn demo_takes_a_scenario_and_an_output() {
        assert_eq!(
            demo_args(&args(&["frename", "--demo", "a.toml", "--out", "a.png"])),
            Some(Ok(DemoArgs {
                scenario: "a.toml".into(),
                out: "a.png".into(),
                batch: false,
                mono: false,
                ai: false,
                settings: None,
                lang: "en".to_string(),
            }))
        );
        assert_eq!(
            demo_args(&args(&[
                "frename", "--batch", "--out", "a.png", "--demo", "a.toml", "--mono", "--lang",
                "ru"
            ])),
            Some(Ok(DemoArgs {
                scenario: "a.toml".into(),
                out: "a.png".into(),
                batch: true,
                mono: true,
                ai: false,
                settings: None,
                lang: "ru".to_string(),
            }))
        );
    }

    #[test]
    fn settings_alone_captures_the_first_page_and_an_unknown_page_is_an_error() {
        let parsed = demo_args(&args(&[
            "frename",
            "--demo",
            "a.toml",
            "--out",
            "a.png",
            "--settings",
        ]));
        assert!(matches!(
            parsed,
            Some(Ok(DemoArgs {
                settings: Some(settings::Page::Interface),
                ..
            }))
        ));
        assert!(matches!(
            demo_args(&args(&[
                "frename",
                "--demo",
                "a.toml",
                "--out",
                "a.png",
                "--settings",
                "general"
            ])),
            Some(Err(_))
        ));
        let parsed = demo_args(&args(&[
            "frename",
            "--demo",
            "a.toml",
            "--out",
            "a.png",
            "--settings",
            "ai",
            "--lang",
            "ru",
        ]));
        assert!(matches!(
            parsed,
            Some(Ok(DemoArgs {
                settings: Some(settings::Page::Ai),
                ..
            }))
        ));
        assert!(
            matches!(
                demo_args(&args(&[
                    "frename",
                    "--demo",
                    "a.toml",
                    "--out",
                    "a.png",
                    "--settings",
                    "--batch"
                ])),
                Some(Err(_))
            ),
            "batch mode reads the real keys"
        );
    }

    #[test]
    fn demo_without_a_scenario_or_an_output_is_an_error() {
        assert!(matches!(
            demo_args(&args(&["frename", "--demo", "a.toml"])),
            Some(Err(_))
        ));
        assert!(matches!(
            demo_args(&args(&["frename", "--demo", "--out", "a.png"])),
            Some(Err(_))
        ));
    }

    fn scenario() -> DemoScenario {
        DemoScenario::parse(
            "source = \".\"\nopen = \"a.mp4\"\nseek = 2.5\nwindow = [4, 2]\n\
             panels = [1, 1]\n[[files]]\nfrom = \"a.mp4\"\nname = \"a.mp4\"\n",
        )
        .unwrap()
    }

    #[test]
    fn a_scenario_can_turn_the_clip_and_run_a_named_batch_action() {
        let mut turned = scenario();
        turned.rotate = 1;
        assert!(matches!(
            steps(&turned, false, false).last(),
            Some(folder_workspace::Message::RotateVideo(1))
        ));

        let mut run = scenario();
        run.batch_action = Some("Rotate videos".to_string());
        let batch_steps = steps(&run, true, false);
        assert!(matches!(
            batch_steps.last(),
            Some(folder_workspace::Message::Batch(
                batch::Message::SelectAction(batch::Action::Rotate)
            ))
        ));
    }

    #[test]
    fn the_video_is_paused_at_the_scenario_time_and_batch_mode_only_when_asked() {
        let plain = steps(&scenario(), false, false);
        assert!(matches!(
            plain.as_slice(),
            [folder_workspace::Message::MediaViewer(media_viewer::Message::Video(
                video::Message::Seek(s)
            ))] if *s == 2.5
        ));
        let batch = steps(&scenario(), true, false);
        assert_eq!(batch.len(), 3);
        assert!(matches!(
            batch[1],
            folder_workspace::Message::Folder(folder::Message::SetBatchMode(true))
        ));
        assert!(matches!(
            batch[2],
            folder_workspace::Message::Folder(folder::Message::ToggleAllChecked)
        ));
    }

    #[test]
    fn the_subtitle_list_opens_only_when_the_scenario_asks() {
        let mut with_list = scenario();
        with_list.subtitle_list = true;
        let steps = steps(&with_list, false, false);
        assert_eq!(steps.len(), 2);
        assert!(matches!(
            steps[1],
            folder_workspace::Message::MediaViewer(media_viewer::Message::Video(
                video::Message::ToggleCueList
            ))
        ));
    }

    #[test]
    fn ai_selects_describe_with_ai_in_batch_mode() {
        assert_eq!(
            steps(&scenario(), false, true).len(),
            1,
            "the open file's description is in its comment box already"
        );
        let batch_mode = steps(&scenario(), true, true);
        assert!(matches!(
            batch_mode.last(),
            Some(folder_workspace::Message::Batch(
                batch::Message::SelectAction(batch::Action::DescribeAi)
            ))
        ));
    }

    #[test]
    fn only_the_first_ready_video_sets_up_the_scenario() {
        let args = DemoArgs {
            scenario: "a.toml".into(),
            out: "o.png".into(),
            batch: false,
            mono: false,
            ai: false,
            settings: None,
            lang: "en".to_string(),
        };
        let mut run = DemoRun::new(scenario(), "o.png".into(), "w".into(), &args);
        assert!(run.video_ready().is_some());
        assert!(run.video_ready().is_none());
    }

    #[test]
    fn a_screenshot_of_the_expected_size_is_saved_as_is() {
        let path = std::env::temp_dir().join(format!("frename-demo-{}.png", std::process::id()));
        let rgba: Vec<u8> = (0..32).collect();
        let shot = window::Screenshot::new(rgba.clone(), iced::Size::new(4, 2), 1.0);

        save_png(&shot, (4, 2), &path).unwrap();

        assert_eq!(image::open(&path).unwrap().to_rgba8().into_raw(), rgba);
        std::fs::remove_file(&path).unwrap();
    }

    /// The committed README scenarios stage clips CI can decode, and only tags a new folder
    /// already has (any other tag would show up as unsaved) — except `mono.toml`, whose point
    /// is exactly one tag that is not in the list, so the monochrome screenshot can show it next
    /// to a listed one (issue #53).
    #[test]
    fn the_readme_scenarios_are_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let decodable = std::fs::read_to_string(root.join("tests/self-test-clips.txt")).unwrap();
        let known: Vec<&str> = frename_core::DEFAULT_TAGS.iter().map(|t| t.name).collect();
        let screenshots = root.join("docs/screenshots");
        for entry in std::fs::read_dir(&screenshots).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "toml") {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let scenario = DemoScenario::parse(&std::fs::read_to_string(&path).unwrap())
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            let source = path.parent().unwrap().join(&scenario.source);
            for file in &scenario.files {
                assert!(source.join(&file.from).is_file(), "{name}: {}", file.from);
                assert!(
                    decodable
                        .lines()
                        .any(|l| l.ends_with(&format!("/{}", file.from))),
                    "{name}: {} is not in tests/self-test-clips.txt",
                    file.from
                );
                // These stage tags the folder does not have, on purpose: the mono scenario for
                // tags it does not know, the other for an unsaved tag's "+" (#216).
                if name == "mono.toml" || name == "unsaved-tag-plus.toml" {
                    continue;
                }
                for tag in frename_core::FileSnapshot::parse(&file.name).tags() {
                    assert!(known.contains(&tag.as_str()), "{name}: tag {tag}");
                }
            }
        }
    }

    #[test]
    fn the_work_folder_goes_when_dropped() {
        let work = WorkDir::new();
        std::fs::create_dir_all(work.path().join("data")).unwrap();
        let path = work.path().to_path_buf();
        drop(work);
        assert!(!path.exists());
    }

    #[test]
    fn a_screenshot_of_another_size_is_refused() {
        let shot = window::Screenshot::new(vec![0u8; 32], iced::Size::new(4, 2), 1.0);
        let error = save_png(&shot, (8, 4), Path::new("never.png")).unwrap_err();
        assert!(error.contains("4x2"), "{error}");
        assert!(!Path::new("never.png").exists());
    }
}
