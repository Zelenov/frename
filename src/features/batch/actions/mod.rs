//! The batch actions. Each available action lives in its own module with everything that is
//! its own: its options and their messages, the panel that edits them, and what it does to one
//! file. This module only lists them and dispatches to them; the checked files and the job
//! that runs an action over them are shared (see [`super::state`]).
//!
//! Adding an action: a module with `label()`, `view` and `run` (plus `Options` with `Message`
//! and `update` when it has settings), then one line in each match below. An action that needs
//! more than "Run on N files" can also give the run button its own label and readiness
//! ([`Actions::panel`]), a line pinned next to it ([`Actions::footer`]), and the files its job
//! runs over ([`Actions::job_files`]), as "Describe with AI" does.

pub mod describe_ai;
mod fix_tags;
pub mod generate_subtitles;
mod markers_comment;
pub use markers_comment::Direction as MarkersDirection;
mod move_comments;
mod move_in_out;
mod reload_files;
mod rotate;
mod tag_commented;
mod tag_spacing;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use clipscribe::AiUsage;
use clipscribe::Model;
use frename_core::ai::key::ApiKey;
use frename_core::{
    CommentStorage, File, FileId, FileSnapshot, FileTagger, FolderInfo, InOutStorage, MoveOutcome,
};
use iced::Element;

use super::{ItemProgress, ItemResult, ItemStatus};
use crate::ui::icons::Icon;

/// An entry of the action list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MoveComments,
    MoveInOut,
    MarkersComment,
    Rotate,
    TagCommented,
    FixTags,
    RespaceTags,
    ReloadFiles,
    DescribeAi,
    GenerateSubtitles,
}

impl Action {
    /// Every action, in list order.
    pub const ALL: [Action; 10] = [
        Action::MoveComments,
        Action::MoveInOut,
        Action::MarkersComment,
        Action::Rotate,
        Action::TagCommented,
        Action::FixTags,
        Action::RespaceTags,
        Action::ReloadFiles,
        Action::DescribeAi,
        Action::GenerateSubtitles,
    ];

    pub fn label(self) -> String {
        match self {
            Self::MoveComments => move_comments::label(),
            Self::MoveInOut => move_in_out::label(),
            Self::MarkersComment => markers_comment::label(),
            Self::Rotate => rotate::label(),
            Self::TagCommented => tag_commented::label(),
            Self::FixTags => fix_tags::label(),
            Self::RespaceTags => tag_spacing::label(),
            Self::ReloadFiles => reload_files::label(),
            Self::DescribeAi => describe_ai::label(),
            Self::GenerateSubtitles => generate_subtitles::label(),
        }
    }

    /// The paid service the action bills, when it is one.
    pub fn service(self) -> Option<ApiKey> {
        match self {
            Self::DescribeAi => Some(ApiKey::Anthropic),
            Self::GenerateSubtitles => Some(ApiKey::Soniox),
            _ => None,
        }
    }

