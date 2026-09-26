//! "Generate subtitles": sends each checked video's audio to Soniox (through the sonisub
//! library) and writes the subtitles next to it as `clip.srt`, the file frename already shows.
//!
//! Before the run the panel shows a plan with the cost, worked out from the files and their
//! headers only; the job then skips what the plan excluded. The key and the languages come
//! from the settings (see [`Config`]).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use frename_core::{subtitle_path, CueLength};
use iced::widget::{button, checkbox, column, text};
use iced::Element;
use sonisub::batch::{self as plan_batch, Action as Planned, Totals};
use sonisub::cancel::CancelToken;
use sonisub::job::{self, Outcome};
use sonisub::soniox::{self, api_error, Client};
use sonisub::{audio, srt, usage};

use super::super::{ItemResult, ItemStatus, JobStop};
use super::ActionMessage;
use crate::soniox_key::SonioxKey;
use crate::theme;

pub const LABEL: &str = "Generate subtitles";

/// How long the price lookup may take before the typical price is used.
const PRICE_TIMEOUT: Duration = Duration::from_secs(10);
/// What the plan says about files the built-in decoder cannot read while ffmpeg is missing.
const INSTALL_FFMPEG: &str =
    "to read .mkv, .m2ts, .avi …, install ffmpeg from ffmpeg.org, add it to PATH, restart frename";
/// Shown when Soniox refuses the key, in the plan and when it stops a job.
pub const KEY_REJECTED: &str = "Soniox rejected the key";

/// What the action needs from the settings. Sent by the app whenever one of them changes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Config {
    pub key: KeyStatus,
    /// Language hints; empty: detect automatically.
    pub languages: Vec<String>,
    pub cue_length: CueLength,
}

/// Whether a key is set, as far as the settings know.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum KeyStatus {
    /// Not read from the credential store yet.
    #[default]
    Unknown,
    Missing,
    Present(SonioxKey),
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
    /// The settings the action uses changed (or were read for the first time).
    SetConfig(Config),
    /// Transcribe again videos that already have subtitles.
    SetReplace(bool),
    /// A key was saved in the settings: look its price up again, even for the same key (it may
    /// have been refused before and fixed on the account since).
    KeySaved,
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

/// Where the plan of the checked files stands.
#[derive(Debug, Clone, Default)]
struct PlanState {
    /// Bumped by every change the plan depends on; an answer for an older one is dropped.
    generation: u64,
    /// The plan of the current generation, once worked out.
    plan: Option<Plan>,
    /// A plan for the current generation has to be asked for.
    wanted: bool,
}

/// A request for the plan of the checked files, answered on a blocking thread by [`plan`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanRequest {
    pub generation: u64,
    pub replace: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    config: Config,
    /// "Replace existing subtitles".
    replace: bool,
    plan: PlanState,
    /// The price for a key: looked up once per key per session.
    price: Option<(SonioxKey, Option<Price>)>,
}

