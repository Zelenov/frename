# Prepare clips: the whole-folder pipeline (#17, stage 3)

Written by the agent; nothing here waited for an answer. It builds on stage 1 (Describe with AI, 0.71),
on clipscribe 0.11 to 0.13 (`describe_clip`, `Budget`, `Cache`, `group_clips`, `detect_shots`, all
already pinned) and on the task queue of #249 (`docs/design/task-queue.md`): a run is a batch job, so it
runs in the background and goes through the one queue.

## What the editor gets

One action in the batch panel, **Prepare clips**, over the clips checked in the file list (All, None,
Invert, Ctrl+click and Shift+click work as in batch mode; batch mode checks the open file, and All is
one click away). For each clip, in this order, in one pass:

1. **Subtitles** (Soniox), when the clip has none and a Soniox key is saved. Free when a transcript
   is already saved next to the video (the existing "rebuilt free" case).
2. **Description** (Claude or ChatGPT, the model of Settings): the summary and the time-ranged
   moments, from the key frames and the subtitles just made, in one request.
3. **Tags**: the request also suggests tags from the folder's own tags (Settings → Describe with AI,
   as today). They are stored with the description for the editor to review, as today; Prepare clips
   does not apply them.
4. **Markers**: the moments become AI markers (in the video or in the comment, as the settings say).
5. **Comment**: the summary goes into the AI block of the comment; the editor's own text is never
   touched.

Steps 4 and 5 are what `Describe with AI` already writes in one save; the pipeline adds step 1
around it and one plan, one cap and one progress view over all of it.

The panel shows, before Run: how many clips, what each step will do or skip (already subtitled, already
described, too long, unreadable), the price of each paid step and the total, and the cap.
While it runs, the job page of #249 shows one progress bar over the clips and, under it, the clip in
work and its step ("subtitles", "frame 12 of 40", "waiting for Claude", "saving"). The strip of #249
shows it with the panel closed.

### Budget cap

**Stop spending at** (US dollars, empty = no cap; the field remembers its last value). The cap covers
both paid steps of the whole run:

- the description goes through clipscribe's `Budget`: its request's upper bound is set aside before
  it is sent, and a clip that could take the run past the cap is not sent (`describe_clip` answers
  `None`);