    /// The action's name in English only: logs never switch language.
    pub fn log_id(self) -> &'static str {
        match self {
            Self::MoveComments => "Move comments",
            Self::MoveInOut => "In/out points: comment <-> video",
            Self::MarkersComment => "Markers <-> comment",
            Self::Rotate => "Rotate videos",
            Self::TagCommented => "Tag commented videos",
            Self::FixTags => "Fix tags by priority",
            Self::RespaceTags => "Apply tag spacing",
            Self::ReloadFiles => "Reset cache and reload",
            Self::DescribeAi => "Describe with AI",
            Self::GenerateSubtitles => "Generate subtitles",
        }
    }

    /// The group of the action list it is under.
    pub fn group(self) -> Group {
        match self {
            Self::MoveComments | Self::MoveInOut | Self::MarkersComment => Group::MoveBetweenPlaces,
            Self::Rotate
            | Self::TagCommented
            | Self::FixTags
            | Self::RespaceTags
            | Self::ReloadFiles => Group::FixFiles,
            Self::DescribeAi | Self::GenerateSubtitles => Group::PaidServices,
        }
    }

    /// The icon the action list shows it with (design system §13.6.3).
    pub fn icon(self) -> Icon {
        match self {
            Self::MoveComments => Icon::MessageSquareText,
            Self::MoveInOut => Icon::Scissors,
            Self::MarkersComment => Icon::MapPin,
            Self::Rotate => Icon::RotateCw,
            Self::TagCommented => Icon::Tag,
            Self::FixTags => Icon::ListOrdered,
            Self::RespaceTags => Icon::TextCursorInput,
            Self::ReloadFiles => Icon::RotateCcw,
            Self::DescribeAi => Icon::Sparkles,
            Self::GenerateSubtitles => Icon::Captions,
        }
    }

    /// The run button's label for `count` files: a verb and the count, or with nothing checked
    /// the action's name alone (§13.6.4). The paid actions label it themselves.
    pub fn run_label(self, count: usize) -> String {
        if count == 0 {
            return self.label();
        }
        let count = count as i64;
        match self {
            Self::MoveComments => fl!("batch-run-move-comments", count = count),
            Self::MoveInOut => fl!("batch-run-move-in-out", count = count),
            Self::MarkersComment => fl!("batch-run-convert", count = count),
            Self::Rotate => fl!("batch-run-rotate", count = count),
            Self::TagCommented => fl!("batch-run-tag", count = count),
            Self::FixTags => fl!("batch-run-fix-tags", count = count),
            Self::RespaceTags => fl!("batch-run-rename", count = count),
            Self::ReloadFiles => fl!("batch-run-reload", count = count),
            Self::DescribeAi | Self::GenerateSubtitles => self.label(),
        }
    }

    /// The figures' word for a file the action did its work on.
    pub fn done_label(self) -> String {
        match self {
            Self::GenerateSubtitles => fl!("batch-done-label-subtitled"),
            _ => fl!("batch-done-label-changed"),
        }
    }

    /// Stable id for remembering the last action run (#65): unlike `label()` it never changes
    /// with the UI language, and unlike `log_id()` it is never shown, so it can change wording
    /// there without breaking a saved id.
    pub fn id(self) -> &'static str {
        match self {
            Self::MoveComments => "move_comments",
            Self::MoveInOut => "move_in_out",
            Self::MarkersComment => "markers_comment",
            Self::Rotate => "rotate",
            Self::TagCommented => "tag_commented",
            Self::FixTags => "fix_tags",
            Self::RespaceTags => "respace_tags",
            Self::ReloadFiles => "reload_files",
            Self::DescribeAi => "describe_ai",
            Self::GenerateSubtitles => "generate_subtitles",
        }
    }

    /// Parse a persisted id; `None` for one no action has (e.g. an action removed since).
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }
}

/// An action as a dropdown shows it: its name.
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label())
    }
}

/// A group of the action list, under its caption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    MoveBetweenPlaces,
    FixFiles,
    PaidServices,
}

impl Group {
    pub fn label(self) -> String {
        match self {
            Self::MoveBetweenPlaces => fl!("batch-group-move"),
            Self::FixFiles => fl!("batch-group-fix"),
            Self::PaidServices => fl!("batch-group-paid"),
        }
    }
}

/// What a job does to each file, with the options it was started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    MoveComments(CommentStorage),
    MoveInOut(InOutStorage),
    MarkersComment(markers_comment::Direction),
    /// Turn each video by changing its rotation flag.
    Rotate(rotate::Turn),
    TagCommented,
    FixTags,
    /// Rename files to the tag spacing chosen in the settings.
    RespaceTags,
    ReloadFiles,
    /// Describe each video with AI.
    DescribeAi(describe_ai::Run),
    /// Transcribe each video with Soniox into `clip.srt`.
    GenerateSubtitles(generate_subtitles::Run),
}

