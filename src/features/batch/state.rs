//! State for batch mode: the checked files, the chosen action, and the job running it.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use frename_core::ai::provider::AiUsage;
use frename_core::{File, FileId, FileSnapshot};

use super::actions::Actions;
use super::{Action, Message, Operation};

/// Where one file of a job stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStatus {
    /// Not reached yet; after a cancel, never reached.
    Pending,
    Running,
    /// Changed.
    Done,
    /// Nothing to do for this file.
    Skipped,
    /// Could not be done; the reason is in the failed list when the action gives one, and in
    /// the log.
    Failed,
}

/// What an operation did to one file.
#[derive(Debug, Clone)]
pub struct ItemResult {
    /// `Pending` when the file was left before it was done (a cancel reached it).
    pub status: ItemStatus,
    /// The file's path and snapshot afterwards, when the operation touched it.
    pub update: Option<(PathBuf, FileSnapshot)>,
    /// Why the file failed, shown after its name in the failed list.
    pub reason: Option<String>,
    /// What the AI requests for this file cost, summed into the job's spend.
    pub usage: Option<AiUsage>,
    /// A request for this file may have been billed without saying what it cost (it timed
    /// out), so the job's spend is a lower bound.
    pub usage_unknown: bool,
    /// Stop the job after this file, leaving the rest not reached, with this summary line.
    pub stop_job: Option<String>,
    /// Stop the job with this summary line once [`REPEATS_THAT_STOP`] files in a row end with
    /// it (e.g. the network is gone), instead of failing every file left the same way.
    pub stop_if_repeated: Option<String>,
}

/// How many files in a row may fail the same way before the job stops.
pub const REPEATS_THAT_STOP: usize = 3;

impl ItemResult {
    pub fn new(status: ItemStatus, update: Option<(PathBuf, FileSnapshot)>) -> Self {
        Self {
            status,
            update,
            reason: None,
            usage: None,
            usage_unknown: false,
            stop_job: None,
            stop_if_repeated: None,
        }
    }

    /// A failure with the reason the failed list shows.
    pub fn failed(reason: impl Into<String>) -> Self {
        Self {
            reason: Some(reason.into()),
            ..Self::new(ItemStatus::Failed, None)
        }
    }
}

/// One run of an operation over the files checked when it started.
#[derive(Debug, Clone)]
struct Job {
    operation: Operation,
    /// Files in the order they are processed.
    order: Vec<FileId>,
    statuses: HashMap<FileId, ItemStatus>,
    /// Index into `order` of the next file to start.
    next: usize,
    cancelled: bool,
    /// Set together with `cancelled`, for the file in work to see.
    cancel: Arc<AtomicBool>,
    /// Why failed files failed, when their action said.
    reasons: HashMap<FileId, String>,
    /// What the job's AI requests cost so far; `None` when it made none.
    usage: Option<AiUsage>,
    /// Some request may have been billed without reporting its cost.
    usage_unknown: bool,
    /// Why a file stopped the job, if one did.
    stopped: Option<String>,
    /// The last files' repeated failure and how many in a row ended with it.
    repeated: Option<(String, usize)>,
    /// False once the job has finished or stopped; the report stays until it is closed.
    running: bool,
    /// Files whose check changed after the job: the list shows their check box again, while
    /// the report still counts them.
    dismissed: HashSet<FileId>,
}

impl Job {
    fn count(&self, status: ItemStatus) -> usize {
        self.statuses.values().filter(|s| **s == status).count()
    }
}

/// How far a job is, for the job panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub total: usize,
    /// Files finished, whatever their outcome.
    pub finished: usize,
    pub done: usize,
    pub skipped: usize,
    pub failed: usize,
    pub running: bool,
    pub cancelled: bool,
    /// What the job's AI requests cost so far; `None` when it made none.
    pub usage: Option<AiUsage>,
    /// `usage` is a lower bound: some request did not report its cost.
    pub usage_unknown: bool,
}

/// Batch mode state.
#[derive(Debug, Clone)]
pub struct BatchState {
    active: bool,
    action: Action,
    /// The options of every action.
    actions: Actions,
    checked: HashSet<FileId>,
    job: Option<Job>,
}

