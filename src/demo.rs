//! Demo mode: `frename --demo <scenario.toml> --out <png>` opens a staged folder in a known state,
//! saves a screenshot of the main window and exits. CI uses it for the README screenshots
//! (`docs/screenshots/`).
//!
//! Everything runs in a throwaway work folder: the staged clips and the app's own database and
//! log (`FRENAME_DATA_DIR`), so the user's files and settings are never touched.

use std::path::{Path, PathBuf};
use std::time::Duration;

use frename_core::demo::DemoScenario;
use iced::{window, Task};

use crate::features::{folder, folder_workspace, media_viewer, media_viewer::video};

/// Time from the video being ready to the screenshot: covers the seek, the subtitles, and the
/// comments that load in the background.
const SETTLE: Duration = Duration::from_secs(3);

/// A demo that has not produced its screenshot by then has failed.
const TIMEOUT: Duration = Duration::from_secs(90);

/// Demo mode messages.
#[derive(Debug, Clone)]
pub enum Message {
    /// The main window's screenshot.
    Captured(window::Screenshot),
    /// The demo took too long.
    TimedOut,
}

/// A demo in progress.
#[derive(Debug, Clone)]
pub struct DemoRun {
    scenario: DemoScenario,
    out: PathBuf,
    work: PathBuf,
    video_ready: bool,
}

impl DemoRun {
    /// `work` is the throwaway folder; `main` removes it after the app has exited.
    pub fn new(scenario: DemoScenario, out: PathBuf, work: PathBuf) -> Self {
        Self {
            scenario,
            out,
            work,
            video_ready: false,
        }
    }

    /// Start the watchdog; call once the main window is open.
    pub fn start() -> Task<Message> {
        Task::future(async {
            tokio::time::sleep(TIMEOUT).await;
            Message::TimedOut
        })
    }

    /// The steps to take when the video is ready: the first time, set up the scenario's state
    /// and schedule the screenshot of `main_window`; afterwards nothing.
    pub fn video_ready(
        &mut self,
        main_window: window::Id,
    ) -> Option<(Vec<folder_workspace::Message>, Task<Message>)> {
        if std::mem::replace(&mut self.video_ready, true) {
            return None;
        }
        // The screenshot re-renders what was last drawn. A message would rebuild the UI first,
        // and a text editor's rebuilt text is not in the last drawing, so the comment box would
        // come out empty: chain the screenshot straight onto the wait, with no message between.
        let capture = Task::future(async { tokio::time::sleep(SETTLE).await })
            .then(move |()| window::screenshot(main_window))
            .map(Message::Captured);
        Some((steps(&self.scenario), capture))
    }

