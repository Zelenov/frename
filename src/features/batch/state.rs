//! State for batch mode: the checked files, the chosen action, and the job running it.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use frename_core::{FileId, FileSnapshot};

use super::actions::generate_subtitles::PlanRequest;
use super::actions::Actions;
use super::{Action, ActionMessage, Message, Operation};
use crate::soniox_key::SonioxKey;

/// Where one file of a job stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStatus {
    /// Not reached yet; after a cancel, never reached. An operation cancelled in the middle of
    /// a file returns it too: that file was not done, and not failed either.
    Pending,
    Running,
    /// Changed.
    Done,
    /// Nothing to do for this file.
    Skipped,
    /// Could not be done; the details are in the log.
    Failed,
}

/// What an operation did to one file.
#[derive(Debug, Clone)]
pub struct ItemResult {
    pub status: ItemStatus,
    /// The file's path and snapshot afterwards, when the operation touched it.
    pub update: Option<(PathBuf, FileSnapshot)>,
    /// Why the file was left alone or failed, shown after its name in the job's results.
    pub reason: Option<String>,
    /// Stop the job after this file: the rest would fail the same way (e.g. a rejected key).
    pub stop_job: Option<JobStop>,
}

impl ItemResult {
    pub fn new(status: ItemStatus, update: Option<(PathBuf, FileSnapshot)>) -> Self {
        Self {
            status,
            update,
            reason: None,
            stop_job: None,
        }
    }
}

/// Why a job stopped before its last file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobStop {
    /// The job's summary line.
    pub message: String,
    /// Whether the settings window can fix it (e.g. a rejected key).
    pub open_settings: bool,
}

/// One run of an operation over the files checked when it started.
#[derive(Debug, Clone)]
struct Job {
    operation: Operation,
    /// Files in the order they are processed.
    order: Vec<FileId>,
    statuses: HashMap<FileId, ItemStatus>,
    /// Why files were left alone or failed, for the results list.
    reasons: HashMap<FileId, String>,
    /// Index into `order` of the next file to start.
    next: usize,
    cancelled: bool,
    /// Set when a file's result stopped the job.
    stopped: Option<JobStop>,
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
        // Nothing changes under a running job but its own cancel, and answers from outside
        // (the settings, the price lookup) that the next run needs.
        let passes = matches!(
            message,
            Message::Cancel
                | Message::SubtitlePriceReady { .. }
                | Message::Action(ActionMessage::GenerateSubtitles(
                    super::actions::generate_subtitles::Message::SetConfig(_)
                ))
        );
        if self.is_running() && !passes {
            return;
        }
        // Whatever changes the checked files changes what they need.
        if matches!(
            message,
            Message::Prepare { .. }
                | Message::Toggle(_)
                | Message::CheckAll(_)
                | Message::CheckNone
                | Message::Invert(_)
                | Message::SelectAction(_)
                | Message::CloseReport
                // Files may have been renamed or got subtitles outside batch mode.
                | Message::SetActive(true)
        ) {
            self.actions.generate_subtitles().invalidate_plan();
        }
        match message {
            Message::SetActive(active) => self.active = active,
            Message::SelectAction(action) => self.action = action,
            Message::Action(message) => self.actions.update(message),
            Message::SubtitlePlanReady { generation, plan } => self
                .actions
                .generate_subtitles()
                .plan_ready(generation, *plan),
            Message::SubtitlePriceReady { key, price } => {
                self.actions.generate_subtitles().price_ready(key, price)
            }
            Message::Prepare { operation, files } => {
                self.actions.prepare(&operation);
                self.action = operation.action();
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
                if let Some(job) = self.job.as_mut().filter(|job| job.running) {
                    job.cancelled = true;
                    job.operation.cancel();
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
            reasons: HashMap::new(),
            next: 0,
            cancelled: false,
            stopped: None,
            running: true,
            dismissed: HashSet::new(),
        });
        true
    }

    /// Mark the next file running and return it with the operation, or end the job when it is
    /// cancelled or has no files left (`None`).
    pub fn begin_next(&mut self) -> Option<(FileId, Operation)> {
        let job = self.job.as_mut().filter(|job| job.running)?;
        let stop = job.cancelled || job.stopped.is_some();
        let Some(id) = job.order.get(job.next).copied().filter(|_| !stop) else {
            job.running = false;
            // The job changed the files: what they need is worked out again.
            self.actions.generate_subtitles().invalidate_plan();
            return None;
        };
        job.next += 1;
        job.statuses.insert(id, ItemStatus::Running);
        Some((id, job.operation.clone()))
    }

    /// Record how a file of the running job went.
    pub fn finish(&mut self, id: FileId, result: &ItemResult) {
        if let Some(job) = self.job.as_mut() {
            job.statuses.insert(id, result.status);
            if let Some(reason) = &result.reason {
                job.reasons.insert(id, reason.clone());
            }
            if job.stopped.is_none() {
                job.stopped = result.stop_job.clone();
            }
        }
    }

    /// The plan and price the selected subtitle action is waiting for, to be worked out by
    /// the workspace; each is handed out once per change.
    pub fn take_subtitle_requests(&mut self) -> (Option<PlanRequest>, Option<SonioxKey>) {
        if !self.active || self.action != Action::GenerateSubtitles || self.is_running() {
            return (None, None);
        }
        let options = self.actions.generate_subtitles();
        (options.take_plan_request(), options.take_price_request())
    }

