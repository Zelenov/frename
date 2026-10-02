//! "Describe with AI": for each checked video, key frames (chosen where the picture changes the
//! most) and its subtitles go to Claude, and the summary and time-ranged segments it returns —
//! only what stands out, possibly none for a static or uniform clip — are written into the AI
//! block of the video's comment (see `frename_core::ai::block`). The editor's own text is never
//! touched.
//!
//! Before running, the panel shows what will be sent and about what it costs: clip lengths are
//! read in the background (see [`Options::missing_probes`]) and kept per file.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use clipscribe::{
    self as describe, AiError, AiUsage, FrameSampling, Model, MomentsMode, Stage, SummaryLanguage,
    MAX_DURATION_S,
};
use frename_core::ai::block;
use frename_core::ai::key::{self, KeyState};
use frename_core::{File, FileId, FileKind, FileTagger, FolderInfo, MarkerStorage};
use iced::widget::column;
use iced::Element;

use super::super::page::{self, Change};
use super::super::{ItemProgress, ItemResult, ItemStatus};
use super::{ActionMessage, Panel};
use crate::ui::layout::{self, NoticeKind};
use crate::ui::tokens::SPACE_S;
use crate::ui::{button, form, text};

pub fn label() -> String {
    fl!("batch-action-describe-ai")
}

/// Clip lengths read at once in the background.
const PROBES_AT_ONCE: usize = 4;
/// Clip lengths read before the panel hears of them.
const PROBES_PER_MESSAGE: usize = 16;
/// For the time estimate: one request, and sampling one frame.
const SECONDS_PER_REQUEST: f64 = 15.0;
const SECONDS_PER_FRAME: f64 = 0.3;

#[derive(Debug, Clone)]
pub enum Message {
    /// Redo videos that already have an AI description.
    SetRedo(bool),
    /// Clip lengths read in the background.
    Probed(Vec<(FileId, Probe)>),
    /// Whether an API key is saved, from the settings (which read it).
    KeyState(KeyState),
    /// The language descriptions are written in, from the settings.
    SetLanguage(SummaryLanguage),
    /// The model descriptions are written with, from the settings.
    SetModel(Model),
}

/// What the job does to each file: the options it started with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    pub language: SummaryLanguage,
    pub redo: bool,
    /// The model's id (see [`Model::from_id`]).
    pub model: &'static str,
}

