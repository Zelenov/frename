//! "Describe with AI": for each checked video, key frames (chosen where the picture changes the
//! most) and its subtitles go to Claude, and the summary and time-ranged segments it returns —
//! only what stands out, possibly none for a static or uniform clip — are written into the AI
//! block of the video's comment (see `frename_core::ai::block`). The editor's own text is never
//! touched. While tag suggestions are on (Settings) and the folder has tags of its own, the same
//! request also suggests which of them fit; they go into the block as suggestions, and the clip's
//! tags change only when the editor adds them.
//!
//! Before running, the panel shows what will be sent and about what it costs: clip lengths are
//! read in the background (see [`Options::missing_probes`]) and kept per file.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use clipscribe::{
    self as describe, AiError, AiUsage, FrameSampling, Model, MomentsMode, RetryReason, Stage,
    SummaryLanguage, MAX_DURATION_S,
};
use frename_core::ai::block;
use frename_core::ai::key::{self, KeyState};
use frename_core::{
    AppDatabase, AppStateStore, File, FileId, FileKind, FileTagger, FolderInfo, MarkerStorage,
};
use iced::widget::{column, text_editor};
use iced::Element;

use super::super::page::{self, Change};
use super::super::{ItemProgress, ItemResult, ItemStatus};
use super::{ActionMessage, Panel};
use crate::ui::layout::{self, NoticeKind};
use crate::ui::tokens::SPACE_S;
use crate::ui::{button, form, text};

/// Id of the folder notes' editor, to ask whether it has the keys.
pub const NOTES_EDITOR_ID: &str = "ai-notes-editor";

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
    /// Whether the `ApiKey` is saved, from the settings (which read it).
    KeyState(key::ApiKey, KeyState),
    /// The language descriptions are written in, from the settings.
    SetLanguage(SummaryLanguage),
    /// The model descriptions are written with, from the settings.
    SetModel(Model),
    /// Which moments descriptions get, from the settings.
    SetMoments(MomentsMode),
    /// Whether to suggest tags from the folder's tags too, from the settings.
    SetTagSuggestions(bool),
    /// The editor typed in the folder's notes for the AI (#310).
    NotesAction(text_editor::Action),
    /// The one-off line for the next run was edited (#310).
    SetOneOff(String),
}

/// What the job does to each file: the options it started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub language: SummaryLanguage,
    pub redo: bool,
    /// The model's id (see [`Model::from_id`]).
    pub model: &'static str,
    /// Only what stands out (which alone suggests an In/Out), or the whole clip.
    pub moments: MomentsMode,
    /// The folder's tags to suggest from; none when tag suggestions are off.
    pub tags: Vec<String>,
    /// The tags were asked for on their own (Suggest tags on the open clip), whatever Settings
    /// say: a clip already described, but without suggestions, is described again to get them.
    pub tags_requested: bool,
    /// The editor's notes for the AI, the folder's and the line for this run (#310); sent in
    /// every request of the run. `None` when there are none.
    pub instructions: Option<String>,
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
    moments: MomentsMode,
    /// What is known about each checked video, kept while the folder is open.
    probes: HashMap<FileId, Probe>,
    /// Videos whose length is being read.
    probing: HashSet<FileId>,
    /// Each provider's key: `None` until read (only when the action is first shown: reading may
    /// unlock a keyring). The model chosen decides which one counts ([`Self::key`]).
    anthropic_key: Option<KeyState>,
    openai_key: Option<KeyState>,
    gemini_key: Option<KeyState>,
    /// The key's state has been asked for; the answer is on its way.
    key_requested: bool,
    /// Suggest tags from the folder's tags too (Settings).
    suggest_tags: bool,
    /// The next run is a "Suggest tags" of the open clip: it suggests whatever Settings say.
    tags_requested: bool,
    /// The open folder's own tags, kept up to date by the folder workspace.
    folder_tags: Vec<String>,
    /// The folder's notes for the AI (#310), edited in the panel and kept in its `.frename`.
    notes: Notes,
    /// An extra line for the next run only; never saved.
    one_off: String,
}

/// The folder's notes in their editor. The editor's content cannot be cloned, so a clone is a
/// new editor with the same text.
#[derive(Debug, Default)]
struct Notes(text_editor::Content);

