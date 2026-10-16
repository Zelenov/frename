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
    /// Clips whose asked-for "Describe with AI" is a "Suggest tags" (#250): the run suggests tags
    /// whatever Settings say, and describes a clip that has no suggestions yet.
    tags: HashSet<FileId>,
    /// The clip whose "Suggest tags" job is in work.
    pub(super) tags_running: Option<FileId>,
}

/// A run that checked its clip and waits for what the action needs to know.
struct Preparing {
    action: Action,
    file: FileId,
    /// What the batch panel had selected and checked, which the run borrowed.
    saved: (Action, HashSet<FileId>),
    since: Instant,
    /// It is a "Suggest tags" run.
    tags: bool,
}

impl FolderWorkspace {
    /// The open clip's `action` has been asked for: it runs now, or waits behind the job.
    pub(super) fn run_on_open_clip(&mut self, action: Action) -> Task<Message> {
        let Some(id) = self.file_workspace.file().map(|f| f.id()) else {
            return Task::none();
        };
        let label = action.label();
        // Pressed again while asked for or in work: the button takes the run back.
        if self.clip_action(action, id) != ClipAction::Idle {
            return self.withdraw_run(action, id);
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

    /// "Suggest tags" for the open clip (#250): a "Describe with AI" run that suggests the folder's
    /// tags whatever Settings say, through the same line as every slow run. The suggestions show
    /// in the file-name panel (F6 adds them), like those of a batch run.
    pub(super) fn suggest_tags_on_open_clip(&mut self) -> Task<Message> {
        let Some(id) = self.file_workspace.file().map(|f| f.id()) else {
            return Task::none();
        };
        // Pressed again while asked for or in work: the button takes the run back.
        if self.suggest_tags_state(id) != ClipAction::Idle {
            self.solo.tags.remove(&id);
            return self.withdraw_run(Action::DescribeAi, id);
        }
        if self.file_workspace.tag_list().saved_tag_names().is_empty() {
            return Self::notice(&fl!("batch-suggest-no-tags"));
        }
        if self.clip_action(Action::DescribeAi, id) != ClipAction::Idle {
            // A description of this clip waits in the line: it suggests tags too. One in a job
            // (waiting for its turn there, or in work) was made without them: a note says so.
            let in_line = self.solo.waiting.contains(&(Action::DescribeAi, id))
                || self
                    .solo
                    .preparing
                    .as_ref()
                    .is_some_and(|run| run.action == Action::DescribeAi && run.file == id);
            if !in_line {
                return Self::notice(&fl!("batch-suggest-in-job"));
            }
            self.solo.tags.insert(id);
            // Already preparing: its options were set when it began, set them again.
            if let Some(run) = self.solo.preparing.as_mut().filter(|run| run.file == id) {
                run.tags = true;
                self.batch.request_tags(true);
            }
            return Task::none();
        }
        self.solo.tags.insert(id);
        self.run_on_open_clip(Action::DescribeAi)
    }

    /// Forget the "Suggest tags" marks of runs that are not in the line any more (the job that
    /// they started has ended).
    pub(super) fn prune_suggest_tags(&mut self) {
        let waiting = &self.solo.waiting;
        let preparing = self.solo.preparing.as_ref().map(|run| run.file);
        self.solo
            .tags
            .retain(|id| preparing == Some(*id) || waiting.contains(&(Action::DescribeAi, *id)));
    }

    /// Where "Suggest tags" stands for `clip`.
    pub fn suggest_tags_state(&self, clip: FileId) -> ClipAction {
        if self.solo.tags.contains(&clip) || self.solo.tags_running == Some(clip) {
            self.clip_action(Action::DescribeAi, clip)
        } else {
            ClipAction::Idle
        }
    }

    /// Take the run of `action` on `clip` back: from the line, from the preparing, from a job that
    /// has not reached the clip. A job already working on the clip is not stopped from here (that
    /// would stop all its clips): a note says where to cancel it.
    pub(super) fn withdraw_run(&mut self, action: Action, clip: FileId) -> Task<Message> {
        let label = action.label();
        if action == Action::DescribeAi {
            self.solo.tags.remove(&clip);
        }
        if self
            .solo
            .preparing
            .as_ref()
            .is_some_and(|run| run.action == action && run.file == clip)
        {
            self.end_preparing();
        } else if let Some(at) = self.solo.waiting.iter().position(|w| *w == (action, clip)) {
            self.solo.waiting.remove(at);
        } else if self.batch.job_action() == Some(action) && self.batch.withdraw(clip) {
            // Taken out of the running job's line before its turn.
        } else if self.batch.is_running() && self.batch.current() == Some(clip) {
            // In work: stopping it stops the whole job, which is the strip's and the page's Cancel.
            return Self::notice(&fl!("batch-solo-in-work", action = label));
        } else {
            return Task::none();
        }
        Self::notice(&fl!("batch-solo-withdrawn", action = label))
    }

    /// Demo: two runs on the last two clips of the list wait in line, as if asked for.
    pub(super) fn show_waiting_runs(&mut self) {
        let Some(dir) = self.directory.as_ref() else {
            return;
        };
        let ids: Vec<FileId> = dir.files_in_order().map(|f| f.id()).collect();
        for (action, id) in [Action::DescribeAi, Action::GenerateSubtitles]
            .into_iter()
            .zip(ids.iter().rev())
        {
            self.solo.waiting.push_back((action, *id));
        }
    }

    /// The runs of one clip waiting for their turn, the one preparing first: for the queue list of
    /// the job page.
    pub fn waiting_runs(&self) -> Vec<(Action, FileId)> {
        // A clip a rescan renewed is not listed: its row would have no name.
        self.solo
            .preparing
            .iter()
            .map(|run| (run.action, run.file))
            .chain(self.solo.waiting.iter().copied())
            .filter(|(_, id)| {
                self.directory
                    .as_ref()
                    .is_some_and(|d| d.file_by_id(*id).is_some())
            })
            .collect()
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
                suggest_tags: ClipAction::Idle,
            };
        };
        ClipActions {
            describe: self.clip_action(Action::DescribeAi, id),
            subtitles: self.clip_action(Action::GenerateSubtitles, id),
            suggest_tags: self.suggest_tags_state(id),
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
                // A "Suggest tags" job keeps its button latched until it ends.
                if run.tags {
                    self.solo.tags_running = Some(run.file);
                }
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
                    // Waiting for the marker requests to let go of the clip: the job starts by
                    // itself then.
                    if self.batch.is_waiting_for_markers() {
                        return Task::none();
                    }
                    let tags = run.tags;
                    let file_id = run.file;
                    let started = self.start_batch();
                    if self.batch.is_running() {
                        if tags {
                            self.solo.tags_running = Some(file_id);
                        }
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
                self.solo.tags.remove(&file);
                continue;
            }
            let Some(saved) = self.batch.borrow_for_one(action, file) else {
                self.solo.waiting.push_front((action, file));
                return Task::none();
            };
            let tags = action == Action::DescribeAi && self.solo.tags.contains(&file);
            self.solo.preparing = Some(Preparing {
                action,
                file,
                saved,
                since: Instant::now(),
                tags,
            });
            self.share_folder_tags();
            self.batch.request_tags(tags);
            let reads = Task::batch([self.describe_ai_reads(), self.subtitle_reads()]);
            // Nothing to read (a free action, or everything cached): it may start at once.
            return Task::batch([reads, self.advance_solo()]);
        }
        Task::none()
    }

    /// Cancel stops the line too: the run that prepares and the ones that wait.
    pub(super) fn cancel_solo(&mut self) {
        self.solo.waiting.clear();
        self.solo.tags.clear();
        self.end_preparing();
    }

    /// The folder changed: the runs asked for belong to the one that was open.
    pub(super) fn forget_solo(&mut self) {
        // A run that prepared had set the flag of the next description job.
        self.batch.request_tags(false);
        self.solo = Solo::default();
    }

    /// The run is over with preparing (started, blocked, or given up): the selection comes back.
    fn end_preparing(&mut self) {
        if let Some(run) = self.solo.preparing.take() {
            self.solo.tags.remove(&run.file);
            self.batch.request_tags(false);
            // A start that waited for the marker requests is not wanted any more.
            self.batch.update(batch::Message::StopWaiting);
            self.batch.restore_selection(run.saved);
        }
    }
}
