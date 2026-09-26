//! "Describe with AI": for each checked video, frames sampled every 2 s and its subtitles go to
//! Claude, and the summary and time-ranged segments it returns are written into the AI block of
//! the video's comment (see `frename_core::ai::block`). The editor's own text is never touched.
//!
//! Before running, the panel shows what will be sent and about what it costs: clip lengths are
//! read in the background (see [`Options::missing_probes`]) and kept per file.

mod frames;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use frename_core::ai::describe::{self, SummaryLanguage, MAX_DURATION_S, MODEL};
use frename_core::ai::key::{self, KeyState};
use frename_core::ai::provider::{AiError, AiProvider, AiUsage};
use frename_core::ai::{anthropic::Anthropic, block};
use frename_core::{File, FileId, FileKind, FileTagger, FolderInfo};
use iced::widget::{button, checkbox, column, row, text};
use iced::{Element, Length};

pub use frames::Probe;

use super::super::{ItemResult, ItemStatus};
use super::ActionMessage;
use crate::theme;

pub const LABEL: &str = "Describe with AI";

/// Clip lengths read at once in the background.
const PROBES_AT_ONCE: usize = 4;
/// For the time estimate: one request, and sampling one frame.
const SECONDS_PER_REQUEST: f64 = 15.0;
const SECONDS_PER_FRAME: f64 = 0.3;

#[derive(Debug, Clone)]
pub enum Message {
    /// Redo videos that already have an AI description.
    SetRedo(bool),
    /// Clip lengths read in the background.
    Probed(Vec<(FileId, Probe)>),
    /// Whether an API key is saved, read in the background or after a change in the settings.
    KeyState(KeyState),
}

/// What the job does to each file: the options it started with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    pub language: SummaryLanguage,
    pub redo: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    redo: bool,
    /// What is known about each checked video, kept while the folder is open.
    probes: HashMap<FileId, Probe>,
    /// Videos whose length is being read.
    probing: HashSet<FileId>,
    /// `None` until read (only when the action is first shown: reading may unlock a keyring).
    key: Option<KeyState>,
}