impl Default for BatchState {
    fn default() -> Self {
        Self {
            active: false,
            action: Action::MoveComments,
            actions: Actions::default(),
            checked: HashSet::new(),
            job: None,
        }
    }
}

impl BatchState {
    /// Apply a message. `Run` is started by the workspace through [`Self::start`].
    pub fn update(&mut self, message: Message) {
        // Nothing changes under a running job but its own cancel, and what background reads
        // bring in (clip lengths, the key's state).
        let background = matches!(&message, Message::Action(m) if m.applies_while_running());
        if self.is_running() && !matches!(message, Message::Cancel) && !background {
            return;
        }
        match message {
            Message::SetActive(active) => self.active = active,
            Message::SelectAction(action) => self.action = action,
            Message::Action(message) => self.actions.update(message),
            Message::Prepare { operation, files } => {
                self.action = operation.action();
                self.actions.prepare(operation);
                self.active = true;
                self.job = None;
                self.checked = files.into_iter().collect();
            }
            Message::Toggle(id) => {
                self.dismiss([id]);
                if !self.checked.remove(&id) {
                    self.checked.insert(id);
                }
            }
            Message::CheckAll(ids) => {
                self.dismiss(ids.iter().copied());
                self.checked.extend(ids);
            }
            Message::CheckNone => {
                self.dismiss(self.checked.clone());
                self.checked.clear();
            }
            Message::Invert(ids) => {
                self.dismiss(ids.iter().copied());
                for id in ids {
                    if !self.checked.remove(&id) {
                        self.checked.insert(id);
                    }
                }
            }
            Message::Cancel => {
                if let Some(job) = self.job.as_mut() {
                    job.cancelled = true;
                    job.cancel.store(true, Ordering::Relaxed);
                }
            }
            Message::CloseReport => self.job = None,
            Message::Run => {}
        }
    }

    /// Start the selected action on `files` (the checked ones, in list order). Returns false
    /// when there is nothing to run: no files, an action that is not available, or a job
    /// already running.
    pub fn start(&mut self, files: Vec<FileId>) -> bool {
        let Some(operation) = self
            .operation()
            .filter(|_| !files.is_empty() && !self.is_running())
        else {
            return false;
        };
        let statuses = files.iter().map(|id| (*id, ItemStatus::Pending)).collect();
        self.job = Some(Job {
            operation,
            order: files,
            statuses,
            next: 0,
            cancelled: false,
            cancel: Arc::new(AtomicBool::new(false)),
            reasons: HashMap::new(),
            usage: None,
            usage_unknown: false,
            stopped: None,
            repeated: None,
            running: true,
            dismissed: HashSet::new(),
        });
        true
    }

    /// Mark the next file running and return it with the operation and the cancel flag it
    /// checks, or end the job when it is cancelled, stopped, or has no files left (`None`).
    pub fn begin_next(&mut self) -> Option<(FileId, Operation, Arc<AtomicBool>)> {
        let job = self.job.as_mut().filter(|job| job.running)?;
        let Some(id) = job
            .order
            .get(job.next)
            .copied()
            .filter(|_| !job.cancelled && job.stopped.is_none())
        else {
            job.running = false;
            if let Some(usage) = job.usage {
                log::info!(
                    "batch: {} spent {}",
                    job.operation.action().label(),
                    super::actions::spend_line(usage)
                );
            }
            return None;
        };
        job.next += 1;
        job.statuses.insert(id, ItemStatus::Running);
        Some((id, job.operation.clone(), job.cancel.clone()))
    }

