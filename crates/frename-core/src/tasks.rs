//! The queue of slow tasks (#249): what is waiting, what runs, what ended, and which clip is busy.
//!
//! Pure bookkeeping, no threads and no I/O: the app starts the work and reports back. The queue
//! is generic over the action so the batch actions of the app stay out of the core.

use crate::FileId;
use std::collections::VecDeque;

/// Identifies one task for the life of the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

/// Where a task is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    Queued,
    Running,
    Done,
    /// Ended with an error; the text is what the editor reads.
    Failed(String),
    Cancelled,
}

impl TaskState {
    /// The task will not run again unless it is retried.
    pub fn is_finished(&self) -> bool {
        !matches!(self, Self::Queued | Self::Running)
    }
}

/// One action on one clip.
#[derive(Debug, Clone, PartialEq)]
pub struct Task<A> {
    pub id: TaskId,
    pub file: FileId,
    pub action: A,
    pub state: TaskState,
}

/// Tasks in the order they were added. One runs at a time; the rest wait.
#[derive(Debug, Clone)]
pub struct TaskQueue<A> {
    tasks: VecDeque<Task<A>>,
    next_id: u64,
}

impl<A> Default for TaskQueue<A> {
    fn default() -> Self {
        Self {
            tasks: VecDeque::new(),
            next_id: 0,
        }
    }
}

