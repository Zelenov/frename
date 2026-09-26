//! The batch actions. Each available action lives in its own module with everything that is
//! its own: its options and their messages, the panel that edits them, and what it does to one
//! file. This module only lists them and dispatches to them; the checked files and the job
//! that runs an action over them are shared (see [`super::state`]).
//!
//! Adding an action: a module with `LABEL`, `view` and `run` (plus `Options` with `Message`
//! and `update` when it has settings), then one line in each match below.

mod fix_tags;
pub mod generate_subtitles;
mod move_comments;
mod move_in_out;
mod reload_files;
mod tag_commented;
mod tag_spacing;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use frename_core::{
    CommentStorage, FileSnapshot, FileTagger, FolderInfo, InOutStorage, MoveOutcome,
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
    GenerateSubtitles,
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
        Action::GenerateSubtitles,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::MoveComments => move_comments::LABEL,
            Self::MoveInOut => move_in_out::LABEL,
            Self::TagCommented => tag_commented::LABEL,
            Self::FixTags => fix_tags::LABEL,
            Self::RespaceTags => tag_spacing::LABEL,
            Self::ReloadFiles => reload_files::LABEL,
            Self::GenerateSubtitles => generate_subtitles::LABEL,
        }
    }

    /// The counts line's word for a file the action did its work on.
    pub fn done_label(self) -> &'static str {
        match self {
            Self::GenerateSubtitles => "subtitled",
            _ => "changed",
        }
    }

    /// Heading of the list of files the action did not do, once the job has ended.
    pub fn results_heading(self) -> &'static str {
        match self {
            Self::GenerateSubtitles => "Not subtitled:",
            _ => "Failed (see the log for why):",
        }
    }
}

/// What a job does to each file, with the options it was started with.
#[derive(Debug, Clone)]
pub enum Operation {
    MoveComments(CommentStorage),
    MoveInOut(InOutStorage),
    TagCommented,
    FixTags,
    /// Rename files to the tag spacing chosen in the settings.
    RespaceTags,
    ReloadFiles,
    /// Transcribe the videos with Soniox; the job carries the key, the plan's exclusions and
    /// its cancel token.
    GenerateSubtitles(Arc<generate_subtitles::SubtitleJob>),
}

/// Operations are equal when they do the same; two subtitle jobs only when they are the same.
impl PartialEq for Operation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MoveComments(a), Self::MoveComments(b)) => a == b,
            (Self::MoveInOut(a), Self::MoveInOut(b)) => a == b,
            (Self::GenerateSubtitles(a), Self::GenerateSubtitles(b)) => Arc::ptr_eq(a, b),
            (a, b) => {
                std::mem::discriminant(a) == std::mem::discriminant(b)
                    && !matches!(a, Self::GenerateSubtitles(_))
            }
        }
    }
}

impl Operation {
    /// Do it to the file at `path`. Blocking: runs on a worker thread.
    pub fn run(&self, path: &Path) -> ItemResult {
        match self {
            Self::MoveComments(to) => move_comments::run(*to, path),
            Self::MoveInOut(to) => move_in_out::run(*to, path),
            Self::TagCommented => tag_commented::run(path),
            Self::FixTags => fix_tags::run(path),
            Self::RespaceTags => tag_spacing::run(path),
            Self::ReloadFiles => reload_files::run(path),
            Self::GenerateSubtitles(job) => generate_subtitles::run(job, path),
        }
    }

    /// Stop the file in work, for actions that can (the others stop after it).
    pub fn cancel(&self) {
        if let Self::GenerateSubtitles(job) = self {
            job.cancel();
        }
    }

    /// A line for the job's summary, for actions that have more to say than the counts.
    pub fn report(&self) -> Option<String> {
        match self {
            Self::GenerateSubtitles(job) => job.report(),
            _ => None,
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
            Self::GenerateSubtitles(_) => Action::GenerateSubtitles,
        }
    }
}

/// A message of one action's panel.
#[derive(Debug, Clone)]
pub enum ActionMessage {
    MoveComments(move_comments::Message),
    MoveInOut(move_in_out::Message),
    GenerateSubtitles(generate_subtitles::Message),
    /// Open the settings window, where an action's global settings live (e.g. the commented
    /// tag). Handled by the app, which owns the windows.
    OpenSettings,
    /// Open the settings window at its Subtitles section (the Soniox key). Handled by the app.
    OpenSubtitleSettings,
}

/// The options of every action, kept while the user switches between them.
#[derive(Debug, Clone, Default)]
pub struct Actions {
    move_comments: move_comments::Options,
    move_in_out: move_in_out::Options,
    generate_subtitles: generate_subtitles::Options,
}

impl Actions {
    pub fn update(&mut self, message: ActionMessage) {
        match message {
            ActionMessage::MoveComments(message) => self.move_comments.update(message),
            ActionMessage::MoveInOut(message) => self.move_in_out.update(message),
            ActionMessage::GenerateSubtitles(message) => self.generate_subtitles.update(message),
            ActionMessage::OpenSettings | ActionMessage::OpenSubtitleSettings => {}
        }
    }

    /// Set the options of `operation`'s action to do what it does.
    pub fn prepare(&mut self, operation: &Operation) {
        match operation {
            Operation::MoveComments(to) => self.move_comments.prepare(*to),
            Operation::MoveInOut(to) => self.move_in_out.prepare(*to),
            Operation::TagCommented
            | Operation::FixTags
            | Operation::RespaceTags
            | Operation::ReloadFiles
            | Operation::GenerateSubtitles(_) => {}
        }
    }

    /// The subtitle action's options: its plan and price are worked out by the workspace.
    pub fn generate_subtitles(&mut self) -> &mut generate_subtitles::Options {
        &mut self.generate_subtitles
    }

    /// The run button of `action` for `count` checked files: label, and whether it can run.
    pub fn run_button(&self, action: Action, count: usize) -> (String, bool) {
        match action {
            Action::GenerateSubtitles => self.generate_subtitles.run_button(),
            _ => (
                format!(
                    "Run on {count} {}",
                    if count == 1 { "file" } else { "files" }
                ),
                self.operation(action).is_some(),
            ),
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
            Action::GenerateSubtitles => self.generate_subtitles.operation(),
        }
    }

    /// The panel of `action`: its title, what it does, and its options.
    pub fn view(&self, action: Action) -> Element<'_, ActionMessage> {
        match action {
            Action::MoveComments => self.move_comments.view().map(ActionMessage::MoveComments),
            Action::MoveInOut => self.move_in_out.view().map(ActionMessage::MoveInOut),
            Action::TagCommented => tag_commented::view(),
            Action::FixTags => fix_tags::view(),
            Action::RespaceTags => tag_spacing::view(),
            Action::ReloadFiles => reload_files::view(),
            Action::GenerateSubtitles => self.generate_subtitles.view(),
        }
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