impl Run {
    pub fn model(&self) -> Model {
        Model::from_id(self.model)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    redo: bool,
    language: SummaryLanguage,
    model: Model,
    /// What is known about each checked video, kept while the folder is open.
    probes: HashMap<FileId, Probe>,
    /// Videos whose length is being read.
    probing: HashSet<FileId>,
    /// `None` until read (only when the action is first shown: reading may unlock a keyring).
    key: Option<KeyState>,
    /// The key's state has been asked for; the answer is on its way.
    key_requested: bool,
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
            Message::SetLanguage(language) => self.language = language,
            Message::SetModel(model) => self.model = model,
        }
    }

    pub fn operation(&self) -> super::Operation {
        super::Operation::DescribeAi(Run {
            language: self.language,
            redo: self.redo,
            model: self.model.id,
        })
    }

    /// The model and language set in the settings.
    pub fn model_and_language(&self) -> (Model, SummaryLanguage) {
        (self.model, self.language)
    }

    /// Whether clip lengths are being read (they keep the clips open: no job may rename them).
    pub fn is_probing(&self) -> bool {
        !self.probing.is_empty()
    }

    /// Whether the key's state still has to be read; marks it as asked for.
    pub(in crate::features::batch) fn request_key_state(&mut self) -> bool {
        let needed = self.key.is_none() && !self.key_requested;
        self.key_requested = true;
        needed
    }

    /// Forget what was read about the files of the folder that was open.
    pub fn reset_files(&mut self) {
        self.probes.clear();
        self.probing.clear();
    }

    /// The next checked videos whose length is not known, at most [`PROBES_PER_MESSAGE`] and
    /// only once the last ones are in, so the estimate counts up as they arrive. They are
    /// marked as being read; each answer asks for the next ones.
    pub(in crate::features::batch) fn missing_probes<'a>(
        &mut self,
        checked: impl Iterator<Item = &'a File>,
    ) -> Vec<(FileId, PathBuf)> {
        if !self.probing.is_empty() {
            return Vec::new();
        }
        let missing: Vec<(FileId, PathBuf)> = checked
            .filter(|f| f.kind() == FileKind::Video)
            .filter(|f| !self.probes.contains_key(&f.id()))
            .take(PROBES_PER_MESSAGE)
            .map(|f| (f.id(), f.file_path().to_path_buf()))
            .collect();
        self.probing.extend(missing.iter().map(|(id, _)| *id));
        missing
    }

    /// The videos of `checked` a run will send, in order: the ones its plan counts.
    pub fn files_to_send(&self, checked: &[&File]) -> Vec<FileId> {
        self.plan(checked).send
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
            // The file list only lists videos, so nothing else is ever checked; a file of another
            // kind is left out quietly (it has no length to read, so the estimate would wait
            // for it forever).
            if file.kind() != FileKind::Video {
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
                    plan.send.push(file.id());
                    plan.seconds += d;
                    plan.usage += describe::estimate_usage(self.model, d, probe.subtitle_bytes);
                    plan.run_seconds +=
                        SECONDS_PER_REQUEST + SECONDS_PER_FRAME * describe::frame_count(d) as f64;
                    if !file.has_subtitles() {
                        plan.no_subtitles += 1;
                    }
                }
            }
        }
        plan
    }

    /// Whether the settings said no key is saved: the action list shows "No key".
    pub fn key_missing(&self) -> bool {
        self.key == Some(KeyState::Missing)
    }

    /// The page for `checked` and its run button (`Describe 12 videos · about $0.35` once the
    /// estimate and the key are known). The plan is made once.
    pub fn panel(&self, checked: &[&File]) -> Panel<'_> {
        let plan = self.plan(checked);
        let ready = !plan.estimating && !plan.send.is_empty() && self.key == Some(KeyState::Saved);
        let reason = if plan.estimating {
            Some(fl!("batch-reason-estimate"))
        } else if plan.send.is_empty() && !checked.is_empty() {
            Some(fl!("batch-action-describe-ai-none"))
        } else {
            None
        };
        Panel {
            page: self.view(&plan),
            run: self.run_label(&plan),
            ready,
            reason,
        }
    }

    /// The run button's label: the videos and the cost, or only the verb while estimating.
    fn run_label(&self, plan: &Plan) -> String {
        if plan.estimating {
            fl!("batch-action-describe-ai-run-waiting")
        } else if plan.send.is_empty() {
            label()
        } else {
            fl!(
                "batch-action-describe-ai-run",
                videos = videos(plan.send.len()),
                dollars = dollars(self.model.cost_usd(plan.usage))
            )
        }
    }

    fn view(&self, plan: &Plan) -> Element<'_, ActionMessage> {
        let settings = [
            page::linked_row(
                fl!("settings-ai-model-label"),
                self.model.label,
                fl!("batch-ai-change"),
                ActionMessage::OpenAiSettings,
            ),
            page::linked_row(
                fl!("batch-option-language"),
                language_name(self.language),
                fl!("batch-ai-change"),
                ActionMessage::OpenAiSettings,
            ),
            page::option_row(
                fl!("batch-option-described"),
                layout::choices([
                    form::checkbox(fl!("batch-action-describe-ai-redo"), self.redo)
                        .on_toggle(|redo| ActionMessage::DescribeAi(Message::SetRedo(redo)))
                        .into(),
                ]),
            ),
        ];
        page::page(
            label(),
            fl!("batch-action-describe-ai-hint-panel"),
            &[Change::IntoComments],
            settings
                .into_iter()
                .chain(std::iter::once(self.estimate(plan)))
                .chain(self.key_notice()),
        )
    }

    /// The plan's figures and what is skipped, or how far the estimate is.
    fn estimate(&self, plan: &Plan) -> Element<'_, ActionMessage> {
        if plan.estimating {
            return text::body(fl!(
                "batch-action-describe-ai-estimating",
                known = ((plan.probed + plan.described) as i64),
                total = (plan.checked_videos as i64)
            ))
            .into();
        }
        let mut notes: Vec<String> = Vec::new();
        if !plan.send.is_empty() {
            notes.push(fl!("batch-action-describe-ai-hint"));
        }
        // Also when nothing is sent: it says why.
        notes.extend(plan.skipped_line());
        if plan.no_subtitles > 0 {
            notes.push(fl!(
                "batch-action-describe-ai-no-subtitles",
                videos = videos(plan.no_subtitles)
            ));
        }
        let rows = (!plan.send.is_empty()).then(|| {
            page::plan([
                (fl!("batch-plan-videos"), videos(plan.send.len())),
                (fl!("batch-plan-length"), minutes(plan.seconds)),
                (
                    fl!("batch-plan-cost"),
                    dollars(self.model.cost_usd(plan.usage)),
                ),
                (fl!("batch-plan-time"), duration_text(plan.run_seconds)),
            ])
        });
        column![]
            .push(rows)
            .push(page::notes(notes))
            .spacing(SPACE_S)
            .into()
    }

    /// Why the key keeps the run button off, with the button that fixes it.
    fn key_notice(&self) -> Option<Element<'_, ActionMessage>> {
        let (kind, headline, fix) = match self.key {
            Some(KeyState::Missing) => (
                NoticeKind::Error,
                fl!("batch-ai-key-missing"),
                fl!("batch-set-key"),
            ),
            // Opening the settings reads the key's state again.
            Some(KeyState::Unavailable) => (
                NoticeKind::Warning,
                fl!("batch-ai-key-unavailable"),
                fl!("batch-check-key"),
            ),
            Some(KeyState::Saved) | None => return None,
        };
        Some(layout::notice(
            kind,
            headline,
            None,
            [button::secondary(fix)
                .on_press(ActionMessage::OpenAiSettings)
                .into()],
        ))
    }
}