    /// Whether the selected subtitle action waits for the key to be read from the store.
    pub fn needs_subtitle_key(&mut self) -> bool {
        self.active
            && self.action == Action::GenerateSubtitles
            && self.actions.generate_subtitles().needs_key()
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
        self.actions.generate_subtitles().invalidate_plan();
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

    /// Files of the current job to list after it: the failed ones, and those left alone for a
    /// reason, in job order.
    pub fn results(&self) -> Vec<(FileId, ItemStatus, Option<&str>)> {
        self.job.as_ref().map_or_else(Vec::new, |job| {
            job.order
                .iter()
                .filter_map(|id| {
                    let status = *job.statuses.get(id)?;
                    let reason = job.reasons.get(id).map(String::as_str);
                    let listed = status == ItemStatus::Failed
                        || (status == ItemStatus::Skipped && reason.is_some());
                    listed.then_some((*id, status, reason))
                })
                .collect()
        })
    }

    /// Why the current job stopped before its last file, if a file stopped it.
    pub fn stopped(&self) -> Option<&JobStop> {
        self.job.as_ref().and_then(|job| job.stopped.as_ref())
    }

    /// The action of the current job, which the panel's labels follow while it is shown.
    pub fn job_action(&self) -> Option<Action> {
        self.job.as_ref().map(|job| job.operation.action())
    }

    /// The current job's own summary line (e.g. what it spent), when it has one.
    pub fn report(&self) -> Option<String> {
        self.job.as_ref().and_then(|job| job.operation.report())
    }

    /// The run button of the selected action: label, and whether it can run now.
    pub fn run_button(&self) -> (String, bool) {
        let (label, enabled) = self.actions.run_button(self.action, self.checked_count());
        (
            label,
            enabled && self.checked_count() > 0 && !self.is_running(),
        )
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

        let (first, operation) = batch.begin_next().expect("first file");
        assert_eq!(
            (first, operation),
            (files[0], Operation::MoveComments(CommentStorage::InVideo))
        );
        assert_eq!(batch.current(), Some(files[0]));
        batch.finish(first, &ItemResult::new(ItemStatus::Done, None));

        batch.update(Message::Toggle(files[0]));
        assert!(
            !batch.is_checked(files[0]),
            "checks are locked while running"
        );

        batch.update(Message::Cancel);
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
        let (id, _) = batch.begin_next().expect("the file");
        batch.finish(id, &ItemResult::new(ItemStatus::Failed, None));
        assert!(batch.begin_next().is_none());
        assert_eq!(batch.results(), vec![(files[0], ItemStatus::Failed, None)]);
    }

    #[test]
    fn unchecking_a_finished_file_shows_its_check_box_again_and_keeps_the_report() {
        let files = ids(2);
        let mut batch = state();
        batch.update(Message::CheckAll(files.clone()));
        batch.start(files.clone());
        for _ in 0..2 {
            let (id, _) = batch.begin_next().expect("a file");
            batch.finish(id, &ItemResult::new(ItemStatus::Done, None));
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
    fn a_result_that_stops_the_job_ends_it_and_says_why() {
        let files = ids(3);
        let mut batch = state();
        batch.start(files.clone());
        let (id, _) = batch.begin_next().expect("first file");
        let stop = JobStop {
            message: "Soniox rejected the key".to_string(),
            open_settings: true,
        };
        batch.finish(
            id,
            &ItemResult {
                reason: Some("failed: Soniox: bad key".to_string()),
                stop_job: Some(stop.clone()),
                ..ItemResult::new(ItemStatus::Failed, None)
            },
        );
        assert!(batch.begin_next().is_none(), "the rest is not reached");
        assert_eq!(batch.stopped(), Some(&stop));
        assert_eq!(
            batch.results(),
            vec![(
                files[0],
                ItemStatus::Failed,
                Some("failed: Soniox: bad key")
            )]
        );
        assert_eq!(batch.status(files[1]), Some(ItemStatus::Pending));
    }

    #[test]
    fn files_left_alone_for_a_reason_are_listed_after_the_job() {
        let files = ids(3);
        let mut batch = state();
        batch.start(files.clone());
        let results = [
            ItemResult::new(ItemStatus::Done, None),
            ItemResult {
                reason: Some("no speech".to_string()),
                ..ItemResult::new(ItemStatus::Skipped, None)
            },
            ItemResult::new(ItemStatus::Skipped, None),
        ];
        for result in &results {
            let (id, _) = batch.begin_next().expect("a file");
            batch.finish(id, result);
        }
        assert_eq!(
            batch.results(),
            vec![(files[1], ItemStatus::Skipped, Some("no speech"))],
            "a file left alone without a reason is not listed"
        );
    }

    #[test]
    fn a_file_cancelled_in_the_middle_is_not_reached() {
        let files = ids(2);
        let mut batch = state();
        batch.start(files.clone());
        let (id, _) = batch.begin_next().expect("first file");
        batch.update(Message::Cancel);
        batch.finish(id, &ItemResult::new(ItemStatus::Pending, None));
        assert!(batch.begin_next().is_none());
        let progress = batch.progress().expect("report");
        assert_eq!((progress.finished, progress.failed), (0, 0));
        assert_eq!(batch.status(files[0]), Some(ItemStatus::Pending));
    }

    #[test]
    fn every_action_runs_and_none_runs_without_files() {
        let mut batch = state();
        assert!(!batch.start(Vec::new()), "nothing checked");
        for action in Action::ALL {
            batch.update(Message::SelectAction(action));
            if action == Action::GenerateSubtitles {
                // Waits for its key, plan and price.
                assert_eq!(batch.operation(), None);
                continue;
            }
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
}
