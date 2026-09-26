# Markers

Design for issue #9: Premiere Pro clip markers instead of screenshot timecodes.
Research: [`docs/research/premiere-xmp-markers.md`](../research/premiere-xmp-markers.md).

## Problem

An editor watching a clip marks the interesting moments. Today that takes two tools that do not
reach Premiere Pro: `F12` saves a `.snap.*.jpg` and appends `HH-MM-SS-mmm: ` to the comment, and
the timecode is then plain text. Premiere shows neither on the clip: after import the editor has to
find each moment again by hand.

A **marker** is a separate entity — time, optional duration, color, name, multi-line comment —
written into the video's XMP as a Premiere clip marker, so it appears on the clip in Premiere after
import. The file's **comment** stays free text for the whole file; markers are not part of it.

## User flows

1. **Mark while watching.** Playing a clip, the editor presses `F2` at a moment. A colored tick
   appears on the progress bar at that time. Playback does not stop and keyboard focus does not
   move, so tagging and marking can be mixed.
   **Mark and describe from the keyboard:** `F2` opens a marker's row with the name field focused
   (opening the list if it was closed) instead of adding one when either
   - a marker was added with `F2`/`◆+` less than 1.5 s ago and nothing happened since (no seek,
     no other marker action, no file change); or
   - a marker is under the playhead (nearest within 0.5 s, e.g. after `Shift+F1`/`Shift+F3`).
   Otherwise `F2` always adds, so several moments can be marked in a row while watching.
   `F2 F2`, type, `Enter` names a moment; `Shift+F1` then `F2` names an earlier one.
   New markers are green (Premiere's default: the color key is left out).
2. **Describe a marker.** Besides `F2 F2`, the editor can click a marker's row (or its `✎`) in the
   list: the row opens for editing, with a name field (focused) and a
   multi-line comment editor. `Enter` in the name field, `Esc` in either field, or a click on the
   row's `✓` closes it and unfocuses every widget (as closing the inline rename does), so `Space` plays again and
   `PageDown` moves on. The color is picked from the row's color dot.
3. **Give a marker a length.** With the playhead after a marker, the row's `⇥` button sets its end
   to the playhead (a ranged marker, drawn as a colored band). `⇤` on a ranged marker clears the
   duration.
4. **Navigate.** `Shift+F1` / `Shift+F3` jump to the previous / next marker (`F1` / `F3` already
   seek ±10 s). Clicking a row's time jumps to it. Dragging the progress bar with `Shift` held snaps
   to the nearest marker.
5. **Remove.** The row's `✕`, or `Shift+F2`, which deletes the marker under the playhead (the
   nearest one within 0.5 s; nothing happens when there is none). After `Shift+F1`/`Shift+F3` the
   playhead is exactly on a marker, so jump-then-delete is predictable.
6. **Leave the file.** Markers are written into the video's XMP when the file is saved — when the
   editor moves to another file, like the comment and in/out points today. `Ctrl+Z` undoes adding,
   deleting, recoloring and resizing a marker while the file is open.
7. **In Premiere.** After import (or re-import: Premiere caches XMP), the markers are on the clip
   with their name, comment, duration and color.
8. **Frame grabs.** `F12` still saves the current frame as a JPEG next to the video, like VLC's
   snapshot, and forgets it: no tick, no comment text, not renamed with the video.
9. **Batch.** In batch mode: turn timecoded comment lines (`03:24 — shaky, stabilize`) into
   markers; turn markers into comment lines.

## UI sketch

Windowed, marker list open (the list shares the right-hand overlay with the subtitle list):

```
┌──────────────────────────────────────────────┬──────────────────────────────┐
│                                              │ [Subtitles] [Markers]        │
│                                              │ ┌──────────────────────────┐ │
│                   video                      │ │● 0:12        ⇥  ✕       │ │
│                                              │ │ Take 3________________   │ │
│                                              │ │ nice light, use the pan  │ │
│                                              │ │ at the end               │ │
│                                              │ ├──────────────────────────┤ │
│                                              │ │● 0:41–0:47   ⇤  ✕  (lit) │ │
│                                              │ │ Lion__________________   │ │
│                                              │ │                          │ │
│                                              │ └──────────────────────────┘ │
├──────────────────────────────────────────────┴──────────────────────────────┤
│ ▶ ◀10 10▶ [ ]  ──┃──────▼────█████──────▼──────────────  ◆+ ◆ CC ⛶  🔊    │
└─────────────────────────────────────────────────────────────────────────────┘
  ▼ = marker tick in its color   █ = ranged marker band   ◆+ = add (F2)   ◆ = list
```