/// What a run on the checked files would do.
#[derive(Debug, Default, PartialEq)]
struct Plan {
    /// The videos that will be sent, in order; the job runs over exactly these.
    send: Vec<FileId>,
    seconds: f64,
    usage: AiUsage,
    /// About how long the run takes.
    run_seconds: f64,
    described: usize,
    too_long: usize,
    unreadable: usize,
    no_subtitles: usize,
    /// Some lengths or comments are still being read.
    estimating: bool,
    /// Checked videos, and those whose length is known, for the progress of the estimate.
    checked_videos: usize,
    probed: usize,
}

impl Plan {
    /// `Skipped: 3 already described, 1 over 30 min.`
    fn skipped_line(&self) -> Option<String> {
        let parts: Vec<Option<String>> = vec![
            (self.described > 0)
                .then(|| format!("{} {}", self.described, fl!("batch-ai-skip-described"))),
            (self.too_long > 0)
                .then(|| format!("{} {}", self.too_long, fl!("batch-ai-skip-too-long"))),
            (self.unreadable > 0)
                .then(|| format!("{} {}", self.unreadable, fl!("batch-ai-skip-unreadable"))),
        ];
        let parts: Vec<String> = parts.into_iter().flatten().collect();
        (!parts.is_empty()).then(|| fl!("batch-ai-skipped", parts = parts.join(", ")))
    }
}

/// "12 videos" / "1 video".
fn videos(n: usize) -> String {
    fl!("batch-videos-count", n = (n as i64))
}

