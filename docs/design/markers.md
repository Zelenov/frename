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
2. **Describe a marker.** The editor opens the marker list (◆ button, or it is already open) and
   clicks a marker's row (or its `✎`): the row opens for editing, with a name field (focused) and a
   multi-line comment editor. `Enter` in the name field, `Esc` in either field, or a click on the
   row's `✓` closes it and gives the keyboard back to the workspace, so `Space` plays again and
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
- **Controls bar.** `◆+` adds a marker (tooltip `Add marker (F2)`); `◆` shows or hides the marker
  list (tooltip `Markers`), next to `CC`. `◆` is shown whenever a video is open, in fullscreen too.
- **Side list.** One overlay on the right, as the subtitle list is today. When the video has both
  subtitles and markers (or the marker list was opened), the top of the overlay has two tabs,
  `Subtitles` and `Markers`; with only one of them there are no tabs. `CC` opens the subtitle tab,
  `◆` the marker tab; pressing the button of the open tab closes the overlay. The last tab is kept
  across files. Fullscreen shows the subtitle overlay when subtitles exist, as today; the marker
  tab only when the editor opens it with `◆` (then `◆` again hides the overlay), so marked clips
  are not watched with part of the picture covered.
- **Marker row.** Read-only, like a cue row: color dot (click → a row of the 9 colors), time
  (`m:ss` or `m:ss–m:ss`; click to jump), `⇥`/`⇤`, `✎`, `✕`; then the name and up to two lines
  of the comment as text. Rows have a fixed height so the list scrolls to a row by arithmetic, as
  the cue list does. **One row at a time** opens for editing (flow 2): only that row holds a
  `text_input` and a `text_editor` (one `text_editor::Content` in state, created when the row
  opens), and it is taller; rows below it shift by a known amount, so scrolling stays arithmetic.
  The row of the last marker at or before the playhead is lit; the list follows playback except
  while a row is open.
- **Empty list.** `No markers yet — press F2 to add one`.

## Keyboard shortcuts

| Key | Action |
|---|---|
| `F2` | Add a marker at the playhead |
| `Enter` / `Esc` in an open marker row | Close the row, back to the workspace |
| `Shift+F2` | Delete the marker under the playhead (nearest within 0.5 s) |
| `Shift+F1` / `Shift+F3` | Jump to the previous / next marker |
| `Shift` + drag on the progress bar | Snap to the nearest marker (within 8 px) |
| `F12` | Save the current frame as a JPEG next to the video |

F-keys work while the search bar has focus, as `F1`/`F3`/`F12` do today. Premiere's own `M` is not
used: letters type into the tag search.

While a marker row is open for editing, keys belong to its fields: `[`, `]` and `Shift+Space`
(which today fire whatever has focus) are not handled globally, so typing `[b-roll]` does not set
in/out points. The workspace knows a row is open (it is state), so the global key handler checks
that, not the widget focus. `Enter` (name field) and `Esc` (either field) close the row. `F2` sits between the seek keys, and `F1`/`F3` with
`Shift` keep "back/forward" on the same keys. `F1`/`F3`/`F12` today ignore modifiers; they will
check that `Shift` is not held.

A second `F2` within 100 ms of an existing marker does nothing (same dedupe as screenshots today).

## Data format

### In the video (XMP)

Markers live in `xmpDM:Tracks` on tracks whose `trackType` is `Comment`, as in the research
sample. Reading accepts every form listed under Gotchas in the research: any `frameRate`
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
Every `Marker` read from the file keeps its **as-read values** (id, track, start, duration, name,
comment, color) next to its current ones; a new marker has none. A save computes the changes
*as read → now* and applies only those to the XMP that is in the file at write time:
- a marker is found in the file by id (GUID; for a marker without one, by its as-read track, start
  and name); if it is no longer there, its change is dropped and logged;
- an unchanged marker is not touched, so a Premiere-written marker keeps its exact frame times and
  every field frename does not know (`type`, `speaker`, other `cuePointParams`, …);
- a changed field is rewritten on that marker only, in the track's own rate
  (`round(ms × rate / 1000)`, computed in `u128`, so `f254016000000` cannot overflow);
- a marker the user deleted (it was read, and is gone now) is removed from its track; a track left
  empty by that is removed, and so is `xmpDM:Tracks` when it is left empty (as `set_in_out_range`
  does);
- markers in the file that frename never read (e.g. Premiere added one while the file was open)
  are never deleted;
- a new marker is appended to the first `Comment` track, or to a new `Comment` track
  (`trackName` `Comment`, `frameRate` `f1000`) when there is none;
