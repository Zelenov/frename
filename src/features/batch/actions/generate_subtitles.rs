//! "Generate subtitles": sends each checked video's audio to Soniox (through the sonisub
//! library) and writes the subtitles next to it as `clip.srt`, the file frename already shows.
//!
//! Before the run the panel shows a plan with the cost, worked out in the background from the
//! files and their headers only (see [`plan`]); the job then skips what the plan excluded. The
//! key lives in the credential store (Settings → Subtitles); the languages and cue length come
//! from the settings too (see [`Config`]).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use frename_core::ai::key::{self, ApiKey, KeyState};
use frename_core::{subtitle_path, CueLength, File, FileId, FileTagger};
use iced::widget::{button, checkbox, column, row, text};
use iced::{Element, Length};
use sonisub::batch::{self as plan_batch, Action as Planned, Totals};
use sonisub::cancel::CancelToken;
use sonisub::job::{self, Outcome};
use sonisub::soniox::{self, api_error, Client};
use sonisub::{audio, srt, usage};

use super::super::{ItemProgress, ItemResult, ItemStatus};
use super::ActionMessage;
use crate::theme;

pub const LABEL: &str = "Generate subtitles";

/// Read when no key is saved: agent sessions and CI.
const KEY_VAR: &str = "SONIOX_API_KEY";
/// How long the price lookup may take before the typical price is used.
const PRICE_TIMEOUT: Duration = Duration::from_secs(10);
/// What the plan says about files the built-in decoder cannot read while ffmpeg is missing.
const INSTALL_FFMPEG: &str =
    "to read .mkv, .m2ts, .avi …, install ffmpeg from ffmpeg.org, add it to PATH, restart frename";
/// How often a job's file checks whether the batch was cancelled.
const CANCEL_POLL: Duration = Duration::from_millis(100);

/// What the action needs from the settings, sent by the app whenever one of them changes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Config {
    /// Language hints; empty: detect automatically.
    pub languages: Vec<String>,
    pub cue_length: CueLength,
}

/// The transcription price the estimate uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Price {
    /// From the account's usage in the last 30 days.
    Learned(f64),
    /// No usage to learn from, or the lookup failed or took too long.
    Typical,
    /// Soniox refused the key.
    Rejected,
}