impl Operation {
    /// Do it to the file at `path`. Blocking: runs on a worker thread. Long operations check
    /// `cancel` and stop early, leaving the file not reached.
    pub fn run(&self, path: &Path, cancel: &AtomicBool, progress: &ItemProgress) -> ItemResult {
        match self {
            Self::MoveComments(to) => move_comments::run(*to, path),
            Self::MoveInOut(to) => move_in_out::run(*to, path),
            Self::MarkersComment(direction) => markers_comment::run(*direction, path),
            Self::Rotate(turn) => rotate::run(*turn, path),
            Self::TagCommented => tag_commented::run(path),
            Self::FixTags => fix_tags::run(path),
            Self::RespaceTags => tag_spacing::run(path),
            Self::ReloadFiles => reload_files::run(path),
            Self::DescribeAi(options) => describe_ai::run(*options, path, cancel, progress),
            Self::GenerateSubtitles(run) => generate_subtitles::run(run, path, cancel, progress),
        }
    }

    /// A line for the job's summary, for actions with more to say than the counts.
    pub fn report(&self) -> Option<String> {
        match self {
            Self::GenerateSubtitles(run) => run.report(),
            _ => None,
        }
    }

    /// [`Self::report`] in English only, for logs.
    pub fn log_report(&self) -> Option<String> {
        match self {
            Self::GenerateSubtitles(run) => run.log_report(),
            _ => None,
        }
    }

    /// The model this operation's AI requests go to; the default for actions without any.
    pub fn ai_model(&self) -> Model {
        match self {
            Self::DescribeAi(run) => run.model(),
            _ => Model::default(),
        }
    }

    /// The action this operation belongs to.
    pub fn action(&self) -> Action {
        match self {
            Self::MoveComments(_) => Action::MoveComments,
            Self::MoveInOut(_) => Action::MoveInOut,
            Self::MarkersComment(_) => Action::MarkersComment,
            Self::Rotate(_) => Action::Rotate,
            Self::TagCommented => Action::TagCommented,
            Self::FixTags => Action::FixTags,
            Self::RespaceTags => Action::RespaceTags,
            Self::ReloadFiles => Action::ReloadFiles,
            Self::DescribeAi(_) => Action::DescribeAi,
            Self::GenerateSubtitles(_) => Action::GenerateSubtitles,
        }
    }
}

/// A message of one action's panel.
#[derive(Debug, Clone)]
pub enum ActionMessage {
    MoveComments(move_comments::Message),
    MoveInOut(move_in_out::Message),
    MarkersComment(markers_comment::Message),
    Rotate(rotate::Message),
    DescribeAi(describe_ai::Message),
    GenerateSubtitles(generate_subtitles::Message),
    /// Open the settings window, where an action's global settings live (e.g. the commented
    /// tag). Handled by the app, which owns the windows.
    OpenSettings,
    /// Open the settings window at its AI section (the API key, the summary language).
    OpenAiSettings,
    /// Have the settings read whether an API key is saved; the answer comes back as
    /// `DescribeAi(KeyState)`. Handled by the app.
    ReadKeyState,
    /// Open the settings window at its Subtitles section (the Soniox key). Handled by the app.
    OpenSubtitleSettings,
    /// Have the settings read whether a Soniox key is saved; the answer comes back as
    /// `GenerateSubtitles(KeyState)`. Handled by the app.
    ReadSonioxKeyState,
}

impl ActionMessage {
    /// Whether it may change the options while a job runs: results of background reads.
    pub fn applies_while_running(&self) -> bool {
        match self {
            Self::GenerateSubtitles(message) => message.applies_while_running(),
            _ => matches!(
                self,
                Self::DescribeAi(
                    describe_ai::Message::Probed(_)
                        | describe_ai::Message::KeyState(_)
                        | describe_ai::Message::SetLanguage(_)
                        | describe_ai::Message::SetModel(_)
                )
            ),
        }
    }
}

/// The options of every action, kept while the user switches between them.
#[derive(Debug, Clone, Default)]
pub struct Actions {
    move_comments: move_comments::Options,
    move_in_out: move_in_out::Options,
    markers_comment: markers_comment::Options,
    rotate: rotate::Options,
    describe_ai: describe_ai::Options,
    generate_subtitles: generate_subtitles::Options,
}