- **Progress bar.** Marker ticks in the marker's color replace the teal screenshot ticks; a ranged
  marker also draws a thin band in its color above the in/out highlight.
- **Controls bar.** `◆+` adds a marker (tooltip `Add marker (F2, again to name it)`); `◆` shows or
  hides the marker list (tooltip `Markers (Shift+F1 / Shift+F3 to jump, Shift+drag to snap)`), next to `CC`. `◆` is shown whenever a video is open, in fullscreen too.
- **Side list.** One overlay on the right, as the subtitle list is today. When the video has both
  subtitles and markers (or the marker list was opened), the top of the overlay has two tabs,
  `Subtitles` and `Markers`; with only one of them there are no tabs. `CC` opens the subtitle tab,
  `◆` the marker tab; pressing the button of the open tab closes the overlay. Whether the overlay
  is open, and which tab, is remembered across files, as `CC` is today; it never opens by itself
  because a file has markers. Fullscreen shows the subtitle overlay when subtitles exist, as today; the marker
  tab only when the editor opens it with `◆` or `F2 F2`. `◆` on the open marker tab goes back to the
  subtitle tab when there are subtitles and hides the overlay when there are none, so marked clips
  are not watched with part of the picture covered and subtitles are never lost.
- **Marker row.** Read-only, like a cue row: color dot (click → a row of the 9 colors), time
  (`m:ss` or `m:ss–m:ss`; click to jump), `⇥`/`⇤`, `✎`, `✕`; then the name and up to two lines
  of the comment as text. Rows have a fixed height so the list scrolls to a row by arithmetic, as
  the cue list does. **One row at a time** opens for editing (flow 2): only that row holds a
  `text_input` and a `text_editor` (one `text_editor::Content` in state, created when the row
  opens), and it is taller; rows below it shift by a known amount, so scrolling stays arithmetic.
  The row of the last marker at or before the playhead is lit; the list follows playback except
  while a row is open.
- **Empty list.** `No markers yet — press F2 to add one`.
- **Frame saved.** After `F12`, the controls bar shows `Frame saved` for 2 seconds, since there is
  no tick any more to show it worked.
- **Where the state lives.** The markers of the open file are owned by `TagList` in `frename-core`
  (with in/out and the comment); the open row (which marker, and its `text_editor::Content`) lives
  in `file_workspace` next to `comment_content`; the video view gets both as arguments, the way it
  gets the in/out points today.

## Keyboard shortcuts

| Key | Action |
|---|---|
| `F2` | Add a marker at the playhead |
| `F2` on the marker under the playhead | Open its row to type a name (`F2 F2` = mark and name) |
| `Enter` / `Esc` in an open marker row | Close the row, back to the workspace |
| `Shift+F2` | Delete the marker under the playhead (nearest within 0.5 s); shows `Marker deleted` or `No marker here` |
| `Shift+F1` / `Shift+F3` | Jump to the previous / next marker |
| `Shift` + drag on the progress bar | Snap to the nearest marker (within 8 px) |
| `F12` | Save the current frame as a JPEG next to the video (shows `Frame saved`) |

F-keys work while the search bar has focus, as `F1`/`F3`/`F12` do today. Premiere's own `M` is not
used: letters type into the tag search.

While a marker row is open for editing, keys belong to its fields. `main_window_event` stays a
stateless function; `FolderWorkspace::update` guards the messages instead, the same way it already
ignores `SetSegmentStart` while `inline_rename` is open:
- `SetSegmentStart`/`SetSegmentEnd` (`[`, `]`) and `TagPanel(ToggleSelectedTag)` (`Shift+Space`)
  are ignored, so typing `[b-roll]` does not set in/out points;
- `EscapePressed` closes the row first (before leaving fullscreen or clearing the search);
- `F2` closes the open row (its text kept) and adds a new marker at the playhead without opening
  it, so the next moment can be caught while typing;