impl Price {
    fn usd_per_hour(self) -> f64 {
        match self {
            Self::Learned(price) => price,
            Self::Typical | Self::Rejected => usage::FALLBACK_USD_PER_HOUR,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// The settings the action uses changed (or were read at start).
    SetConfig(Config),
    /// Transcribe again videos that already have subtitles.
    SetReplace(bool),
    /// Whether a Soniox key is saved, from the settings (which read it).
    KeyState(KeyState),
    /// A Soniox key was saved: look its price up again, even for the same key (it may have
    /// been refused before and fixed on the account since).
    KeySaved,
    /// The plan of the checked files, worked out in the background; `generation` ties it to
    /// the checks it was made for.
    PlanReady { generation: u64, plan: Box<Plan> },
    /// The price, looked up in the background.
    PriceReady(Price),
}

impl Message {
    /// Whether it may change the options while a job runs: answers of background reads and
    /// settings, which the next run needs.
    pub fn applies_while_running(&self) -> bool {
        !matches!(self, Self::SetReplace(_))
    }
}

/// What the checked files need, from their names and headers only.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Plan {
    /// Videos to send to Soniox.
    pub transcribe: usize,
    /// Seconds of audio to send, of the videos whose length is known.
    pub audio_s: f64,
    /// Videos to send whose length is unknown.
    pub unknown_length: usize,
    pub already_subtitled: usize,
    /// Subtitles rebuilt from a transcript saved next to the video: free.
    pub rebuilt_free: usize,
    /// Checked before and found to have no speech.
    pub no_speech_before: usize,
    pub no_audio: usize,
    /// Videos whose `.srt` name another checked video already takes.
    pub shared_name: usize,
    /// Videos the built-in decoder cannot read while ffmpeg is not installed.
    pub unreadable: usize,
    /// Files the job must not send, with the reason shown for them.
    pub excluded: HashMap<PathBuf, &'static str>,
}

impl Plan {
    /// Whether running it would do anything at all.
    fn has_work(&self) -> bool {
        self.transcribe + self.rebuilt_free > 0
    }
}

/// What the workspace should work out in the background for the panel.
#[derive(Debug, Default, PartialEq)]
pub struct Reads {
    /// The plan of these files (list order), with Replace on or off, for this generation.
    pub plan: Option<(u64, Vec<PathBuf>, bool)>,
    /// The price of the saved key.
    pub price: bool,
    /// Whether a key is saved.
    pub key_state: bool,
}

/// The checked files a plan was made for, and with Replace on or off.
type PlannedFor = (Vec<(FileId, PathBuf)>, bool);

#[derive(Debug, Clone, Default)]
pub struct Options {
    config: Config,
    /// "Replace existing subtitles".
    replace: bool,
    /// `None` until read (only when the action is first shown: reading may unlock a keyring).
    key: Option<KeyState>,
    key_requested: bool,
    /// Bumped with every new plan asked for; an answer for an older one is dropped.
    generation: u64,
    /// What the last plan asked for was made for.
    planned_for: Option<PlannedFor>,
    /// The plan of the current generation, once worked out.
    plan: Option<Plan>,
    /// The price: `None` until asked for, `Some(None)` while it is looked up.
    price: Option<Option<Price>>,
}

impl Options {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetConfig(config) => self.config = config,
            Message::SetReplace(replace) => self.replace = replace,
            Message::KeyState(state) => {
                if state != KeyState::Saved {
                    self.price = None;
                }
                self.key = Some(state);
            }
            Message::KeySaved => self.price = None,
            Message::PlanReady { generation, plan } => {
                if generation == self.generation {
                    self.plan = Some(*plan);
                }
            }
            Message::PriceReady(price) => {
                if self.price.is_some() {
                    self.price = Some(Some(price));
                }
            }
        }
    }

    /// The files changed on disk (a job ran, the folder changed, batch mode was left): the plan
    /// is made again when next asked for.
    pub fn invalidate_plan(&mut self) {
        self.planned_for = None;
        self.plan = None;
    }

    /// What to work out for `checked` (in list order); each is asked for once per change.
    pub(in crate::features::batch) fn reads(&mut self, checked: &[&File]) -> Reads {
        let planned_for: PlannedFor = (
            checked
                .iter()
                .map(|f| (f.id(), f.file_path().to_path_buf()))
                .collect(),
            self.replace,
        );
        let plan = (self.planned_for.as_ref() != Some(&planned_for)).then(|| {
            self.generation += 1;
            self.plan = None;
            let files = checked
                .iter()
                .map(|f| FileTagger::disk_path(f.file_path()))
                .collect();
            self.planned_for = Some(planned_for);
            (self.generation, files, self.replace)
        });
        let price = self.key == Some(KeyState::Saved) && self.price.is_none();
        if price {
            self.price = Some(None);
        }
        let key_state = self.key.is_none() && !self.key_requested;
        self.key_requested = true;
        Reads {
            plan,
            price,
            key_state,
        }
    }

    /// The price of the saved key, once looked up.
    fn price(&self) -> Option<Price> {
        self.price.flatten()
    }

    /// The plan, when it was made for exactly `checked`.
    fn plan_for(&self, checked: &[&File]) -> Option<&Plan> {
        let (files, replace) = self.planned_for.as_ref()?;
        let same = *replace == self.replace
            && files.len() == checked.len()
            && files.iter().zip(checked).all(|((id, _), f)| *id == f.id());
        self.plan.as_ref().filter(|_| same)
    }

    /// The job, when the plan is known, the key accepted and there is something to do.
    pub fn operation(&self) -> Option<super::Operation> {
        let plan = self.plan.as_ref().filter(|p| p.has_work())?;
        if self.key != Some(KeyState::Saved) {
            return None;
        }
        // Free rebuilds need no price; a transcription waits for the lookup, which also tells
        // whether Soniox accepts the key.
        let price = match self.price() {
            Some(Price::Rejected) => return None,
            Some(price) => price,
            None if plan.transcribe > 0 => return None,
            None => Price::Typical,
        };
        let options = job::Options {
            languages: self.config.languages.clone(),
            layout: match self.config.cue_length {
                CueLength::Short => srt::Layout::default(),
                CueLength::Sentence => srt::Layout::unlimited(),
            },
            force: self.replace,
            keep_json: false,
            audio: audio::Backend::Auto,
            reference: format!(
                "frename-{}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ),
            // sonisub draws progress bars; frename has no console to draw them on.
            progress: Some(indicatif::MultiProgress::with_draw_target(
                indicatif::ProgressDrawTarget::hidden(),
            )),
            ..job::Options::default()
        };
        Some(super::Operation::GenerateSubtitles(Run(Arc::new(
            SubtitleJob {
                options,
                excluded: plan.excluded.clone(),
                usd_per_hour: price.usd_per_hour(),
                uploaded_ms: AtomicU64::new(0),
                failed_deletes: AtomicUsize::new(0),
            },
        ))))
    }

    /// The panel for `checked`, and the run button's label and whether it can run.
    pub fn panel(&self, checked: &[&File]) -> (Element<'_, ActionMessage>, String, bool) {
        let plan = self.plan_for(checked);
        let (label, ready) = match plan {
            None => ("Transcribe".to_string(), false),
            Some(plan) if plan.transcribe > 0 => (
                format!("Transcribe {}", count(plan.transcribe, "video", "videos")),
                self.operation().is_some(),
            ),
            Some(plan) if plan.rebuilt_free > 0 => (
                format!(
                    "Build {} (free)",
                    count(plan.rebuilt_free, "subtitle", "subtitles")
                ),
                self.operation().is_some(),
            ),
            Some(_) => ("Nothing to transcribe".to_string(), false),
        };
        (self.view(plan), label, ready)
    }

    fn view(&self, plan: Option<&Plan>) -> Element<'_, ActionMessage> {
        let muted = |line: &'static str| text(line).size(12).color(theme::TEXT_MUTED);
        let mut lines = column![].spacing(4);
        match plan {
            None => lines = lines.push(text("Estimating…").size(13)),
            Some(plan) => {
                // Without a saved key there is no account to learn the price from.
                let price = match self.key {
                    Some(KeyState::Saved) => self.price(),
                    _ => Some(Price::Typical),
                };
                for line in plan_lines(plan, price) {
                    lines = lines.push(text(line).size(13));
                }
            }
        }
        let options = column![
            lines,
            column![
                checkbox(self.replace)
                    .label("Replace existing subtitles")
                    .text_size(13)
                    .on_toggle(|on| ActionMessage::GenerateSubtitles(Message::SetReplace(on))),
                muted("Transcribes again; costs as shown."),
            ]
            .spacing(4),
            column![
                muted("The audio of these videos is sent to Soniox and deleted there afterwards."),
                muted(
                    "Takes a few minutes per hour of audio; the folder is locked until it ends. \
                     Closing frename stops the run; finished subtitles are kept."
                ),
            ]
            .spacing(4),
        ]
        .spacing(12);
        super::panel(
            LABEL,
            "Transcribes the speech of each checked video with Soniox and saves the subtitles \
             next to it (clip.srt), where frename shows them."
                .to_string(),
            options.into(),
        )
    }

    /// Why the key keeps the run button off, shown next to it so it is never scrolled away.
    pub fn footer(&self) -> Option<Element<'_, ActionMessage>> {
        let line = match (self.key, self.price()) {
            (Some(KeyState::Missing), _) => "Set a Soniox API key in Settings",
            (Some(KeyState::Unavailable), _) => {
                "The system keyring could not be opened: it may be locked, or there is none \
                 (such as GNOME Keyring or KWallet)."
            }
            (Some(KeyState::Saved), Some(Price::Rejected)) => "Soniox rejected the key",
            _ => return None,
        };
        Some(
            row![
                text(line).size(13).color(theme::ERROR).width(Length::Fill),
                button(text("Open Settings").size(12))
                    .on_press(ActionMessage::OpenSubtitleSettings)
                    .padding([3, 10])
                    .style(theme::icon_button_style(true)),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .into(),
        )
    }
}