- `InOut` tracks (frename's in/out) and every other track type are left alone;
- nothing is written when the markers did not change, and file times are kept, as `xmp::write`
  does today.

Markers are always stored in the video: there is no file-name or text-file home, so the Settings
choices for comments and in/out do not apply to them.

### In the file snapshot

`FileSnapshot` gets `markers: Option<Vec<Marker>>`. `None` means "not read": saving leaves the
file's markers as they are. This is the same guard `comment_loading` gives the comment today, so a
folder scan, a batch rename or a tag-only save can never wipe markers it did not read. Markers are
read when a file is opened (one XMP read of one file) and by the batch actions; the folder scan
and the `.frename` file list do not read or cache them.

Every place that builds a new snapshot from the open file's snapshot must carry the markers over,
or the user's unsaved edits are silently dropped. Today that is `submit_rename`
(`src/features/folder_workspace/state.rs`, which copies the comment and screenshots) and the
paste-tags path (`paste_tags`, which calls `FileSnapshot::new`); the implementation lists them all with a grep for
`FileSnapshot::parse`/`FileSnapshot::new` in `src/`, and a test renames an open file with edited
markers and checks they are written.

### Screenshots

`F12` keeps writing `{file name}.snap.HH-MM-SS-mmm.jpg` next to the video (the file list shows only
videos, so it stays hidden there). `FileSnapshot::screenshots`, the ticks,
the comment text and the rename-with-video code are removed. Existing `.snap.` files stay on disk
untouched and are no longer shown as ticks.

## Batch actions

One new batch action, **Markers ⇄ comment**, with a direction switch like **Move comments**:

One line format serves both directions:

```
<time>[–<time>] — <name>[ — <comment>]
```

`<time>` is `m:ss` below an hour and `h:mm:ss` from an hour, with `.mmm` added when the
milliseconds are not zero. The range dash may be `–` or `-`.

- **Comment → markers.** Each comment line that starts with a time (or a range) becomes a marker
  and is removed from the comment. The rest of the line, after a leading separator (`—`, `–`, `-`,
  `:`) and spaces, is split on the first ` — `: the part before is the **name**, the part after the
  **comment**. Also accepted: `mm:ss` and the old screenshot form `HH-MM-SS-mmm`. Lines without a
  time stay in the comment. When the clip length is known, a line whose time is past the end stays
  in the comment and the file's result says so; when it is unknown, every time is accepted. If a
  marker with the same name already exists within 0.5 s, no duplicate is added (the line is still
  removed).
- **Markers → comment.** Each marker becomes a line in the format above, appended to the comment
  in time order, and the markers are removed from the video. The marker comment's line breaks
  become spaces. The result text says how many markers were moved.

Both are moves, like the existing move actions, so re-running one does nothing, and running one
after the other brings back each marker's start, duration, name and one-line comment. Lost on the
way: color, line breaks inside a marker comment, the marker id, and anything Premiere-specific;
a name that itself contains ` — ` comes back split. Files that cannot hold XMP get a red result
(`this format cannot hold markers`) and are not changed. The comment is written through the normal
save path, so it follows the Settings comment storage and the "Commented" tag rule.

## Edge cases

- **Video that cannot hold XMP** (the toolkit has no smart handler for it, e.g. some `.MTS`), or
  a **read-only file** (checked when the file opens: read-only attribute or a read-only folder):
  `◆+` is disabled with the tooltip `This file cannot hold markers` (or `This file is read-only`).
  `F2` on such a file opens the marker list, which shows that same sentence instead of rows, so a
  key press never silently does nothing. The README lists which video formats hold markers.
- **XMP write fails anyway** (file in use, disk full): the unsaved markers are kept in memory for
  that file until frename closes, the failure is logged, and the file's row in the list gets a red
  `✕` with the tooltip `Markers not saved: the file is read-only or in use`. Opening the file again
  shows the kept markers, with that sentence above the list, and the next save of the file tries
  again. Nothing is written to a fallback home.
- **Premiere or a batch job changed the markers** while frename had the file open: the save applies
  only the user's *as read → now* changes, so markers added or edited elsewhere survive. The same
  field changed in both wins for frename (last writer).
- **Markers past the end**, negative or garbage times in a foreign file: shown clamped to the clip
  on the bar, kept unchanged in the file.
- **Many markers** (hundreds, e.g. from a speech-to-text tool): the list is a fixed-row-height
  scrollable, like the cue list with thousands of cues.
- **Rename**: markers live inside the video, so they move with it on disk. In memory, the renamed
  file's snapshot must carry the edited markers over (see "In the file snapshot").
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
   go, markers come in, and the new keys join the Video shortcut table. Body `Refs #9`.
2. **Batch** — Markers ⇄ comment. Body `Closes #9`.

## Test plan

Core unit tests (no network, fixtures in `crates/frename-core/tests/fixtures/`):
- write → read round-trip of name, multi-line comment (Cyrillic and emoji), duration and every
  color; CR line breaks in the file;
- a Premiere-shaped fixture (the research sample: `parseType="Resource"`, `keywordExtDVAv1_`,
  `marker_guid`) plus variants with attribute form, `f254016000000`, `f30000s1001` and
  `xmp:id:`-prefixed GUIDs: read correctly, and after frename edits one marker the others are
  byte-identical in their fields and unknown fields survive;
- `InOut`, `Markers`, `Cue` and Chapter tracks are untouched by marker writes, and marker writes do
  not disturb the in/out marker or the comment;
- saving a snapshot whose markers are `None` does not change the file; saving unchanged markers
  does not write;
- a marker added to the file after frename read it survives frename's save; a marker edited in
  the file after the read keeps the fields frename did not change;
- renaming an open file keeps its edited markers (the snapshot rebuild carries them);
- rate conversion at `f254016000000` for a 10-hour clip does not overflow;
- undo/redo of add, delete, color, duration;
- the timecode parser (every accepted form, ranges, name/comment split, lines without a time,
  past-the-end, unknown clip length) and the formatter (`h:mm:ss`, `.mmm`); markers → comment →
  markers keeps start, duration, name and one-line comment; comment → markers → comment is
  stable; re-running is a no-op.

UI: an Xvfb screenshot of the open-folder screen (the only screen reachable until #14's demo mode),
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
4. **Markers → comment: move or copy?** *Recommended:* move, mirroring the existing move actions,
   so the pair is reversible and re-runs do not duplicate lines.
5. **F12 file name.** *Recommended:* keep `{file}.snap.HH-MM-SS-mmm.jpg` so nothing about it changes
   except that frename forgets it.
6. **Unverified colors.** The color values come from third-party writers and a 2018–2020 sample.
   *Recommended:* ship with the research table; the owner checks colors in Premiere and a follow-up
   fixes any that differ.