impl Options {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetRedo(redo) => self.redo = redo,
            Message::Probed(results) => {
                for (id, probe) in results {
                    self.probing.remove(&id);
                    self.probes.insert(id, probe);
                }
            }
            Message::KeyState(state) => self.key = Some(state),
        }
    }

    pub fn operation(&self) -> super::Operation {
        super::Operation::DescribeAi(Run {
            language: frename_core::ai::summary_language(),
            redo: self.redo,
        })
    }

    /// Whether the key's state still has to be read.
    pub fn needs_key_state(&self) -> bool {
        self.key.is_none()
    }

    /// Forget what was read about the files of the folder that was open.
    pub fn reset_files(&mut self) {
        self.probes.clear();
        self.probing.clear();
    }

    /// Checked videos whose length is not known or being read yet; they are marked as being
    /// read.
    pub fn missing_probes<'a>(
        &mut self,
        checked: impl Iterator<Item = &'a File>,
    ) -> Vec<(FileId, PathBuf)> {
        let missing: Vec<(FileId, PathBuf)> = checked
            .filter(|f| f.kind() == FileKind::Video)
            .filter(|f| !self.probes.contains_key(&f.id()) && !self.probing.contains(&f.id()))
            .map(|f| (f.id(), f.file_path().to_path_buf()))
            .collect();
        self.probing.extend(missing.iter().map(|(id, _)| *id));
        missing
    }

    /// The videos of `checked` a run will send, in order.
    pub fn files_to_send(&self, checked: &[&File]) -> Vec<FileId> {
        checked
            .iter()
            .filter(|f| self.will_send(f))
            .map(|f| f.id())
            .collect()
    }

    fn will_send(&self, file: &File) -> bool {
        file.kind() == FileKind::Video
            && !file.snapshot().comment_loading()
            && (self.redo || block::ai_block(file.comment()).is_none())
            && self
                .probes
                .get(&file.id())
                .and_then(|p| p.duration_s)
                .is_some_and(|d| d <= MAX_DURATION_S)
    }

    /// What a run on `checked` would send, skip and cost.
    fn plan(&self, checked: &[&File]) -> Plan {
        let mut plan = Plan {
            checked_videos: checked
                .iter()
                .filter(|f| f.kind() == FileKind::Video)
                .count(),
            ..Plan::default()
        };
        for file in checked {
            if file.kind() != FileKind::Video {
                plan.not_videos += 1;
                continue;
            }
            if file.snapshot().comment_loading() {
                plan.estimating = true;
                continue;
            }
            if !self.redo && block::ai_block(file.comment()).is_some() {
                plan.described += 1;
                continue;
            }
            let Some(probe) = self.probes.get(&file.id()) else {
                plan.estimating = true;
                continue;
            };
            plan.probed += 1;
            match probe.duration_s {
                None => plan.unreadable += 1,
                Some(d) if d > MAX_DURATION_S => plan.too_long += 1,
                Some(d) => {
                    plan.videos += 1;
                    plan.seconds += d;
                    plan.usage += describe::estimate_usage(d, probe.subtitle_chars);
                    plan.run_seconds += SECONDS_PER_REQUEST
                        + SECONDS_PER_FRAME * describe::sample_times(d).len() as f64;
                    if !file.has_subtitles() {
                        plan.no_subtitles += 1;
                    }
                }
            }
        }
        plan
    }

    /// The run button's label and whether it can run: `Describe 12 videos` once the estimate
    /// and the key are known.
    pub fn run_button(&self, checked: &[&File]) -> (String, bool) {
        let plan = self.plan(checked);
        let label = format!("Describe {}", videos(plan.videos));
        let ready = !plan.estimating && plan.videos > 0 && self.key == Some(KeyState::Saved);
        (label, ready)
    }

    pub fn view(&self, checked: &[&File]) -> Element<'_, ActionMessage> {
        let plan = self.plan(checked);
        let mut lines = column![].spacing(6);
        let muted = |line: String| text(line).size(12).color(theme::TEXT_MUTED);
        if plan.estimating {
            let known = plan.probed + plan.described;
            lines =
                lines.push(text(format!("Estimating… {known} / {}", plan.checked_videos)).size(13));
        } else {
            lines = lines.push(
                text(format!(
                    "{}, {} · about {} with {}",
                    videos(plan.videos),
                    minutes(plan.seconds),
                    dollars(MODEL.cost_usd(plan.usage)),
                    MODEL.label
                ))
                .size(13),
            );
            if plan.videos > 0 {
                lines = lines.push(muted(format!(
                    "Takes about {}. The folder is locked until it ends. Cancel keeps the videos \
                     already described; running it again skips them.",
                    duration_text(plan.run_seconds)
                )));
            }
            if let Some(skipped) = plan.skipped_line() {
                lines = lines.push(muted(skipped));
            }
            if plan.no_subtitles > 0 {
                lines = lines.push(muted(format!(
                    "Without subtitles (only the picture is described): {}.",
                    videos(plan.no_subtitles)
                )));
            }
        }
        lines = lines
            .push(
                row![
                    text(language_line(frename_core::ai::summary_language()))
                        .size(12)
                        .width(Length::Fill),
                    link_button("Change", ActionMessage::OpenAiSettings),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .push(muted(
                "Frames and subtitles of these videos are sent to Anthropic.".to_string(),
            ))
            .push(
                checkbox(self.redo)
                    .label("Redo videos that already have an AI description")
                    .text_size(13)
                    .on_toggle(|redo| ActionMessage::DescribeAi(Message::SetRedo(redo))),
            );
        match self.key {
            Some(KeyState::Missing) => {
                lines = lines.push(
                    row![
                        text("Set an Anthropic API key in Settings")
                            .size(13)
                            .color(theme::ERROR)
                            .width(Length::Fill),
                        link_button("Open Settings", ActionMessage::OpenAiSettings),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
            }
            Some(KeyState::Unavailable) => {
                lines = lines.push(
                    text("Cannot store an API key on this system")
                        .size(13)
                        .color(theme::ERROR),
                )
            }
            Some(KeyState::Saved) | None => {}
        }
        super::panel(
            LABEL,
            "Describes what happens in each checked video, and when: a summary and time-ranged \
             segments go into the AI description of its comment. Your own comment text is kept."
                .to_string(),
            lines.into(),
        )
    }
}

fn link_button(label: &str, message: ActionMessage) -> Element<'_, ActionMessage> {
    button(text(label).size(12))
        .on_press(message)
        .padding([3, 10])
        .style(theme::icon_button_style(true))
        .into()
}

/// What a run on the checked files would do.
#[derive(Debug, Default, PartialEq)]
struct Plan {
    /// Videos that will be sent.
    videos: usize,
    seconds: f64,
    usage: AiUsage,
    /// About how long the run takes.
    run_seconds: f64,
    described: usize,
    too_long: usize,
    not_videos: usize,
    unreadable: usize,
    no_subtitles: usize,
    /// Some lengths or comments are still being read.
    estimating: bool,
    /// Checked videos, and those whose length is known, for the progress of the estimate.
    checked_videos: usize,
    probed: usize,
}

impl Plan {
    /// `Skipped: 3 already described, 1 over 30 min, 200 photos.`
    fn skipped_line(&self) -> Option<String> {
        let parts: Vec<String> = [
            (self.described, "already described".to_string()),
            (self.too_long, "over 30 min".to_string()),
            (
                self.not_videos,
                if self.not_videos == 1 {
                    "photo"
                } else {
                    "photos"
                }
                .to_string(),
            ),
            (self.unreadable, "unreadable".to_string()),
        ]
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, what)| format!("{n} {what}"))
        .collect();
        (!parts.is_empty()).then(|| format!("Skipped: {}.", parts.join(", ")))
    }
}

