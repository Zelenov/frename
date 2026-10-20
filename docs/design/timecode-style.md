# Timecode style (#309)

Settings → Interface → **Time display**: Default, Premiere Pro, DaVinci Resolve.

## What it changes

Display only, plus what Copy (`Ctrl+Shift+C`, right-click on the time) puts on the clipboard and
what the time field starts with (typing a time already reads every style, see `timecode::parse`).

| Where | Default | Premiere Pro / DaVinci Resolve |
|---|---|---|
| Time under the video | `00:10 / 00:30`, `00:10.250` paused | `00:00:10:06 / 00:00:30:00` |
| Marker list | `0:41`, `0:41–0:47` | `00:00:41:00–00:00:47:00` |
| In / Out line in the marker list | `0:41–0:47` | same as above |
| Subtitle list | `0:41` | `00:00:41:00` |
| In / Out chips | `01:05` | `00:01:05:12` |

Unchanged on purpose, because they are read back without the clip's frame rate (the line reader
of Markers ⇄ comment takes `m:ss` only; the marker list's Copy and Export… write that format, so
pasting them into a comment still makes markers; #306's batch export is the same): the In / Out line in comments, markers saved in files, the markers text export and
Copy as text, subtitle files, file names. The notice "stopped at …" keeps its short form.

## Formats

- Frames are the frame the moment falls in: `round(ms × fps / 1000)`, the inverse of what typing a
  timecode does. Fields are `HH:MM:SS:FF`, always two digits; frames per second counted are
  `fps.round()` (23.976 → 24, 59.94 → 60).
- 29.97 and 59.94 fps are **drop-frame** (2 and 4 frame numbers skipped each minute, but every
  tenth), 23.976 and the whole rates count whole frames.
- Separator: Premiere Pro shows a drop-frame timecode with semicolons everywhere
  (`00;01;00;02`), DaVinci Resolve with a semicolon before the frames only (`00:01:00;02`).
  Non-drop-frame timecodes use colons in both.
- Both styles start at `00:00:00:00` at the start of the clip: frename does not read the start
  timecode a camera writes into the file (the sources' own offsets, e.g. Resolve's 01:00:00:00
  timeline start, are not applied).

## Clips without a steady frame rate

A clip GStreamer reports as variable-frame-rate (iPhone recordings), or with a frame rate that is
zero or not a number, has nothing to count frames in: it is written in the default style, and the
Settings help (ⓘ) says so. The player marks such a clip when it opens it (it counts it as 30 fps
only to open it), so the nominal rate is never shown as real frames.

## Decisions made without the owner

- Drop-frame is decided by the frame rate alone (29.97 and 59.94 are always drop-frame). A
  non-drop-frame 29.97 clip is rare, and the app cannot tell from the file; the formatter itself
  supports both through the separator choice only, so no setting is added.
- The two styles differ only by the drop-frame separator. Both are the clip's own time, not a
  timeline time.
- The readout is wider in a timecode style (180 px instead of 152 px); the controls bar folds
  buttons into More earlier to make room.
- The In / Out line in comments and the marker exports keep their format (the issue left this to
  the design).