- the subtitles are counted from the price the plan used (audio length × the account's price per hour),
  checked before a clip's transcription starts and settled with the real audio sent after it.

A clip that does not fit stops the job ("Stopped: the next clip could take the run past the $5.00
cap. Spent $4.62. Raise the cap to do the rest."); the clips not reached stay unreached, and
**Run again** continues from them.

### Resume

Nothing is paid for twice, because each step skips what is already there: a clip with subtitles gets
no new transcription, a clip with an AI description is not described again (Prepare clips has no Redo; use Describe with AI for that), and the description of an unchanged clip is also kept in a cache that frename's own data
folder holds (clipscribe's `Cache`, keyed by the file's size, time and a sample of its content, so a
renamed clip still finds its description). A description that was paid for but not saved (a read-only share) is
taken from the cache when the clip is run again, so a retry does not pay twice. So a stopped run is
resumed by running it again on the same clips: finished clips are skipped at once, at no cost. A clip
stopped between its two steps counts as not reached and is offered again; its subtitles are kept.

"A resumed run offers the same set again": when a run stopped before the end, frename remembers which
clips were not done (their names, in the batch's last-run record, #65). The next time **Prepare
clips** is chosen in that folder, the panel says "The last run stopped with 37 clips left" and has a
button **Check them** that checks exactly those clips. Nothing is checked on its own.

### Similar footage

Describing a clip through clipscribe's `describe_clip` also gives the fingerprints of the frames it was
described from. After the run, `group_clips` groups the clips (and stretches of them) whose pictures are
the same shot: duplicates, re-exports, a clip stored sideways, a camera that did not move. It costs
nothing (no request). The result page lists the groups, each with a label from the description of its
most typical stretch and its clips, and a button **Check these clips** per group, so the editor can
run another batch action (a tag, a comment, a move) on one group at once. Groups by words
(experimental in clipscribe) are not offered. Grouping reads the fingerprints of clips described
earlier from the cache, so a resumed run groups the whole set, not only what it described this time.

### Shots

`detect_shots` finds the cuts of an **edited** video. The footage this app prepares is raw: one take
per file, where the key frames already follow the changes of the picture, and a shot count adds
nothing the description does not say. It is not used by Prepare clips. It stays available for a later
use on edited sequences (filed as an idea).

## How it works

- `Action::PrepareClips`, `Operation::PrepareClips(pipeline::Run)` in `src/features/batch/actions/
  pipeline.rs`. `Run` holds the options of both steps (the Soniox `generate_subtitles::Run` and the
  description `describe_ai::Run`), the clipscribe `Budget` (shared by every item of the run), and the
  cache.
- `Operation::run` for one clip: (1) `generate_subtitles::run` when the plan says transcribe; (2)
  `describe_ai::run_clip` (the existing run, made to call `describe_clip` with the run's budget; it keeps
  the clip, with its fingerprints, in the cache, which is where grouping reads them back); (3) the tag suggestions come out of the same request; the results are added up (the status of the
  clip is the worst of its steps' failures, `usage` is the sum, a stop of either step stops the job).
- The plan (`pipeline::Plan`) is the two existing plans over the checked clips: the subtitles plan
  (`generate_subtitles::plan`, header reads) and the description plan (`describe_ai::Options::plan`,
  clip lengths), read in the background while the panel is shown; `Actions::readiness` says when
  both are in, so the single-clip path of #249 could run it too.
- Per clip, the item's progress is split between the steps, 40 % for the subtitles and 60 % for the
  description (the whole bar for a single step).
- After the last item, the workspace groups the described clips (`group_clips`) in the background
  and the job page shows the groups.

## Decisions made without the owner

- The clips-left record lives in the last-run record, which any other action overwrites; after a
  restart it is offered only if Prepare clips was the last action. Clips are kept by name, so a clip
  renamed meanwhile is not offered. Grouping lists a clip under the group covering most of it, and
  only clips described by Prepare clips are in the cache.

- The cache layout (`cache_dir_in`, `folder_key`) is in the UI crate on purpose: it is frename's own
  data folder policy for a feature of this crate, not logic `frename-core` needs.

- One new action, not a mode of Describe with AI: the panel of a pipeline has its own options (the cap,
  the two steps), and the existing two actions stay as they are.
- Steps cannot be reordered; each can be switched off (no subtitles, no description) so the same action
  serves "only describe what is already subtitled".
- Tags are only suggested and stored for review (the owner's review rule of stage 2 stays).
- The cap is in dollars and per run, not per day.
- Resume is "run again, finished steps are skipped" plus a remembered "clips left" set per folder;
  no background resume after a restart without the editor pressing Run.
- Grouping is by pictures only; words are experimental in clipscribe and off.
- Shots are not used (raw footage): see above.
- Clips run one at a time (the batch job runs one item at a time); clipscribe's concurrency
  (`jobs`) is not used. Revisit when the queue of #249 can run tasks of different providers together.

## Tests

What exists (the requests to Claude and Soniox need keys and credit, so they are covered by their
decisions rather than by a fake service; a live run is the owner's check):

- Plan: counts and skipped reasons for the steps together; the price sums both services; Redo of
  Describe with AI is ignored.
- Steps: a clip whose subtitles failed is not described; a stop of the description after the subtitles leaves the clip unreached with a stop; a
  step's failure and usage are added up.
- Cap: a clip that could pass it is not sent and stops the job; a clip that costs nothing is not
  held back; what was set aside is replaced by what was sent.
- Resume: a kept description is used again at no cost; the left-clips record round-trips and is
  tied to its folder; **Check them** and **Check these clips** check exactly those; Run again waits
  for the subtitles plan instead of leaving the step out.
- Persistence: the options (cap, steps) round-trip through the last-run record.

Not covered by tests: `Run::group` over real fingerprints (needs described frames) and the live
requests; both are for the owner to see in a real run.