    /// Handle a demo message.
    pub fn update(&self, message: Message) -> Task<Message> {
        match message {
            Message::Captured(shot) => {
                let [width, height] = self.scenario.window;
                match save_png(&shot, (width, height), &self.out) {
                    Ok(()) => {
                        log::info!("demo: saved {}", self.out.display());
                        iced::exit()
                    }
                    Err(e) => self.fail(&e),
                }
            }
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

/// What to do once the video is ready: pause at the scenario's time, and turn on batch mode with
/// every file checked when the scenario asks for it.
fn steps(scenario: &DemoScenario) -> Vec<folder_workspace::Message> {
    let mut steps = vec![folder_workspace::Message::MediaViewer(
        media_viewer::Message::Video(video::Message::Seek(scenario.seek as f32)),
    )];
    if scenario.batch {
        steps.push(folder_workspace::Message::Folder(
            folder::Message::SetBatchMode(true),
        ));
        steps.push(folder_workspace::Message::Folder(
            folder::Message::ToggleAllChecked,
        ));
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

/// The scenario and output file given with `--demo <scenario> --out <png>`; `None` for a normal
/// start.
pub fn demo_args(args: &[String]) -> Option<Result<(PathBuf, PathBuf), String>> {
    let at = args.iter().position(|a| a == "--demo")?;
    let value = |flag: &str, index: Option<usize>| {
        index
            .and_then(|i| args.get(i + 1))
            .filter(|v| !v.starts_with("--"))
            .map(PathBuf::from)
            .ok_or(format!("{flag} needs a value"))
    };
    let out_at = args.iter().position(|a| a == "--out");
    Some(value("--demo", Some(at)).and_then(|scenario| {
        let out =
            value("--out", out_at).map_err(|_| "--demo needs --out <file.png>".to_string())?;
        Ok((scenario, out))
    }))
}

/// Stage the scenario in `work`, point the app's stored state at it, and return the run.
/// The app database must already be set up (`FRENAME_DATA_DIR` pointing into `work`).
pub fn prepare(scenario_path: &Path, out: PathBuf, work: PathBuf) -> Result<DemoRun, String> {
    let text = std::fs::read_to_string(scenario_path)
        .map_err(|e| format!("cannot read {}: {e}", scenario_path.display()))?;
    let scenario = DemoScenario::parse(&text).map_err(|e| e.to_string())?;
    let scenario_dir = scenario_path.parent().unwrap_or(Path::new("."));
    let folder = work.join("folder");
    let file = frename_core::demo::stage(&scenario, scenario_dir, &folder)
        .map_err(|e| format!("cannot stage the demo folder: {e}"))?;
    frename_core::demo::seed(&frename_core::AppDatabase::new(), &scenario, &folder, &file);
    Ok(DemoRun::new(scenario, out, work))
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
            Some(Ok((PathBuf::from("a.toml"), PathBuf::from("a.png"))))
        );
        assert_eq!(
            demo_args(&args(&["frename", "--out", "a.png", "--demo", "a.toml"])),
            Some(Ok((PathBuf::from("a.toml"), PathBuf::from("a.png"))))
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

    fn scenario(batch: bool) -> DemoScenario {
        DemoScenario::parse(&format!(
            "source = \".\"\nopen = \"a.mp4\"\nseek = 2.5\nbatch = {batch}\nwindow = [4, 2]\n\
             panels = [1, 1]\n[[files]]\nfrom = \"a.mp4\"\nname = \"a.mp4\"\n"
        ))
        .unwrap()
    }

    #[test]
    fn the_video_is_paused_at_the_scenario_time_and_batch_mode_only_when_asked() {
        let plain = steps(&scenario(false));
        assert!(matches!(
            plain.as_slice(),
            [folder_workspace::Message::MediaViewer(media_viewer::Message::Video(
                video::Message::Seek(s)
            ))] if *s == 2.5
        ));
        let batch = steps(&scenario(true));
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
    fn only_the_first_ready_video_sets_up_the_scenario() {
        let mut run = DemoRun::new(scenario(false), "o.png".into(), "w".into());
        let window = window::Id::unique();
        assert!(run.video_ready(window).is_some());
        assert!(run.video_ready(window).is_none());
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
    /// already has (any other tag would show up as unsaved).
    #[test]
    fn the_readme_scenarios_are_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let decodable = std::fs::read_to_string(root.join("tests/self-test-clips.txt")).unwrap();
        let known: Vec<&str> = frename_core::DEFAULT_TAGS.iter().map(|t| t.name).collect();
        for name in ["main", "batch"] {
            let path = root.join("docs/screenshots").join(format!("{name}.toml"));
            let scenario = DemoScenario::parse(&std::fs::read_to_string(&path).unwrap())
                .unwrap_or_else(|e| panic!("{name}.toml: {e}"));
            let source = path.parent().unwrap().join(&scenario.source);
            if let Some(image) = &scenario.snap_image {
                assert!(source.join(image).is_file(), "{name}.toml: {image}");
            }
            for file in &scenario.files {
                assert!(
                    source.join(&file.from).is_file(),
                    "{name}.toml: {}",
                    file.from
                );
                assert!(
                    decodable
                        .lines()
                        .any(|l| l.ends_with(&format!("/{}", file.from))),
                    "{name}.toml: {} is not in tests/self-test-clips.txt",
                    file.from
                );
                for tag in frename_core::FileSnapshot::parse(&file.name).tags() {
                    assert!(known.contains(&tag.as_str()), "{name}.toml: tag {tag}");
                }
            }
        }
    }

    #[test]
    fn a_screenshot_of_another_size_is_refused() {
        let shot = window::Screenshot::new(vec![0u8; 32], iced::Size::new(4, 2), 1.0);
        let error = save_png(&shot, (8, 4), Path::new("never.png")).unwrap_err();
        assert!(error.contains("4x2"), "{error}");
        assert!(!Path::new("never.png").exists());
    }
}
