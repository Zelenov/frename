//! "Prepare clips" (#17 stage 3, `docs/design/folder-pipeline.md`): per checked clip, in one pass,
//! subtitles (Soniox), then the description (summary, moments as markers, comment, tag
//! suggestions), under one budget cap, with one plan and one progress view. Each step skips what
//! is already there, so a stopped run is resumed by running it again. Afterwards similar footage
//! is grouped (clipscribe's `group_clips`) from the fingerprints of the frames each clip was
//! described from, kept in a cache in frename's own data folder.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};

use clipscribe::{Budget, Cache};
use frename_core::{File, FileId, FileTagger};
use iced::widget::column;
use iced::Element;

use super::super::page::{self, Change};
use super::super::{ItemProgress, ItemResult, ItemStatus};
use super::{describe_ai, generate_subtitles, ActionMessage, Panel, Readiness, Step};
use crate::ui::layout::{self, NoticeKind};
use crate::ui::tokens::SPACE_S;
use crate::ui::{button, form, text};

pub fn label() -> String {
    fl!("batch-action-prepare")
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Make the subtitles first, when a clip has none.
    SetSubtitles(bool),
    /// Describe the clips: summary, markers, comment, tag suggestions.
    SetDescription(bool),
    /// The text of the cap field: dollars, empty for none.
    SetCap(String),
    /// The groups of similar footage the finished run found.
    Groups(Vec<Group>),
}

/// Clips whose footage looks the same, from one run: what the group shows and which clips.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub label: String,
    pub files: Vec<FileId>,
}

/// The options of the action, kept while the editor switches between actions.
#[derive(Debug, Clone)]
pub struct Options {
    subtitles: bool,
    description: bool,
    /// What was typed in the cap field.
    cap_text: String,
    /// The names of the clips the last run of this folder left undone, to offer again.
    left: Vec<String>,
    /// The folder `left` belongs to: the names mean nothing in another one.
    left_folder: String,
    groups: Vec<Group>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            subtitles: true,
            description: true,
            cap_text: String::new(),
            left: Vec::new(),
            left_folder: String::new(),
            groups: Vec::new(),
        }
    }
}