/// "38 min", or "< 1 min".
fn minutes(seconds: f64) -> String {
    match (seconds / 60.0).round() as u64 {
        0 if seconds > 0.0 => fl!("batch-ai-minutes-under"),
        n => fl!("batch-minutes", n = (n as i64)),
    }
}

/// `language`'s name, translated: used here and in the Settings picker. `SummaryLanguage`'s own
/// `Display` is English only (it is the value stored in the settings), so every place that shows
/// the name to the user goes through this instead.
pub fn language_name(language: SummaryLanguage) -> String {
    match language {
        SummaryLanguage::SameAsSubtitles => fl!("ai-language-same-as-subtitles"),
        SummaryLanguage::English => fl!("ai-language-english"),
        SummaryLanguage::Russian => fl!("ai-language-russian"),
        SummaryLanguage::Ukrainian => fl!("ai-language-ukrainian"),
        SummaryLanguage::German => fl!("ai-language-german"),
        SummaryLanguage::Spanish => fl!("ai-language-spanish"),
        SummaryLanguage::French => fl!("ai-language-french"),
    }
}

/// "25 min", "2 h 10 min", "under a minute".
pub fn duration_text(seconds: f64) -> String {
    let minutes = (seconds / 60.0).round() as u64;
    match minutes {
        0 => fl!("batch-ai-duration-under-minute"),
        m if m < 60 => fl!("batch-minutes", n = (m as i64)),
        m if m % 60 == 0 => fl!("batch-ai-duration-hours", h = ((m / 60) as i64)),
        m => fl!(
            "batch-ai-duration-hours-minutes",
            h = ((m / 60) as i64),
            m = ((m % 60) as i64)
        ),
    }
}

/// "$0.35", "under $0.01".
pub fn dollars(usd: f64) -> String {
    if usd > 0.0 && usd < 0.01 {
        fl!("batch-ai-dollars-under")
    } else {
        format!("${usd:.2}")
    }
}

/// What the estimate needs to know about a clip before it runs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Probe {
    /// Length in seconds; `None` when the clip could not be read in time.
    pub duration_s: Option<f64>,
    /// Size of its `.srt` file in bytes: an upper bound on the subtitle text's length.
    pub subtitle_bytes: usize,
}

