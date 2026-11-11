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
  entry once the save is applied *and read back*: not refused, no marker write waiting in
  `unsaved_markers`, and tags, name, comment and in/out points on disk are what was wanted
  (`recovery::saved_what_was_wanted`, deliberately stricter than the live refusal check
  `drag_out::save_failed`). Otherwise it stays. A normal close therefore leaves nothing behind.
  A journal write is one at a time, off the interface's thread; a write that lands after the
  clip was saved is dropped and its entry removed (an epoch counter).
- **Only one frename:** the journal is held with an exclusive lock on `recovery/instance.lock`
  for the life of the process; a second frename leaves it alone.
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
- The message pieces are counts and reasons from the core, worded and pluralised in the app
  (en + ru). The restore message stays 20 s (all messages in one note), then goes.
- The entry kept aside gets a readable `.txt` beside it to retype from.

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
- Atomicity of the existing rename and XMP write paths: audited in #156, see below.

## Atomicity of the save paths (#156, part 3)

What a crash, a kill or a full disk in the middle of each write leaves behind:

| Write | How | Torn write possible? |
|---|---|---|
| Rename of the clip and its sidecars | `std::fs::rename` within one folder | No: the file has the old name or the new one. A crash between the clip and a sidecar leaves the sidecar under the old name; the next save moves it. |
| Folder tag file `.frename` | scratch `.frename.tmp`, renamed over | No (already so). |
| `clip.mp4.comment.txt` | was `std::fs::write` (truncate, then write) | **Was yes:** a crash after the truncate left the comment empty or cut off. Now `write_atomically`: a hidden scratch file next to it, renamed over. |
| `clip.markers.txt` (marker export) | was `std::fs::write` | **Was yes**, same fix. |
| Rotation | in place, a few bytes per track; a failed write puts back what it wrote | A kill between two tracks of a multi-track file leaves them turned differently. Rare (one 36-byte write per track) and visible; not changed. |
| XMP inside the video (comment, in/out, markers) | Adobe XMP Toolkit, in place | Possible: the toolkit rewrites the XMP box in the file. A safe update would copy the whole video (gigabytes) on every save, and `xmp_toolkit` 1.12 does not expose the toolkit's safe-update flag. Not changed: the journal keeps the edits until they read back from the file, so a torn XMP write is restored on the next start. |

The scratch files are not flushed (`fsync`) before the rename: these writes run on the interface
thread, a flush can take tens of milliseconds on Windows, and a power cut is what the journal
(which does flush) covers.

Not done from part 3: writing the XMP comment and markers early while the clip plays on Windows.