impl<A: Clone> TaskQueue<A> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a task at the back.
    pub fn push(&mut self, file: FileId, action: A) -> TaskId {
        let id = TaskId(self.next_id);
        self.next_id += 1;
        self.tasks.push_back(Task {
            id,
            file,
            action,
            state: TaskState::Queued,
        });
        id
    }

    pub fn get(&self, id: TaskId) -> Option<&Task<A>> {
        self.tasks.iter().find(|task| task.id == id)
    }

    pub fn tasks(&self) -> impl Iterator<Item = &Task<A>> {
        self.tasks.iter()
    }

    /// The task that is running now.
    pub fn running(&self) -> Option<&Task<A>> {
        self.tasks
            .iter()
            .find(|task| task.state == TaskState::Running)
    }

    /// True while a task of this clip runs: its own saves and renames wait for the item to end.
    pub fn is_busy(&self, file: FileId) -> bool {
        self.running().is_some_and(|task| task.file == file)
    }

    pub fn queued_count(&self) -> usize {
        self.count(|state| *state == TaskState::Queued)
    }

    /// Queued or running: what the indicator counts.
    pub fn pending_count(&self) -> usize {
        self.count(|state| !state.is_finished())
    }

    pub fn failed_count(&self) -> usize {
        self.count(|state| matches!(state, TaskState::Failed(_)))
    }

    /// Nothing queued or running.
    pub fn is_idle(&self) -> bool {
        self.pending_count() == 0
    }

    /// Starts the first queued task, unless one is running. Returns it for the app to run.
    pub fn start_next(&mut self) -> Option<Task<A>> {
        if self.running().is_some() {
            return None;
        }
        let task = self
            .tasks
            .iter_mut()
            .find(|task| task.state == TaskState::Queued)?;
        task.state = TaskState::Running;
        Some(task.clone())
    }

    /// Records how the running task ended. Anything but a running task is ignored, so a late
    /// answer for a task that was cancelled cannot overwrite that.
    pub fn finish(&mut self, id: TaskId, result: Result<(), String>) {
        if let Some(task) = self
            .tasks
            .iter_mut()
            .find(|task| task.id == id && task.state == TaskState::Running)
        {
            task.state = match result {
                Ok(()) => TaskState::Done,
                Err(reason) => TaskState::Failed(reason),
            };
        }
    }

    /// A queued task is dropped from the line (kept as `Cancelled`). A running task is returned
    /// as `Some(true)` so the app raises its cancel flag; its end is still reported by
    /// [`finish_cancelled`](Self::finish_cancelled). `None` when there was nothing to cancel.
    pub fn cancel(&mut self, id: TaskId) -> Option<bool> {
        let task = self.tasks.iter_mut().find(|task| task.id == id)?;
        match task.state {
            TaskState::Queued => {
                task.state = TaskState::Cancelled;
                Some(false)
            }
            TaskState::Running => Some(true),
            _ => None,
        }
    }

    /// The running task ended because it was cancelled.
    pub fn finish_cancelled(&mut self, id: TaskId) {
        if let Some(task) = self
            .tasks
            .iter_mut()
            .find(|task| task.id == id && task.state == TaskState::Running)
        {
            task.state = TaskState::Cancelled;
        }
    }

    /// Cancels every queued task and returns the running one, if any, to be stopped.
    pub fn cancel_all(&mut self) -> Option<TaskId> {
        for task in &mut self.tasks {
            if task.state == TaskState::Queued {
                task.state = TaskState::Cancelled;
            }
        }
        self.running().map(|task| task.id)
    }

    /// Puts a failed or cancelled task at the back of the line.
    pub fn retry(&mut self, id: TaskId) -> bool {
        let Some(position) = self.tasks.iter().position(|task| {
            task.id == id && matches!(task.state, TaskState::Failed(_) | TaskState::Cancelled)
        }) else {
            return false;
        };
        let Some(mut task) = self.tasks.remove(position) else {
            return false;
        };
        task.state = TaskState::Queued;
        self.tasks.push_back(task);
        true
    }

    /// Forgets the tasks that ended, keeping the line short.
    pub fn clear_finished(&mut self) {
        self.tasks.retain(|task| !task.state.is_finished());
    }

    fn count(&self, matches: impl Fn(&TaskState) -> bool) -> usize {
        self.tasks
            .iter()
            .filter(|task| matches(&task.state))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue_of(count: usize) -> (TaskQueue<&'static str>, Vec<FileId>, Vec<TaskId>) {
        let mut queue = TaskQueue::new();
        let files: Vec<FileId> = (0..count).map(|_| FileId::new()).collect();
        let ids = files
            .iter()
            .map(|file| queue.push(*file, "describe"))
            .collect();
        (queue, files, ids)
    }

    #[test]
    fn tasks_start_in_the_order_they_were_added_one_at_a_time() {
        let (mut queue, _, ids) = queue_of(3);
        assert_eq!(queue.start_next().map(|task| task.id), Some(ids[0]));
        assert!(queue.start_next().is_none(), "one runs at a time");
        queue.finish(ids[0], Ok(()));
        assert_eq!(queue.start_next().map(|task| task.id), Some(ids[1]));
    }

    #[test]
    fn only_the_clip_of_the_running_task_is_busy() {
        let (mut queue, files, ids) = queue_of(2);
        assert!(!queue.is_busy(files[0]), "queued is not busy");
        queue.start_next();
        assert!(queue.is_busy(files[0]));
        assert!(!queue.is_busy(files[1]));
        queue.finish(ids[0], Ok(()));
        assert!(!queue.is_busy(files[0]));
    }

    #[test]
    fn a_failed_task_keeps_its_reason_and_the_line_goes_on() {
        let (mut queue, _, ids) = queue_of(2);
        queue.start_next();
        queue.finish(ids[0], Err("no credit".into()));
        assert_eq!(
            queue.get(ids[0]).map(|task| task.state.clone()),
            Some(TaskState::Failed("no credit".into()))
        );
        assert_eq!(queue.failed_count(), 1);
        assert_eq!(queue.start_next().map(|task| task.id), Some(ids[1]));
    }

    #[test]
    fn cancelling_a_queued_task_skips_it() {
        let (mut queue, _, ids) = queue_of(3);
        assert_eq!(queue.cancel(ids[1]), Some(false));
        queue.start_next();
        queue.finish(ids[0], Ok(()));
        assert_eq!(queue.start_next().map(|task| task.id), Some(ids[2]));
    }

    #[test]
    fn cancelling_the_running_task_waits_for_its_end() {
        let (mut queue, _, ids) = queue_of(1);
        queue.start_next();
        assert_eq!(queue.cancel(ids[0]), Some(true));
        assert_eq!(
            queue.get(ids[0]).map(|task| task.state.clone()),
            Some(TaskState::Running)
        );
        queue.finish_cancelled(ids[0]);
        assert_eq!(
            queue.get(ids[0]).map(|task| task.state.clone()),
            Some(TaskState::Cancelled)
        );
        assert!(queue.is_idle());
    }

    #[test]
    fn a_late_answer_does_not_overwrite_a_task_that_is_not_running() {
        let (mut queue, _, ids) = queue_of(1);
        queue.cancel(ids[0]);
        queue.finish(ids[0], Ok(()));
        assert_eq!(
            queue.get(ids[0]).map(|task| task.state.clone()),
            Some(TaskState::Cancelled)
        );
    }

    #[test]
    fn cancel_all_drops_the_queued_and_names_the_running() {
        let (mut queue, _, ids) = queue_of(3);
        queue.start_next();
        assert_eq!(queue.cancel_all(), Some(ids[0]));
        assert_eq!(queue.queued_count(), 0);
        assert_eq!(queue.pending_count(), 1, "the running one is still pending");
    }

    #[test]
    fn retry_puts_a_failed_task_at_the_back() {
        let (mut queue, _, ids) = queue_of(2);
        queue.start_next();
        queue.finish(ids[0], Err("rate limit".into()));
        assert!(queue.retry(ids[0]));
        assert!(!queue.retry(ids[1]), "a queued task is not retried");
        assert_eq!(queue.start_next().map(|task| task.id), Some(ids[1]));
        queue.finish(ids[1], Ok(()));
        assert_eq!(queue.start_next().map(|task| task.id), Some(ids[0]));
    }

    #[test]
    fn clearing_keeps_only_what_is_pending() {
        let (mut queue, _, ids) = queue_of(3);
        queue.start_next();
        queue.finish(ids[0], Ok(()));
        queue.cancel(ids[1]);
        queue.clear_finished();
        assert_eq!(queue.tasks().count(), 1);
        assert_eq!(queue.queued_count(), 1);
    }
}