    /// Record how a file of the running job went.
    pub fn finish(&mut self, id: FileId, result: &ItemResult) {
        let Some(job) = self.job.as_mut() else {
            return;
        };
        job.statuses.insert(id, result.status);
        if let Some(reason) = result.reason.clone() {
            job.reasons.insert(id, reason);
        }
        if let Some(usage) = result.usage {
            *job.usage.get_or_insert_with(AiUsage::default) += usage;
        }
        if result.usage_unknown {
            job.usage.get_or_insert_with(AiUsage::default);
            job.usage_unknown = true;
        }
        if let Some(stop) = result.stop_job.clone() {
            job.stopped = Some(stop);
        }
        job.repeated = match (job.repeated.take(), result.stop_if_repeated.clone()) {
            (Some((last, n)), Some(stop)) if last == stop => Some((stop, n + 1)),
            (_, Some(stop)) => Some((stop, 1)),
            (_, None) => None,
        };
        if let Some((stop, n)) = &job.repeated {
            if *n >= REPEATS_THAT_STOP && job.stopped.is_none() {
                job.stopped = Some(stop.clone());
            }
        }
    }

    /// Why a file stopped the current job, if one did.
    pub fn stopped(&self) -> Option<&str> {
        self.job.as_ref().and_then(|job| job.stopped.as_deref())
    }

    /// Show the check boxes of `ids` again instead of their outcome in the finished job.
    fn dismiss(&mut self, ids: impl IntoIterator<Item = FileId>) {
        if let Some(job) = self.job.as_mut() {
            job.dismissed.extend(ids);
        }
    }

    /// Forget the checks and the job: they belong to the folder that was open.
    pub fn reset_files(&mut self) {
        self.checked.clear();
        self.job = None;
        self.actions.describe_ai_mut().reset_files();
    }