impl Actions {
    pub fn update(&mut self, message: ActionMessage) {
        match message {
            ActionMessage::MoveComments(message) => self.move_comments.update(message),
            ActionMessage::MoveInOut(message) => self.move_in_out.update(message),
            ActionMessage::MarkersComment(message) => self.markers_comment.update(message),
            ActionMessage::Rotate(message) => self.rotate.update(message),
            ActionMessage::DescribeAi(message) => self.describe_ai.update(message),
            ActionMessage::GenerateSubtitles(message) => self.generate_subtitles.update(message),
            ActionMessage::OpenSettings
            | ActionMessage::OpenAiSettings
            | ActionMessage::ReadKeyState
            | ActionMessage::OpenSubtitleSettings
            | ActionMessage::ReadSonioxKeyState => {}
        }
    }

    /// The model and language of "Describe with AI", which describing a marker uses too.
    pub fn ai_model_and_language(&self) -> (Model, clipscribe::SummaryLanguage) {
        self.describe_ai.model_and_language()
    }

    pub(super) fn describe_ai_mut(&mut self) -> &mut describe_ai::Options {
        &mut self.describe_ai
    }

    pub(super) fn generate_subtitles_mut(&mut self) -> &mut generate_subtitles::Options {
        &mut self.generate_subtitles
    }

    /// Set the options of `operation`'s action to do what it does.
    pub fn prepare(&mut self, operation: Operation) {
        match operation {
            Operation::MoveComments(to) => self.move_comments.prepare(to),
            Operation::MoveInOut(to) => self.move_in_out.prepare(to),
            Operation::MarkersComment(direction) => self.markers_comment.prepare(direction),
            Operation::Rotate(_)
            | Operation::TagCommented
            | Operation::FixTags
            | Operation::RespaceTags
            | Operation::ReloadFiles
            | Operation::DescribeAi(_)
            | Operation::GenerateSubtitles(_) => {}
        }
    }

    /// What `action` would do with its options now; `None` when it cannot run.
    pub fn operation(&self, action: Action) -> Option<Operation> {
        match action {
            Action::MoveComments => Some(self.move_comments.operation()),
            Action::MoveInOut => Some(self.move_in_out.operation()),
            Action::MarkersComment => Some(self.markers_comment.operation()),
            Action::Rotate => Some(self.rotate.operation()),
            Action::TagCommented => tag_commented::operation(),
            Action::FixTags => Some(Operation::FixTags),
            Action::RespaceTags => Some(Operation::RespaceTags),
            Action::ReloadFiles => Some(Operation::ReloadFiles),
            Action::DescribeAi => Some(self.describe_ai.operation()),
            Action::GenerateSubtitles => self.generate_subtitles.operation(),
        }
    }

    /// The files a job of `action` runs over: the checked ones, or for "Describe with AI" only
    /// the videos it will send.
    pub fn job_files(&self, action: Action, checked: &[&File]) -> Vec<FileId> {
        match action {
            Action::DescribeAi => self.describe_ai.files_to_send(checked),
            _ => checked.iter().map(|f| f.id()).collect(),
        }
    }

    /// Whether a background read keeps files open, so no job may start yet.
    pub fn is_reading_files(&self) -> bool {
        self.describe_ai.is_probing()
    }

    /// Which paid service `action` bills through, and whether its key is missing: the badge of
    /// its row in the action list. `None` for the free actions.
    pub fn service(&self, action: Action) -> Option<(String, bool)> {
        match action {
            Action::DescribeAi => Some((
                fl!("batch-service-anthropic"),
                self.describe_ai.key_missing(),
            )),
            Action::GenerateSubtitles => Some((
                fl!("batch-service-soniox"),
                self.generate_subtitles.key_missing(),
            )),
            _ => None,
        }
    }

