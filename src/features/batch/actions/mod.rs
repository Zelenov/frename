//! The batch actions. Each available action lives in its own module with everything that is
//! its own: its options and their messages, the panel that edits them, and what it does to one
//! file. This module only lists them and dispatches to them; the checked files and the job
//! that runs an action over them are shared (see [`super::state`]).
//!
//! Adding an action: a module with `LABEL`, `view` and `run` (plus `Options` with `Message`
//! and `update` when it has settings), then one line in each match below. An action that needs
//! more than "Run on N files" can also give the run button its own label and readiness
//! ([`Actions::panel`]), a line pinned next to it ([`Actions::footer`]), and the files its job
//! runs over ([`Actions::job_files`]), as "Describe with AI" does.

pub mod describe_ai;
mod fix_tags;
mod move_comments;
mod move_in_out;
mod reload_files;
mod tag_commented;
mod tag_spacing;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use frename_core::ai::describe::MODEL;
use frename_core::ai::provider::AiUsage;
use frename_core::{
    CommentStorage, File, FileId, FileSnapshot, FileTagger, FolderInfo, InOutStorage, MoveOutcome,
};
use iced::widget::{column, text};
use iced::Element;

use super::{ItemResult, ItemStatus};
use crate::theme;

/// An entry of the action list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MoveComments,
    MoveInOut,
    TagCommented,
    FixTags,
    RespaceTags,
    ReloadFiles,
    DescribeAi,
}

impl Action {
    /// Every action, in list order.
    pub const ALL: [Action; 7] = [
        Action::MoveComments,
        Action::MoveInOut,
        Action::TagCommented,
        Action::FixTags,
        Action::RespaceTags,
        Action::ReloadFiles,
        Action::DescribeAi,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::MoveComments => move_comments::LABEL,
            Self::MoveInOut => move_in_out::LABEL,
            Self::TagCommented => tag_commented::LABEL,
            Self::FixTags => fix_tags::LABEL,
            Self::RespaceTags => tag_spacing::LABEL,
            Self::ReloadFiles => reload_files::LABEL,
            Self::DescribeAi => describe_ai::LABEL,
        }
    }
}

/// What a job does to each file, with the options it was started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    MoveComments(CommentStorage),
    MoveInOut(InOutStorage),
    TagCommented,
    FixTags,
    /// Rename files to the tag spacing chosen in the settings.
    RespaceTags,
    ReloadFiles,
    /// Describe each video with AI.
    DescribeAi(describe_ai::Run),
}

impl Operation {
    /// Do it to the file at `path`. Blocking: runs on a worker thread. Long operations check
    /// `cancel` and stop early, leaving the file not reached.
    pub fn run(&self, path: &Path, cancel: &AtomicBool) -> ItemResult {
        match self {
            Self::MoveComments(to) => move_comments::run(*to, path),
            Self::MoveInOut(to) => move_in_out::run(*to, path),
            Self::TagCommented => tag_commented::run(path),
            Self::FixTags => fix_tags::run(path),
            Self::RespaceTags => tag_spacing::run(path),
            Self::ReloadFiles => reload_files::run(path),
            Self::DescribeAi(options) => describe_ai::run(*options, path, cancel),
        }
    }

    /// The action this operation belongs to.
    pub fn action(&self) -> Action {
        match self {
            Self::MoveComments(_) => Action::MoveComments,
            Self::MoveInOut(_) => Action::MoveInOut,
            Self::TagCommented => Action::TagCommented,
            Self::FixTags => Action::FixTags,
            Self::RespaceTags => Action::RespaceTags,
            Self::ReloadFiles => Action::ReloadFiles,
            Self::DescribeAi(_) => Action::DescribeAi,
        }
    }
}

/// A message of one action's panel.
#[derive(Debug, Clone)]
pub enum ActionMessage {
    MoveComments(move_comments::Message),
    MoveInOut(move_in_out::Message),
    DescribeAi(describe_ai::Message),
    /// Open the settings window, where an action's global settings live (e.g. the commented
    /// tag). Handled by the app, which owns the windows.
    OpenSettings,
    /// Open the settings window at its AI section (the API key, the summary language).
    OpenAiSettings,
    /// Have the settings read whether an API key is saved; the answer comes back as
    /// `DescribeAi(KeyState)`. Handled by the app.
    ReadKeyState,
}

