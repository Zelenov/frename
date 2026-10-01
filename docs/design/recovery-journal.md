# Recovery journal (#140)

The owner's fear: tagging and marking a long clip, then a crash, a kill or a power cut loses it.
Most edits of the open clip reach the file only when the clip is left (Windows locks a playing
video; the rename waits for it to unload). This is the design notes for the journal that closes
that gap. Written by the agent; nothing here waited for an answer.

## What is built

- **Journal** (`frename_core::recovery`): one JSON file per clip in `recovery/` of the app data
  folder. An entry holds what the editor did and has not saved: tags and name, comment, in/out
  points, markers, plus the clip's path, size and modification time. Rotation is not in it: a
  turn is written into the file at once.
- **Atomic writes:** temporary file, `sync_all`, rename over the entry. A power cut leaves the old
  entry or the new one whole. A truncated, foreign-version or unreadable file is moved to
  `recovery/kept/` at the next start (never deleted, never a crash).
- **When it is written:** a one-second tick (`JournalTick`, a subscription that exists only
  while a clip is open) compares the open clip with its state when it was opened or last saved
  (the baseline). If it differs and the entry on disk is not already that, it writes. If it is
  back to the baseline (undone), the entry is removed. So a crash loses at most about a second.
  A running batch job does not journal (it writes its own files).
- **When it goes:** `apply_file_updated` (leaving the clip, closing, a folder change) removes the
  entry once the save was not refused. A refused save (a file with that name exists, read-only)
  keeps it. A normal close therefore leaves nothing behind.
- **After a crash:** at start, before any clip is open (so nothing locks the clips),
  `restore_all` applies each entry through the normal save path (markers, then name, tags,
  comment) when the clip is still the one it was made on (same size and time). The clip is then
  opened first, with the message *frename closed unexpectedly. Restored your unsaved work on
  `clip.MP4` (5 tags, 3 markers, comment).* over its picture.
- **Never overwrites:** if the clip is gone, was changed after the edits (by Premiere, by a
  rotation), or the save is refused, the entry is moved to `recovery/kept/` and the message says
  where it is. Nothing is applied silently.

## Decisions made without the owner

- The state journaled is the clip's whole pending snapshot, written after every change (not a
  diff): entries are small and a restore is one save.
- The entry's fingerprint is rewritten with each entry, so an edit followed by a rotation (which
  changes the file) writes a fresh entry that still matches the clip.
- The restore runs at start, not when the clip is opened: no video is open yet, so Windows does
  not lock it, and the existing save paths can be reused as they are.
- The `what` in the message (`5 tags, 3 markers, comment`) is English in every UI language.

## Not built (left for the owner / a next session)

- **Undo after a restore:** the restore writes the edits to the clip directly, so Ctrl+Z does
  not know them. Restoring into the open clip's state (as unsaved edits with a history) would
  give the undo the issue asks for, but means opening the clip and replaying the edits as
  commands.
- **The saved/saving indicator** ("All changes saved") in the UI.
- **Writing early** what can be written while playing (XMP comment and markers on Windows): the
  journal covers it for now.
- **A kill test as a separate process in CI.** The tests drop the workspace without saving or
  closing and restore from the journal in the same process; the journal logic is unit-tested in
  `frename-core` (round trip, replacement, truncated file, changed clip, missing clip, applied).
- Atomicity of the existing rename and XMP write paths was not audited.