- `Undo`, `Redo`, `CopyTags`, `PasteTags` and `Shift+F2` are ignored (the fields have no undo of their own; an
  app undo would remove the very marker being edited);
- if the open row's marker disappears anyway, the row closes.
`Enter` in the name field also closes the row. `F2` sits between the seek keys, and `F1`/`F3` with
`Shift` keep "back/forward" on the same keys. `F1`/`F3`/`F12` today ignore modifiers; they will
check that `Shift` is not held.

`F2` never adds a second marker within 0.5 s of an existing one; it opens that one instead.

## Data format

### In the video (XMP)

Markers live in `xmpDM:Tracks` on tracks whose `trackType` is `Comment`, as in the research
sample. Tracks are selected by `trackType` only; `trackName` is ignored. Reading accepts every form listed under Gotchas in the research: any `frameRate`
(`f1000`, `f48000`, `f254016000000`, `f<num>s<den>`), attribute and `parseType="Resource"` forms,
and GUIDs with an `xmp:id:` prefix. Times are converted to milliseconds for the UI.

Per marker, frename reads and writes:

| Field | XMP | Notes |
|---|---|---|
| start | `xmpDM:startTime` | in the track's rate |
| duration | `xmpDM:duration` | 0 / absent = point marker |
| name | `xmpDM:name` | single line |
| comment | `xmpDM:comment` | line breaks written as CR, read as CR, LF or CRLF |
| id | `xmpDM:guid` + `cuePointParams` `marker_guid` | fresh UUID for new markers |
| color | `cuePointParams` `keywordExtDVAv1_<uuid>` = `{"color":N,…}` | green = key absent |

Colors: the 9 in the research table (green, red, orange, yellow, white, blue, cyan, lavender,
magenta).
An unknown color value read from a file is kept as it is and drawn gray until the user picks a
color.

**Writing is an in-place edit, never a rebuild**, so nothing frename does not understand is lost.
Markers are matched by **GUID at write time**, against the XMP that is in the file then; nothing
about the file is remembered inside a marker. Besides the list, frename keeps, per file (keyed by
`FileId`, in `FolderWorkspace`, never in snapshots or undo commands), the **known set**: every
GUID it has read from or written to that file this session. A write:
- for each marker in frename's list: if its GUID is in the file, the fields that differ from the
  file are rewritten on that marker only, in the track's own rate (`round(ms × rate / 1000)`,
  computed in `u128`, so `f254016000000` cannot overflow); fields frename does not know (`type`,
  `speaker`, other `cuePointParams`, …) are never touched; if its GUID is not in the file, it is
  appended to the first `Comment` track, or to a new `Comment` track (`trackName` `Comment`,
  `frameRate` `f1000`) when there is none. A GUID already in the file is never appended again, so
  a marker cannot be duplicated, whatever undo, redo or re-save does;
- a marker in the file whose GUID is in the known set but not in frename's list was deleted by the
  user: it is removed. A track left empty by that is removed, and so is `xmpDM:Tracks` when it is
  left empty (as `set_in_out_range` does);
- a marker in the file whose GUID frename never knew (e.g. Premiere added it while the file was
  open) is kept;
- markers without a GUID (other tools; Premiere always writes one) are shown **read-only**: listed,
  ticked and jumpable, but not editable or deletable, and never touched;
- marker edits are applied before the in/out change in the same write, since `set_in_out_range`
  moves the InOut track to the end of `Tracks`;
