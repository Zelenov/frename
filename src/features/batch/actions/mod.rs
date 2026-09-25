//! The batch actions. Each available action lives in its own module with everything that is
//! its own: its options and their messages, the panel that edits them, and what it does to one
//! file. This module only lists them and dispatches to them; the checked files and the job
//! that runs an action over them are shared (see [`super::state`]).
//!
//! Adding an action: a module with `LABEL`, `view` and `run` (plus `Options` with `Message`
//! and `update` when it has settings), then one line in each match below.

mod fix_tags;
mod move_comments;
mod move_in_out;
mod reload_files;
mod tag_commented;

use std::path::{Path, PathBuf};

use frename_core::{CommentStorage, FileSnapshot, FileTagger, FolderInfo, InOutStorage, MoveOutcome};
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
    ReloadFiles,
}

impl Action {
    /// Every action, in list order.
    pub const ALL: [Action; 5] =
        [Action::MoveComments, Action::MoveInOut, Action::TagCommented, Action::FixTags, Action::ReloadFiles];

    pub fn label(self) -> &'static str {
        match self {
            Self::MoveComments => move_comments::LABEL,
            Self::MoveInOut => move_in_out::LABEL,
            Self::TagCommented => tag_commented::LABEL,
            Self::FixTags => fix_tags::LABEL,
            Self::ReloadFiles => reload_files::LABEL,
        }
    }
}

/// What a job does to each file, with the options it was started with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    MoveComments(CommentStorage),
    MoveInOut(InOutStorage),
    TagCommented,
    FixTags,
    ReloadFiles,
}

impl Operation {
    /// Do it to the file at `path`. Blocking: runs on a worker thread.
    pub fn run(self, path: &Path) -> ItemResult {
        match self {
            Self::MoveComments(to) => move_comments::run(to, path),
            Self::MoveInOut(to) => move_in_out::run(to, path),
            Self::TagCommented => tag_commented::run(path),
            Self::FixTags => fix_tags::run(path),
            Self::ReloadFiles => reload_files::run(path),
        }
    }

    /// The action this operation belongs to.
    pub fn action(self) -> Action {
        match self {
            Self::MoveComments(_) => Action::MoveComments,
            Self::MoveInOut(_) => Action::MoveInOut,
            Self::TagCommented => Action::TagCommented,
            Self::FixTags => Action::FixTags,
            Self::ReloadFiles => Action::ReloadFiles,
        }
    }
}

/// A message of one action's panel.
#[derive(Debug, Clone)]
pub enum ActionMessage {
    MoveComments(move_comments::Message),
    MoveInOut(move_in_out::Message),
    /// Open the settings window, where an action's global settings live (e.g. the commented
    /// tag). Handled by the app, which owns the windows.
    OpenSettings,
}

/// The options of every action, kept while the user switches between them.
#[derive(Debug, Clone, Default)]
pub struct Actions {
    move_comments: move_comments::Options,
    move_in_out: move_in_out::Options,
}

impl Actions {
    pub fn update(&mut self, message: ActionMessage) {
        match message {
            ActionMessage::MoveComments(message) => self.move_comments.update(message),
            ActionMessage::MoveInOut(message) => self.move_in_out.update(message),
            ActionMessage::OpenSettings => {}
        }
    }

    /// Set the options of `operation`'s action to do what it does.
    pub fn prepare(&mut self, operation: Operation) {
        match operation {
            Operation::MoveComments(to) => self.move_comments.prepare(to),
            Operation::MoveInOut(to) => self.move_in_out.prepare(to),
            Operation::TagCommented | Operation::FixTags | Operation::ReloadFiles => {}
        }
    }

    /// What `action` would do with its options now; `None` when it cannot run.
    pub fn operation(&self, action: Action) -> Option<Operation> {
        match action {
            Action::MoveComments => Some(self.move_comments.operation()),
            Action::MoveInOut => Some(self.move_in_out.operation()),
            Action::TagCommented => tag_commented::operation(),
            Action::FixTags => Some(Operation::FixTags),
            Action::ReloadFiles => Some(Operation::ReloadFiles),
        }
    }

    /// The panel of `action`: its title, what it does, and its options.
    pub fn view(&self, action: Action) -> Element<'_, ActionMessage> {
        match action {
            Action::MoveComments => self.move_comments.view().map(ActionMessage::MoveComments),
            Action::MoveInOut => self.move_in_out.view().map(ActionMessage::MoveInOut),
            Action::TagCommented => tag_commented::view(),
            Action::FixTags => fix_tags::view(),
            Action::ReloadFiles => reload_files::view(),
        }
    }
}

/// An action's panel as every action shows it: title, what it does, then its options.
fn panel<'a, M: 'a>(title: &'a str, hint: String, options: Element<'a, M>) -> Element<'a, M> {
    column![text(title).size(15), text(hint).size(12).color(theme::TEXT_MUTED), options]
        .spacing(12)
        .into()
}

/// The job's record of a file an action changed, failed on or left alone.
fn item_result(outcome: MoveOutcome) -> ItemResult {
    match outcome {
        MoveOutcome::NothingToMove => ItemResult { status: ItemStatus::Skipped, update: None },
        MoveOutcome::Moved(new_path) => ItemResult { status: ItemStatus::Done, update: Some(reparsed(new_path)) },
        MoveOutcome::Failed(new_path) => ItemResult { status: ItemStatus::Failed, update: Some(reparsed(new_path)) },
    }
}

/// The file at `path` as the list shows it after a change: parsed again, like after a save.
fn reparsed(path: PathBuf) -> (PathBuf, FileSnapshot) {
    let snapshot = FileTagger::parse(&path, &FolderInfo::default());
    (path, snapshot)
}
