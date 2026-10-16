# Suggest tags for the open clip (#250)

Written by the agent; nothing here waited for an answer.

## Today (what #249 and #175 already gave)

- A "Describe with AI" request can also suggest tags from the folder's list (Settings → Describe
  with AI → tag suggestions). The answer goes into the clip's comment as `Suggested tags: …` and
  `Tag ideas: …`; there is no typed value kept after the run.
- With the clip open, `file_name_panel` shows the suggestions as chips (confidence in the label,
  `F6` adds the first, **Add all** / `Shift+F6` the rest). Each accept is one undo step
  (`ToggleTagCommand` / `CheckTagsCommand`), saved and journaled like tagging by hand. Ideas the
  folder does not have are listed apart and never added. Ideas the folder has since gained are
  hidden.
- The sparkles button in the comment's corner runs "Describe with AI" for the open clip through
  the queue of #249 (waits behind a running job, can be withdrawn, cancelled from the strip). The
  result lands in the comment and the clip reopens.

So the chips, accept / dismiss (the chips go when the line is edited away), undo and the queue
exist. What the issue still asks for is a way to **ask for the suggestions on their own**: it
needed Settings → tag suggestions on, plus the whole description.

## What this adds

- A **tag button** in the comment's corner (before the sparkles and captions buttons) and
  **`Ctrl+F6`** ("always, no repeat", next to `F6` / `Shift+F6`). It queues a "Describe with AI"
  run for the open clip that suggests the folder's tags **whatever the setting says**.
- It is lit while it waits or works (the sparkles button is lit too: it is the same run), pressing
  it again takes the run back out of the line, a run in work is cancelled from the strip.
- A clip that already has suggestions (or ideas) is left as it is: they are showing. A clip that
  is described but has none is described again; its description is replaced by the new one.
- A folder without tags says so in a notice and nothing is asked for.

## Decisions made without the owner

1. **Same request, not a cheaper one.** clipscribe has `suggest_tags` (tags from a description
   already made, no frames). frename never reads a clip's description back into clipscribe's type
   (it keeps the text in the comment), so the cheap path needs a parser of the AI block into a
   `Description`, which is more than this change should carry. A clip that has a description but no
   suggestions is therefore described again, and the price shown in the guide says so. The cheap
   path is filed as an idea.
2. **No new batch action.** A new `Action` touches about forty places (panel, plan, price,
   readiness, job page); the job is the existing "Describe with AI" with one flag on its run
   (`tags_requested`), so the queue, the key and budget checks, the strip, cancel, results and
   cache all stay as they are.
3. **Suggestions show where they already show** (under the clip's name), not as a new style in the
   tag grid; the chips are the accept / dismiss UI the issue asks for, with their keys.
4. **The shortcut is `Ctrl+F6`**, next to `F6` / `Shift+F6`; it fits the keymap of #168 as the
   "ask" half of the pair.
5. **Leaving the clip while it waits** needs nothing new: a run in the line belongs to its clip,
   not to the one that is open. It runs on that clip when its turn comes (while a job works, other
   clips wait, #249), and the chips show when the clip is opened again.

## Not built

- The "describe too" option and a price shown before running (the strip and the batch panel show
  the cost of the job as for any description).
- A distinct style for suggestions inside the tag grid; moving through suggestions by keyboard
  beyond `F6` and `Shift+F6`.
- Reading an existing description into clipscribe's `suggest_tags` (decision 1).

## Test plan

- A run asked for from the open clip shows on its button, is withdrawn by a second press, and a
  run that cannot start (no key) leaves no state behind (`suggest_tags_on_the_open_clip_is_a_run_that_can_be_withdrawn`).
- The run suggests the folder's tags whatever Settings say, and the next ordinary run does not
  (`a_suggest_tags_run_suggests_whatever_the_settings_say`).
- A comment with tags or ideas counts as having suggestions
  (`a_comment_has_suggestions_when_it_holds_tags_or_ideas`).
- `Ctrl+F6` asks, `F6` and `Shift+F6` still add (`app/state.rs` key test).
- Accepting, undo and saving are covered by the tests of #175
  (`the_ai_suggested_tags_are_added_only_on_request_and_undone_in_one_step`).