impl ActionMessage {
    /// Whether it may change the options while a job runs: results of background reads.
    pub fn applies_while_running(&self) -> bool {
        matches!(
            self,
            Self::DescribeAi(
                describe_ai::Message::Probed(_)
                    | describe_ai::Message::KeyState(_)
                    | describe_ai::Message::SetLanguage(_)
            )
        )
    }
}

/// The options of every action, kept while the user switches between them.
#[derive(Debug, Clone, Default)]
pub struct Actions {
    move_comments: move_comments::Options,
    move_in_out: move_in_out::Options,
    describe_ai: describe_ai::Options,
}

impl Actions {
    pub fn update(&mut self, message: ActionMessage) {
        match message {
            ActionMessage::MoveComments(message) => self.move_comments.update(message),
            ActionMessage::MoveInOut(message) => self.move_in_out.update(message),
            ActionMessage::DescribeAi(message) => self.describe_ai.update(message),
            ActionMessage::OpenSettings
            | ActionMessage::OpenAiSettings
            | ActionMessage::ReadKeyState => {}
        }
    }

    pub(super) fn describe_ai_mut(&mut self) -> &mut describe_ai::Options {
        &mut self.describe_ai
    }

    /// Set the options of `operation`'s action to do what it does.
    pub fn prepare(&mut self, operation: Operation) {
        match operation {
            Operation::MoveComments(to) => self.move_comments.prepare(to),
            Operation::MoveInOut(to) => self.move_in_out.prepare(to),
            Operation::TagCommented
            | Operation::FixTags
            | Operation::RespaceTags
            | Operation::ReloadFiles
            | Operation::DescribeAi(_) => {}
        }
    }

    /// What `action` would do with its options now; `None` when it cannot run.
    pub fn operation(&self, action: Action) -> Option<Operation> {
        match action {
            Action::MoveComments => Some(self.move_comments.operation()),
            Action::MoveInOut => Some(self.move_in_out.operation()),
            Action::TagCommented => tag_commented::operation(),
            Action::FixTags => Some(Operation::FixTags),
            Action::RespaceTags => Some(Operation::RespaceTags),
            Action::ReloadFiles => Some(Operation::ReloadFiles),
            Action::DescribeAi => Some(self.describe_ai.operation()),
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

    /// What `action` shows next to the run button (why it cannot run), if anything.
    pub fn footer(&self, action: Action) -> Option<Element<'_, ActionMessage>> {
        match action {
            Action::DescribeAi => self.describe_ai.footer(),
            _ => None,
        }
    }

    /// The panel of `action` for the `checked` files (its title, what it does, and its options),
    /// the run button's label, and whether it can run.
    pub fn panel(
        &self,
        action: Action,
        checked: &[&File],
    ) -> (Element<'_, ActionMessage>, String, bool) {
        let view = match action {
            Action::DescribeAi => return self.describe_ai.panel(checked),
            Action::MoveComments => self.move_comments.view().map(ActionMessage::MoveComments),
            Action::MoveInOut => self.move_in_out.view().map(ActionMessage::MoveInOut),
            Action::TagCommented => tag_commented::view(),
            Action::FixTags => fix_tags::view(),
            Action::RespaceTags => tag_spacing::view(),
            Action::ReloadFiles => reload_files::view(),
        };
        let label = format!("Run on {}", files(checked.len()));
        let ready = !checked.is_empty() && self.operation(action).is_some();
        (view, label, ready)
    }
}

/// An action's panel as every action shows it: title, what it does, then its options.
fn panel<'a, M: 'a>(title: &'a str, hint: String, options: Element<'a, M>) -> Element<'a, M> {
    column![
        text(title).size(15),
        text(hint).size(12).color(theme::TEXT_MUTED),
        options
    ]
    .spacing(12)
    .into()
}

/// "5 files" / "1 file".
pub fn files(n: usize) -> String {
    format!("{n} {}", if n == 1 { "file" } else { "files" })
}

/// What a job's AI requests cost: `$0.31 (Claude Haiku 4.5)`.
pub fn spend_line(usage: AiUsage) -> String {
    format!(
        "{} ({})",
        describe_ai::dollars(MODEL.cost_usd(usage)),
        MODEL.label
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
