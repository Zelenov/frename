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

1. **Busy clip.** `BatchState::current` is the clip in work. In step 1 a message that would edit
   it, open it or change its file is **dropped and the editor is told why** in the video's note
   ("In use: <action>"; "Busy: <action>" for what the folder-wide hold stops: another folder, file moves, Undo, marker descriptions); the arrow
   keys of the list step over it. Holding such an edit until the item ends (the
   `pending_file_updates` queue the media viewer uses for renames) is for a later step.
2. **Path follows renames.** A task reads and writes the clip's **current** path, never the one
   captured at the start. While a task is running its clip cannot be renamed (guard 1), so the
   path read at item start is valid for the whole item; the result's `update` is applied to
   whatever the clip is called then.
3. **Open clip.** A task on the clip that is open does not close the video at the start of the
   run. Step 1: when the job reaches the open clip, its pending edits are saved, the video
   unloads, the clip closes for the length of its item (`run_batch_item`) and opens again under
   its current name (`reopen_selected_if_closed`). While the job waits for the unload, no other
   clip opens (`job_holds`).
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

1. **Background batch (0.152).** The batch job is the queue already (one job, its files in
   order, per-file outcome, cancel, retry, progress, cost). It stops locking the app:
   - the lock of `FolderWorkspace::is_blocked` shrinks from the app to the **clip in work**
     (`BatchState::current`): that clip cannot be opened or edited until its item ends;
   - the panel can be closed while the job runs (the ✕, `Esc`, the toolbar toggle); a strip above
     the file list's toolbar (`batch::view::indicator`) shows the action, `finished of total`,
     the cost so far, the clip in work and a progress bar, and brings the panel back on click;
   - the job no longer closes the open clip at its start. When the next item is the clip that is
     open, the pending saves are applied, the video unloads, the clip closes, the item runs, and
     the clip opens again under its current name (`run_batch_item`, `reopen_selected_if_closed`);
   - a job that ends or stops while its panel is closed brings the panel back when something went
     wrong (a failed file, a stop, files not reached), otherwise a note says it finished;
   - still locked while a job runs: opening another folder (the job's files belong to this
     folder, so a per-folder background queue is left for later), file actions that delete or
     move files, Undo and Redo (the job renames files behind the history; the history is reset
     when it ends), and marker descriptions (`marker_ai`).
2. **Single-clip actions (step 2).** "Describe with AI" and "Generate subtitles" start for the open
   clip from two buttons in the corner of the comment box (`file_workspace::view`), without the
   batch panel. A run borrows the batch machinery (`folder_workspace::state::solo`): it selects the
   action and checks only that clip (the panel's checks are put back), asks the action whether it
   can start (`Actions::readiness`: ready, waiting for a read, or blocked with a note and the
   settings page that fixes it), starts the job, and gives the checks back. Many runs are a line of
   `(action, clip)`: a run asked for while a job works, or while another prepares, waits and
   starts once the job's end is reported (`batch_end_pending`), so everything slow goes through one
   queue, one clip at a time. While a run prepares (a few moments: clip length, key, price), what
   would change the checks waits (`solo_holds`, given up after 30 s). The button is lit while its
   clip is queued or in work. Marker description keeps its own path until step 3, and tag
   suggestions are part of Describe with AI (#250).
3. **One model for every AI button (0.155, shipped).** Every slow button follows the model above: it is lit
   while its run is queued or in work, and a press on a lit button *withdraws* the run if it is
   still waiting (a run in work is cancelled from the job page or the strip, as before). Runs that
   wait are listed on the job page ("Waiting", each with Remove) and counted on the strip. The
   marker descriptions stop being a separate path that waits behind a job: they join the line.

   | Button (today) | Becomes |
   |---|---|
   | Describe with AI, comment corner (step 2) | Lit while queued or in work; a press while waiting withdraws it. |
   | Generate subtitles, comment corner (step 2) | The same. |
   | Marker row ✨ (describe one marker) | While a job works the request waits and starts when it ends (note "waits for the running job"); no longer refused. |
   | "Describe all unnamed" markers | The same: the markers wait as one batch of requests. |
   | In/Out AI pill (name a new range) | The same as the marker ✨. |
   | Batch panel actions (Describe with AI, Generate subtitles, Prepare clips, ...) | Unchanged: a batch is the queue's own job; the panel and the strip show it. |

   A keyboard shortcut for the two single-clip actions is not added: the issue does not ask for one.

## Test plan

Step 1 (in `folder_workspace::state` and `batch::state`): the clip in work is held and the
others are not, nothing opens while the item waits for the unload, a held message tells why, the
arrow keys step over the clip in work, a pending save of the clip is applied before its item, a
cancel before the item starts leaves the clip alone, the panel closes under a running job and the
strip shows it. For steps 2 and 3:

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