/// The plan as the panel shows it: what is sent and what it costs, then one line per kind of
/// file that is not sent.
fn plan_lines(plan: &Plan, price: Option<Price>) -> Vec<String> {
    let mut lines = Vec::new();
    if plan.transcribe > 0 {
        let cost = match price {
            _ if plan.audio_s == 0.0 => "cost unknown".to_string(),
            None => "Estimating…".to_string(),
            Some(price) => {
                let usd = price.usd_per_hour() * plan.audio_s / 3600.0;
                match price {
                    Price::Learned(_) => usd_text(usd),
                    Price::Typical | Price::Rejected => {
                        format!("{} (typical price)", usd_text(usd))
                    }
                }
            }
        };
        let unknown = if plan.unknown_length > 0 {
            format!(" + {} of unknown length", plan.unknown_length)
        } else {
            String::new()
        };
        lines.push(format!(
            "{} to transcribe, {} of audio{unknown} · {cost}",
            count(plan.transcribe, "video", "videos"),
            duration_text(plan.audio_s),
        ));
    }
    let not_sent = [
        (
            plan.already_subtitled,
            "already has subtitles",
            "already have subtitles",
        ),
        (
            plan.rebuilt_free,
            "rebuilt free from a saved transcript",
            "rebuilt free from a saved transcript",
        ),
        (
            plan.no_speech_before,
            "had no speech last time",
            "had no speech last time",
        ),
        (plan.no_audio, "has no audio", "have no audio"),
        (
            plan.shared_name,
            "shares its subtitle name with another video",
            "share their subtitle name with another video",
        ),
    ];
    for (n, one, many) in not_sent {
        if n > 0 {
            lines.push(format!("{n} {}", if n == 1 { one } else { many }));
        }
    }
    if plan.unreadable > 0 {
        lines.push(format!(
            "{} cannot be read ({INSTALL_FFMPEG})",
            plan.unreadable
        ));
    }
    if !plan.has_work() {
        lines.push("Nothing to transcribe".to_string());
    }
    lines
}

/// Work out what each of `files` (on-disk paths, list order) needs. Blocking: reads file
/// headers. Of several videos whose subtitles would have the same name, the first keeps it.
pub fn plan(files: &[PathBuf], replace: bool) -> Plan {
    plan_with(files, replace, ffmpeg_installed())
}