- `InOut` tracks (frename's in/out) and every other track type are left alone;
- the known set reaches the core write as an argument, never through the snapshot:
  `save(snapshot, path, known: &HashSet<Guid>) -> (PathBuf, MarkersSaved)`; batch actions and
  `move_metadata` pass an empty set (they never delete markers);
- nothing is written when nothing differs, and file times are kept, as `xmp::write` does today.
- after a successful write, the known set gains the GUIDs just written.

Markers are always stored in the video: there is no file-name or text-file home, so the Settings
choices for comments and in/out do not apply to them.

The save reports what happened to the markers: `FileTaggerBackend::save` / `SaveAndReparse` (and
`InMemoryFileTagger`) return the new path plus a `MarkersSaved` outcome — `Untouched` (snapshot had
`None`), `Written(guids)` or `Failed(reason)` — instead of swallowing the error as `save_to_xmp`
does for the comment today. `apply_file_updated` uses it for the known set and for failures.

### In memory

`FileSnapshot` gets `markers: Option<Vec<Marker>>`, current values only. `None` means "not read":
saving leaves the file's markers as they are. This is the same guard `comment_loading` gives the
comment today, so a folder scan, a batch rename or a tag-only save can never wipe markers it did
not read. The folder scan and the `.frename` file list do not read or cache markers, so snapshots
in the `Directory` have `None`.

`FileTagger::parse` and `save_and_reparse` never fill markers: they return `None`. Only the
open-file path and the Markers ⇄ comment action read them.

- **Opening a file** with a different `FileId` from the open one: if the unsaved-markers memory
  (see "XMP write fails") holds markers for that `FileId`, those are used; otherwise its markers
  are read from the file (one XMP read of one file) and their GUIDs join its known set.
- **Re-opening the same file** (`FileOpened` with the open file's `FileId`: after an inline
  rename, an undo or redo refresh, a re-click): markers are **not** read from disk — the save of the
  current state may still be waiting for the video to unload. They come from the snapshot being
  opened (undo/redo) or stay as they are in `TagList`. Today `FileWorkspace::set_file`
  (`already_loaded`) and `apply_file_opened` (`same_file`) compare paths, and `submit_rename` opens
  the pre-rename `File`; the Markers PR changes both checks to compare `FileId`, and a same-id open
  only updates the `File` (its path) and keeps the `TagList` markers.
- **While open**, `TagList` owns the markers, as it owns in/out and the comment: `TagList::new`
  takes them from the snapshot and `TagList::file_snapshot()` puts them back. Marker undo commands
  act on `ctx.tag_list`, like `SetSegmentStartCommand`. The command for *add* captures the marker
  as it is at undo time, so a redo brings back the name typed after adding it. Since undo/redo
  snapshots hold current values only and writes match by GUID, restoring any of them can neither
  duplicate a marker nor hide one from the next write.

Every place that builds a new snapshot from the open file's state must carry the markers over,
or the user's unsaved edits are silently dropped: `TagList::file_snapshot()`, `submit_rename`
(`src/features/folder_workspace/state.rs`, which copies the comment and screenshots) and the
paste-tags path (`paste_tags`, which calls `FileSnapshot::new`); the implementation lists them all
with a grep for `FileSnapshot::parse`/`FileSnapshot::new` in `src/`.

**Saving the open file** happens on every path that leaves it: moving to another file and starting
a batch (both today), and — new — opening another folder and closing the window. Today neither of
the last two saves (`scan_folder` drops `pending_file_updated` and calls `set_file(None)`;
`CloseRequested` only unloads the video). The Markers PR adds it, in the order unload →
`apply_file_updated` → close/scan. This also newly saves tags, comment and in/out (and a pending
rename) on close and on folder change; `version.md` says so.

### Screenshots

`F12` keeps writing `{file name}.snap.HH-MM-SS-mmm.jpg` next to the video (the file list shows only
videos; `.snap.` jpgs are hidden from it by `is_screenshot_sidecar`, which with
`Screenshot::parse_time` stays, covered by a test). `FileSnapshot::screenshots`, the ticks, the
comment text and the rename-with-video code are removed. Existing `.snap.` files stay on disk
untouched and are no longer shown as ticks.

## Batch actions

One new batch action, **Markers ⇄ comment**, with a direction switch like **Move comments**.
One line format serves both directions:

```
<time>[–<time>] — <name>[ — <comment>]
```

`<time>` is `m:ss` below an hour and `h:mm:ss` from an hour, with `.mmm` added when the
milliseconds are not zero. A range is two times joined by `–` or `-` with no spaces around it
(`0:41-0:47`); `0:41 - 0:47 is the best part` is a point marker at 0:41 named `0:47 is the best
part`.

- **Comment → markers** (a move). Each comment line that starts with a time (or a range)
  **followed by a separator** (`—`, `–`, `-`, `:`) becomes a marker; the line is removed from the
  comment only after the marker write succeeded (on a failed write the comment stays as it is and
  the result is red: `Markers not saved: the file is read-only or in use`); a line like `12:30 call the client back` has no separator and stays. The rest of the
  line is split on the first ` — ` or ` -- ` (`--` is easy to type on a Windows keyboard): the part
  before is the **name**, the part after the **comment**. A plain ` - ` does not split, since names
  often contain it; the README says so. Also accepted: `mm:ss` and the old screenshot form
  `HH-MM-SS-mmm:`. Lines without a time stay. When the clip length is known, a line whose time is
  past the end stays and the file's result says so; when it is unknown, every time is accepted. If
  a marker with the same name already exists within 0.5 s, no duplicate is added (the line is still
  removed). Each file's result says how many lines became markers.
- **Markers → comment** (a copy). Each marker becomes a line in the format above, appended to the
  comment in time order, unless that exact line is already in the comment. The markers stay in the
  video, untouched. The marker comment's line breaks become spaces. Each file's result says how
  many lines were added.

Re-running either direction does nothing new. Comment → markers after Markers → comment removes the
copied lines and adds no duplicate markers. Files that cannot hold XMP get a red result
(`this format cannot hold markers`) for Comment → markers and are not changed. The comment is
written through the normal save path, so it follows the Settings comment storage and the
"Commented" tag rule. Only Markers ⇄ comment clears a file's unsaved-markers memory; other batch
actions leave it alone.

## Edge cases

- **Video that cannot hold XMP** (the toolkit has no smart handler for it, e.g. some `.MTS`):
  `◆+` is disabled with the tooltip `This file cannot hold markers`; `F2` opens the marker list,
  which shows that sentence, so a key press never silently does nothing. The README lists which
  video formats hold markers.
- **Read-only file** (checked when the file opens: read-only attribute or a read-only folder): the
  list shows the file's markers read-only (jump works), with `This file is read-only` above them;
  `◆+`, `F2`-add, edit and delete are disabled.
- **XMP write fails anyway** (file in use — Premiere often holds clips it has imported — or disk
  full): the unsaved markers are kept in memory, keyed by the file's path (moved along if the
  rename still happened; `FileId`s change on every folder scan), the failure is logged, and the
  file's row gets a red `✕` with the tooltip
  `Markers not saved: the file is read-only or in use (close it in Premiere and try again)`.
  Opening the file again shows the kept markers, with that sentence above the list, and the next
  save of the file tries again. Closing the window or opening another folder while any are kept
  asks `Markers of 2 files could not be saved.` with **Retry**, **Close anyway** and **Cancel**.
  Nothing is written to a fallback home.
