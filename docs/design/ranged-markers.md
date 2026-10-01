# Ranged markers and AI markers

Part 1 implemented; part 2 in part (see As built). Follows `docs/design/markers.md` (point markers, 0.70).

## As built

Part 1 (ranged markers) in full, and the comment side of part 2, in one PR:

- **Hold `F2`**, **`Alt`+drag** (paused), **handles** on the active range with `Shift` snapping
  (to other markers' starts and ends, and to in/out), swap when an end passes the other, **back to
  a point** (ends within 100 ms, or `Alt`+click the band), **lanes** (3 at most), and **click a
  band to play it**. One undo step each; the new core command is `SetMarkerSpanCommand` (start and
  length together). The active band is drawn fully opaque, the others at 60 %.
- A held `F2` is one `AddMarkerCommand`: it captures the marker when undone, so the range comes
  back whole. The naming window of `F2 F2` restarts on release, so `F2` right after a range
  names it. `F2` auto-repeat is ignored (it used to open the marker's row while held).
- **Markers ⇄ comment and the AI block.** The block's segment lines (`0:00–0:14 Street.`) have no
  separator after the time, so the batch action used to skip them. Now *comment → markers* turns
  them into markers in the AI color (White), points and ranges, with the segment text as the name;
  the block stays in the comment (it is the source of the text), and a re-run adds nothing twice.
  *Markers → comment* leaves out markers the block already holds (same start, length and name) and
  writes its lines above the block, so a new AI run does not replace them. Before, they were
  appended at the end of the comment, which is inside the block.
- **Not in this PR** (part 2 UI): the picker's AI group, Mine / AI sections, ↻ and 📌, turning a
  white marker Green on edit, regeneration, AI lanes, and "Describe with AI" writing markers itself.

Two parts:

1. **Ranged markers**: a marker with a length (`0:41–0:47 — Lion`), made and edited by hand.
2. **AI markers**: points and ranges written by "Describe with AI" as real markers, told apart
   from the user's own by one reserved color, and regenerated without touching the user's.

## 1. Ranged markers

### What exists already

- `Marker` has `start_ms` and `duration_ms`; `end_ms()` gives the end. Premiere reads and writes
  the length, and frename already reads, keeps and writes it (`markers_xmp.rs`).
- Undo: `SetMarkerDurationCommand` was in core (its UI was removed before 0.70); removed in #139, `SetMarkerSpanCommand` covers it.
- The progress bar draws a band for a marker with a length (`BarMarker { start, end }`).
- `marker_at` already treats the playhead as "on" a ranged marker from its start to its end, so
  the label (the pin's head) stays up for the whole range.
- The batch line format already has ranges: `0:41-0:47 — Lion`.

So the model and storage are done. The work is input, drawing and editing.

### Ranges versus in/out

In/out is one segment per clip: the usable part. It goes into the file name or becomes a
Premiere subclip. Ranges are notes, and a clip can have many. They must never look alike:

| | In/out | Ranged marker |
|---|---|---|
| How many | 0 or 1 per clip | any |
| Drawn | fill of the bar itself (`SEGMENT` color) | band under the bar, in the marker's color |
| Keys | `[` `]` | hold `F2` |
| Stored | file name or XMP in/out | XMP clip marker with a duration |

### Making a range

**Hold `F2`** (main way, works while playing):
- Key down: a marker starts at the playhead, like today's `F2`.
- Held more than 400 ms: it becomes a range, and its end follows the playhead live (the band
  grows as the video plays).
- Key up: the end is set there. One undo step (`AddMarkerCommand` with the duration).
- A short press stays a point marker, so `F2 F2` to name it still works.

What this needs:
- `video_controls` handles `KeyPressed(F2)` only today. It needs `KeyReleased(F2)` too, and has to
  ignore auto-repeat `KeyPressed` events while the key is held (iced marks them `repeat`).
- `MarkersState` gets `recording: Option<(guid, pressed_at)>`. Each video tick while recording
  sets `duration_ms` to `playhead − start`; release commits it.
- If playback is paused while recording, release after 400 ms still makes a range only when the
  playhead moved (by `F3` or a seek); otherwise it stays a point.

**Alt+drag on the bar** (when paused): press sets the start, release sets the end. `Shift`
still snaps both ends to other markers and to in/out.

### Drawing

- Pin at the start, as for points; a 3 px band under the bar from start to end.
- Overlapping ranges go into lanes: 3 lanes at most, each range in the first lane where it fits,
  and anything beyond that shares the last lane. The widget grows by the lanes in use.
- Active range (playhead inside): brighter band, label head over its start pin, visible for the
  whole range.
- The folder list stays the same: 📍 and the count cover points and ranges alike.

### Editing

- **On the bar.** The active range shows 6 px handles at both ends; dragging one moves that end.
  `Shift` snaps. Dragging the start past the end swaps them. One undo step per drag
  (`SetMarkerDurationCommand`, plus a start-moving command, which is new).
- **Back to a point.** Drag the ends together (within 100 ms), or `Alt`+click the band.
- **Marker list.** The time reads `0:41–0:47`, and clicking it jumps to the start. No ⇥/⇤
  buttons (they were removed as confusing): the length is edited where it is seen.
- **Play the range.** Clicking the band plays from start to end and pauses, to check a moment was
  caught.

### Tests

- Core: a range round-trips through XMP (it already does); undo of add, resize and move.
- UI: hold and release give the duration; a short press gives a point; auto-repeat is ignored;
  lane assignment for overlaps; handle drag with a swap.

### Order

1. Hold `F2` and the band under the bar: enough to make and see ranges.
2. Handles on the active range.
3. Lanes for overlaps, then click-to-play.

## 2. AI markers

"Describe with AI" already writes an AI block into the comment, with timed segments. The next
step is to write those moments as real markers, points and ranges, straight into the video.
There is no accept/reject step: an AI marker is a marker from the start, and Premiere shows it.

### The rule: the color is the owner

**A marker is AI if and only if its color is the AI color.** Nothing else is stored, so what
frename treats as AI is exactly what the user sees, in frename and in Premiere.

- **The AI color is White.** Of Premiere's 9 marker colors it is the only one without a hue, so
  it stands apart from all the others on the bar and in Premiere's timeline. The user's default
  stays Green.
- **Touching an AI marker makes it human, and that shows as a color change.** Renaming, moving or
  resizing an AI marker turns it Green (the default) in the same step. Undo brings back the white
  marker as it was. Regeneration only ever looks at white markers, so an edited marker is safe
  because it is no longer white.
- **Recoloring is the explicit way.** Picking any other color makes the marker human; picking
  White makes any marker AI, and the next regeneration may replace it. The picker says so (below).
- **Premiere.** A marker recolored in Premiere follows the same rule: frename reads the color on
  the next open. Filtering by White in Premiere shows the AI's markers only.

Nothing about ownership is kept in `.frename` or XMP beyond the color. A file copied anywhere,
with or without its `.frename`, keeps its AI markers as AI.

### The color picker

The picker is the row of dots that opens from a marker's color dot. It gets two groups:

```
 ● ● ● ● ● ● ● ●  │  AI
                  │  ○
```

- The 8 human colors on the left, in Premiere's order without White.
- A divider, then White on its own under the caption **AI**, with the tooltip "AI marker: replaced
  when the AI describes this clip again".
- Picking White on a human marker asks nothing; the caption has already said what it means.

### How they look

The color does the work; the rest only adds to it:

| | Human marker | AI marker |
|---|---|---|
| Pin | its color | White |
| Range band | its color, lane 1 (under the bar) | White, lane 2 (below lane 1) |
| Label (active) | name | `AI · name` |
| Marker list | in "Mine" | in its own section **AI** below, with a white left edge |
| Folder list | 📍 3 | 📍 3 · AI 5 |

The marker list is split in two sections, both in time order:

```
 Mine ────────────────────────────────
 ● 0:12   Lion entering            ✎ ✕
 ● 0:41–0:47 Cub                    ✎ ✕
 AI ──────────────── ↻ All   ✕ All ──
 ○ 0:03   Car on the road        ↻ 📌 ✎ ✕
 ○ 0:20–0:26 Birds take off      ↻ 📌 ✎ ✕
```

### Buttons on an AI row

| Button | Does | Afterwards |
|---|---|---|
| time (`0:03`, `0:20–0:26`) | jump to the start | stays AI |
| ○ color dot | open the picker | any other color makes it human |
| ↻ Regenerate | describe just this moment again (see Regenerating) | replaced by the new AI marker(s) |
| 📌 Keep | make it the user's without changing it: turns Green | moves to "Mine" |
| ✎ Rename | open the name field | turns Green on the first typed letter, moves to "Mine" when the row closes |
| ✕ Delete | delete it | gone until the next regeneration, which may bring the moment back (see Deleting) |

The AI section header has:

| Button | Does |
|---|---|
| ↻ All | describe the whole clip again: every white marker is replaced |
| ✕ All | delete every white marker (one undo step) |

A human row keeps today's buttons (✎ ✕ and its color dot). No ↻ there: the AI never replaces a
human marker.

The active label over the bar shows `AI · name` with ↻ and ✎ next to it, the same as the row.

### Regenerating

**All** (↻ All, or "Describe with AI" on the file or in batch):
1. The clip is described again.
2. Every white marker of the file is removed. Markers of any other color stay.
3. The new moments are written as white markers, except those that land on a human marker (see
   Overlaps).
4. One undo step brings the previous white markers back.

**One** (↻ on a row or on the label):
1. The window sent to the model is the marker's range plus 3 s on each side, or 5 s on each side
   of a point.
2. The answer replaces the white markers inside that window only, with the same overlap rules.
3. One undo step.

**A stretch** (Alt+drag on the bar selects a window, then ↻ on the selection): the same as "one",
for any window. It fills in a part the AI missed.

While a regeneration runs, the white markers it will replace are shown striped, so it is clear
what is about to change. On failure (no key, network), nothing is removed.

### Overlaps

| Case | Rule |
|---|---|
| AI range vs AI range from the same run | The model is asked for ranges that do not overlap. If they still do, the later one is cut to start where the earlier ends; if nothing is left of it, it is dropped. |
| AI range vs human range | Both are kept, in their own lanes (human lane 1, AI lane 2), so they never cover each other. An AI range lying entirely inside a human one is dropped: the human already marked it. |
| AI point vs human point | An AI point within 1 s of a human marker is dropped. |
| Regenerating one range next to others | Only white markers inside the window are replaced; its neighbours stay, and a new range that overlaps a neighbour is cut, as in the first row. |

### Deleting

Deleting an AI marker is not remembered: frename cannot tell whether a later marker is "the same"
moment, since the model words it differently and moves it by a second or two each run. So
**regenerating may bring a deleted moment back**. To keep the AI off a moment for good, the user
keeps or renames the marker instead: it turns Green and regeneration leaves it.

This keeps ownership in the color alone, with nothing stored outside it. If returning moments turn
out to annoy in practice, the follow-up is to remember deleted time windows per file, pass them to
the model as "do not describe", and filter by time overlap as a safety net.

On the bar: lane 1 holds human ranges, lane 2 AI ranges. Within a lane, overlapping ranges (only
possible for human ones, drawn freely) stack into up to 3 sub-lanes, as in part 1. The active
label is the latest started marker under the playhead, human first when both start together.

### Storage and Premiere

- AI markers are ordinary XMP clip markers: White, with their name, length, and the AI text as
  the marker comment, which Premiere shows.
- The AI block in the comment stays the source of the text. Markers are written from it.

### Tests

- Core: rename, move, resize and Keep turn a white marker Green in the same undo step; "regenerate
  all" removes only white markers; the overlap rules in the table; undo of a regeneration; a
  deleted white marker does not block the next run.
- UI: the picker's AI group and caption; the Mine / AI sections; the row buttons in the tables
  above; ↻ and ✎ on the active label.

### Defaults chosen here (change if needed)

- AI color: White. Human default: Green.
- Regenerate-one window: 3 s around a range, 5 s around a point.
- AI point next to a human marker: dropped within 1 s.
