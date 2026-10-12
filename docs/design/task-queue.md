# Task queue: slow work in the background (#249)

Written by the agent; nothing here waited for an answer. It builds on a map of today's code
(see "Today") and splits the issue into three PRs.

## Today

- A batch job locks the app. `FolderWorkspace::is_blocked` ends with
  `batch.is_running() && (edits_open_file || changes_files)`: opening clips, folders, selection,
  tags, rename, undo, markers and rotation wait until the job ends.
- `start_batch` unloads the video and `run_batch` does `file_workspace.set_file(None)`: the open
  clip is closed for the whole job. Renames the job makes are applied by `batch_item_done`, and
  `batch_finished` resets the undo history because "the job renamed files behind it".
- A job runs one file at a time. The file's path is looked up by id when the item **starts**
  (`next_batch_item`), the item runs in `spawn_blocking`, and the result carries the new path
  (`ItemResult.update`). Cancel is a flag checked inside the item; the job stops after it.
- Marker descriptions are a second, separate path (`state/marker_ai.rs`): requests counted in
  `marker_requests`, readers in `marker_readers`, and a batch run waits for them
  (`WaitForMarkers`) or stops them.
- No per-file "in use by a task" state exists. Only global signals: `batch.is_running()`,
  `batch.current()`, `batch.status(id)`.

## Target model

One **queue** of **tasks**. A task is one action on one clip (describe, subtitles, suggest tags,
describe marker, and the existing batch actions). A batch run is a queue filled with one task per
checked clip. The editor keeps working while tasks run.

```
Task { id, file: FileId, action: Operation, state, cost, error }
state: Queued -> Running -> Done | Failed(reason) | Cancelled
```

- **Concurrency:** one running task per provider at a time to begin with (what a batch does
  today); tasks of different providers (Soniox, Anthropic, local) may run together later. The
  queue is FIFO; Retry puts a failed task at the back.
- **Cancel:** per task and for all. A queued task is dropped; a running one gets its cancel flag
  (as today) and ends at the next check. Nothing half-written: the existing rule stays.
- **Budget:** the cost estimate and the cap apply to the whole queue, as the batch run's do.

### Guards (the core of "never freezes")

The lock moves from the whole app to **one clip**:

1. **Busy clip.** `Queue::busy(file)` is true while a task of that clip is running. A busy
   clip's own saves, renames and rotation **wait** for the task's item to end and say so
   ("Waiting for the description to finish"). This reuses `pending_file_updates`, the queue the
   media viewer already uses to hold a rename until the video is unloaded.
2. **Path follows renames.** A task reads and writes the clip's **current** path, never the one
   captured at the start. While a task is running its clip cannot be renamed (guard 1), so the
   path read at item start is valid for the whole item; the result's `update` is applied to
   whatever the clip is called then.
3. **Open clip.** A task on the clip that is open does not close the video. Tasks that only read
   (frames, audio) let the video play; tasks that write the file (comment, markers) wait for the
   editor to leave the clip, or run when the viewer is idle. The first PR keeps today's rule for
   the open clip: a write task waits (state `Queued`, reason "open") until the clip is left.
4. **Undo.** A task's result applied to the **open** clip is one undoable step (later PR). A
   result for a clip that is not open is written straight to the file, as a batch does, and
   resets nothing: undo history is per open clip.
5. **Folder change.** The queue is **per folder**: changing folder keeps the queue running in the
   background for the old folder, shown in the indicator with its folder name. Opening the old
   folder again shows its clips' status. (Decision: waiting or asking would freeze the very
   thing this feature removes.)
6. **App exit.** With tasks running or queued: ask "Wait" or "Stop and keep what is done".
   Unfinished tasks are not offered again on the next start in the first version (the batch
   run's own resume, `set_batch_run`, stays as it is).

### UI

- **Indicator** in the window (count, current item, progress, cost so far). Click opens the
  **queue panel**: queued, running, done, failed, Cancel per task, Cancel all, Retry on failed.
- **One model for every AI button:** the button shows a spinner while its task is queued or
  running; click again cancels; the price shows before a paid action that is not tiny (#230);
  errors go to the queue panel and a note, never a modal.

## Steps (each its own PR)

1. **Background batch (this PR).** The batch job is the queue already (one job, its files in
   order, per-file outcome, cancel, retry, progress, cost). It stops locking the app:
   - the lock of `FolderWorkspace::is_blocked` shrinks from the app to the **clip in work**
     (`BatchState::current`): that clip cannot be opened or edited until its item ends;
   - the panel can be closed while the job runs (the ✕, `Esc`, the toolbar toggle); a strip above
     the file list's toolbar (`batch::view::indicator`) shows the action, `finished of total`,
     the cost so far, the clip in work and a progress bar, and brings the panel back on click;
   - the job no longer closes the open clip at its start. When the next item is the clip that is
     open, the pending saves are applied, the video unloads, the clip closes, the item runs, and
     the clip opens again under its current name (`run_batch_item`, `reopen_selected_if_closed`);
   - still locked while a job runs: opening another folder (the job's files belong to this
     folder, so a per-folder background queue is left for later), file actions that delete or
     move files, Undo and Redo (the job renames files behind the history; the history is reset
     when it ends), and marker descriptions (`marker_ai`).
2. **Single-clip actions** start directly (describe, subtitles, suggest tags per #250, marker
   describe) and enter the same queue. This needs the job to hold items of different actions
   and accept new ones while it runs; the queue model is designed then (the pure `TaskQueue`
   of the first draft was dropped because nothing used it yet).
3. **Button clean-up:** every AI button follows the model above; list of today's buttons and
   what each becomes is filled in then (marker rows ✨, "describe all unnamed", the In/Out AI
   pill, batch actions).

## Test plan

- Core: queue FIFO, cancel of queued and running, retry, busy guard, per-folder separation,
  budget cap stops the queue.
- App: a rename/save of a busy clip waits and is applied after the item; a result for a clip
  renamed between items is written to its current name; changing folder keeps the queue.

## Decisions made without the owner

- One running task per provider; no parallel tasks on one clip.
- Step 1 keeps the folder locked while a job runs (a queue per folder is a later step).
- Busy-clip guard instead of closing the video; a write task on the open clip waits.
- No resume of the queue after a restart in step 1.
- Marker describe joins the queue in step 2 (not step 1), so its own request path stays
  untouched until then.