impl Options {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetConfig(config) => self.config = config,
            Message::SetReplace(replace) => {
                self.replace = replace;
                self.invalidate_plan();
            }
            Message::KeySaved => self.price = None,
        }
    }

    /// The checked files or what they have on disk changed: the plan must be worked out again.
    pub fn invalidate_plan(&mut self) {
        self.plan.generation += 1;
        self.plan.plan = None;
        self.plan.wanted = true;
    }

    /// The plan to ask for, once per change.
    pub fn take_plan_request(&mut self) -> Option<PlanRequest> {
        if !std::mem::take(&mut self.plan.wanted) {
            return None;
        }
        Some(PlanRequest {
            generation: self.plan.generation,
            replace: self.replace,
        })
    }

    /// Take the plan worked out for `generation`, unless the files changed since.
    pub fn plan_ready(&mut self, generation: u64, plan: Plan) {
        if generation == self.plan.generation {
            self.plan.plan = Some(plan);
        }
    }

    /// The key whose price has to be looked up: once per key per session.
    pub fn take_price_request(&mut self) -> Option<SonioxKey> {
        let KeyStatus::Present(key) = &self.config.key else {
            return None;
        };
        if self.price.as_ref().is_some_and(|(known, _)| known == key) {
            return None;
        }
        self.price = Some((key.clone(), None));
        Some(key.clone())
    }

    pub fn price_ready(&mut self, key: SonioxKey, price: Price) {
        if let Some((known, slot)) = self.price.as_mut() {
            if *known == key {
                *slot = Some(price);
            }
        }
    }

    /// The key needs reading from the credential store before the action can run.
    pub fn needs_key(&self) -> bool {
        self.config.key == KeyStatus::Unknown
    }

    fn key(&self) -> Option<&SonioxKey> {
        match &self.config.key {
            KeyStatus::Present(key) => Some(key),
            KeyStatus::Unknown | KeyStatus::Missing => None,
        }
    }

    /// The price of the current key, once known.
    fn price(&self) -> Option<Price> {
        let key = self.key()?;
        self.price
            .as_ref()
            .filter(|(known, _)| known == key)
            .and_then(|(_, price)| *price)
    }

    /// The job, when the plan is known, the key accepted and there is something to do.
    pub fn operation(&self) -> Option<super::Operation> {
        let plan = self.plan.plan.as_ref().filter(|p| p.has_work())?;
        let key = self.key()?.clone();
        // Free rebuilds need no key check; a transcription waits for the price lookup, which
        // also tells whether Soniox accepts the key.
        let price = match self.price() {
            Some(Price::Rejected) => return None,
            Some(price) => price,
            None if plan.transcribe > 0 => return None,
            None => Price::Typical,
        };
        let cancel = CancelToken::new();
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
            cancel: cancel.clone(),
            ..job::Options::default()
        };
        Some(super::Operation::GenerateSubtitles(Arc::new(SubtitleJob {
            key,
            options,
            excluded: plan.excluded.clone(),
            usd_per_hour: price.usd_per_hour(),
            cancel,
            uploaded_ms: AtomicU64::new(0),
            failed_deletes: AtomicUsize::new(0),
        })))
    }

    /// The run button: its label, and whether it can be pressed.
    pub fn run_button(&self) -> (String, bool) {
        let Some(plan) = self.plan.plan.as_ref() else {
            return ("Transcribe".to_string(), false);
        };
        let runnable = self.operation().is_some();
        if plan.transcribe > 0 {
            (
                format!("Transcribe {}", count(plan.transcribe, "video", "videos")),
                runnable,
            )
        } else if plan.rebuilt_free > 0 {
            (
                format!(
                    "Build {} (free)",
                    count(plan.rebuilt_free, "subtitle", "subtitles")
                ),
                runnable,
            )
        } else {
            ("Nothing to transcribe".to_string(), false)
        }
    }

    pub fn view(&self) -> Element<'_, ActionMessage> {
        let mut lines = column![].spacing(4);
        let key_line = match &self.config.key {
            KeyStatus::Unknown => Some(("Reading the Soniox API key…", false)),
            KeyStatus::Missing => Some(("Set a Soniox API key in Settings", true)),
            KeyStatus::Present(_) if self.price() == Some(Price::Rejected) => {
                Some((KEY_REJECTED, true))
            }
            KeyStatus::Present(_) => None,
        };
        if let Some((line, error)) = key_line {
            let line = text(line).size(13).color(if error {
                theme::ERROR
            } else {
                theme::TEXT_MUTED
            });
            lines = lines.push(line);
            if error {
                lines = lines.push(settings_button());
            }
        }

        match &self.plan.plan {
            None => lines = lines.push(text("Estimating…").size(13)),
            Some(plan) => {
                // Without a key there is no account to learn the price from.
                let price = match self.config.key {
                    KeyStatus::Present(_) => self.price(),
                    KeyStatus::Unknown | KeyStatus::Missing => Some(Price::Typical),
                };
                for line in plan_lines(plan, price) {
                    lines = lines.push(text(line).size(13));
                }
            }
        }

        let replace = column![
            checkbox(self.replace)
                .label("Replace existing subtitles")
                .text_size(13)
                .on_toggle(|on| ActionMessage::GenerateSubtitles(Message::SetReplace(on))),
            text("Transcribes again; costs as shown.")
                .size(12)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(4);
        let notes = column![
            text("The audio of these videos is sent to Soniox and deleted there afterwards.")
                .size(12)
                .color(theme::TEXT_MUTED),
            text("Takes a few minutes per hour of audio; frename is busy meanwhile.")
                .size(12)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(4);

        super::panel(
            LABEL,
            "Transcribes the speech of each checked video with Soniox and saves the subtitles \
             next to it (clip.srt), where frename shows them."
                .to_string(),
            column![lines, replace, notes].spacing(12).into(),
        )
    }
}