- **Premiere or a batch job changed the markers** while frename had the file open: markers added
  elsewhere survive (unknown GUIDs are kept). A marker changed both in Premiere and in frename ends
  with frename's values for the fields frename shows (last writer).
- **Markers past the end**, negative or garbage times in a foreign file: shown clamped to the clip
  on the bar, kept unchanged in the file.
- **Many markers** (hundreds, e.g. from a speech-to-text tool): the list is a fixed-row-height
  scrollable, like the cue list with thousands of cues.
- **Rename**: markers live inside the video, so they move with it on disk. In memory, the renamed
  file's snapshot must carry the edited markers over (see "In memory").
- **Undo**: add, delete, color and duration are undo commands in `frename-core/src/undo`, scoped to
  the open file like in/out. Typing in name and comment is not undoable, like the comment box.
- **Leaving the file with an open row** (e.g. a click on another file): the typed text is in state
  already (every keystroke is a message), so it is saved and the row closes.
- **Image files**: no markers (no video timeline); the controls are not shown.

## Out of scope

- Marker types other than Comment (Chapter, WebLink, FLV cue points, Segmentation): kept intact,
  not shown.
- Sidecar `.xmp` files for formats that cannot hold XMP.
- A "has markers" filter in the file list, and caching markers in `.frename`.
- Converting existing `.snap.` screenshots into markers.
- Moving a marker by dragging its tick.
- A tooltip with the marker name when hovering a tick on the progress bar.

## Delivery

Two PRs, each through the full review gate:
1. **Markers** — core (XMP read/edit, snapshot, undo) and UI (ticks, hotkeys, list with tabs,
   snap, `F12` change), README and `version.md`. The README section "Timeline markers, comments,
   and IN/OUT points" is rewritten: screenshot ticks and the non-existent comment timestamp button
   go, markers come in, the new keys join the Video shortcut table, and the `F12` row becomes
   "Save the current frame as a JPEG", and one sentence says that in fullscreen `◆` shows the
   marker list. Body `Refs #9`.
