//! Runs of one clip, started from the open clip without the batch panel (#249).
//!
//! "Describe with AI" and "Generate subtitles" for the clip in front of the editor. A run borrows
//! the batch machinery: it checks only that clip, waits for the reads the action's plan needs
//! (clip length, key, price), starts the job, and gives the checks back. A run asked for while a
//! job works waits its turn in a line, so everything slow goes through one queue.

use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

use frename_core::FileId;
use iced::Task;

use super::FolderWorkspace;
use crate::features::batch::{self, Action, Readiness};
use crate::features::file_workspace::{ClipAction, ClipActions};
use crate::features::folder_workspace::Message;

/// How long a run may wait for its reads before it is given up, so a read that never answers
/// cannot hold the batch controls for good.
const PREPARE_LIMIT: Duration = Duration::from_secs(30);

/// The actions of one clip: the one preparing and the ones waiting.
#[derive(Default)]
pub(super) struct Solo {
    preparing: Option<Preparing>,
    waiting: VecDeque<(Action, FileId)>,
}

/// A run that checked its clip and waits for what the action needs to know.
struct Preparing {
    action: Action,
    file: FileId,
    /// The checks of the batch panel, which the run borrowed.
    saved: HashSet<FileId>,
    since: Instant,
}

impl FolderWorkspace {
    /// The open clip's `action` has been asked for: it runs now, or waits behind the job.
    pub(super) fn run_on_open_clip(&mut self, action: Action) -> Task<Message> {
        let Some(id) = self.file_workspace.file().map(|f| f.id()) else {
            return Task::none();
        };
        let label = action.label();
        if self.clip_action(action, id) != ClipAction::Idle {
            return Self::notice(&fl!("batch-solo-already", action = label));
        }
        let busy = self.batch.is_running()
            || self.solo.preparing.is_some()
            || !self.solo.waiting.is_empty();
        self.solo.waiting.push_back((action, id));
        if busy {
            return Self::notice(&fl!("batch-solo-queued", action = label));
        }
        // `advance_solo` starts it right after this message.
        Task::none()
    }

    /// Where `action` stands for `clip`.
    pub fn clip_action(&self, action: Action, clip: FileId) -> ClipAction {
        if let Some(run) = &self.solo.preparing {
            if run.action == action && run.file == clip {
                return ClipAction::Running;
            }
        }
        if self.batch.job_action() == Some(action) && self.batch.is_running() {
            match self.batch.status(clip) {
                Some(batch::ItemStatus::Running) => return ClipAction::Running,
                Some(batch::ItemStatus::Pending) => return ClipAction::Queued,
                _ => {}
            }
        }
        if self.solo.waiting.contains(&(action, clip)) {
            return ClipAction::Queued;
        }
        ClipAction::Idle
    }

    /// Where the open clip's paid actions stand, for the buttons of the comment's corner.
    pub fn clip_actions(&self) -> ClipActions {
        let Some(id) = self.file_workspace.file().map(|f| f.id()) else {
            return ClipActions {
                describe: ClipAction::Idle,
                subtitles: ClipAction::Idle,
            };
        };
        ClipActions {
            describe: self.clip_action(Action::DescribeAi, id),
            subtitles: self.clip_action(Action::GenerateSubtitles, id),
        }
    }

    /// While a run prepares, the checks and the job's start are its own: what would change them
    /// waits for the few moments it takes.
    pub(super) fn solo_holds(&self, message: &Message) -> bool {
        self.solo.preparing.is_some()
            && matches!(
                message,
                Message::Batch(
                    batch::Message::Run
                        | batch::Message::Retry
                        | batch::Message::Toggle(_)
                        | batch::Message::CheckAll(_)
                        | batch::Message::CheckNone
                        | batch::Message::Invert(_)
                        | batch::Message::SelectAction(_)
                        | batch::Message::Prepare { .. }
                ) | Message::PrepareBatch(_)
                    | Message::Folder(
                        crate::features::folder::Message::ToggleChecked(_)
                            | crate::features::folder::Message::ToggleAllChecked
                            | crate::features::folder::Message::InvertChecks
                            | crate::features::folder::Message::SetBatchMode(_)
                    )
            )
    }

    /// Move the line on: start the run that waits for the job once it is idle, and the run that
    /// prepares once what it needs is read. Called after every message; cheap when nothing waits.
    pub(super) fn advance_solo(&mut self) -> Task<Message> {
        if self.solo.preparing.is_none() && self.solo.waiting.is_empty() {
            return Task::none();
        }
        if let Some(run) = &self.solo.preparing {
            // The job started (here, or once the marker requests let go): give the checks back.
            if self.batch.is_running() {
                self.end_preparing();
                return Task::none();
            }
            if run.since.elapsed() > PREPARE_LIMIT {
                self.end_preparing();
                return Self::notice(&fl!("batch-solo-gave-up"));
            }
            let Some(file) = self
                .directory
                .as_ref()
                .and_then(|d| d.file_by_id(run.file))
                .cloned()
            else {
                self.end_preparing();
                return Task::none();
            };
            let action = run.action;
            return match self.batch.actions().readiness(action, &[&file]) {
                Readiness::Waiting => Task::none(),
                Readiness::Blocked { note, fix } => {
                    self.end_preparing();
                    let fix = fix.map_or_else(Task::none, |fix| {
                        Task::done(Message::Batch(batch::Message::Action(fix)))
                    });
                    Task::batch([Self::notice(&note), fix, self.advance_solo()])
                }
                Readiness::Ready => {
                    let started = self.start_batch();
                    if self.batch.is_running() {
                        self.end_preparing();
                    }
                    started
                }
            };
        }
        if self.batch.is_running() || self.batch.is_waiting_for_markers() || self.batch_end_pending
        {
            return Task::none();
        }
        while let Some((action, file)) = self.solo.waiting.pop_front() {
            if self
                .directory
                .as_ref()
                .and_then(|d| d.file_by_id(file))
                .is_none()
            {
                continue;
            }
            let Some(saved) = self.batch.borrow_for_one(action, file) else {
                self.solo.waiting.push_front((action, file));
                return Task::none();
            };
            self.solo.preparing = Some(Preparing {
                action,
                file,
                saved,
                since: Instant::now(),
            });
            self.share_folder_tags();
            let reads = Task::batch([self.describe_ai_reads(), self.subtitle_reads()]);
            // Nothing to read (a free action, or everything cached): it may start at once.
            return Task::batch([reads, self.advance_solo()]);
        }
        Task::none()
    }

    /// The run is over with preparing (started, blocked, or given up): the checks come back.
    fn end_preparing(&mut self) {
        if let Some(run) = self.solo.preparing.take() {
            self.batch.restore_checked(run.saved);
        }
    }
}