fn settings_button<'a>() -> Element<'a, ActionMessage> {
    button(text("Open Settings").size(12))
        .on_press(ActionMessage::OpenSubtitleSettings)
        .padding([3, 10])
        .style(theme::icon_button_style(true))
        .into()
}

/// The plan as the panel shows it: what is sent and what it costs, then one line per kind of
/// file that is not sent.
fn plan_lines(plan: &Plan, price: Option<Price>) -> Vec<String> {
    let mut lines = Vec::new();
    if plan.transcribe > 0 {
        let cost = match price {
            None => "Estimating…".to_string(),
            Some(price) => {
                let usd = price.usd_per_hour() * plan.audio_s / 3600.0;
                match price {
                    Price::Learned(_) => format!("about {}", usd_text(usd)),
                    Price::Typical | Price::Rejected => {
                        format!("about {} (typical price)", usd_text(usd))
                    }
                }
            }
        };
        let unknown = if plan.unknown_length > 0 {
            format!(" + {} of unknown length", plan.unknown_length)
        } else {
            String::new()
        };
        // Nothing to price when no length is known.
        let cost = if plan.audio_s == 0.0 {
            "cost unknown".to_string()
        } else {
            cost
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

/// Work out what each of `files` needs (blocking: reads file headers). Files are in list order;
/// of several videos whose subtitles would have the same name, the first one keeps it.
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

/// Look up the transcription price from the account's usage in the last 30 days (blocking,
/// at most [`PRICE_TIMEOUT`]). A refused key is reported; any other failure gives the typical
/// price.
pub fn price(key: &SonioxKey) -> Price {
    let (sender, receiver) = std::sync::mpsc::channel();
    let key = key.clone();
    // Its own thread, so a slow lookup can be given up; the client is created and dropped there.
    std::thread::spawn(move || {
        let price = (|| {
            let client = Client::new(soniox::DEFAULT_BASE, key.expose())?;
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

/// One run of the action: the options every file gets, and what the run has sent so far.
pub struct SubtitleJob {
    key: SonioxKey,
    options: job::Options,
    /// Files the plan excluded, with their reason.
    excluded: HashMap<PathBuf, &'static str>,
    /// The price the estimate used, for what the run spent.
    usd_per_hour: f64,
    /// Stops the file in work (extraction, upload, polling) on Cancel.
    cancel: CancelToken,
    /// Audio sent to Soniox by the files that finished.
    uploaded_ms: AtomicU64,
    /// Uploads that could not be deleted on Soniox afterwards.
    failed_deletes: AtomicUsize,
}

impl std::fmt::Debug for SubtitleJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubtitleJob")
            .field("key", &self.key)
            .field("languages", &self.options.languages)
            .field("force", &self.options.force)
            .field("excluded", &self.excluded.len())
            .finish_non_exhaustive()
    }
}

impl SubtitleJob {
    /// Stop the file in work within a few seconds; it counts as not reached.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// What the run spent, and uploads left on Soniox; `None` when nothing was sent.
    pub fn report(&self) -> Option<String> {
        let seconds = self.uploaded_ms.load(Ordering::Relaxed) as f64 / 1000.0;
        let failed_deletes = self.failed_deletes.load(Ordering::Relaxed);
        let mut parts = Vec::new();
        if seconds > 0.0 {
            parts.push(format!(
                "Soniox: at least {} · about {}",
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

/// Generate the subtitles of the video at `path`. Blocking: runs on the batch's worker thread,
/// where the Soniox client is created and dropped (a blocking HTTP client must not be dropped
/// on an async runtime thread).
pub fn run(job: &SubtitleJob, path: &Path) -> ItemResult {
    if let Some(reason) = job.excluded.get(path) {
        log::info!("subtitles: {} skipped: {reason}", path.display());
        return with_reason(ItemStatus::Skipped, reason);
    }
    if job.cancel.is_cancelled() {
        return ItemResult::new(ItemStatus::Pending, None);
    }
    let result = Client::new(soniox::DEFAULT_BASE, job.key.expose()).and_then(|client| {
        let client = client.with_cancel(job.cancel.clone());
        let result = job::process(path, &subtitle_path(path), &job.options, Some(&client));
        job.failed_deletes
            .fetch_add(client.failed_deletes(), Ordering::Relaxed);
        result
    });
    if let Some(seconds) = result.as_ref().ok().and_then(Outcome::uploaded_s) {
        job.uploaded_ms
            .fetch_add((seconds * 1000.0) as u64, Ordering::Relaxed);
    }
    item_result(path, result, job.cancel.is_cancelled())
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
    let mut result = with_reason(ItemStatus::Failed, &failure_reason(&e));
    if let Some(api) = api_error(&e).filter(|api| api.is_fatal()) {
        let key_rejected = matches!(api.status, Some(401))
            || api.error_type == "unauthenticated"
            || api.error_type == "permission_denied";
        let message = match api.error_type.as_str() {
            _ if key_rejected => KEY_REJECTED.to_string(),
            "organization_balance_exhausted" => {
                "Soniox balance is empty. Top it up at console.soniox.com.".to_string()
            }
            t if t.ends_with("budget_exhausted") => {
                "The Soniox monthly budget is used up. Raise it at console.soniox.com.".to_string()
            }
            _ => format!("Soniox stopped the job ({}).", api.error_type),
        };
        result.stop_job = Some(JobStop {
            message,
            open_settings: key_rejected,
        });
    }
    result
}

/// Why a file failed, short enough for the result list; the log has the whole error.
fn failure_reason(e: &anyhow::Error) -> String {
    if let Some(api) = api_error(e) {
        return format!("failed: Soniox: {}", api.message);
    }
    let chain = format!("{e:#}");
    // sonisub's context for a request that got no answer (network, proxy, timeout).
    if chain.contains("request to Soniox failed") {
        return "failed: cannot reach Soniox".to_string();
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
    let root = e.root_cause().to_string();
    let short: String = root.chars().take(120).collect();
    format!("failed: {short}")
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

/// "$0.07"; small amounts are not shown as zero.
fn usd_text(usd: f64) -> String {
    if usd > 0.0 && usd < 0.01 {
        "less than $0.01".to_string()
    } else {
        format!("${usd:.2}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSCRIPT: &str =
        include_str!("../../../../tests/fixtures/subtitles/dialog.soniox.json");

    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("frename-subtitles-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("temp dir");
            Self(dir)
        }

        /// A file that is not a readable video (no header), with `content`.
        fn file(&self, name: &str, content: &str) -> PathBuf {
            let path = self.0.join(name);
            std::fs::write(&path, content).expect("write");
            path
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/subtitles")
            .join(name)
    }

    fn job(excluded: HashMap<PathBuf, &'static str>, force: bool) -> SubtitleJob {
        job_with_key(excluded, force, SonioxKey::new("test").expect("key"))
    }

    fn job_with_key(
        excluded: HashMap<PathBuf, &'static str>,
        force: bool,
        key: SonioxKey,
    ) -> SubtitleJob {
        let cancel = CancelToken::new();
        SubtitleJob {
            key,
            options: job::Options {
                force,
                cancel: cancel.clone(),
                ..job::Options::default()
            },
            excluded,
            usd_per_hour: 0.12,
            cancel,
            uploaded_ms: AtomicU64::new(0),
            failed_deletes: AtomicUsize::new(0),
        }
    }

    #[test]
    fn a_saved_transcript_becomes_subtitles_without_calling_soniox() {
        let folder = Folder::new("cached");
        let video = folder.0.join("clip.mp4");
        std::fs::copy(fixture("dialog.mp4"), &video).expect("copy the video");
        folder.file("clip.soniox.json", TRANSCRIPT);

        let result = run(&job(HashMap::new(), false), &video);
        assert_eq!(result.status, ItemStatus::Done);
        let srt = std::fs::read_to_string(subtitle_path(&video)).expect("clip.srt written");
        let subtitles = frename_core::Subtitles::parse(&srt);
        assert!(subtitles.cues().len() > 3, "{srt}");
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
        let files = vec![subtitled, cached, silent, first.clone(), second.clone()];

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
        let result = run(&job(excluded, true), &video);
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
        let stop = rejected.stop_job.expect("stops the job");
        assert_eq!(stop.message, KEY_REJECTED);
        assert!(stop.open_settings);

        let empty = item_result(
            path,
            Err(soniox_error(402, "organization_balance_exhausted")),
            false,
        );
        assert_eq!(
            empty.stop_job.map(|s| s.message).as_deref(),
            Some("Soniox balance is empty. Top it up at console.soniox.com.")
        );

        let ordinary = item_result(path, Err(soniox_error(400, "invalid_audio_file")), false);
        assert!(ordinary.stop_job.is_none());
        assert_eq!(
            ordinary.reason.as_deref(),
            Some("failed: Soniox: message from Soniox")
        );
    }

    #[test]
    fn a_network_failure_reads_as_such() {
        let e = anyhow::anyhow!("error sending request").context("request to Soniox failed");
        assert_eq!(failure_reason(&e), "failed: cannot reach Soniox");
        let e = anyhow::anyhow!("unsupported container");
        assert_eq!(failure_reason(&e), "audio format not supported");
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
    }

    #[test]
    fn a_job_after_a_cancelled_one_runs() {
        let folder = Folder::new("after-cancel");
        let video = folder.file("clip.mp4", "x");
        folder.file("clip.soniox.json", TRANSCRIPT);

        let cancelled = job(HashMap::new(), false);
        cancelled.cancel();
        assert_eq!(run(&cancelled, &video).status, ItemStatus::Pending);
        assert!(!subtitle_path(&video).exists());

        assert_eq!(
            run(&job(HashMap::new(), false), &video).status,
            ItemStatus::Done
        );
    }

    #[test]
    fn the_report_says_what_was_sent_at_least() {
        let job = job(HashMap::new(), false);
        assert_eq!(job.report(), None);
        job.uploaded_ms.store(41 * 60 * 1000, Ordering::Relaxed);
        job.failed_deletes.store(1, Ordering::Relaxed);
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

    /// The real Soniox, with the key from `SONIOX_API_KEY` (a CI secret); skipped without it.
    /// Costs about 30 s of transcription.
    #[test]
    fn live_a_video_gets_subtitles_from_soniox() {
        let Some(key) = std::env::var("SONIOX_API_KEY")
            .ok()
            .and_then(|k| SonioxKey::new(&k))
        else {
            eprintln!("skipped: SONIOX_API_KEY is not set");
            return;
        };
        let folder = Folder::new("live");
        let video = folder.0.join("dialog.mp4");
        std::fs::copy(fixture("dialog.mp4"), &video).expect("copy the video");

        let job = job_with_key(HashMap::new(), false, key);
        let result = run(&job, &video);
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

    #[test]
    fn the_job_prints_no_key() {
        let job = job(HashMap::new(), false);
        assert!(!format!("{job:?}").contains("test"));
    }

    #[test]
    fn the_run_button_names_what_is_sent() {
        let mut options = Options::default();
        options.update(Message::SetConfig(Config {
            key: KeyStatus::Present(SonioxKey::new("k").expect("key")),
            ..Config::default()
        }));
        let request = options.take_plan_request();
        assert!(request.is_none(), "nothing asked for before a change");
        options.invalidate_plan();
        let request = options.take_plan_request().expect("plan request");
        options.plan_ready(
            request.generation,
            Plan {
                transcribe: 8,
                ..Plan::default()
            },
        );
        assert_eq!(
            options.run_button(),
            ("Transcribe 8 videos".to_string(), false),
            "price pending"
        );
        let key = options.take_price_request().expect("price request");
        assert!(options.take_price_request().is_none(), "once per key");
        options.price_ready(key.clone(), Price::Learned(0.1));
        assert_eq!(
            options.run_button(),
            ("Transcribe 8 videos".to_string(), true)
        );
        options.price_ready(key, Price::Rejected);
        assert!(!options.run_button().1, "key rejected");

        // An answer for an older set of files is dropped.
        options.update(Message::SetReplace(true));
        let old = request.generation;
        options.plan_ready(old, Plan::default());
        assert_eq!(options.run_button(), ("Transcribe".to_string(), false));
    }
}