    /// `action`'s current options, as simple key/value pairs to remember for next time (#65).
    /// Actions with nothing to choose return none. Reads each action's own options directly
    /// (not through [`Self::operation`]), so it does not depend on the action being ready to
    /// run right now (e.g. "Generate subtitles" before its plan and key are known).
    pub fn persist(&self, action: Action) -> Vec<(String, String)> {
        match action {
            // Each field's own `operation()` always returns its own action's variant.
            Action::MoveComments => match self.move_comments.operation() {
                Operation::MoveComments(to) => vec![("to".to_string(), to.as_str().to_string())],
                _ => unreachable!(),
            },
            Action::MoveInOut => match self.move_in_out.operation() {
                Operation::MoveInOut(to) => vec![("to".to_string(), to.as_str().to_string())],
                _ => unreachable!(),
            },
            Action::MarkersComment => match self.markers_comment.operation() {
                Operation::MarkersComment(direction) => {
                    vec![("direction".to_string(), direction.as_str().to_string())]
                }
                _ => unreachable!(),
            },
            Action::Rotate => match self.rotate.operation() {
                Operation::Rotate(turn) => vec![("turn".to_string(), turn.as_str().to_string())],
                _ => unreachable!(),
            },
            // `language` and `model` are not a batch-local choice: the panel only shows them,
            // set from Settings (`FrenameApp::update`'s `WindowReady` sync, which would
            // overwrite a restored value right on startup) — like "Generate subtitles"'s
            // `config` below, only `redo`, the panel's own checkbox, is worth remembering.
            Action::DescribeAi => match self.describe_ai.operation() {
                Operation::DescribeAi(run) => vec![("redo".to_string(), run.redo.to_string())],
                _ => unreachable!(),
            },
            Action::GenerateSubtitles => {
                let formats = self.generate_subtitles.formats();
                vec![
                    (
                        "replace".to_string(),
                        self.generate_subtitles.replaces().to_string(),
                    ),
                    ("srt".to_string(), formats.srt.to_string()),
                    ("premiere".to_string(), formats.premiere.to_string()),
                ]
            }
            Action::TagCommented | Action::FixTags | Action::RespaceTags | Action::ReloadFiles => {
                Vec::new()
            }
        }
    }

    /// Apply `options`, as [`Self::persist`] returned them for `action`, to its own options. A
    /// key it does not recognise, or a value it cannot parse, is simply left at its default
    /// (removed choices, or ones only another version understands, are never an error).
    pub fn restore(&mut self, action: Action, options: &[(String, String)]) {
        let get = |key: &str| {
            options
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        match action {
            Action::MoveComments => {
                if let Some(v) = get("to") {
                    self.move_comments
                        .update(move_comments::Message::SetTo(CommentStorage::from_name(v)));
                }
            }
            Action::MoveInOut => {
                if let Some(v) = get("to") {
                    self.move_in_out
                        .update(move_in_out::Message::SetTo(InOutStorage::from_name(v)));
                }
            }
            Action::MarkersComment => {
                if let Some(v) = get("direction") {
                    self.markers_comment
                        .update(markers_comment::Message::SetDirection(
                            markers_comment::Direction::from_name(v),
                        ));
                }
            }
            Action::Rotate => {
                if let Some(v) = get("turn") {
                    self.rotate
                        .update(rotate::Message::SetTurn(rotate::Turn::from_name(v)));
                }
            }
            Action::DescribeAi => {
                if let Some(v) = get("redo") {
                    self.describe_ai
                        .update(describe_ai::Message::SetRedo(v == "true"));
                }
            }
            Action::GenerateSubtitles => {
                if let Some(v) = get("replace") {
                    self.generate_subtitles
                        .update(generate_subtitles::Message::SetReplace(v == "true"));
                }
                if let Some(v) = get("srt") {
                    self.generate_subtitles
                        .update(generate_subtitles::Message::SetSrt(v == "true"));
                }
                if let Some(v) = get("premiere") {
                    self.generate_subtitles
                        .update(generate_subtitles::Message::SetPremiere(v == "true"));
                }
            }
            Action::TagCommented | Action::FixTags | Action::RespaceTags | Action::ReloadFiles => {}
        }
    }

    /// The page of `action` for the `checked` files, and its run button.
    pub fn panel(&self, action: Action, checked: &[&File]) -> Panel<'_> {
        let page = match action {
            Action::DescribeAi => return self.describe_ai.panel(checked),
            Action::GenerateSubtitles => return self.generate_subtitles.panel(checked),
            Action::MoveComments => self.move_comments.view().map(ActionMessage::MoveComments),
            Action::MoveInOut => self.move_in_out.view().map(ActionMessage::MoveInOut),
            Action::MarkersComment => self
                .markers_comment
                .view()
                .map(ActionMessage::MarkersComment),
            Action::Rotate => self.rotate.view().map(ActionMessage::Rotate),
            Action::TagCommented => tag_commented::view(),
            Action::FixTags => fix_tags::view(),
            Action::RespaceTags => tag_spacing::view(),
            Action::ReloadFiles => reload_files::view(),
        };
        Panel {
            page,
            run: action.run_label(checked.len()),
            ready: !checked.is_empty() && self.operation(action).is_some(),
            reason: None,
        }
    }
}