impl Clone for Notes {
    fn clone(&self) -> Self {
        Self(text_editor::Content::with_text(&self.0.text()))
    }
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
            Message::KeyState(which, state) => match which {
                key::ApiKey::Anthropic => self.anthropic_key = Some(state),
                key::ApiKey::OpenAi => self.openai_key = Some(state),
                key::ApiKey::Gemini => self.gemini_key = Some(state),
                key::ApiKey::Soniox => {}
            },
            Message::SetLanguage(language) => self.language = language,
            Message::SetModel(model) => self.model = model,
            Message::SetMoments(moments) => self.moments = moments,
            Message::SetTagSuggestions(on) => self.suggest_tags = on,
            Message::NotesAction(action) => {
                // The cap is the library's: a longer text is typed but refused, so a run
                // cannot be started with it (the panel says so).
                self.notes.0.perform(action);
            }
            Message::SetOneOff(line) => self.one_off = line,
        }
    }

    /// The open folder's notes for the AI, as saved in its `.frename`: replaces what the editor
    /// holds when a folder opens.
    pub fn set_folder_notes(&mut self, notes: &str) {
        self.notes = Notes(text_editor::Content::with_text(notes));
    }

    /// The folder's notes as typed, without the blank lines at the ends.
    pub fn folder_notes(&self) -> String {
        self.notes.0.text().trim().to_string()
    }

    /// Forget the one-off line once a run has taken it.
    pub fn clear_one_off(&mut self) {
        self.one_off.clear();
    }

    /// What every request of the next run carries (#310): the folder's notes, then the line for
    /// this run. `None` when both are empty.
    fn instructions(&self) -> Option<String> {
        instructions_of(&self.folder_notes(), &self.one_off)
    }

    /// Why the notes cannot be sent: they are longer than the library takes.
    fn notes_too_long(&self) -> Option<usize> {
        let chars = self.instructions()?.chars().count();
        (chars > describe::MAX_INSTRUCTIONS_CHARS).then_some(chars)
    }

    /// The open folder's own tags: what a run may suggest from.
    pub fn set_folder_tags(&mut self, tags: Vec<String>) {
        self.folder_tags = tags;
    }

    /// The next run is (or is not) a "Suggest tags" of the open clip (#250).
    pub fn request_tags(&mut self, on: bool) {
        self.tags_requested = on;
    }

    /// The tags a run suggests from: the folder's, while tag suggestions are on or asked for.
    fn tags_to_suggest(&self) -> &[String] {
        if self.suggest_tags || self.tags_requested {
            &self.folder_tags
        } else {
            &[]
        }
    }

    pub fn operation(&self) -> super::Operation {
        super::Operation::DescribeAi(self.run())
    }

    /// The run these options make.
    pub fn run(&self) -> Run {
        Run {
            instructions: self.instructions(),
            ..self.run_with(self.redo)
        }
    }

    /// The run these options make, with Redo set by the caller: Prepare clips never redoes, whatever
    /// the Describe with AI panel remembers.
    pub fn run_with(&self, redo: bool) -> Run {
        Run {
            language: self.language,
            redo,
            model: self.model.id,
            moments: self.moments,
            tags: self.tags_to_suggest().to_vec(),
            tags_requested: self.tags_requested,
            // The line for one run belongs to the Describe with AI panel's own run.
            instructions: instructions_of(&self.folder_notes(), ""),
        }
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
        let needed = (self.anthropic_key.is_none()
            || self.openai_key.is_none()
            || self.gemini_key.is_none())
            && !self.key_requested;
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
        self.plan_with(checked, self.redo)
    }

    fn plan_with(&self, checked: &[&File], redo: bool) -> Plan {
        let mut plan = Plan {
            checked_videos: checked
                .iter()
                .filter(|f| f.kind() == FileKind::Video)
                .count(),
            ..Plan::default()
        };
        let vocabulary = vocabulary(self.tags_to_suggest());
        // The notes ride along in every request, like subtitles.
        let notes_bytes = self.instructions().map_or(0, |notes| notes.len());
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
            // Suggest tags on a clip that is described but has no suggestions describes it again.
            let redo = redo || (self.tags_requested && !has_suggestions(file.comment()));
            if !redo && block::ai_block(file.comment()).is_some() {
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
                    plan.usage += if vocabulary.is_empty() {
                        describe::estimate_usage(self.model, d, probe.subtitle_bytes + notes_bytes)
                    } else {
                        describe::estimate_tags_usage(
                            self.model,
                            d,
                            probe.subtitle_bytes + notes_bytes,
                            &vocabulary,
                            None,
                        )
                    };
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

    /// The state of the key of the chosen model's provider.
    fn key(&self) -> Option<KeyState> {
        match frename_core::ai::key_of(self.model) {
            key::ApiKey::OpenAi => self.openai_key,
            key::ApiKey::Gemini => self.gemini_key,
            _ => self.anthropic_key,
        }
    }

    /// The company of the chosen model, by name.
    pub fn provider_label(&self) -> &'static str {
        self.model.provider.label()
    }

    /// Whether the settings said no key is saved for the chosen model: the action list shows
    /// "No key".
    pub fn key_missing(&self) -> bool {
        self.key() == Some(KeyState::Missing)
    }

    /// Whether a run on `checked` can start now, and if not, why (#249).
    pub fn readiness(&self, checked: &[&File]) -> super::Readiness {
        self.step(checked).readiness()
    }

    /// Where a run on `checked` stands as a step of a longer run (#17).
    pub fn step(&self, checked: &[&File]) -> super::Step<Run> {
        self.step_with(checked, self.redo)
    }

    /// [`Self::step`] with Redo set by the caller (see [`Self::run_with`]).
    pub fn step_with(&self, checked: &[&File], redo: bool) -> super::Step<Run> {
        use super::Step;
        let blocked = |note: String| Step::Blocked {
            note,
            fix: Some(ActionMessage::OpenAiSettings),
        };
        match self.key() {
            None => return Step::Wait,
            Some(KeyState::Missing) => {
                return blocked(fl!(
                    "batch-ai-key-missing",
                    provider = self.provider_label()
                ))
            }
            Some(KeyState::Unavailable) => return blocked(fl!("batch-ai-key-unavailable")),
            Some(KeyState::Saved) => {}
        }
        let plan = self.plan_with(checked, redo);
        if plan.estimating {
            return Step::Wait;
        }
        if plan.send.is_empty() {
            return Step::Skip(
                plan.skipped_line()
                    .unwrap_or_else(|| fl!("batch-action-describe-ai-none")),
            );
        }
        Step::Ready(self.run_with(redo))
    }

    /// What a run on `checked` would send and cost, for a page that adds up several steps:
    /// videos, seconds of footage, dollars, seconds it takes. `None` while it is estimated.
    pub fn totals(&self, checked: &[&File]) -> Option<(usize, f64, f64, f64)> {
        let plan = self.plan_with(checked, false);
        (!plan.estimating).then(|| {
            (
                plan.send.len(),
                plan.seconds,
                self.model.cost_usd(plan.usage),
                plan.run_seconds,
            )
        })
    }

    /// The page for `checked` and its run button (`Describe 12 videos · about $0.35` once the
    /// estimate and the key are known). The plan is made once.
    pub fn panel(&self, checked: &[&File]) -> Panel<'_> {
        let plan = self.plan(checked);
        let too_long = self.notes_too_long();
        let ready = !plan.estimating
            && !plan.send.is_empty()
            && self.key() == Some(KeyState::Saved)
            && too_long.is_none();
        let reason = if too_long.is_some() {
            Some(fl!("batch-ai-notes-too-long-reason"))
        } else if plan.estimating {
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
            page::linked_row(
                fl!("batch-option-tag-suggestions"),
                self.tag_suggestions_text(),
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
            self.notes_row(plan),
            page::option_row(
                fl!("batch-option-notes-once"),
                form::text_field(&fl!("batch-ai-notes-once-placeholder"), &self.one_off)
                    .on_input(|line| ActionMessage::DescribeAi(Message::SetOneOff(line)))
                    .width(iced::Length::Fill),
            ),
        ];
        page::page(
            label(),
            fl!(
                "batch-action-describe-ai-hint-panel",
                provider = self.provider_label()
            ),
            &[Change::IntoComments],
            settings
                .into_iter()
                .chain(std::iter::once(self.estimate(plan)))
                .chain(self.key_notice()),
        )
    }

    /// The folder's notes for the AI: the editor, and what it costs to send them.
    fn notes_row(&self, plan: &Plan) -> Element<'_, ActionMessage> {
        let editor = form::text_area(&self.notes.0, &fl!("batch-ai-notes-placeholder"))
            .id(iced::widget::Id::new(NOTES_EDITOR_ID))
            .on_action(|action| ActionMessage::DescribeAi(Message::NotesAction(action)));
        let hint = match self.notes_too_long() {
            Some(chars) => text::error(fl!(
                "batch-ai-notes-too-long",
                chars = (chars as i64),
                max = (describe::MAX_INSTRUCTIONS_CHARS as i64)
            )),
            None => text::secondary(fl!("batch-ai-notes-hint", provider = self.provider_label())),
        };
        // Notes about what is said need the words: say so for clips without subtitles.
        let no_subtitles = (talks_about_speech(&self.instructions().unwrap_or_default())
            && plan.no_subtitles > 0)
            .then(|| {
                text::secondary(fl!(
                    "batch-ai-notes-no-subtitles",
                    videos = videos(plan.no_subtitles)
                ))
            });
        page::option_row_with_info(
            fl!("batch-option-notes"),
            fl!("batch-ai-notes-info"),
            column![editor, hint].push(no_subtitles).spacing(SPACE_S),
        )
    }

    /// What the tag suggestions row says: off, or how many of the folder's tags it suggests from.
    fn tag_suggestions_text(&self) -> String {
        if !self.suggest_tags {
            fl!("batch-ai-tag-suggestions-off")
        } else if self.folder_tags.is_empty() {
            fl!("batch-ai-tag-suggestions-no-tags")
        } else {
            fl!(
                "batch-ai-tag-suggestions-on",
                tags = (self.folder_tags.len() as i64)
            )
        }
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
    pub(super) fn key_notice(&self) -> Option<Element<'_, ActionMessage>> {
        let (kind, headline, fix) = match self.key() {
            Some(KeyState::Missing) => (
                NoticeKind::Error,
                fl!("batch-ai-key-missing", provider = self.provider_label()),
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

/// Words in notes about what is said.
const SPEECH_WORDS: &str = "take line said says speak speech repeat words";

/// Whether the notes seem to be about what is said (takes, lines, repeats): they need the clip's
/// subtitles. A rough test of English words; another language's notes simply get no hint.
fn talks_about_speech(notes: &str) -> bool {
    let notes = notes.to_lowercase();
    SPEECH_WORDS
        .split_whitespace()
        .any(|word| notes.contains(word))
}

/// The folder's notes and the line for one run as the one text a request carries.
fn instructions_of(folder_notes: &str, one_off: &str) -> Option<String> {
    let (notes, line) = (folder_notes.trim(), one_off.trim());
    match (notes.is_empty(), line.is_empty()) {
        (true, true) => None,
        (false, true) => Some(notes.to_string()),
        (true, false) => Some(line.to_string()),
        (false, false) => Some(format!("{notes}\nFor this clip or run only: {line}")),
    }
}

/// `tags` as clipscribe's vocabulary: names only, frename keeps no hints.
fn vocabulary(tags: &[String]) -> Vec<describe::Tag> {
    tags.iter()
        .map(|name| describe::Tag {
            name: name.clone(),
            hint: None,
        })
        .collect()
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
    let chosen = AppDatabase::new().get_subtitle_choice(path);
    // A debug build renames in memory only: GStreamer needs the name on disk.
    let path = &FileTagger::disk_path(path);
    Probe {
        duration_s: describe::frames::clip_duration_s(path),
        subtitle_bytes: frename_core::subtitle_bytes(path, chosen),
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

/// What an item says while its request waits `after` before it is sent again: the whole seconds
/// of the wait, rounded up, once (it is not counted down).
fn retry_label(after: std::time::Duration, reason: RetryReason) -> String {
    let seconds = (after.as_secs() + u64::from(after.subsec_nanos() > 0)).max(1) as i64;
    match reason {
        RetryReason::RateLimit => fl!("batch-ai-progress-retry-rate-limit", seconds = seconds),
        _ => fl!("batch-ai-progress-retry-temporary", seconds = seconds),
    }
}

/// Demo: `progress` as it is while the request waits `seconds` for a rate limit.
pub fn show_retry(progress: &ItemProgress, seconds: u64) {
    let after = std::time::Duration::from_secs(seconds);
    progress.set(
        0.45,
        fl!(
            "batch-ai-progress-waiting",
            provider = clipscribe::Provider::Anthropic.label()
        ),
    );
    progress.note(retry_label(after, RetryReason::RateLimit), after * 3600);
}

/// Describe the video at `path` and write the description into its comment, and its moments
/// into the video as markers while markers are kept there (Settings), where Premiere Pro shows
/// them on the clip; with markers kept in comments, into the comment too.
pub fn run(options: &Run, path: &Path, cancel: &AtomicBool, progress: &ItemProgress) -> ItemResult {
    run_clip(
        options,
        &describe::Budget::new(None),
        None,
        path,
        cancel,
        progress,
    )
}

/// [`run`] within `budget` (a cap shared by the clips of a run: a clip whose request could take
/// the run past it is not sent, and the job stops). The described clip is also kept in `cache`
/// (its fingerprints, for grouping similar footage); a clip the cache already has, described
/// with the same settings, is not asked for again, so a retry after a failed save costs nothing.
pub fn run_clip(
    options: &Run,
    budget: &describe::Budget,
    cache: Option<&describe::Cache>,
    path: &Path,
    cancel: &AtomicBool,
    progress: &ItemProgress,
) -> ItemResult {
    let is_video = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| FileKind::from_extension(e) == FileKind::Video);
    if !is_video {
        return ItemResult::new(ItemStatus::Skipped, None);
    }
    // Read from the file itself, so a comment the folder had not loaded yet is kept.
    let mut snapshot = FileTagger::parse(path, &FolderInfo::default());
    // Suggest tags on a clip that is described but has no suggestions yet describes it again.
    let redo = options.redo || (options.tags_requested && !has_suggestions(snapshot.comment()));
    if !redo && block::ai_block(snapshot.comment()).is_some() {
        return ItemResult::new(ItemStatus::Skipped, None);
    }
    // Removed in the settings while the job runs: every later video would fail the same way.
    let Some(request) = Request::for_clip(
        path,
        options.model(),
        options.language,
        options.moments,
        options.instructions.as_deref(),
    ) else {
        return ItemResult {
            stop_job: Some(fl!(
                "batch-ai-stop-no-key",
                provider = options.model().provider.label()
            )),
            ..ItemResult::failed(fl!("batch-ai-fail-no-key"))
        };
    };
    let Request {
        on_disk,
        subtitles,
        options: describe_options,
    } = request;
    // The file's share of reading frames, as the estimate counts it; waiting for the answer
    // takes the rest.
    let mut asked_at = 0.0;
    let on_stage = |stage: Stage| match stage {
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
            fl!(
                "batch-ai-progress-waiting",
                provider = options.model().provider.label()
            ),
        ),
        // The wait is said for as long as it lasts; the bar keeps creeping, and the "waiting"
        // text comes back when the request is sent again.
        Stage::Retrying { after, reason } => progress.note(retry_label(after, reason), after),
        // A stage this version does not know keeps the text it has.
        _ => {}
    };
    let vocabulary = vocabulary(&options.tags);
    let vocabulary_ref = (!vocabulary.is_empty()).then_some(vocabulary.as_slice());
    // Described before, with these settings, but not saved (a read-only share): no second request.
    let cached = cache
        .filter(|_| !redo)
        .and_then(|cache| reusable(cache, &on_disk, &describe_options, vocabulary_ref));
    // One request either way: with tags to suggest from, it suggests them too.
    let described = match cached {
        Some(clip) => Ok(Some(clip)),
        None => describe::describe_clip(
            &on_disk,
            &subtitles,
            vocabulary_ref,
            &describe_options,
            budget,
            cancel,
            on_stage,
        ),
    };
    let described_clip: describe::DescribedClip;
    let (description, tags, usage, frames) = match described {
        // Its frames were read, but the request could pass the cap: nothing was sent, and the
        // clips after it would be refused the same way.
        Ok(None) => return over_budget(budget),
        Ok(Some(clip)) => {
            described_clip = clip.clone();
            (
                clip.description,
                clip.tags.filter(|_| !vocabulary.is_empty()),
                clip.usage,
                clip.frames.len(),
            )
        }
        Err(describe::Error::Cancelled) => return ItemResult::new(ItemStatus::Pending, None),
        Err(describe::Error::TooLong(_)) => return ItemResult::new(ItemStatus::Skipped, None),
        Err(e @ describe::Error::Unreadable(_)) => {
            return ItemResult::failed(failure_reason(&e, path));
        }
        Err(describe::Error::Ai(e)) => return ai_failed(e, options.model().provider),
        Err(describe::Error::BadAnswer { reason, usage }) => {
            return ItemResult {
                usage: Some(usage),
                ..ItemResult::failed(reason)
            }
        }
        // The panel keeps a run with notes over the cap from starting; fail the item with the
        // library's text if one gets here anyway.
        Err(e @ describe::Error::InstructionsTooLong { .. }) => {
            return ItemResult::failed(e.to_string());
        }
    };
    progress.set(0.98, fl!("batch-ai-progress-saving"));
    let mut new_block = match frename_core::marker_storage() {
        MarkerStorage::Comment => block::format_block(&description),
        MarkerStorage::InVideo => {
            // Markers first, before the comment's save may rename the file.
            let segments = block::segment_lines(&description);
            match FileTagger::save_ai_markers(path, &segments) {
                Ok(()) => block::format_summary_block(&description),
                // A format without markers, or a file in use: the moments go into the
                // comment instead, so the paid answer is not lost.
                Err(e) => {
                    log::warn!(
                        "ai: moments of {} not written as markers: {e}",
                        path.display()
                    );
                    block::format_block(&description)
                }
            }
        }
    };
    if let Some(tags) = &tags {
        block::push_tag_lines(&mut new_block, tags);
    }
    snapshot.set_comment(block::replace_block(snapshot.comment(), &new_block));
    let new_path = FileTagger::save(&snapshot, path);
    log::info!(
        "ai: described {} ({} frames, {} in / {} out tokens)",
        new_path.display(),
        frames,
        usage.input_tokens,
        usage.output_tokens
    );
    let usage = Some(usage);
    // A save that failed (a read-only share) only logs: check the description is there, so a
    // paid answer that was lost is reported, not counted as done.
    let (new_path, saved) = super::reparsed(new_path);
    // Kept under the file as it is now: the save wrote into it (XMP), which changes the identity
    // the cache finds it by, so a key made before the save would never be found again.
    if let Some(cache) = cache {
        let saved_on_disk = FileTagger::disk_path(&new_path);
        keep_in_cache(
            cache,
            &saved_on_disk,
            &describe_options,
            &vocabulary,
            &described_clip,
        );
    }
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

/// Whether the comment already holds tag suggestions or tag ideas from an earlier run.
fn has_suggestions(comment: &str) -> bool {
    !block::suggested_tags(comment).is_empty() || !block::tag_ideas(comment).is_empty()
}

/// Keep the described clip in `cache` (its fingerprints are what grouping reads later); a cache
/// that cannot be written only logs: the description itself is saved to the clip.
fn keep_in_cache(
    cache: &describe::Cache,
    on_disk: &Path,
    options: &describe::Options,
    vocabulary: &[describe::Tag],
    clip: &describe::DescribedClip,
) {
    let key = describe::CacheKey::new(
        on_disk,
        options,
        (!vocabulary.is_empty()).then_some(vocabulary),
        true,
    );
    let record = key.map(|key| describe::ClipRecord {
        file: on_disk.to_path_buf(),
        key,
        model: options.model.id.to_string(),
        clip: clip.clone(),
    });
    match record {
        Ok(record) => {
            if let Err(e) = cache.put(&record) {
                log::warn!("ai: {} not kept in the cache: {e}", on_disk.display());
            }
        }
        Err(e) => log::warn!(
            "ai: {} cannot be keyed for the cache: {e}",
            on_disk.display()
        ),
    }
}

/// The clip at `path` as `cache` has it for the settings of `options`: its fingerprints, for
/// grouping. `None` when it was not described with them, or the key is gone.
pub fn cached_clip(
    options: &Run,
    cache: &describe::Cache,
    path: &Path,
) -> Option<describe::DescribedClip> {
    let request = Request::for_clip(
        path,
        options.model(),
        options.language,
        options.moments,
        options.instructions.as_deref(),
    )?;
    let vocabulary = vocabulary(&options.tags);
    cached_in(
        cache,
        &request.on_disk,
        &request.options,
        (!vocabulary.is_empty()).then_some(vocabulary.as_slice()),
    )
}

/// A description the cache keeps for this clip and these settings, to use again at no cost: what
/// it cost was paid when it was made.
fn reusable(
    cache: &describe::Cache,
    on_disk: &Path,
    options: &describe::Options,
    vocabulary: Option<&[describe::Tag]>,
) -> Option<describe::DescribedClip> {
    let mut clip = cached_in(cache, on_disk, options, vocabulary)?;
    clip.usage = Default::default();
    Some(clip)
}

/// The result of a clip whose request could take the run past its cap: not sent, not reached, and
/// the clips after it would be refused the same way, so the job stops.
fn over_budget(budget: &describe::Budget) -> ItemResult {
    ItemResult {
        stop_job: Some(over_budget_text(budget)),
        ..ItemResult::new(ItemStatus::Pending, None)
    }
}

fn cached_in(
    cache: &describe::Cache,
    on_disk: &Path,
    options: &describe::Options,
    vocabulary: Option<&[describe::Tag]>,
) -> Option<describe::DescribedClip> {
    let key = describe::CacheKey::new(on_disk, options, vocabulary, true).ok()?;
    cache.get(&key).map(|record| record.clip)
}

/// Why a run stopped before the next clip: what the cap is, what was spent, and what to do.
pub fn over_budget_text(budget: &describe::Budget) -> String {
    fl!(
        "batch-stop-over-budget",
        cap = dollars(budget.max_usd().unwrap_or_default()),
        spent = dollars(budget.spent_usd())
    )
}

/// The result of a file whose request failed; some failures stop the whole job, and losing
/// the connection stops it after a few files in a row.
fn ai_failed(error: AiError, provider: clipscribe::Provider) -> ItemResult {
    log::warn!("ai: request failed: {error:?}");
    let offline = matches!(error, AiError::Network(_) | AiError::Timeout);
    ItemResult {
        // A request that timed out may have been answered and billed after all.
        usage_unknown: error == AiError::Timeout,
        out_of_credit: matches!(error, AiError::OutOfCredit(_)),
        key_rejected: matches!(error, AiError::KeyRejected(_)),
        stop_job: error.stops_job(),
        stop_if_repeated: offline
            .then(|| fl!("batch-ai-stop-offline", provider = provider.label())),
        ..ItemResult::failed(error.reason())
    }
}

/// What a request about one clip sends besides its frames: the same for a batch run and for
/// "Describe with AI" on a marker, so the two cannot drift apart.
pub struct Request {
    /// Where the clip is on disk: a debug build renames in memory only, so the clip and its
    /// subtitles are read by the name on disk.
    pub on_disk: PathBuf,
    /// The clip's `.srt`, if it has one (none when it cannot be read).
    pub subtitles: Vec<describe::Cue>,
    pub options: describe::Options,
}

impl Request {
    /// The request for the clip at `path` with `model`, `language` and `moments`; `None` when
    /// no key is saved for the model's provider. Reads the key and the subtitles: blocking.
    pub fn for_clip(
        path: &Path,
        model: Model,
        language: SummaryLanguage,
        moments: MomentsMode,
        instructions: Option<&str>,
    ) -> Option<Self> {
        let api_key = key::read_key(frename_core::ai::key_of(model))?;
        let on_disk = FileTagger::disk_path(path);
        let chosen = AppDatabase::new().get_subtitle_choice(path);
        let subtitles = frename_core::subtitle_cues(&on_disk, chosen);
        Some(Self {
            on_disk,
            subtitles,
            options: describe::Options {
                api_key,
                model,
                language,
                frame_sampling: FrameSampling::KeyFrames,
                moments,
                instructions: instructions.map(str::to_string),
            },
        })
    }
}

/// Why a request about the clip at `path` failed, as the batch report and the marker notice
/// say it: the AI's own reason (a rejected key, no credit left, the network…), or the clip's.
/// A cancelled request has no reason to show: its callers handle it before asking.
pub fn failure_reason(error: &describe::Error, path: &Path) -> String {
    match error {
        describe::Error::Unreadable(e) => {
            log::warn!("ai: cannot read {}: {e}", path.display());
            fl!("batch-ai-fail-unreadable")
        }
        describe::Error::TooLong(_) => fl!("batch-ai-fail-too-long"),
        describe::Error::Ai(e) => e.reason(),
        describe::Error::BadAnswer { reason, .. } => reason.clone(),
        describe::Error::Cancelled => String::new(),
        describe::Error::InstructionsTooLong { .. } => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_providers_key_and_credit_errors_each_get_their_own_fix() {
        use clipscribe::Provider;
        for provider in [Provider::Anthropic, Provider::OpenAi, Provider::Gemini] {
            let label = provider.label().to_string();
            let rejected = super::ai_failed(AiError::KeyRejected(label.clone()), provider);
            assert!(rejected.key_rejected && !rejected.out_of_credit, "{label}");
            assert!(rejected.stop_job.is_some(), "a rejected key stops the job");
            let broke = super::ai_failed(AiError::OutOfCredit(label.clone()), provider);
            assert!(broke.out_of_credit && !broke.key_rejected, "{label}");
            let busy = super::ai_failed(AiError::Network("x".into()), provider);
            assert!(!busy.out_of_credit && !busy.key_rejected, "{label}");
        }
    }

    use super::*;
    use std::time::SystemTime;

    /// A description kept in the cache is used again for nothing, and a clip over the cap is
    /// neither sent nor reached and stops the job.
    #[test]
    fn a_kept_description_costs_nothing_again_and_an_over_budget_clip_is_not_reached() {
        let dir = frename_core::test_support::fresh_dir("describe-reuse");
        let video = dir.join("clip.mp4");
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("crates/frename-core/tests/fixtures/tiny.mov");
        std::fs::copy(source, &video).expect("clip");
        let cache = describe::Cache::open(&dir.join("cache.jsonl")).expect("cache");
        let options = describe::Options {
            api_key: String::new(),
            model: describe::MODELS[0],
            language: Default::default(),
            frame_sampling: Default::default(),
            instructions: None,
            moments: Default::default(),
        };
        assert!(
            reusable(&cache, &video, &options, None).is_none(),
            "nothing kept yet"
        );

        let clip = describe::DescribedClip {
            description: describe::Description {
                summary: "A lion.".to_string(),
                segments: Vec::new(),
                main: None,
            },
            tags: None,
            usage: clipscribe::AiUsage {
                input_tokens: 1000,
                output_tokens: 200,
            },
            duration_s: 1.0,
            frames: Vec::new(),
        };
        keep_in_cache(&cache, &video, &options, &[], &clip);
        let again = reusable(&cache, &video, &options, None).expect("kept");
        assert_eq!(again.description, clip.description);
        assert_eq!(again.usage, clipscribe::AiUsage::default(), "paid once");

        let budget = describe::Budget::new(Some(0.5));
        let result = over_budget(&budget);
        assert_eq!(result.status, ItemStatus::Pending);
        assert!(result.stop_job.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

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

    /// A run of one clip, started without the panel, waits for the key and the length, and
    /// says why it cannot start (#249).
    #[test]
    fn a_run_without_a_panel_waits_for_its_reads_and_says_why_it_cannot_start() {
        use super::super::Readiness;
        let files = [file("a.mp4", ""), file("b.mp4", BLOCK)];
        let mut options = Options::default();
        let which = frename_core::ai::key_of(options.model);
        let plain: Vec<&File> = vec![&files[0]];
        let described: Vec<&File> = vec![&files[1]];

        assert!(matches!(options.readiness(&plain), Readiness::Waiting));
        options.update(Message::KeyState(which, KeyState::Missing));
        assert!(matches!(
            options.readiness(&plain),
            Readiness::Blocked { fix: Some(_), .. }
        ));
        options.update(Message::KeyState(which, KeyState::Saved));
        assert!(matches!(options.readiness(&plain), Readiness::Waiting));
        options.update(Message::Probed(vec![(files[0].id(), probe(Some(60.0)))]));
        assert!(matches!(options.readiness(&plain), Readiness::Ready));
        // Already described: nothing to send, and it says so.
        assert!(matches!(
            options.readiness(&described),
            Readiness::Blocked { fix: None, .. }
        ));
    }

    #[test]
    fn a_retry_wait_is_told_in_whole_seconds_with_its_reason() {
        use std::time::Duration;
        let text = |ms, reason| retry_label(Duration::from_millis(ms), reason);
        let strip = |s: String| s.replace(['\u{2068}', '\u{2069}'], "");
        assert_eq!(
            strip(text(8000, RetryReason::RateLimit)),
            "retrying in 8 s (rate limit)"
        );
        assert_eq!(
            strip(text(2100, RetryReason::Temporary)),
            "retrying in 3 s (temporary failure)"
        );
        assert_eq!(
            strip(text(0, RetryReason::RateLimit)),
            "retrying in 1 s (rate limit)"
        );
    }

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
        options.update(Message::KeyState(key::ApiKey::Anthropic, KeyState::Missing));
        assert!(!run_button(&options, &checked).1);
        options.update(Message::KeyState(key::ApiKey::Anthropic, KeyState::Saved));
        assert!(run_button(&options, &checked).1);
    }

    #[test]
    fn the_key_that_counts_is_the_one_of_the_chosen_models_provider() {
        let files = [file("a.mp4", "")];
        let checked: Vec<&File> = files.iter().collect();
        let mut options = Options::default();
        options.update(Message::Probed(vec![(files[0].id(), probe(Some(10.0)))]));
        options.update(Message::KeyState(key::ApiKey::Anthropic, KeyState::Saved));
        options.update(Message::KeyState(key::ApiKey::OpenAi, KeyState::Missing));
        assert!(run_button(&options, &checked).1, "Claude with its key");
        assert!(!options.key_missing());

        options.update(Message::SetModel(frename_core::ai::model_from_id(
            "gpt-4.1",
        )));
        assert!(!run_button(&options, &checked).1, "ChatGPT without its key");
        assert!(options.key_missing());
        assert_eq!(options.provider_label(), "OpenAI");

        options.update(Message::KeyState(key::ApiKey::OpenAi, KeyState::Saved));
        assert!(run_button(&options, &checked).1);
        assert!(!options.key_missing());
    }

    #[test]
    fn a_run_suggests_from_the_folders_tags_only_while_suggestions_are_on() {
        let files = [file("a.mp4", "")];
        let checked: Vec<&File> = files.iter().collect();
        let mut options = Options::default();
        options.update(Message::Probed(vec![(files[0].id(), probe(Some(60.0)))]));
        let tags_of = |options: &Options| match options.operation() {
            super::super::Operation::DescribeAi(run) => run.tags,
            _ => unreachable!(),
        };
        let without = options.plan(&checked).usage;
        assert_eq!(
            options.tag_suggestions_text(),
            fl!("batch-ai-tag-suggestions-off")
        );

        options.update(Message::SetTagSuggestions(true));
        assert_eq!(
            options.tag_suggestions_text(),
            fl!("batch-ai-tag-suggestions-no-tags")
        );
        assert!(tags_of(&options).is_empty(), "the folder has no tags");

        options.set_folder_tags(vec!["night".to_string(), "beach".to_string()]);
        assert_eq!(tags_of(&options), ["night", "beach"]);
        assert_eq!(
            options.tag_suggestions_text(),
            fl!("batch-ai-tag-suggestions-on", tags = 2)
        );
        let with = options.plan(&checked).usage;
        assert!(
            with.input_tokens > without.input_tokens && with.output_tokens > without.output_tokens,
            "the estimate counts the tags sent and the suggestions that come back"
        );

        options.update(Message::SetTagSuggestions(false));
        assert!(tags_of(&options).is_empty());
        assert_eq!(options.plan(&checked).usage, without);
    }

    #[test]
    fn a_suggest_tags_run_suggests_whatever_the_settings_say() {
        let mut options = Options::default();
        options.set_folder_tags(vec!["night".to_string(), "beach".to_string()]);
        let run_of = |options: &Options| match options.operation() {
            super::super::Operation::DescribeAi(run) => run,
            _ => unreachable!(),
        };
        let plain = run_of(&options);
        assert!(
            plain.tags.is_empty() && !plain.tags_requested,
            "Settings say no"
        );

        options.request_tags(true);
        let asked = run_of(&options);
        assert_eq!(asked.tags, ["night", "beach"]);
        assert!(asked.tags_requested);

        options.request_tags(false);
        assert_eq!(run_of(&options), plain, "the next run is an ordinary one");
    }

    #[test]
    fn suggest_tags_sends_a_described_clip_that_has_no_suggestions_yet() {
        let described = file("a.mp4", "AI: A walk.");
        let suggested = file("b.mp4", "AI: A walk.\nSuggested tags: night (90%)");
        let checked = [&described, &suggested];
        let mut options = Options::default();
        options.set_folder_tags(vec!["night".to_string()]);
        for file in checked {
            options.update(Message::Probed(vec![(file.id(), probe(Some(60.0)))]));
        }
        let plain = options.plan(&checked);
        assert_eq!(plain.described, 2);
        assert!(
            plain.send.is_empty(),
            "an ordinary run leaves described clips alone"
        );

        options.request_tags(true);
        let asked = options.plan(&checked);
        assert_eq!(
            asked.send,
            [described.id()],
            "the clip with no suggestions is sent"
        );
        assert_eq!(asked.described, 1, "the one that has them is not");
    }

    #[test]
    fn a_comment_has_suggestions_when_it_holds_tags_or_ideas() {
        assert!(!has_suggestions("Mine\n\nAI: A walk."));
        assert!(has_suggestions("AI: A walk.\nSuggested tags: night (90%)"));
        assert!(has_suggestions("AI: A walk.\nTag ideas: kayak race"));
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
            &Run {
                language: SummaryLanguage::English,
                redo: false,
                model: Model::default().id,
                moments: MomentsMode::default(),
                tags: Vec::new(),
                tags_requested: false,
                instructions: None,
            },
            Path::new("C:/clips/photo.jpg"),
            &AtomicBool::new(false),
            &ItemProgress::default(),
        );
        assert_eq!(result.status, ItemStatus::Skipped);
    }

    #[test]
    fn the_moments_set_in_the_settings_go_into_the_run() {
        let mut options = Options::default();
        let moments = |options: &Options| match options.operation() {
            super::super::Operation::DescribeAi(run) => run.moments,
            other => panic!("not a Describe with AI run: {other:?}"),
        };
        assert_eq!(moments(&options), MomentsMode::Important, "the default");
        options.update(Message::SetMoments(MomentsMode::Full));
        assert_eq!(moments(&options), MomentsMode::Full);
    }

    fn type_notes(options: &mut Options, text: &str) {
        options.update(Message::NotesAction(text_editor::Action::Edit(
            text_editor::Edit::Paste(std::sync::Arc::new(text.to_string())),
        )));
    }

    fn run_of(options: &Options) -> Run {
        match options.operation() {
            super::super::Operation::DescribeAi(run) => run,
            other => panic!("not a Describe with AI run: {other:?}"),
        }
    }

    #[test]
    fn the_notes_and_the_one_off_line_make_one_text_for_every_request_of_the_run() {
        let mut options = Options::default();
        assert_eq!(
            run_of(&options).instructions,
            None,
            "no notes, nothing extra"
        );

        type_notes(&mut options, "The man is Eugene.\n");
        assert_eq!(
            run_of(&options).instructions.as_deref(),
            Some("The man is Eugene."),
            "trimmed"
        );
        options.update(Message::SetOneOff("the woman in red is Ana".into()));
        // Prepare clips carries the folder's notes, never the line for one run.
        assert_eq!(
            options.run_with(false).instructions.as_deref(),
            Some("The man is Eugene.")
        );
        let text = run_of(&options).instructions.expect("both");
        assert!(text.starts_with("The man is Eugene.\n"), "{text}");
        assert!(text.ends_with("the woman in red is Ana"), "{text}");

        options.clear_one_off();
        assert_eq!(
            run_of(&options).instructions.as_deref(),
            Some("The man is Eugene.")
        );
        options.update(Message::SetOneOff("only this".into()));
        options.set_folder_notes("");
        assert_eq!(run_of(&options).instructions.as_deref(), Some("only this"));
    }

    #[test]
    fn a_folder_opened_replaces_the_notes_in_the_editor() {
        let mut options = Options::default();
        options.set_folder_notes("Mark the moon.");
        assert_eq!(options.folder_notes(), "Mark the moon.");
        options.set_folder_notes("");
        assert_eq!(options.folder_notes(), "");
        // A clone is an editor with the same text.
        type_notes(&mut options, "Suggest tea.");
        assert_eq!(options.clone().folder_notes(), "Suggest tea.");
    }

    #[test]
    fn the_notes_are_counted_in_the_price() {
        let files = [file("a.mp4", "")];
        let checked: Vec<&File> = files.iter().collect();
        let mut options = Options::default();
        options.update(Message::Probed(vec![(files[0].id(), probe(Some(60.0)))]));
        let without = options.plan(&checked).usage;
        type_notes(&mut options, &"Mark the moon. ".repeat(100));
        let with = options.plan(&checked).usage;
        assert!(
            with.input_tokens > without.input_tokens,
            "{with:?} {without:?}"
        );
        assert_eq!(with.output_tokens, without.output_tokens);
    }

    #[test]
    fn notes_over_the_library_cap_keep_the_run_off() {
        let files = [file("a.mp4", "")];
        let checked: Vec<&File> = files.iter().collect();
        let mut options = Options::default();
        options.update(Message::Probed(vec![(files[0].id(), probe(Some(10.0)))]));
        options.update(Message::KeyState(key::ApiKey::Anthropic, KeyState::Saved));
        assert!(run_button(&options, &checked).1);

        type_notes(&mut options, &"x".repeat(describe::MAX_INSTRUCTIONS_CHARS));
        assert_eq!(options.notes_too_long(), None, "exactly the cap is fine");
        assert!(run_button(&options, &checked).1);
        type_notes(&mut options, "y");
        assert_eq!(
            options.notes_too_long(),
            Some(describe::MAX_INSTRUCTIONS_CHARS + 1)
        );
        assert!(!run_button(&options, &checked).1);
    }
}