impl Options {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetSubtitles(on) => self.subtitles = on,
            Message::SetDescription(on) => self.description = on,
            Message::SetCap(text) => self.cap_text = text,
            Message::Groups(groups) => self.groups = groups,
        }
    }

    /// The cap in dollars: none when the field is empty, `Err` when it holds no positive number.
    pub fn cap(&self) -> Result<Option<f64>, ()> {
        let text = self.cap_text.trim().replace(',', ".");
        if text.is_empty() {
            return Ok(None);
        }
        match text.parse::<f64>() {
            Ok(usd) if usd.is_finite() && usd > 0.0 => Ok(Some(usd)),
            _ => Err(()),
        }
    }

    pub fn subtitles(&self) -> bool {
        self.subtitles
    }

    pub fn description(&self) -> bool {
        self.description
    }

    /// The groups the last finished run found.
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }

    /// Forget the groups: a new run starts.
    pub fn clear_groups(&mut self) {
        self.groups.clear();
    }

    /// The clips the last run left in `folder`, by name; none when that run was in another folder.
    pub fn left(&self, folder: &Path) -> &[String] {
        if self.left_folder == folder.to_string_lossy() {
            &self.left
        } else {
            &[]
        }
    }

    pub fn set_left(&mut self, names: Vec<String>, folder: &Path) {
        self.left = names;
        self.left_folder = folder.to_string_lossy().into_owned();
    }

    /// The options as key/value pairs to remember for next time (#65), with the clips left.
    pub fn persist(&self) -> Vec<(String, String)> {
        let mut saved = vec![
            (
                "subtitles".to_string(),
                u8::from(self.subtitles).to_string(),
            ),
            (
                "description".to_string(),
                u8::from(self.description).to_string(),
            ),
            ("cap".to_string(), self.cap_text.trim().to_string()),
        ];
        if !self.left.is_empty() {
            // One name per line: a name with a line break cannot be told apart, and is not kept.
            let names: Vec<&str> = self
                .left
                .iter()
                .map(String::as_str)
                .filter(|name| !name.contains('\n'))
                .collect();
            saved.push(("left".to_string(), names.join("\n")));
            saved.push(("left_folder".to_string(), self.left_folder.clone()));
        }
        saved
    }

    /// Restore what [`Self::persist`] gave; a missing or unreadable value keeps the default.
    pub fn restore(&mut self, saved: &[(String, String)]) {
        let get = |key: &str| {
            saved
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        if let Some(on) = get("subtitles").and_then(flag) {
            self.subtitles = on;
        }
        if let Some(on) = get("description").and_then(flag) {
            self.description = on;
        }
        if let Some(cap) = get("cap") {
            self.cap_text = cap.to_string();
        }
        self.left_folder = get("left_folder").unwrap_or_default().to_string();
        self.left = get("left")
            .map(|names| {
                names
                    .lines()
                    .filter(|name| !name.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
    }
}

fn flag(value: &str) -> Option<bool> {
    match value {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

/// What the two steps say together: whether the run can start, or why not yet.
pub enum Plan {
    Wait,
    Blocked {
        note: String,
        fix: Option<ActionMessage>,
    },
    /// Nothing to do for either step: the reasons.
    Nothing(Vec<String>),
    Ready,
}

/// Put the steps together. A step that is switched off is skipped; a step that is held by
/// something the editor can fix holds the run; with both skipped there is nothing to do.
pub fn plan<S, D>(options: &Options, subtitles: Step<S>, description: Step<D>) -> Plan {
    fn off<R>() -> Step<R> {
        Step::Skip(fl!("batch-prepare-step-off"))
    }
    let subtitles = if options.subtitles { subtitles } else { off() };
    let description = if options.description {
        description
    } else {
        off()
    };
    if let Some((note, fix)) = as_note(&subtitles).or_else(|| as_note(&description)) {
        return Plan::Blocked { note, fix };
    }
    if matches!(subtitles, Step::Wait) || matches!(description, Step::Wait) {
        return Plan::Wait;
    }
    fn why<R>(step: Step<R>) -> Option<String> {
        match step {
            Step::Skip(why) => Some(why),
            _ => None,
        }
    }
    let (subtitles_why, description_why) = (why(subtitles), why(description));
    if subtitles_why.is_some() && description_why.is_some() {
        return Plan::Nothing(
            [subtitles_why, description_why]
                .into_iter()
                .flatten()
                .collect(),
        );
    }
    Plan::Ready
}

/// What holds a step, when something does.
fn as_note<R>(step: &Step<R>) -> Option<(String, Option<ActionMessage>)> {
    match step {
        Step::Blocked { note, fix } => Some((note.clone(), fix.clone())),
        _ => None,
    }
}

/// The readiness of the action for a run of one clip or a check from outside the panel.
pub fn readiness(plan: Plan) -> Readiness {
    match plan {
        Plan::Wait => Readiness::Waiting,
        Plan::Blocked { note, fix } => Readiness::Blocked { note, fix },
        Plan::Nothing(why) => Readiness::Blocked {
            note: why
                .into_iter()
                .next()
                .unwrap_or_else(|| fl!("batch-prepare-nothing")),
            fix: None,
        },
        Plan::Ready => Readiness::Ready,
    }
}

/// One run of the action: the options of both steps, and what the steps share.
#[derive(Debug, Clone)]
pub struct Run(Arc<PipelineJob>);

pub struct PipelineJob {
    subtitles: Option<generate_subtitles::Run>,
    description: Option<describe_ai::Run>,
    /// The cap on what the run spends, shared by every clip: the description takes its request's
    /// upper bound from it before sending, the subtitles take theirs from the clip's length.
    budget: Budget,
    usd_per_hour: f64,
    /// Where the described clips are kept for grouping; none when frename has no data folder.
    cache_dir: Option<PathBuf>,
    /// The cache of the folder the clips are in, opened by the first clip.
    cache: OnceLock<Option<Cache>>,
}

impl std::fmt::Debug for PipelineJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(PipelineJob))
            .field(stringify!(subtitles), &self.subtitles.is_some())
            .field(stringify!(description), &self.description.is_some())
            .field(stringify!(cap), &self.budget.max_usd())
            .finish_non_exhaustive()
    }
}

impl PartialEq for Run {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Run {}

impl Run {
    /// A run of these steps (at least one), spending at most `cap` dollars in all, keeping the
    /// described clips under `cache_dir`.
    pub fn new(
        subtitles: Option<generate_subtitles::Run>,
        description: Option<describe_ai::Run>,
        cap: Option<f64>,
        cache_dir: Option<PathBuf>,
    ) -> Self {
        Self(Arc::new(PipelineJob {
            usd_per_hour: subtitles.as_ref().map_or(0.0, |s| s.usd_per_hour()),
            subtitles,
            description,
            budget: Budget::new(cap),
            cache_dir,
            cache: OnceLock::new(),
        }))
    }

    /// The description step's options, for the model of the spend line and for grouping.
    pub fn description(&self) -> Option<&describe_ai::Run> {
        self.0.description.as_ref()
    }

    /// What the subtitles step spent, and the cap and spend of the run, as a line for the report.
    pub fn report(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        if let Some(subtitles) = self.0.subtitles.as_ref().and_then(|s| s.report()) {
            parts.push(subtitles);
        }
        if let Some(cap) = self.0.budget.max_usd() {
            parts.push(fl!(
                "batch-prepare-report-cap",
                spent = describe_ai::dollars(self.0.budget.spent_usd()),
                cap = describe_ai::dollars(cap)
            ));
        }
        (!parts.is_empty()).then(|| parts.join(". "))
    }

    /// [`Self::report`] in English only, for logs.
    pub fn log_report(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        if let Some(subtitles) = self.0.subtitles.as_ref().and_then(|s| s.log_report()) {
            parts.push(subtitles);
        }
        if let Some(cap) = self.0.budget.max_usd() {
            parts.push(format!(
                "spent ${:.2} of the ${cap:.2} cap",
                self.0.budget.spent_usd()
            ));
        }
        (!parts.is_empty()).then(|| parts.join(". "))
    }

    /// A clip the run has not reached was renamed: what the subtitles plan keeps by path moves.
    pub fn path_changed(&self, old: &Path, new: &Path) {
        if let Some(subtitles) = &self.0.subtitles {
            subtitles.path_changed(old, new);
        }
    }

    /// Group the clips of `files` (id and path) that were described by this run or an earlier one
    /// with the same settings, from what the cache kept of them: no request, no frames read.
    /// Blocking.
    pub fn group(&self, files: &[(FileId, PathBuf)]) -> Vec<Group> {
        let Some(description) = &self.0.description else {
            return Vec::new();
        };
        // Opened by the first clip of the run, or here when every clip was skipped.
        let Some(Some(cache)) = files.first().and_then(|(_, path)| self.0.cache_for(path)) else {
            return Vec::new();
        };
        let mut found: Vec<(FileId, clipscribe::DescribedClip)> = Vec::new();
        for (id, path) in files {
            if let Some(clip) = describe_ai::cached_clip(description, cache, path) {
                found.push((*id, clip));
            }
        }
        let clips: Vec<&clipscribe::DescribedClip> = found.iter().map(|(_, clip)| clip).collect();
        let grouping = clipscribe::group_clips(&clips, &clipscribe::GroupOptions::default());
        let mut by_group: HashMap<usize, Vec<FileId>> = HashMap::new();
        for ((id, _), groups) in found.iter().zip(&grouping.clips) {
            by_group.entry(groups.group).or_default().push(*id);
        }
        grouping
            .groups
            .iter()
            .filter_map(|group| {
                let files = by_group.remove(&group.id)?;
                (files.len() > 1).then(|| Group {
                    label: group.label.clone(),
                    files,
                })
            })
            .collect()
    }
}

/// Prepare the clip at `path`: its subtitles when it has none, then its description. The result
/// is the two steps' put together: done when one did something, failed when one failed, and a
/// stop of either (a rejected key, the cap) stops the job.
pub fn run(job: &Run, path: &Path, cancel: &AtomicBool, progress: &ItemProgress) -> ItemResult {
    let job = &job.0;
    let mut result = ItemResult::new(ItemStatus::Skipped, None);
    // The share of the bar each step may fill: the subtitles take a few minutes per hour of
    // audio, the description about a quarter minute.
    let (subtitles_share, description_share) = match (&job.subtitles, &job.description) {
        (Some(_), Some(_)) => (0.4, 0.6),
        (Some(_), None) => (1.0, 0.0),
        _ => (0.0, 1.0),
    };
    if let Some(subtitles) = &job.subtitles {
        progress.window(0.0, subtitles_share);
        let step = subtitles_step(job, subtitles, path, cancel, progress);
        if step.stop_job.is_some() || step.status == ItemStatus::Pending {
            return step;
        }
        merge(&mut result, step);
    }
    if let Some(description) = &job.description {
        progress.window(1.0 - description_share, 1.0);
        let cache = job.cache_for(path);
        let step = describe_ai::run_clip(
            description,
            &job.budget,
            cache.as_ref().and_then(|c| c.as_ref()),
            path,
            cancel,
            progress,
        );
        if step.stop_job.is_some() || step.status == ItemStatus::Pending {
            return stopped_after_subtitles(result, step);
        }
        merge(&mut result, step);
    }
    result
}

/// The clip's result when the description step stopped the job or was cancelled, after the
/// subtitles step (whose result is `result`) ran. The clip is left unreached, so it is offered
/// again and a retry resumes it; the subtitles are kept (and not paid for again).
fn stopped_after_subtitles(mut result: ItemResult, step: ItemResult) -> ItemResult {
    let failed = step.status == ItemStatus::Failed;
    merge(&mut result, step);
    if !failed {
        result.status = ItemStatus::Pending;
    }
    result
}

/// What a clip of unreadable length is priced at, in seconds.
const UNKNOWN_LENGTH_S: f64 = 3600.0;

/// The subtitles of the clip, inside the cap: a clip that has none (or is to be replaced) needs
/// its audio's price set aside first.
fn subtitles_step(
    job: &PipelineJob,
    subtitles: &generate_subtitles::Run,
    path: &Path,
    cancel: &AtomicBool,
    progress: &ItemProgress,
) -> ItemResult {
    let disk = FileTagger::disk_path(path);
    let reservation = if let Some(audio_s) = subtitles.audio_to_send(&disk) {
        // A length that cannot be read is priced as an hour, so the cap is not passed unseen.
        let seconds = audio_s
            .or_else(|| clipscribe::frames::clip_duration_s(&disk))
            .unwrap_or(UNKNOWN_LENGTH_S);
        let bound = job.usd_per_hour * seconds / 3600.0;
        match job.budget.reserve(bound) {
            Some(reservation) => Some(reservation),
            None => {
                return ItemResult {
                    stop_job: Some(describe_ai::over_budget_text(&job.budget)),
                    ..ItemResult::new(ItemStatus::Pending, None)
                };
            }
        }
    } else {
        None
    };
    let before = subtitles.uploaded_seconds();
    let result = generate_subtitles::run(subtitles, path, cancel, progress);
    if let Some(reservation) = reservation {
        settle_upload(
            reservation,
            job.usd_per_hour,
            subtitles.uploaded_seconds() - before,
        );
    }
    result
}

/// Replace what was set aside for a clip's audio by what was really sent: `uploaded_s` seconds
/// at `usd_per_hour`.
fn settle_upload(reservation: clipscribe::Reservation<'_>, usd_per_hour: f64, uploaded_s: f64) {
    reservation.settle(usd_per_hour * uploaded_s.max(0.0) / 3600.0);
}

/// Add one step's result to the clip's.
fn merge(total: &mut ItemResult, step: ItemResult) {
    total.status = match (total.status, step.status) {
        (ItemStatus::Failed, _) | (_, ItemStatus::Failed) => ItemStatus::Failed,
        (ItemStatus::Done, _) | (_, ItemStatus::Done) => ItemStatus::Done,
        (ItemStatus::Pending, other) | (other, ItemStatus::Pending) => other,
        (a, _) => a,
    };
    if total.reason.is_none() || step.status == ItemStatus::Failed {
        total.reason = step.reason.or(total.reason.take());
    }
    total.update = step.update.or(total.update.take());
    total.usage = match (total.usage, step.usage) {
        (Some(mut a), Some(b)) => {
            a += b;
            Some(a)
        }
        (a, b) => a.or(b),
    };
    total.usage_unknown |= step.usage_unknown;
    total.stop_job = total.stop_job.take().or(step.stop_job);
    total.stop_if_repeated = total.stop_if_repeated.take().or(step.stop_if_repeated);
    total.out_of_credit |= step.out_of_credit;
}

impl PipelineJob {
    /// The cache of the folder `path` is in, opened once; `None` when it cannot be.
    fn cache_for(&self, path: &Path) -> Option<&Option<Cache>> {
        let dir = self.cache_dir.as_ref()?;
        let folder = path.parent()?;
        Some(self.cache.get_or_init(|| {
            let dir = dir.join(folder_key(folder));
            match Cache::open(&clipscribe::cache_path(folder, Some(&dir))) {
                Ok(cache) => Some(cache),
                Err(e) => {
                    log::warn!("prepare: the clip cache is not available: {e}");
                    None
                }
            }
        }))
    }
}

/// A stable name for a folder's cache: the cache outlives the build, so no `std` hasher.
fn folder_key(folder: &Path) -> String {
    let hash = folder
        .to_string_lossy()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("{hash:016x}")
}

/// Where the described clips of every folder are kept for grouping, under frename's data folder.
pub fn cache_dir_in(data_dir: &Path) -> PathBuf {
    data_dir.join("clip-cache")
}

/// The page of the action for `checked` (its steps, the cap, the plan, what stops it).
pub fn panel<'a>(
    options: &'a Options,
    subtitles: &'a generate_subtitles::Options,
    description: &'a describe_ai::Options,
    checked: &[&File],
    plan_readiness: &Readiness,
    left_clips: usize,
) -> Panel<'a> {
    let steps = page::option_row(
        fl!("batch-option-steps"),
        layout::choices([
            form::checkbox(fl!("batch-prepare-step-subtitles"), options.subtitles)
                .on_toggle(|on| ActionMessage::PrepareClips(Message::SetSubtitles(on)))
                .into(),
            form::checkbox(fl!("batch-prepare-step-description"), options.description)
                .on_toggle(|on| ActionMessage::PrepareClips(Message::SetDescription(on)))
                .into(),
        ]),
    );
    let cap_ok = options.cap().is_ok();
    let field = if cap_ok {
        form::text_field(&fl!("batch-prepare-cap-none"), &options.cap_text)
    } else {
        form::invalid_text_field(&fl!("batch-prepare-cap-none"), &options.cap_text)
    }
    .on_input(|text| ActionMessage::PrepareClips(Message::SetCap(text)));
    let cap = page::option_row(fl!("batch-prepare-cap"), Element::from(field));

    let mut parts: Vec<Element<'a, ActionMessage>> = vec![steps, cap];
    if left_clips > 0 {
        parts.push(layout::notice(
            NoticeKind::Info,
            fl!("batch-prepare-left", count = (left_clips as i64)),
            None,
            [button::secondary(fl!("batch-prepare-check-left"))
                .on_press(ActionMessage::CheckLeft)
                .into()],
        ));
    }
    parts.push(estimate(options, subtitles, description, checked));
    if options.subtitles {
        parts.extend(subtitles.key_notice());
    }
    if options.description {
        parts.extend(description.key_notice());
    }
    let (run, ready, reason) = match plan_readiness {
        Readiness::Ready => (
            run_label(options, subtitles, description, checked),
            cap_ok,
            (!cap_ok).then(|| fl!("batch-prepare-cap-invalid")),
        ),
        Readiness::Waiting => (
            fl!("batch-action-prepare"),
            false,
            (!checked.is_empty()).then(|| fl!("batch-reason-estimate")),
        ),
        Readiness::Blocked { note, .. } => (fl!("batch-action-prepare"), false, Some(note.clone())),
    };
    Panel {
        page: page::page(
            label(),
            fl!("batch-action-prepare-hint"),
            &[Change::Subtitles, Change::IntoComments],
            parts,
        ),
        run,
        ready,
        reason,
    }
}

/// The button's label: how many clips and what it costs.
fn run_label(
    options: &Options,
    subtitles: &generate_subtitles::Options,
    description: &describe_ai::Options,
    checked: &[&File],
) -> String {
    let usd = totals(options, subtitles, description, checked).map(|t| t.usd);
    match usd {
        Some(usd) => fl!(
            "batch-prepare-run",
            clips = (checked.len() as i64),
            dollars = describe_ai::dollars(usd)
        ),
        None => fl!("batch-action-prepare"),
    }
}

/// What the steps will do, added up.
struct Totals {
    transcribe: usize,
    audio_s: f64,
    describe: usize,
    usd: f64,
}

fn totals(
    options: &Options,
    subtitles: &generate_subtitles::Options,
    description: &describe_ai::Options,
    checked: &[&File],
) -> Option<Totals> {
    let mut totals = Totals {
        transcribe: 0,
        audio_s: 0.0,
        describe: 0,
        usd: 0.0,
    };
    if options.subtitles {
        let (n, audio_s, usd) = subtitles.totals(checked)?;
        totals.transcribe = n;
        totals.audio_s = audio_s;
        totals.usd += usd;
    }
    if options.description {
        let (n, _, usd, _) = description.totals(checked)?;
        totals.describe = n;
        totals.usd += usd;
    }
    Some(totals)
}

/// The plan's figures: what each step will do, and the price of both.
fn estimate<'a>(
    options: &Options,
    subtitles: &generate_subtitles::Options,
    description: &describe_ai::Options,
    checked: &[&File],
) -> Element<'a, ActionMessage> {
    let Some(totals) = totals(options, subtitles, description, checked) else {
        return text::body(fl!("batch-prepare-estimating")).into();
    };
    let mut rows: Vec<(String, String)> = Vec::new();
    if options.subtitles {
        rows.push((
            fl!("batch-prepare-plan-subtitles"),
            if totals.transcribe == 0 {
                fl!("batch-prepare-plan-none")
            } else {
                fl!(
                    "batch-prepare-plan-subtitles-line",
                    videos = (totals.transcribe as i64),
                    duration = describe_ai::duration_text(totals.audio_s)
                )
            },
        ));
    }
    if options.description {
        rows.push((
            fl!("batch-prepare-plan-description"),
            if totals.describe == 0 {
                fl!("batch-prepare-plan-none")
            } else {
                fl!(
                    "batch-prepare-plan-description-line",
                    videos = (totals.describe as i64)
                )
            },
        ));
    }
    rows.push((fl!("batch-plan-cost"), describe_ai::dollars(totals.usd)));
    column![
        page::plan(rows),
        page::notes([fl!("batch-prepare-resume-note")])
    ]
    .spacing(SPACE_S)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipscribe::AiUsage;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn the_cap_is_a_positive_number_or_nothing() {
        let mut options = Options::default();
        assert_eq!(options.cap(), Ok(None));
        for (text, cap) in [
            ("5", Ok(Some(5.0))),
            (" 2,5 ", Ok(Some(2.5))),
            ("0.30", Ok(Some(0.3))),
            ("0", Err(())),
            ("-1", Err(())),
            ("abc", Err(())),
            ("inf", Err(())),
        ] {
            options.update(Message::SetCap(text.to_string()));
            assert_eq!(options.cap(), cap, "{text:?}");
        }
    }

    #[test]
    fn a_description_stopped_by_the_cap_leaves_the_clip_unreached_with_its_subtitles_kept() {
        let subtitles = ItemResult::new(ItemStatus::Done, None);
        let stop = ItemResult {
            stop_job: Some("cap".to_string()),
            ..ItemResult::new(ItemStatus::Pending, None)
        };
        let result = stopped_after_subtitles(subtitles, stop);
        assert_eq!(result.status, ItemStatus::Pending);
        assert_eq!(result.stop_job.as_deref(), Some("cap"));

        // A key the service rejected is a failure of the clip, and stops the job too.
        let subtitles = ItemResult::new(ItemStatus::Done, None);
        let rejected = ItemResult {
            stop_job: Some("key".to_string()),
            ..ItemResult::failed("rejected".to_string())
        };
        let result = stopped_after_subtitles(subtitles, rejected);
        assert_eq!(result.status, ItemStatus::Failed);
        assert!(result.stop_job.is_some());
    }

    /// The cap is checked before a clip is sent: a clip whose audio could pass it is not sent, and
    /// the job stops with the clip unreached. A clip that costs nothing (its subtitles exist) is
    /// not held back by a cap it cannot touch.
    #[test]
    fn the_cap_stops_the_job_before_a_clip_that_could_pass_it() {
        let dir = frename_core::test_support::fresh_dir("pipeline-cap");
        let video = dir.join("clip.mp4");
        std::fs::write(&video, b"not a video").expect("clip");
        let run = Run::new(
            Some(generate_subtitles::Run::priced_for_test(0.12)),
            None,
            Some(0.01),
            None,
        );
        let result = run_now(&run, &video);
        assert_eq!(result.status, ItemStatus::Pending, "not reached");
        assert!(result.stop_job.is_some(), "the job stops");

        // Its subtitles exist: nothing would be sent, nothing is set aside, the job goes on.
        std::fs::write(dir.join("clip.srt"), b"1\n").expect("srt");
        let result = run_now(&run, &video);
        assert!(result.stop_job.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn run_now(run: &Run, path: &Path) -> ItemResult {
        super::run(run, path, &AtomicBool::new(false), &ItemProgress::default())
    }

    /// What was set aside for a clip is replaced by what was sent, so the cap counts each clip's
    /// own audio and nothing stays held.
    #[test]
    fn the_cap_counts_the_audio_each_clip_sent() {
        let budget = Budget::new(Some(1.0));
        let first = budget.reserve(0.5).expect("fits");
        settle_upload(first, 1.0, 360.0);
        assert!((budget.spent_usd() - 0.1).abs() < 1e-9, "6 minutes at $1/h");
        let second = budget.reserve(0.9).expect("the first one's hold is gone");
        settle_upload(second, 1.0, -5.0);
        assert!(
            (budget.spent_usd() - 0.1).abs() < 1e-9,
            "nothing sent, nothing spent"
        );
        assert!(budget.reserve(0.95).is_none(), "passing the cap is refused");
    }

    #[test]
    fn the_options_and_the_clips_left_round_trip() {
        let mut options = Options::default();
        options.update(Message::SetSubtitles(false));
        options.update(Message::SetCap("4.5".to_string()));
        options.set_left(
            vec!["a.mp4".to_string(), "b c.mov".to_string()],
            Path::new("/clips"),
        );
        let saved = options.persist();
        let mut restored = Options::default();
        restored.restore(&saved);
        assert!(!restored.subtitles());
        assert!(restored.description());
        assert_eq!(restored.cap(), Ok(Some(4.5)));
        assert_eq!(restored.left(Path::new("/clips")), ["a.mp4", "b c.mov"]);
        // The names mean nothing in another folder.
        assert!(restored.left(Path::new("/other")).is_empty());

        // Nothing saved, or nonsense: the defaults.
        let mut fresh = Options::default();
        fresh.restore(&[("subtitles".to_string(), "maybe".to_string())]);
        assert!(fresh.subtitles() && fresh.description());
        assert!(fresh.left(Path::new("/clips")).is_empty());
    }

    #[test]
    fn a_step_that_is_off_or_has_nothing_to_do_is_skipped_and_both_skipped_is_nothing_to_do() {
        let options = Options::default();
        let skip = |why: &str| Step::<describe_ai::Run>::Skip(why.to_string());
        let nothing = plan(
            &options,
            Step::<generate_subtitles::Run>::Skip("no speech".to_string()),
            skip("all described"),
        );
        match nothing {
            Plan::Nothing(why) => assert_eq!(why, ["no speech", "all described"]),
            _ => panic!("expected nothing to do"),
        }
        let off = Options {
            description: false,
            ..Options::default()
        };
        assert!(matches!(
            plan(&off, Step::<generate_subtitles::Run>::Wait, skip("x")),
            Plan::Wait
        ));
    }

    #[test]
    fn a_blocked_step_holds_the_run_even_when_the_other_is_ready_to_wait() {
        let options = Options::default();
        let blocked = Step::<generate_subtitles::Run>::Blocked {
            note: "no key".to_string(),
            fix: None,
        };
        let held = plan(&options, blocked, Step::<describe_ai::Run>::Wait);
        assert!(matches!(held, Plan::Blocked { .. }));
        // A switched-off step is not held by its key.
        let off = Options {
            subtitles: false,
            ..Options::default()
        };
        let blocked = Step::<generate_subtitles::Run>::Blocked {
            note: "no key".to_string(),
            fix: None,
        };
        assert!(matches!(
            plan(&off, blocked, Step::<describe_ai::Run>::Wait),
            Plan::Wait
        ));
    }

    fn result(status: ItemStatus, reason: Option<&str>) -> ItemResult {
        ItemResult {
            reason: reason.map(str::to_string),
            ..ItemResult::new(status, None)
        }
    }

    #[test]
    fn the_steps_results_add_up_to_the_clips() {
        let merged = |a: ItemResult, b: ItemResult| {
            let mut total = ItemResult::new(ItemStatus::Skipped, None);
            merge(&mut total, a);
            merge(&mut total, b);
            total
        };
        assert_eq!(
            merged(
                result(ItemStatus::Skipped, None),
                result(ItemStatus::Skipped, None)
            )
            .status,
            ItemStatus::Skipped
        );
        assert_eq!(
            merged(
                result(ItemStatus::Done, Some("clip.srt")),
                result(ItemStatus::Skipped, None)
            )
            .status,
            ItemStatus::Done
        );
        let failed = merged(
            result(ItemStatus::Done, Some("clip.srt")),
            result(ItemStatus::Failed, Some("the AI said no")),
        );
        assert_eq!(failed.status, ItemStatus::Failed, "a failed step shows");
        assert_eq!(failed.reason.as_deref(), Some("the AI said no"));
        let usage = AiUsage {
            input_tokens: 10,
            ..AiUsage::default()
        };
        let both = merged(
            ItemResult {
                usage: Some(usage),
                ..result(ItemStatus::Done, None)
            },
            ItemResult {
                usage: Some(usage),
                ..result(ItemStatus::Done, None)
            },
        );
        assert_eq!(both.usage.map(|u| u.input_tokens), Some(20));
    }

    #[test]
    fn a_folders_cache_has_a_stable_name_of_its_own() {
        assert_eq!(
            folder_key(Path::new("C:/footage/day1")),
            folder_key(Path::new("C:/footage/day1"))
        );
        assert_ne!(
            folder_key(Path::new("C:/footage/day1")),
            folder_key(Path::new("C:/footage/day2"))
        );
    }
}