fn plan_with(files: &[PathBuf], replace: bool, ffmpeg: bool) -> Plan {
    let mut plan = Plan::default();
    let mut names = HashSet::new();
    let mut items = Vec::new();
    for file in files {
        let output = subtitle_path(file);
        // Case-insensitive: clip.MP4 and clip.mov both write clip.srt on Windows.
        if !names.insert(output.to_string_lossy().to_lowercase()) {
            plan.shared_name += 1;
            plan.excluded
                .insert(file.clone(), "shares its subtitle name with another video");
            continue;
        }
        items.push(plan_batch::Item {
            input: file.clone(),
            output,
            name: String::new(),
        });
    }
    let options = job::Options {
        force: replace,
        ..job::Options::default()
    };
    let mut to_send = Vec::new();
    for planned in plan_batch::plan(&items, &options) {
        let unreadable = matches!(planned.action, Planned::Transcribe { audio_s: None })
            && !ffmpeg
            && audio::probe(&planned.item.input).is_err();
        if unreadable {
            plan.unreadable += 1;
            plan.excluded
                .insert(planned.item.input.clone(), "audio format not supported");
        } else {
            to_send.push(planned);
        }
    }
    let totals = Totals::of(&to_send);
    plan.transcribe = totals.transcribe;
    plan.audio_s = totals.audio_s;
    plan.unknown_length = totals.unknown_length;
    plan.already_subtitled = totals.skip;
    plan.rebuilt_free = totals.cached + totals.from_transcript;
    plan.no_speech_before = totals.cached_silent;
    plan.no_audio = totals.no_audio;
    plan
}

/// Whether `ffmpeg` is on `PATH`, for the formats the built-in decoder cannot read.
fn ffmpeg_installed() -> bool {
    let exe = if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(exe).is_file()))
}

/// The saved Soniox key, or the `SONIOX_API_KEY` environment variable when none is saved.
/// Blocking (may unlock a keyring).
fn soniox_key() -> Option<String> {
    key::read_key(ApiKey::Soniox).or_else(|| {
        std::env::var(KEY_VAR)
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
    })
}

/// Look up the transcription price from the account's usage in the last 30 days (blocking,
/// at most [`PRICE_TIMEOUT`]). A refused key is reported; any other failure gives the typical
/// price.
pub fn price() -> Price {
    let Some(key) = soniox_key() else {
        return Price::Typical;
    };
    let (sender, receiver) = std::sync::mpsc::channel();
    // Its own thread, so a slow lookup can be given up; the client is created and dropped
    // there (a blocking HTTP client must not be dropped on an async runtime thread).
    std::thread::spawn(move || {
        let price = (|| {
            let client = Client::new(soniox::DEFAULT_BASE, &key)?;
            let now = SystemTime::now();
            let logs = usage::fetch(&client, now - Duration::from_secs(30 * 86_400), now)?;
            anyhow::Ok(usage::Summary::of(&logs).stt_usd_per_hour())
        })();
        let _ = sender.send(price);
    });
    match receiver.recv_timeout(PRICE_TIMEOUT) {
        Ok(Ok(Some(price))) => Price::Learned(price),
        Ok(Ok(None)) => Price::Typical,
        Ok(Err(e)) if api_error(&e).is_some_and(|a| a.status == Some(401)) => {
            log::warn!("subtitles: Soniox rejected the key: {e:#}");
            Price::Rejected
        }
        Ok(Err(e)) => {
            log::warn!("subtitles: price lookup failed, using the typical price: {e:#}");
            Price::Typical
        }
        Err(_) => {
            log::warn!("subtitles: price lookup took too long, using the typical price");
            Price::Typical
        }
    }
}

/// What the job does to each file: the options it started with, and what it sent so far.
/// Operations compare equal only when they are the same run.
#[derive(Debug, Clone)]
pub struct Run(Arc<SubtitleJob>);

impl PartialEq for Run {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Run {}

impl Run {
    /// What the run spent, and uploads left on Soniox; `None` when nothing was sent.
    pub fn report(&self) -> Option<String> {
        self.0.report()
    }
}

/// One run of the action.
pub struct SubtitleJob {
    options: job::Options,
    /// Files the plan excluded, with their reason.
    excluded: HashMap<PathBuf, &'static str>,
    /// The price the estimate used, for what the run spent.
    usd_per_hour: f64,
    /// Audio sent to Soniox by the files that finished.
    uploaded_ms: AtomicU64,
    /// Uploads that could not be deleted on Soniox afterwards.
    failed_deletes: AtomicUsize,
}

impl std::fmt::Debug for SubtitleJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubtitleJob")
            .field("languages", &self.options.languages)
            .field("force", &self.options.force)
            .field("excluded", &self.excluded.len())
            .finish_non_exhaustive()
    }
}