/// An action's page and its run button.
pub struct Panel<'a> {
    pub page: Element<'a, ActionMessage>,
    /// The run button's label: a verb and the count.
    pub run: String,
    pub ready: bool,
    /// Why it cannot run, for the button bar, when the page does not already say it in a
    /// notice. The batch panel adds the reason every action shares: nothing checked.
    pub reason: Option<String>,
}

/// What a job's AI requests to `model` cost: `$0.31 (Claude Haiku 4.5)`.
pub fn spend_line(model: Model, usage: AiUsage) -> String {
    format!(
        "{} ({})",
        describe_ai::dollars(model.cost_usd(usage)),
        model.label
    )
}

/// The job's record of a file an action changed, failed on or left alone.
fn item_result(outcome: MoveOutcome) -> ItemResult {
    match outcome {
        MoveOutcome::NothingToMove => ItemResult::new(ItemStatus::Skipped, None),
        MoveOutcome::Moved(new_path) => ItemResult::new(ItemStatus::Done, Some(reparsed(new_path))),
        MoveOutcome::Failed(new_path) => {
            ItemResult::new(ItemStatus::Failed, Some(reparsed(new_path)))
        }
    }
}

/// The file at `path` as the list shows it after a change: parsed again, like after a save.
fn reparsed(path: PathBuf) -> (PathBuf, FileSnapshot) {
    let snapshot = FileTagger::parse(&path, &FolderInfo::default());
    (path, snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_paid_actions_have_a_service() {
        assert_eq!(Action::DescribeAi.service(), Some(ApiKey::Anthropic));
        assert_eq!(Action::GenerateSubtitles.service(), Some(ApiKey::Soniox));
        assert_eq!(Action::Rotate.service(), None);
    }

    #[test]
    fn every_action_s_id_round_trips_and_an_unknown_one_is_none() {
        for action in Action::ALL {
            assert_eq!(Action::from_id(action.id()), Some(action));
        }
        assert_eq!(Action::from_id("no-such-action"), None);
    }

    #[test]
    fn move_comments_and_move_in_out_persist_and_restore() {
        let mut actions = Actions::default();
        actions.update(ActionMessage::MoveComments(move_comments::Message::SetTo(
            CommentStorage::TextFile,
        )));
        let saved = actions.persist(Action::MoveComments);
        let mut restored = Actions::default();
        restored.restore(Action::MoveComments, &saved);
        assert_eq!(
            restored.operation(Action::MoveComments),
            Some(Operation::MoveComments(CommentStorage::TextFile))
        );

        actions.update(ActionMessage::MoveInOut(move_in_out::Message::SetTo(
            InOutStorage::Comment,
        )));
        let saved = actions.persist(Action::MoveInOut);
        let mut restored = Actions::default();
        restored.restore(Action::MoveInOut, &saved);
        assert_eq!(
            restored.operation(Action::MoveInOut),
            Some(Operation::MoveInOut(InOutStorage::Comment))
        );
    }

    #[test]
    fn markers_comment_and_rotate_persist_and_restore() {
        let mut actions = Actions::default();
        actions.update(ActionMessage::MarkersComment(
            markers_comment::Message::SetDirection(markers_comment::Direction::MarkersToComment),
        ));
        let saved = actions.persist(Action::MarkersComment);
        let mut restored = Actions::default();
        restored.restore(Action::MarkersComment, &saved);
        assert_eq!(
            restored.operation(Action::MarkersComment),
            Some(Operation::MarkersComment(
                markers_comment::Direction::MarkersToComment
            ))
        );

        actions.update(ActionMessage::Rotate(rotate::Message::SetTurn(
            rotate::Turn::Half,
        )));
        let saved = actions.persist(Action::Rotate);
        let mut restored = Actions::default();
        restored.restore(Action::Rotate, &saved);
        assert_eq!(
            restored.operation(Action::Rotate),
            Some(Operation::Rotate(rotate::Turn::Half))
        );
    }

    #[test]
    fn describe_ai_and_generate_subtitles_options_persist_and_restore() {
        // `language`/`model` are not persisted: the panel only shows them, set from Settings
        // (`FrenameApp::update`'s `WindowReady` sync would overwrite a restored value on
        // startup anyway); only `redo`, the panel's own checkbox, is a batch-local choice.
        let mut actions = Actions::default();
        actions.update(ActionMessage::DescribeAi(describe_ai::Message::SetRedo(
            true,
        )));
        let saved = actions.persist(Action::DescribeAi);
        assert_eq!(saved, vec![("redo".to_string(), "true".to_string())]);
        let mut restored = Actions::default();
        restored.restore(Action::DescribeAi, &saved);
        match restored.operation(Action::DescribeAi) {
            Some(Operation::DescribeAi(run)) => assert!(run.redo),
            other => panic!("expected DescribeAi, got {other:?}"),
        }

        actions.update(ActionMessage::GenerateSubtitles(
            generate_subtitles::Message::SetReplace(true),
        ));
        let saved = actions.persist(Action::GenerateSubtitles);
        assert_eq!(
            saved,
            vec![
                ("replace".to_string(), "true".to_string()),
                ("srt".to_string(), "true".to_string()),
                ("premiere".to_string(), "false".to_string()),
            ]
        );
        let mut restored = Actions::default();
        restored.restore(Action::GenerateSubtitles, &saved);
        assert!(restored.generate_subtitles.replaces());

        actions.update(ActionMessage::GenerateSubtitles(
            generate_subtitles::Message::SetSrt(false),
        ));
        actions.update(ActionMessage::GenerateSubtitles(
            generate_subtitles::Message::SetPremiere(true),
        ));
        let saved = actions.persist(Action::GenerateSubtitles);
        let mut restored = Actions::default();
        restored.restore(Action::GenerateSubtitles, &saved);
        let formats = restored.generate_subtitles.formats();
        assert!(!formats.srt && formats.premiere);
        // A run saved before the formats existed keeps the default: SRT only.
        let mut old = Actions::default();
        old.restore(
            Action::GenerateSubtitles,
            &[("replace".to_string(), "false".to_string())],
        );
        assert_eq!(
            old.generate_subtitles.formats(),
            generate_subtitles::Formats::default()
        );
    }

    #[test]
    fn an_unknown_option_value_falls_back_to_its_default() {
        let mut actions = Actions::default();
        actions.restore(Action::Rotate, &[("turn".to_string(), "??".to_string())]);
        assert_eq!(
            actions.operation(Action::Rotate),
            Some(Operation::Rotate(rotate::Turn::Right)),
            "an unknown turn falls back to the default"
        );
    }

    #[test]
    fn an_unrecognised_option_key_is_ignored() {
        let mut actions = Actions::default();
        actions.restore(
            Action::Rotate,
            &[
                ("turn".to_string(), "half".to_string()),
                ("bogus".to_string(), "x".to_string()),
            ],
        );
        assert_eq!(
            actions.operation(Action::Rotate),
            Some(Operation::Rotate(rotate::Turn::Half))
        );
    }

    #[test]
    fn actions_without_a_choice_persist_nothing() {
        let actions = Actions::default();
        for action in [
            Action::TagCommented,
            Action::FixTags,
            Action::RespaceTags,
            Action::ReloadFiles,
        ] {
            assert!(actions.persist(action).is_empty());
        }
    }
}