/// Read what the estimate needs about the clip at `path`.
fn probe(path: &Path) -> Probe {
    // A debug build renames in memory only: GStreamer needs the name on disk.
    let path = &FileTagger::disk_path(path);
    Probe {
        duration_s: describe::frames::clip_duration_s(path),
        subtitle_bytes: std::fs::metadata(frename_core::subtitle_path(path))
            .map(|m| m.len() as usize)
            .unwrap_or(0),
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
                        .map(|(id, path)| (*id, probe(path)))
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

/// Describe the video at `path` and write the description into its comment, and its moments
/// into the video as markers while markers are kept there (Settings), where Premiere Pro shows
/// them on the clip; with markers kept in comments, into the comment too.
pub fn run(options: Run, path: &Path, cancel: &AtomicBool, progress: &ItemProgress) -> ItemResult {
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
    let Some(api_key) = key::read_key(key::ApiKey::Anthropic) else {
        return ItemResult {
            stop_job: Some(fl!("batch-ai-stop-no-key")),
            ..ItemResult::failed(fl!("batch-ai-fail-no-key"))
        };
    };
    // A debug build renames in memory only: the clip and its subtitles are read by the name on disk.
    let on_disk = FileTagger::disk_path(path);
    let subtitles = describe::srt::load_for(&on_disk).unwrap_or_else(|e| {
        log::warn!("ai: subtitles of {} not read: {e}", path.display());
        Vec::new()
    });
    let describe_options = describe::Options {
        api_key,
        model: options.model(),
        language: options.language,
        frame_sampling: FrameSampling::KeyFrames,
        moments: MomentsMode::Important,
    };
    // The file's share of reading frames, as the estimate counts it; waiting for the answer
    // takes the rest.
    let mut asked_at = 0.0;
    let described =
        describe::describe(
            &on_disk,
            &subtitles,
            &describe_options,
            cancel,
            |stage| match stage {
                Stage::Frame { done, total } => {
                    let sampling_s = SECONDS_PER_FRAME * total as f64;
                    asked_at = (sampling_s / (sampling_s + SECONDS_PER_REQUEST)) as f32;
                    progress.set(
                        asked_at * done as f32 / total.max(1) as f32,
                        fl!(
                            "batch-ai-progress-frame",
                            done = ((done + 1) as i64),
                            total = (total as i64)
                        ),
                    );
                }
                Stage::Asking { .. } => progress.creep(
                    asked_at,
                    0.97,
                    std::time::Duration::from_secs_f64(SECONDS_PER_REQUEST),
                    fl!("batch-ai-progress-waiting"),
                ),
                // A wait before the request is sent again keeps the "waiting" text it has.
                _ => {}
            },
        );
    let described = match described {
        Ok(described) => described,
        Err(describe::Error::Cancelled) => return ItemResult::new(ItemStatus::Pending, None),
        Err(describe::Error::TooLong(_)) => return ItemResult::new(ItemStatus::Skipped, None),
        Err(describe::Error::Unreadable(e)) => {
            log::warn!("ai: cannot read {}: {e}", path.display());
            return ItemResult::failed(fl!("batch-ai-fail-unreadable"));
        }
        Err(describe::Error::Ai(e)) => return ai_failed(e),
        Err(describe::Error::BadAnswer { reason, usage }) => {
            return ItemResult {
                usage: Some(usage),
                ..ItemResult::failed(reason)
            }
        }
    };
    let usage = Some(described.usage);
    progress.set(0.98, fl!("batch-ai-progress-saving"));
    let new_block = match frename_core::marker_storage() {
        MarkerStorage::Comment => block::format_block(&described.description),
        MarkerStorage::InVideo => {
            // Markers first, before the comment's save may rename the file.
            let segments = block::segment_lines(&described.description);
            match FileTagger::save_ai_markers(path, &segments) {
                Ok(()) => block::format_summary_block(&described.description),
                // A format without markers, or a file in use: the moments go into the
                // comment instead, so the paid answer is not lost.
                Err(e) => {
                    log::warn!(
                        "ai: moments of {} not written as markers: {e}",
                        path.display()
                    );
                    block::format_block(&described.description)
                }
            }
        }
    };
    snapshot.set_comment(block::replace_block(snapshot.comment(), &new_block));
    let new_path = FileTagger::save(&snapshot, path);
    log::info!(
        "ai: described {} ({} frames, {} in / {} out tokens)",
        new_path.display(),
        described.frames,
        described.usage.input_tokens,
        described.usage.output_tokens
    );
    // A save that failed (a read-only share) only logs: check the description is there, so a
    // paid answer that was lost is reported, not counted as done.
    let (new_path, saved) = super::reparsed(new_path);
    if block::ai_block(saved.comment()) != Some(new_block.as_str()) {
        return ItemResult {
            usage,
            update: Some((new_path, saved)),
            ..ItemResult::failed(fl!("batch-ai-fail-not-saved"))
        };
    }
    // A cancel that came while the answer was on its way still leaves this video done.
    ItemResult {
        usage,
        ..ItemResult::new(ItemStatus::Done, Some((new_path, saved)))
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
        out_of_credit: matches!(error, AiError::OutOfCredit(_)),
        stop_job: error.stops_job(),
        stop_if_repeated: offline.then(|| fl!("batch-ai-stop-offline")),
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
            subtitle_bytes: 0,
        }
    }

    const BLOCK: &str = "AI: x";

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
            "asked once, and the next ones only after the answer"
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
                plan.send.len(),
                plan.described,
                plan.too_long,
                plan.unreadable,
            ),
            (1, 1, 1, 1),
            "the photo is not counted anywhere"
        );
        assert_eq!(
            plan.skipped_line(),
            Some(fl!(
                "batch-ai-skipped",
                parts = format!(
                    "1 {}, 1 {}, 1 {}",
                    fl!("batch-ai-skip-described"),
                    fl!("batch-ai-skip-too-long"),
                    fl!("batch-ai-skip-unreadable"),
                )
            ))
        );
        assert_eq!(plan.no_subtitles, 1);

        assert_eq!(options.files_to_send(&checked), vec![files[0].id()]);
        options.update(Message::SetRedo(true));
        assert_eq!(
            options.plan(&checked).send.len(),
            2,
            "redo sends described ones"
        );
        assert_eq!(options.files_to_send(&checked).len(), 2);
    }

    fn run_button(options: &Options, checked: &[&File]) -> (String, bool) {
        let panel = options.panel(checked);
        (panel.run, panel.ready)
    }

    #[test]
    fn lengths_are_read_a_few_at_a_time_so_the_estimate_counts_up() {
        let files: Vec<File> = (0..20).map(|i| file(&format!("{i}.mp4"), "")).collect();
        let checked: Vec<&File> = files.iter().collect();
        let mut options = Options::default();
        let first = options.missing_probes(checked.iter().copied());
        assert_eq!(first.len(), PROBES_PER_MESSAGE);
        options.update(Message::Probed(
            first
                .iter()
                .map(|(id, _)| (*id, probe(Some(10.0))))
                .collect(),
        ));
        assert_eq!(options.plan(&checked).probed, PROBES_PER_MESSAGE);
        let rest = options.missing_probes(checked.iter().copied());
        assert_eq!(rest.len(), 20 - PROBES_PER_MESSAGE);
    }

    #[test]
    fn the_run_button_waits_for_the_estimate_and_the_key() {
        let files = [file("a.mp4", "")];
        let checked: Vec<&File> = files.iter().collect();
        let mut options = Options::default();
        options.update(Message::Probed(vec![(files[0].id(), probe(Some(10.0)))]));
        let cost = dollars(options.model.cost_usd(options.plan(&checked).usage));
        assert_eq!(
            run_button(&options, &checked),
            (
                fl!(
                    "batch-action-describe-ai-run",
                    videos = videos(1),
                    dollars = cost
                ),
                false
            ),
            "key not read yet"
        );
        assert!(options.request_key_state(), "asked once");
        assert!(!options.request_key_state());
        options.update(Message::KeyState(KeyState::Missing));
        assert!(!run_button(&options, &checked).1);
        options.update(Message::KeyState(KeyState::Saved));
        assert!(run_button(&options, &checked).1);
    }

    #[test]
    fn money_and_minutes_read_short() {
        assert_eq!(dollars(0.347), "$0.35");
        assert_eq!(dollars(0.001), fl!("batch-ai-dollars-under"));
        assert_eq!(dollars(0.0), "$0.00");
        assert_eq!(minutes(20.0), fl!("batch-ai-minutes-under"));
        assert_eq!(minutes(38.0 * 60.0), fl!("batch-minutes", n = 38));
        assert_eq!(duration_text(20.0), fl!("batch-ai-duration-under-minute"));
        assert_eq!(duration_text(25.0 * 60.0), fl!("batch-minutes", n = 25));
        assert_eq!(
            duration_text(130.0 * 60.0),
            fl!("batch-ai-duration-hours-minutes", h = 2, m = 10)
        );
    }

    #[test]
    fn a_photo_is_left_alone() {
        let result = run(
            Run {
                language: SummaryLanguage::English,
                redo: false,
                model: Model::default().id,
            },
            Path::new("C:/clips/photo.jpg"),
            &AtomicBool::new(false),
            &ItemProgress::default(),
        );
        assert_eq!(result.status, ItemStatus::Skipped);
    }
}