/// "12 videos" / "1 video".
fn videos(n: usize) -> String {
    format!("{n} {}", if n == 1 { "video" } else { "videos" })
}

/// "38 min", or "< 1 min".
fn minutes(seconds: f64) -> String {
    match (seconds / 60.0).round() as u64 {
        0 if seconds > 0.0 => "< 1 min".to_string(),
        n => format!("{n} min"),
    }
}

/// "Descriptions in Russian", or in the subtitles' language.
fn language_line(language: SummaryLanguage) -> String {
    match language {
        SummaryLanguage::SameAsSubtitles => {
            "Descriptions in the subtitles' language (English if none)".to_string()
        }
        language => format!("Descriptions in {language}"),
    }
}

/// "25 min", "2 h 10 min", "under a minute".
fn duration_text(seconds: f64) -> String {
    let minutes = (seconds / 60.0).round() as u64;
    match minutes {
        0 => "under a minute".to_string(),
        m if m < 60 => format!("{m} min"),
        m if m % 60 == 0 => format!("{} h", m / 60),
        m => format!("{} h {} min", m / 60, m % 60),
    }
}

/// "$0.35", "under $0.01".
pub fn dollars(usd: f64) -> String {
    if usd > 0.0 && usd < 0.01 {
        "under $0.01".to_string()
    } else {
        format!("${usd:.2}")
    }
}

/// Read the lengths of `clips`, a few at once. Blocking: runs on a worker thread.
pub fn probe_all(clips: Vec<(FileId, PathBuf)>) -> Vec<(FileId, Probe)> {
    let chunk = clips.len().div_ceil(PROBES_AT_ONCE).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = clips
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|(id, path)| (*id, frames::probe(path)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap_or_default())
            .collect()
    })
}

/// Read whether a key is saved. Blocking: runs on a worker thread.
pub fn read_key_state() -> KeyState {
    key::key_state()
}