impl SubtitleJob {
    fn report(&self) -> Option<String> {
        let seconds = self.uploaded_ms.load(Ordering::Relaxed) as f64 / 1000.0;
        let failed_deletes = self.failed_deletes.load(Ordering::Relaxed);
        let mut parts = Vec::new();
        if seconds > 0.0 {
            parts.push(format!(
                "Soniox: at least {} · {}",
                duration_text(seconds),
                usd_text(self.usd_per_hour * seconds / 3600.0)
            ));
        }
        if failed_deletes > 0 {
            parts.push(format!(
                "{} could not be deleted on Soniox (see the log)",
                count(failed_deletes, "upload", "uploads")
            ));
        }
        (!parts.is_empty()).then(|| parts.join(". "))
    }
}

/// Generate the subtitles of the video at `path`. Blocking: runs on the batch's worker
/// thread, where the Soniox client is created and dropped. `cancel` (the batch's Cancel) stops
/// the file in work within a fraction of a second; it then counts as not reached.
pub fn run(job: &Run, path: &Path, cancel: &AtomicBool, progress: &ItemProgress) -> ItemResult {
    let job = &job.0;
    // A debug build renames in memory only: the clip is on disk under its old name.
    let path = FileTagger::disk_path(path);
    if let Some(reason) = job.excluded.get(&path) {
        log::info!("subtitles: {} skipped: {reason}", path.display());
        return with_reason(ItemStatus::Skipped, reason);
    }
    if cancel.load(Ordering::Relaxed) {
        return ItemResult::new(ItemStatus::Pending, None);
    }
    progress.set(0.0, "transcribing");
    let token = CancelToken::new();
    let options = job::Options {
        cancel: token.clone(),
        ..job.options.clone()
    };
    let finished = AtomicBool::new(false);
    let result = std::thread::scope(|scope| {
        // The batch cancels with its own flag; sonisub listens to its token.
        scope.spawn(|| {
            while !finished.load(Ordering::Relaxed) {
                if cancel.load(Ordering::Relaxed) {
                    token.cancel();
                    return;
                }
                std::thread::sleep(CANCEL_POLL);
            }
        });
        let result = transcribe(job, &path, &options, &token);
        finished.store(true, Ordering::Relaxed);
        result
    });
    if let Some(seconds) = result.as_ref().ok().and_then(Outcome::uploaded_s) {
        job.uploaded_ms
            .fetch_add((seconds * 1000.0) as u64, Ordering::Relaxed);
    }
    item_result(&path, result, token.is_cancelled())
}

/// One file through sonisub. A saved transcript next to it needs no key.
fn transcribe(
    job: &SubtitleJob,
    path: &Path,
    options: &job::Options,
    token: &CancelToken,
) -> anyhow::Result<Outcome> {
    let output = subtitle_path(path);
    let Some(key) = soniox_key() else {
        return job::process(path, &output, options, None);
    };
    let client = Client::new(soniox::DEFAULT_BASE, &key)?.with_cancel(token.clone());
    let result = job::process(path, &output, options, Some(&client));
    job.failed_deletes
        .fetch_add(client.failed_deletes(), Ordering::Relaxed);
    result
}

/// The job's record of one file. A file whose work was cancelled is not reached, not failed.
fn item_result(path: &Path, result: anyhow::Result<Outcome>, cancelled: bool) -> ItemResult {
    let e = match result {
        Ok(outcome) => {
            log::info!("subtitles: {}: {outcome:?}", path.display());
            return match outcome {
                Outcome::Written { .. } => ItemResult::new(ItemStatus::Done, None),
                Outcome::Skipped { .. } => {
                    with_reason(ItemStatus::Skipped, "already had subtitles")
                }
                Outcome::NoAudio { .. } => with_reason(ItemStatus::Skipped, "no audio"),
                Outcome::NoSpeech { .. } => with_reason(ItemStatus::Skipped, "no speech"),
            };
        }
        Err(e) => e,
    };
    if cancelled {
        log::info!("subtitles: {} cancelled: {e:#}", path.display());
        return ItemResult::new(ItemStatus::Pending, None);
    }
    log::warn!("subtitles: {} failed: {e:#}", path.display());
    let reason = failure_reason(&e);
    let mut result = ItemResult::failed(reason.clone());
    if let Some(api) = api_error(&e).filter(|api| api.is_fatal()) {
        let key_rejected = matches!(api.status, Some(401))
            || api.error_type == "unauthenticated"
            || api.error_type == "permission_denied";
        result.stop_job = Some(match api.error_type.as_str() {
            _ if key_rejected => {
                "Stopped: Soniox rejected the key. Check it in Settings → Subtitles.".to_string()
            }
            "organization_balance_exhausted" => {
                "Stopped: the Soniox balance is empty. Top it up at console.soniox.com.".to_string()
            }
            t if t.ends_with("budget_exhausted") => {
                "Stopped: the Soniox monthly budget is used up. Raise it at console.soniox.com."
                    .to_string()
            }
            t => format!("Stopped: Soniox refused to go on ({t})."),
        });
    } else if reason == CANNOT_REACH {
        result.stop_if_repeated =
            Some("Stopped: Soniox cannot be reached. Check the internet connection.".to_string());
    }
    result
}