2. **Batch** — Markers ⇄ comment. Body `Closes #9`.

## Test plan

Core unit tests (no network, fixtures in `crates/frename-core/tests/fixtures/`):
- write → read round-trip of name, multi-line comment (Cyrillic and emoji), duration and every
  color; CR line breaks in the file;
- a Premiere-shaped fixture (the research sample: `parseType="Resource"`, `keywordExtDVAv1_`,
  `marker_guid`) plus variants with attribute form, `f254016000000`, `f30000s1001` and
  `xmp:id:`-prefixed GUIDs: read correctly, and after frename edits one marker the others are
  byte-identical in their fields and unknown fields survive;
- tracks whose `trackType` is not `Comment` (`InOut`, `Chapter`, `Cue`, …) are untouched by
  marker writes, and marker writes do
  not disturb the in/out marker or the comment;
- saving a snapshot whose markers are `None` does not change the file; saving unchanged markers
  does not write;
- a marker added to the file after frename read it survives frename's save; fields frename does
  not know (`type`, other `cuePointParams`) survive an edit of that marker;
- renaming an open file keeps its edited markers (the snapshot rebuild carries them);
- `TagList::new(snapshot).file_snapshot()` keeps the markers;
- add → save with the file still open → edit the name → save again gives exactly one marker, with
  the new name; add → leave → undo the navigation → leave again gives no duplicate;
- add M → paste tags → undo → redo → save gives one M; add M → toggle a tag → undo → undo (removes
  M) → redo → save leaves M in the file;
- inline rename → delete a marker → undo refresh: the deleted marker does not come back from disk;
- a marker whose GUID the known set lacks survives a save; GUID-less markers are never changed;
- a marker edit and an in/out change in the same write both land;
- the range parser: `0:41-0:47` is a range, `0:41 - 0:47 …` is a point marker;
- comment → markers on a file whose XMP write fails leaves the comment unchanged;
- `.snap.` jpgs stay hidden from the file list;
- the open file is saved (markers included) on window close and on opening another folder;
- rate conversion at `f254016000000` for a 10-hour clip does not overflow;
- undo/redo of add, delete, color, duration;
- the timecode parser (every accepted form, ranges, name/comment split, lines without a time,
  past-the-end, unknown clip length) and the formatter (`h:mm:ss`, `.mmm`); markers → comment →
  markers keeps start, duration, name and one-line comment; comment → markers → comment is
  stable; re-running is a no-op.

UI: the sequence `Enter` in a name field → `Space` resumes playback is checked by hand or under
Xvfb once #14 makes the screen reachable; an Xvfb screenshot of the open-folder screen (the only screen reachable until #14's demo mode),
and review of the `view` code with a written description of the list, ticks and controls bar.

By hand, by the owner: markers set in frename show on the clip in Premiere Pro after import, with
name, comment, duration and color; a Premiere-written marker survives a frename edit.

## Open questions (with recommended answers)

1. **Hotkeys.** `F2` add, `Shift+F2` delete, `Shift+F1`/`Shift+F3` previous/next. *Recommended:* as
   above; Premiere's `M` conflicts with type-to-search.
2. **Where the marker list goes.** Same side overlay as subtitles, with tabs. *Recommended:* tabs —
   two overlays at once would cover most of a windowed video.
3. **Does a marker's text go to name or comment in the batch conversion?** *Recommended:* name —
   Premiere's marker panel lists names; the comment is for longer notes.
4. **Markers → comment: move or copy?** *Recommended:* copy — an export that never destroys colors
   or Premiere data, with no undo for batch jobs; re-runs add no duplicate lines. Comment → markers
   stays a move so the timecoded lines do not remain twice.
5. **F12 file name.** *Recommended:* keep `{file}.snap.HH-MM-SS-mmm.jpg` so nothing about it changes
   except that frename forgets it.
6. **Unverified colors.** The color values come from third-party writers and a 2018–2020 sample.
   *Recommended:* ship with the research table; the owner checks colors in Premiere and a follow-up
   fixes any that differ.