/// Describe the video at `path` and write the description into its comment.
pub fn run(options: Run, path: &Path, cancel: &AtomicBool) -> ItemResult {
    let is_video = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| FileKind::from_extension(e) == FileKind::Video);
    if !is_video {
        return ItemResult::new(ItemStatus::Skipped, None);
    }
    // Read from the file itself, so a comment the folder had not loaded yet is kept.
    let mut snapshot = FileTagger::parse(path, &FolderInfo::default());
    if !options.redo && block::ai_block(snapshot.comment()).is_some() {
        return ItemResult::new(ItemStatus::Skipped, None);
    }
    // Removed in the settings while the job runs: every later video would fail the same way.
    let Some(api_key) = key::read_key() else {
        return ItemResult {
            stop_job: Some("Stopped: no Anthropic API key. Set one in Settings.".to_string()),
            ..ItemResult::failed("No API key")
        };
    };
    let unreadable = |e: String| {
        log::warn!("ai: cannot read {}: {e}", path.display());
        ItemResult::failed("Video could not be read")
    };
    let clip = match frames::Clip::open(path, frames::OPEN_TIMEOUT) {
        Ok(clip) => clip,
        Err(e) => return unreadable(e),
    };
    let Some(duration_s) = clip.duration_s() else {
        return unreadable("no duration".to_string());
    };
    if duration_s > MAX_DURATION_S {
        return ItemResult::new(ItemStatus::Skipped, None);
    }
    let frames = match clip.sample(duration_s, cancel) {
        Ok(Some(frames)) if !frames.is_empty() => frames,
        Ok(Some(_)) => return unreadable("no frames".to_string()),
        Ok(None) => return ItemResult::new(ItemStatus::Pending, None),
        Err(e) => return unreadable(e),
    };
    drop(clip);
    let subtitles = frename_core::load_subtitles(path);
    let request =
        describe::build_request(&frames, subtitles.as_ref(), duration_s, options.language);
    let provider = match Anthropic::new(api_key) {
        Ok(provider) => provider,
        Err(e) => return ai_failed(e),
    };
    let response = match provider.complete(&request, cancel) {
        Ok(response) => response,
        Err(AiError::Cancelled) => return ItemResult::new(ItemStatus::Pending, None),
        Err(e) => return ai_failed(e),
    };
    let usage = Some(response.usage);
    let description = match describe::parse_answer(&response, duration_s) {
        Ok(description) => description,
        Err(reason) => {
            return ItemResult {
                usage,
                ..ItemResult::failed(reason)
            }
        }
    };
    let new_block = block::format_block(&description, MODEL.label, &block::today());
    snapshot.set_comment(block::replace_block(snapshot.comment(), &new_block));
    let new_path = FileTagger::save(&snapshot, path);
    log::info!(
        "ai: described {} ({} frames, {} in / {} out tokens)",
        new_path.display(),
        frames.len(),
        response.usage.input_tokens,
        response.usage.output_tokens
    );
    // A cancel that came while the answer was on its way still leaves this video done.
    ItemResult {
        usage,
        ..ItemResult::new(ItemStatus::Done, Some(super::reparsed(new_path)))
    }
}