const CANNOT_REACH: &str = "cannot reach Soniox";

/// Why a file failed, short enough for the result list; the log has the whole error.
fn failure_reason(e: &anyhow::Error) -> String {
    if let Some(api) = api_error(e) {
        return format!("Soniox: {}", api.message);
    }
    let chain = format!("{e:#}");
    // sonisub's context for a request that got no answer (network, proxy, timeout).
    if chain.contains("request to Soniox failed") {
        return CANNOT_REACH.to_string();
    }
    if [
        "unsupported container",
        "unsupported audio codec",
        "ffmpeg is not on PATH",
    ]
    .iter()
    .any(|m| chain.contains(m))
    {
        return "audio format not supported".to_string();
    }
    e.root_cause().to_string().chars().take(120).collect()
}

fn with_reason(status: ItemStatus, reason: &str) -> ItemResult {
    ItemResult {
        reason: Some(reason.to_string()),
        ..ItemResult::new(status, None)
    }
}

/// "1 video" / "8 videos".
fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// "41 min", "1 h 5 min", "20 s".
fn duration_text(seconds: f64) -> String {
    let s = seconds.round() as u64;
    match s {
        0..60 => format!("{s} s"),
        60..3600 => format!("{} min", (s + 30) / 60),
        _ => format!("{} h {} min", s / 3600, s / 60 % 60),
    }
}