    /// What "Describe with AI" needs read from disk while its panel is shown for `checked`: the
    /// videos whose length is not known or asked for yet, and whether the key's state still has
    /// to be read. Both are marked as asked for.
    pub fn describe_ai_reads<'a>(
        &mut self,
        checked: impl Iterator<Item = &'a File>,
    ) -> (Vec<(FileId, PathBuf)>, bool) {
        if !self.active || self.action != Action::DescribeAi {
            return (Vec::new(), false);
        }
        let options = self.actions.describe_ai_mut();
        (options.missing_probes(checked), options.request_key_state())
    }

    /// Whether the right half shows the batch panel instead of the open file.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Whether a job is running; the folder and its files are locked meanwhile.
    pub fn is_running(&self) -> bool {
        self.job.as_ref().is_some_and(|job| job.running)
    }

    pub fn action(&self) -> Action {
        self.action
    }

    /// The options of every action, for the selected action's panel.
    pub fn actions(&self) -> &Actions {
        &self.actions
    }

    /// What the selected action would do with its options; `None` when it cannot run.
    pub fn operation(&self) -> Option<Operation> {
        self.actions.operation(self.action)
    }

    pub fn is_checked(&self, id: FileId) -> bool {
        self.checked.contains(&id)
    }

    pub fn checked_count(&self) -> usize {
        self.checked.len()
    }

    /// Where a file stands in the current job, if it is part of one and its check has not
    /// changed since.
    pub fn status(&self, id: FileId) -> Option<ItemStatus> {
        let job = self
            .job
            .as_ref()
            .filter(|job| !job.dismissed.contains(&id))?;
        job.statuses.get(&id).copied()
    }

    /// The file the job is working on.
    pub fn current(&self) -> Option<FileId> {
        let job = self.job.as_ref().filter(|job| job.running)?;
        job.order
            .iter()
            .copied()
            .find(|id| job.statuses.get(id) == Some(&ItemStatus::Running))
    }

    /// Files of the current job that failed, in job order, with the reason when their action
    /// gave one.
    pub fn failed(&self) -> Vec<(FileId, Option<&str>)> {
        self.job.as_ref().map_or_else(Vec::new, |job| {
            job.order
                .iter()
                .copied()
                .filter(|id| job.statuses.get(id) == Some(&ItemStatus::Failed))
                .map(|id| (id, job.reasons.get(&id).map(String::as_str)))
                .collect()
        })
    }

    /// How far the current job is; `None` without one.
    pub fn progress(&self) -> Option<Progress> {
        let job = self.job.as_ref()?;
        let done = job.count(ItemStatus::Done);
        let skipped = job.count(ItemStatus::Skipped);
        let failed = job.count(ItemStatus::Failed);
        Some(Progress {
            total: job.order.len(),
            finished: done + skipped + failed,
            done,
            skipped,
            failed,
            running: job.running,
            cancelled: job.cancelled,
            usage: job.usage,
            usage_unknown: job.usage_unknown,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frename_core::{CommentStorage, InOutStorage};

    fn ids(n: usize) -> Vec<FileId> {
        (0..n)
            .map(|i| {
                frename_core::File::from_path(
                    format!("C:/none/{i}.mp4"),
                    std::time::SystemTime::UNIX_EPOCH,
                )
                .id()
            })
            .collect()
    }

    fn done() -> ItemResult {
        ItemResult::new(ItemStatus::Done, None)
    }

    fn state() -> BatchState {
        let mut state = BatchState::default();
        state.update(Message::Prepare {
            operation: Operation::MoveComments(CommentStorage::InVideo),
            files: Vec::new(),
        });
        state.update(Message::SetActive(false));
        state
    }

    #[test]
    fn checks_toggle_add_and_invert() {
        let files = ids(3);
        let mut batch = state();
        batch.update(Message::Toggle(files[0]));
        assert!(batch.is_checked(files[0]));
        batch.update(Message::Invert(files.clone()));
        assert!(!batch.is_checked(files[0]));
        assert_eq!(batch.checked_count(), 2);
        batch.update(Message::CheckAll(files.clone()));
        assert_eq!(batch.checked_count(), 3);
        batch.update(Message::CheckNone);
        assert_eq!(batch.checked_count(), 0);
    }

    #[test]
    fn a_job_runs_its_files_in_order_and_stops_after_a_cancel() {
        let files = ids(3);
        let mut batch = state();
        assert!(batch.start(files.clone()));
        assert!(!batch.start(files.clone()), "one job at a time");

        let (first, operation, cancel) = batch.begin_next().expect("first file");
        assert_eq!(
            (first, operation),
            (files[0], Operation::MoveComments(CommentStorage::InVideo))
        );
        assert_eq!(batch.current(), Some(files[0]));
        batch.finish(first, &done());

        batch.update(Message::Toggle(files[0]));
        assert!(
            !batch.is_checked(files[0]),
            "checks are locked while running"
        );

        batch.update(Message::Cancel);
        assert!(
            cancel.load(Ordering::Relaxed),
            "the file in work sees the cancel"
        );
        assert!(
            batch.begin_next().is_none(),
            "cancelled before the second file"
        );
        assert!(!batch.is_running());
        let progress = batch.progress().expect("report");
        assert_eq!(
            (progress.finished, progress.done, progress.total),
            (1, 1, 3)
        );
        assert_eq!(batch.status(files[1]), Some(ItemStatus::Pending));

        batch.update(Message::CloseReport);
        assert!(batch.progress().is_none());
    }

    #[test]
    fn a_job_ends_after_its_last_file() {
        let files = ids(1);
        let mut batch = state();
        batch.start(files.clone());
        let (id, _, _) = batch.begin_next().expect("the file");
        batch.finish(id, &ItemResult::new(ItemStatus::Failed, None));
        assert!(batch.begin_next().is_none());
        assert_eq!(batch.failed(), vec![(files[0], None)]);
    }

    #[test]
    fn unchecking_a_finished_file_shows_its_check_box_again_and_keeps_the_report() {
        let files = ids(2);
        let mut batch = state();
        batch.update(Message::CheckAll(files.clone()));
        batch.start(files.clone());
        for _ in 0..2 {
            let (id, _, _) = batch.begin_next().expect("a file");
            batch.finish(id, &done());
        }
        assert!(batch.begin_next().is_none());

        batch.update(Message::Toggle(files[0]));
        assert!(!batch.is_checked(files[0]));
        assert_eq!(batch.status(files[0]), None, "check box shown again");
        assert_eq!(batch.status(files[1]), Some(ItemStatus::Done));
        assert_eq!(
            batch.progress().map(|p| p.done),
            Some(2),
            "the report still counts it"
        );
    }

    #[test]
    fn every_action_runs_and_none_runs_without_files() {
        let mut batch = state();
        assert!(!batch.start(Vec::new()), "nothing checked");
        for action in Action::ALL {
            batch.update(Message::SelectAction(action));
            assert_eq!(batch.operation().map(|op| op.action()), Some(action));
        }
        batch.update(Message::SelectAction(Action::FixTags));
        assert!(batch.start(ids(2)));
    }

    #[test]
    fn tagging_commented_videos_runs_with_the_default_tag() {
        let mut batch = state();
        batch.update(Message::SelectAction(Action::TagCommented));
        assert_eq!(batch.operation(), Some(Operation::TagCommented));
        assert!(batch.start(ids(1)));
    }

    #[test]
    fn prepare_checks_the_files_and_sets_up_the_action() {
        let files = ids(2);
        let mut batch = state();
        batch.update(Message::Prepare {
            operation: Operation::MoveInOut(InOutStorage::InVideo),
            files: files.clone(),
        });
        assert!(batch.is_active());
        assert_eq!(
            batch.operation(),
            Some(Operation::MoveInOut(InOutStorage::InVideo))
        );
        assert_eq!(batch.checked_count(), 2);
    }

    #[test]
    fn a_failed_files_reason_is_kept_and_usage_is_summed() {
        let files = ids(3);
        let mut batch = state();
        batch.start(files.clone());
        let usage = AiUsage {
            input_tokens: 1000,
            output_tokens: 100,
        };
        let (id, _, _) = batch.begin_next().expect("first");
        batch.finish(
            id,
            &ItemResult {
                usage: Some(usage),
                ..ItemResult::failed("Network error")
            },
        );
        let (id, _, _) = batch.begin_next().expect("second");
        batch.update(Message::Cancel);
        batch.finish(
            id,
            &ItemResult {
                usage: Some(usage),
                ..done()
            },
        );
        assert!(batch.begin_next().is_none());
        assert_eq!(batch.failed(), vec![(files[0], Some("Network error"))]);
        let progress = batch.progress().expect("report");
        assert_eq!(
            progress.usage,
            Some(AiUsage {
                input_tokens: 2000,
                output_tokens: 200
            }),
            "usage counts after a cancel too"
        );
    }

    #[test]
    fn a_file_can_stop_the_job_and_leave_the_rest_not_reached() {
        let files = ids(3);
        let mut batch = state();
        batch.start(files.clone());
        let (id, _, _) = batch.begin_next().expect("first");
        batch.finish(
            id,
            &ItemResult {
                stop_job: Some("Stopped: the key was rejected.".to_string()),
                ..ItemResult::failed("Anthropic rejected the key")
            },
        );
        assert!(batch.begin_next().is_none());
        assert_eq!(batch.stopped(), Some("Stopped: the key was rejected."));
        assert_eq!(batch.status(files[1]), Some(ItemStatus::Pending));
        assert_eq!(batch.status(files[2]), Some(ItemStatus::Pending));
    }

    #[test]
    fn a_file_left_by_a_cancel_stays_not_reached() {
        let files = ids(2);
        let mut batch = state();
        batch.start(files.clone());
        let (id, _, _) = batch.begin_next().expect("first");
        batch.update(Message::Cancel);
        batch.finish(id, &ItemResult::new(ItemStatus::Pending, None));
        assert!(batch.begin_next().is_none());
        assert_eq!(batch.progress().map(|p| p.finished), Some(0));
    }

    #[test]
    fn the_same_failure_three_times_in_a_row_stops_the_job() {
        let files = ids(6);
        let mut batch = state();
        batch.start(files.clone());
        let offline = || ItemResult {
            stop_if_repeated: Some("Stopped: no connection.".to_string()),
            ..ItemResult::failed("Network error")
        };
        for result in [offline(), offline(), done(), offline(), offline()] {
            let (id, _, _) = batch.begin_next().expect("a file");
            batch.finish(id, &result);
        }
        assert!(
            batch.stopped().is_none(),
            "a success in between resets the count"
        );
        let (id, _, _) = batch.begin_next().expect("the sixth file");
        batch.finish(id, &offline());
        assert!(batch.begin_next().is_none());
        assert_eq!(batch.stopped(), Some("Stopped: no connection."));
    }
}