/// The result of a file whose request failed; some failures stop the whole job, and losing
/// the connection stops it after a few files in a row.
fn ai_failed(error: AiError) -> ItemResult {
    log::warn!("ai: request failed: {error:?}");
    let offline = matches!(error, AiError::Network(_) | AiError::Timeout);
    ItemResult {
        // A request that timed out may have been answered and billed after all.
        usage_unknown: error == AiError::Timeout,
        stop_job: error.stops_job(),
        stop_if_repeated: offline.then(|| {
            "Stopped: no connection to Anthropic. Run it again to describe the rest.".to_string()
        }),
        ..ItemResult::failed(error.reason())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn file(name: &str, comment: &str) -> File {
        let mut file = File::from_path(format!("C:/clips/{name}"), SystemTime::UNIX_EPOCH);
        let mut snapshot = file.snapshot().clone();
        snapshot.set_comment(comment.to_string());
        file.set_file_snapshot(&snapshot);
        file
    }

    fn probe(duration_s: Option<f64>) -> Probe {
        Probe {
            duration_s,
            subtitle_chars: 0,
        }
    }

    const BLOCK: &str = "AI: x\n— Claude Haiku 4.5, 2026-09-26 —";

    #[test]
    fn the_plan_sends_only_readable_short_undescribed_videos() {
        let files = [
            file("a.mp4", ""),
            file("b.mp4", BLOCK),
            file("c.mp4", ""),
            file("d.mp4", ""),
            file("e.jpg", ""),
        ];
        let checked: Vec<&File> = files.iter().collect();
        let mut options = Options::default();
        assert_eq!(options.missing_probes(checked.iter().copied()).len(), 4);
        assert!(
            options.missing_probes(checked.iter().copied()).is_empty(),
            "asked once"
        );
        assert!(options.plan(&checked).estimating);

        options.update(Message::Probed(vec![
            (files[0].id(), probe(Some(60.0))),
            (files[1].id(), probe(Some(60.0))),
            (files[2].id(), probe(Some(3600.0))),
            (files[3].id(), probe(None)),
        ]));
        let plan = options.plan(&checked);
        assert!(!plan.estimating);
        assert_eq!(
            (
                plan.videos,
                plan.described,
                plan.too_long,
                plan.unreadable,
                plan.not_videos
            ),
            (1, 1, 1, 1, 1)
        );
        assert_eq!(
            plan.skipped_line().as_deref(),
            Some("Skipped: 1 already described, 1 over 30 min, 1 photo, 1 unreadable.")
        );
        assert_eq!(plan.no_subtitles, 1);

        assert_eq!(options.files_to_send(&checked), vec![files[0].id()]);
        options.update(Message::SetRedo(true));
        assert_eq!(
            options.plan(&checked).videos,
            2,
            "redo sends described ones"
        );
        assert_eq!(options.files_to_send(&checked).len(), 2);
    }

    #[test]
    fn the_run_button_waits_for_the_estimate_and_the_key() {
        let files = [file("a.mp4", "")];
        let checked: Vec<&File> = files.iter().collect();
        let mut options = Options::default();
        options.update(Message::Probed(vec![(files[0].id(), probe(Some(10.0)))]));
        assert_eq!(
            options.run_button(&checked),
            ("Describe 1 video".to_string(), false),
            "key not read yet"
        );
        options.update(Message::KeyState(KeyState::Missing));
        assert!(!options.run_button(&checked).1);
        options.update(Message::KeyState(KeyState::Saved));
        assert!(options.run_button(&checked).1);
    }

    #[test]
    fn money_and_minutes_read_short() {
        assert_eq!(dollars(0.347), "$0.35");
        assert_eq!(dollars(0.001), "under $0.01");
        assert_eq!(dollars(0.0), "$0.00");
        assert_eq!(minutes(20.0), "< 1 min");
        assert_eq!(minutes(38.0 * 60.0), "38 min");
        assert_eq!(duration_text(20.0), "under a minute");
        assert_eq!(duration_text(25.0 * 60.0), "25 min");
        assert_eq!(duration_text(130.0 * 60.0), "2 h 10 min");
    }

    #[test]
    fn a_photo_is_left_alone() {
        let result = run(
            Run {
                language: SummaryLanguage::English,
                redo: false,
            },
            Path::new("C:/clips/photo.jpg"),
            &AtomicBool::new(false),
        );
        assert_eq!(result.status, ItemStatus::Skipped);
    }

    /// A real request, only when `FRENAME_ANTHROPIC_API_KEY` is set (never in CI without the
    /// secret): one test clip, Claude Haiku 4.5, a summary and segments inside the clip. Prints
    /// the real token usage for the estimate to be checked against.
    #[cfg(target_os = "linux")]
    #[test]
    fn live_description_of_a_test_clip() {
        let Some(api_key) = std::env::var("FRENAME_ANTHROPIC_API_KEY")
            .ok()
            .filter(|k| !k.trim().is_empty())
        else {
            eprintln!("FRENAME_ANTHROPIC_API_KEY not set: live test skipped");
            return;
        };
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/folder/file_example_MP4_480_1_5MG.mp4");
        let clip = frames::Clip::open(&path, std::time::Duration::from_secs(20)).expect("clip");
        let duration_s = clip.duration_s().expect("duration");
        let frames = clip
            .sample(duration_s, &AtomicBool::new(false))
            .expect("frames")
            .expect("not cancelled");
        let request = describe::build_request(&frames, None, duration_s, SummaryLanguage::English);
        let provider = Anthropic::new(api_key.trim().to_string()).expect("client");
        let response = match provider.complete(&request, &AtomicBool::new(false)) {
            Ok(response) => response,
            // The key works but its account cannot pay: nothing about the code to test.
            Err(e @ (AiError::OutOfCredit | AiError::LimitReached(_))) => {
                eprintln!("live test skipped: {}", e.reason());
                return;
            }
            Err(e) => panic!("answer: {e:?}"),
        };
        let description = describe::parse_answer(&response, duration_s).expect("description");
        eprintln!(
            "live: {} frames, usage {:?} (estimated {:?}), cost {}\n{}",
            frames.len(),
            response.usage,
            describe::estimate_usage(duration_s, 0),
            dollars(MODEL.cost_usd(response.usage)),
            block::format_block(&description, MODEL.label, &block::today())
        );
        assert!(!description.summary.is_empty());
        assert!(!description.segments.is_empty());
        assert!(description
            .segments
            .iter()
            .all(|s| s.start_s >= 0.0 && s.end_s <= duration_s));
    }
}