/// "about $0.07"; small amounts are not shown as zero.
fn usd_text(usd: f64) -> String {
    if usd > 0.0 && usd < 0.005 {
        "less than $0.01".to_string()
    } else {
        format!("about ${usd:.2}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSCRIPT: &str =
        include_str!("../../../../tests/fixtures/subtitles/dialog.soniox.json");

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/subtitles")
            .join(name)
    }

    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("frename-subtitles-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("temp dir");
            Self(dir)
        }

        /// A file with `content`; not a readable video unless copied from the fixture.
        fn file(&self, name: &str, content: &str) -> PathBuf {
            let path = self.0.join(name);
            std::fs::write(&path, content).expect("write");
            path
        }

        fn video(&self, name: &str) -> PathBuf {
            let path = self.0.join(name);
            std::fs::copy(fixture("dialog.mp4"), &path).expect("copy the video");
            path
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn job(excluded: HashMap<PathBuf, &'static str>, force: bool) -> Run {
        Run(Arc::new(SubtitleJob {
            options: job::Options {
                force,
                ..job::Options::default()
            },
            excluded,
            usd_per_hour: 0.12,
            uploaded_ms: AtomicU64::new(0),
            failed_deletes: AtomicUsize::new(0),
        }))
    }

    fn run_now(job: &Run, path: &Path) -> ItemResult {
        run(job, path, &AtomicBool::new(false), &ItemProgress::default())
    }

    #[test]
    fn a_saved_transcript_becomes_subtitles_without_calling_soniox() {
        let folder = Folder::new("cached");
        let video = folder.video("clip.mp4");
        folder.file("clip.soniox.json", TRANSCRIPT);

        let result = run_now(&job(HashMap::new(), false), &video);
        assert_eq!(result.status, ItemStatus::Done, "{:?}", result.reason);
        let srt = std::fs::read_to_string(subtitle_path(&video)).expect("clip.srt written");
        assert!(
            frename_core::Subtitles::parse(&srt).cues().len() > 3,
            "{srt}"
        );
        assert!(srt.to_lowercase().contains("subtitle generator"), "{srt}");
    }

    #[test]
    fn the_plan_counts_every_kind_of_file() {
        let folder = Folder::new("plan");
        let subtitled = folder.file("done.mp4", "x");
        folder.file("done.srt", "1\n00:00:01,000 --> 00:00:02,000\nHi\n");
        let cached = folder.file("cached.mp4", "x");
        folder.file("cached.soniox.json", TRANSCRIPT);
        let silent = folder.file("silent.mp4", "x");
        folder.file("silent.soniox.json", r#"{"text": "", "tokens": []}"#);
        let first = folder.file("pair.MP4", "x");
        let second = folder.file("pair.mov", "x");
        let files = vec![subtitled, cached, silent, first, second.clone()];

        let plan = plan_with(&files, false, false);
        assert_eq!(plan.already_subtitled, 1);
        assert_eq!(plan.rebuilt_free, 1);
        assert_eq!(plan.no_speech_before, 1);
        assert_eq!(plan.shared_name, 1, "pair.mov maps to pair.srt too");
        assert_eq!(
            plan.excluded.get(&second),
            Some(&"shares its subtitle name with another video")
        );
        assert_eq!(plan.unreadable, 1, "pair.MP4 has no header and no ffmpeg");
        assert_eq!(plan.transcribe, 0);
        assert!(plan.has_work(), "the free rebuild");

        // With ffmpeg there, an unreadable header is left to the real run: sent, length unknown.
        let plan = plan_with(&files, false, true);
        assert_eq!(
            (plan.unreadable, plan.transcribe, plan.unknown_length),
            (0, 1, 1)
        );

        // Replace ignores existing subtitles and saved transcripts, but not shared names.
        let plan = plan_with(&files, true, true);
        assert_eq!(plan.already_subtitled + plan.rebuilt_free, 0);
        assert_eq!(plan.transcribe, 4);
        assert_eq!(plan.shared_name, 1);
    }

    #[test]
    fn excluded_files_are_skipped_with_their_reason_even_with_replace() {
        let folder = Folder::new("excluded");
        let video = folder.file("clip.mov", "x");
        let excluded =
            HashMap::from([(video.clone(), "shares its subtitle name with another video")]);
        let result = run_now(&job(excluded, true), &video);
        assert_eq!(result.status, ItemStatus::Skipped);
        assert_eq!(
            result.reason.as_deref(),
            Some("shares its subtitle name with another video")
        );
    }

    #[test]
    fn outcomes_map_to_statuses_and_reasons() {
        let path = Path::new("clip.mp4");
        let status = |outcome| {
            let result = item_result(path, Ok(outcome), false);
            (result.status, result.reason)
        };
        assert_eq!(
            status(Outcome::NoSpeech {
                marker: None,
                cached: false,
                uploaded_s: Some(3.0)
            }),
            (ItemStatus::Skipped, Some("no speech".to_string()))
        );
        assert_eq!(
            status(Outcome::NoAudio {
                reason: "no audio track".into()
            }),
            (ItemStatus::Skipped, Some("no audio".to_string()))
        );
        assert_eq!(
            status(Outcome::Skipped {
                srt: PathBuf::from("clip.srt")
            }),
            (
                ItemStatus::Skipped,
                Some("already had subtitles".to_string())
            )
        );
    }

    fn soniox_error(status: u16, error_type: &str) -> anyhow::Error {
        soniox::ApiError {
            status: Some(status),
            error_type: error_type.to_string(),
            message: "message from Soniox".to_string(),
            request_id: None,
        }
        .into()
    }

    #[test]
    fn a_fatal_soniox_error_stops_the_job_with_a_short_message() {
        let path = Path::new("clip.mp4");
        let rejected = item_result(path, Err(soniox_error(401, "unauthenticated")), false);
        assert_eq!(rejected.status, ItemStatus::Failed);
        assert!(rejected
            .stop_job
            .as_deref()
            .is_some_and(|s| s.contains("Soniox rejected the key")));

        let empty = item_result(
            path,
            Err(soniox_error(402, "organization_balance_exhausted")),
            false,
        );
        assert!(empty
            .stop_job
            .as_deref()
            .is_some_and(|s| s.contains("balance is empty")));

        let ordinary = item_result(path, Err(soniox_error(400, "invalid_audio_file")), false);
        assert!(ordinary.stop_job.is_none());
        assert_eq!(
            ordinary.reason.as_deref(),
            Some("Soniox: message from Soniox")
        );
    }

    #[test]
    fn network_failures_stop_the_job_once_they_repeat() {
        let e = anyhow::anyhow!("error sending request").context("request to Soniox failed");
        let result = item_result(Path::new("clip.mp4"), Err(e), false);
        assert_eq!(result.reason.as_deref(), Some(CANNOT_REACH));
        assert!(result.stop_job.is_none());
        assert!(result.stop_if_repeated.is_some());
        assert_eq!(
            failure_reason(&anyhow::anyhow!("unsupported container")),
            "audio format not supported"
        );
    }

    #[test]
    fn a_cancelled_file_is_not_reached_not_failed() {
        let result = item_result(
            Path::new("clip.mp4"),
            Err(soniox_error(401, "unauthenticated")),
            true,
        );
        assert_eq!(result.status, ItemStatus::Pending);
        assert!(result.stop_job.is_none());

        let folder = Folder::new("cancelled");
        let video = folder.video("clip.mp4");
        folder.file("clip.soniox.json", TRANSCRIPT);
        let result = run(
            &job(HashMap::new(), false),
            &video,
            &AtomicBool::new(true),
            &ItemProgress::default(),
        );
        assert_eq!(result.status, ItemStatus::Pending);
        assert!(!subtitle_path(&video).exists());
        // The next job is not stopped by the cancelled one.
        assert_eq!(
            run_now(&job(HashMap::new(), false), &video).status,
            ItemStatus::Done
        );
    }

    #[test]
    fn the_report_says_what_was_sent_at_least() {
        let job = job(HashMap::new(), false);
        assert_eq!(job.report(), None);
        job.0.uploaded_ms.store(41 * 60 * 1000, Ordering::Relaxed);
        job.0.failed_deletes.store(1, Ordering::Relaxed);
        assert_eq!(
            job.report().as_deref(),
            Some(
                "Soniox: at least 41 min · about $0.08. 1 upload could not be deleted on Soniox \
                 (see the log)"
            )
        );
    }

    #[test]
    fn the_plan_reads_in_plain_words() {
        let plan = Plan {
            transcribe: 8,
            audio_s: 41.0 * 60.0,
            unknown_length: 2,
            already_subtitled: 3,
            rebuilt_free: 2,
            shared_name: 1,
            ..Plan::default()
        };
        assert_eq!(
            plan_lines(&plan, Some(Price::Typical)),
            [
                "8 videos to transcribe, 41 min of audio + 2 of unknown length · about $0.07 (typical price)",
                "3 already have subtitles",
                "2 rebuilt free from a saved transcript",
                "1 shares its subtitle name with another video",
            ]
        );
        assert_eq!(
            plan_lines(&Plan::default(), None),
            ["Nothing to transcribe"]
        );
        let unknown = Plan {
            transcribe: 2,
            unknown_length: 2,
            unreadable: 1,
            ..Plan::default()
        };
        assert_eq!(
            plan_lines(&unknown, Some(Price::Typical)),
            [
                "2 videos to transcribe, 0 s of audio + 2 of unknown length · cost unknown",
                "1 cannot be read (to read .mkv, .m2ts, .avi …, install ffmpeg from ffmpeg.org, \
                 add it to PATH, restart frename)",
            ]
        );
    }

    fn file(name: &str) -> File {
        File::from_path(format!("C:/none/{name}"), SystemTime::UNIX_EPOCH)
    }

    #[test]
    fn reads_are_asked_for_once_per_change_and_old_answers_are_dropped() {
        let (a, b) = (file("a.mp4"), file("b.mp4"));
        let mut options = Options::default();
        let reads = options.reads(&[&a, &b]);
        assert!(reads.key_state);
        let (generation, files, replace) = reads.plan.expect("a plan");
        assert_eq!((files.len(), replace), (2, false));
        assert_eq!(
            options.reads(&[&a, &b]),
            Reads::default(),
            "nothing changed"
        );

        // The checks change: a new plan; the answer for the old one is dropped.
        let newer = options.reads(&[&a]).plan.expect("a new plan").0;
        options.update(Message::PlanReady {
            generation,
            plan: Box::new(Plan::default()),
        });
        assert!(options.plan_for(&[&a]).is_none(), "stale answer dropped");
        options.update(Message::PlanReady {
            generation: newer,
            plan: Box::new(Plan {
                transcribe: 1,
                ..Plan::default()
            }),
        });
        assert!(options.plan_for(&[&a]).is_some());
        assert!(
            options.plan_for(&[&a, &b]).is_none(),
            "not for other checks"
        );

        // The price once the key is known to be saved, once.
        options.update(Message::KeyState(KeyState::Saved));
        assert!(options.reads(&[&a]).price);
        assert!(!options.reads(&[&a]).price);
        assert!(options.operation().is_none(), "waits for the price");
        options.update(Message::PriceReady(Price::Learned(0.1)));
        assert!(options.operation().is_some());
        let (_, label, ready) = options.panel(&[&a]);
        assert_eq!((label.as_str(), ready), ("Transcribe 1 video", true));

        options.update(Message::PriceReady(Price::Rejected));
        assert!(options.operation().is_none(), "key rejected");
        options.update(Message::KeySaved);
        assert!(options.reads(&[&a]).price, "a saved key is looked up again");

        // Replace changes the plan too.
        options.update(Message::SetReplace(true));
        assert!(options.reads(&[&a]).plan.is_some_and(|(_, _, r)| r));
    }

    #[test]
    fn the_job_prints_no_key_and_equals_only_itself() {
        let a = job(HashMap::new(), false);
        assert_eq!(a, a.clone());
        assert_ne!(a, job(HashMap::new(), false));
        assert!(!format!("{a:?}").contains("key"));
    }

    /// The real Soniox, with the key from `SONIOX_API_KEY` (a CI secret); skipped without it.
    /// Costs about 30 s of transcription.
    #[test]
    fn live_a_video_gets_subtitles_from_soniox() {
        if std::env::var(KEY_VAR).map_or(true, |k| k.trim().is_empty()) {
            eprintln!("skipped: {KEY_VAR} is not set");
            return;
        }
        let folder = Folder::new("live");
        let video = folder.video("dialog.mp4");

        let job = job(HashMap::new(), false);
        let result = run_now(&job, &video);
        assert_eq!(result.status, ItemStatus::Done, "{:?}", result.reason);
        let srt = std::fs::read_to_string(subtitle_path(&video)).expect("clip.srt written");
        assert!(!frename_core::Subtitles::parse(&srt).is_empty(), "{srt}");
        assert!(
            job.report()
                .is_some_and(|r| r.starts_with("Soniox: at least")),
            "{:?}",
            job.report()
        );
    }
}
