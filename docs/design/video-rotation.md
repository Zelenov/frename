# Rotate a video 90°

Design for issue #66. Research: [`docs/research/video-rotation.md`](../research/video-rotation.md).

## Problem

Some clips are shot sideways: the phone was held the wrong way, or its orientation sensor guessed
wrong. The editor wants to turn such a clip upright in frename while reviewing it, and Premiere Pro
must import it upright. The picture must not be re-encoded: rotation is a flag in the file.

MP4 and MOV keep that flag in the video track's `tkhd` matrix, which Premiere, GStreamer, VLC,
Windows and clipscribe read. Other formats frename lists (MKV/WebM, MTS/M2TS, AVI, MPG) have no
flag any editor reads; they show "cannot rotate".

A side finding: frename's player ignores the flag today, so phone portrait clips play lying on
their side. Showing the flag is part of this change.

## User flows

1. **Turn the open clip.** The editor sees a sideways clip and presses `Ctrl+Alt+→` (or clicks
   `↻`). The flag in the file changes, the video opens again at the same moment in the same
   play/pause state, now upright; a note over the picture names the file's rotation flag now
   (`Rotation: 90° right`, `90° left`, `180°`, `none`). It names the flag, not the picture: a
   phone's portrait clip plays upright with a 90° flag, and `none` would show it sideways.
   `Ctrl+Alt+←` / `↺` turns it back. Four presses the same way bring the file back to exactly
   the bytes it had. A held key turns once. With no video shown nothing happens.
2. **Undo.** `Ctrl+Z` right after takes the turn back (the file is written again and the video
   reopens once, even when the turn's own reopen is still loading, with the same note); `Ctrl+Y`
   turns it again. Each press is one undo step. The step refreshes nothing else: a turn changes
   no tags, name or selection.
3. **A file that cannot turn.** An MKV, a read-only file, a file Premiere has open, a damaged
   or moved file: nothing changes and a note over the picture says why, like the note for
   failed marker writes: `Not rotated: the file is read-only or in use`, `Not rotated: this format has no rotation
   flag`, `Not rotated: the file is damaged`. The batch's failed list gives the same reasons.
4. **Many files.** Batch mode ▸ "Rotate videos" with four choices: 90° right, 90° left, 180°,
   "Reset: no rotation (0°)". Run on the checked files. Files with no rotation count as skipped
   for "Reset"; MKVs and locked files count as failed with the reason. "Reset" also removes a
   turn a phone recorded, which the hint says.
5. **Premiere.** The editor imports the clips: they come in portrait. Whether a clip that was
   already in a project follows the change is not known yet (see the research; the owner checks by
   hand).

## UI sketch

Video controls bar, after 📷 and 📍: ↺ ↻ as one tight pair of 24 px buttons; the row's spacing
is 4 px (was 8), so the volume still fits a player of the default width (460 px):

```
⏪ ▶ ⏩ [ ] 📷 📍 ↺↻                    🔊 ━━━━
```

Tooltips: `Rotate left (Ctrl+Alt+←)`, `Rotate right (Ctrl+Alt+→)`. For a clip whose rotation
cannot be read or changed the buttons are off with the reason as the tooltip (`Cannot rotate: this
format has no rotation flag`, `… the file is damaged`), like 📍 for a file that cannot hold
markers; while a clip loads they stay on. The keys still show the reason in a note (a key press
never does nothing silently).

Short notes (`Frame saved`, `Rotation: 90° right`, `Not rotated: …`) show over the bottom left
of the picture, 260 px wide at most (they wrap); in fullscreen at the top left, clear of the
subtitles. They used to sit in the controls bar, which has no room left for them.

Batch panel "Rotate videos": the title, a hint ("Changes the rotation flag of MP4 and MOV files;
the picture is not re-encoded. Premiere Pro is expected to show the clip turned when it imports
it. Running it again turns the files again; “Reset” removes any turn, including one a phone
recorded."), four radio buttons.

## Keyboard shortcuts

| Key | Action | Context |
|---|---|---|
| `Ctrl+Alt+→` | rotate the open video 90° clockwise | not in the comment box or a marker name |
| `Ctrl+Alt+←` | rotate the open video 90° counter-clockwise | not in the comment box or a marker name |

They follow the rules proposed in #62: a combination with Ctrl/Alt, never a plain character. They
act also after typing in a search field (like `[`, `]` and the F-keys), so they never do nothing
silently; in the comment box and a marker's name they stay with the text (the app asks whether
the comment box has focus). A held key turns once. Plain `←`/`→` stay with the tag grid.

## Data format

Only the 36-byte matrix in `tkhd` of every track whose `mdia/hdlr` is `vide` changes, in place.

- Reading: the turn is taken from `a b c d`; accepted values are exact multiples of 90° with or
  without a mirror (entries 0 and ±1.0). Anything else (a scaled or skewed matrix) is "cannot
  rotate: unusual orientation matrix".
- Writing: new `a b c d` = old × the 90° step, so a mirror is kept. The translation is the one
  that puts the turned picture at the origin (`x = height` for 90°, `width, height` for 180°,
  `0, width` for 270°), as phones and exiftool write it. Reading accepts that translation or none
  (ffmpeg's style); any other translation is "unusual". Four turns give back the original bytes
  for every file in the phones' style and every file that starts upright; an ffmpeg-style file
  comes back with the phones' translation (same orientation in every reader). Keeping ffmpeg's
  style is not possible: at 0° both styles are the same bytes, so the style is lost there.
- `u v w`, `mvhd`, `tapt`, `clap`, XMP, `udta`: untouched.
- The file's modified and created times are put back after the write, on the handle that wrote,
  as frename does for XMP. A write that fails part way puts back the tracks already written.
- A box that points past its parent or the end of the file makes the file "damaged"; the walk
  stops after `moov`, so leftovers after it do not matter.
- Formats: `.mp4`, `.m4v`, `.mov` whose content is MP4/MOV. Everything else: "cannot rotate".

## Where it lives

- **Core** `crates/frename-core/src/metadata/rotation.rs`: box walk to the video `tkhd`s, matrix
  maths, `rotation::read(path)` and `rotation::rotate(path, quarter_turns)`. `FileTagger::video_rotation` and
  `FileTagger::rotate_video` go through the backend: `ProductionFileTagger` writes the file,
  `InMemoryFileTagger` (debug builds) keeps the turn in memory by disk path, like its markers.
- **Undo**: `RotateVideoCommand { file: FileId, path: PathBuf, quarter_turns }`; undo turns the
  other way. The file's current path comes from the directory by `FileId`, so a rename in between
  does not break it; `path` is the fallback when the folder no longer lists it. The command says it
  `turns_a_video`, so only such a step makes the player check whether to reopen.
- **Player**: the pipeline gets `videoconvert ! videoflip video-direction=90r|180|90l` in front of
  the existing sink chain when the file is turned (`auto` for a mirrored one); an upright file keeps
  exactly today's pipeline. After a rotation the video reopens at its position and play state. The
  Windows bundle adds `videofilter` (the plugin with `videoflip`); without it (an old system
  GStreamer) the video plays as stored, as before.
- **Batch**: action `Rotate videos` in `src/features/batch/actions/rotate.rs`.

## Edge cases

- **Video open in frename while writing**: GStreamer opens files sharing read and write on
  Windows, so the in-place write works while the clip plays (tested); the clip is reopened right
  after.
- **Premiere has the file open** (or it is read-only): the write fails; nothing changed; the note
  says so. The batch lists it as failed.
- **Several video tracks**: all are turned the same step. The turn shown is the first track's.
- **Movie header matrix not identity**: ignored for reading and left alone.
- **Cloud placeholder (online-only)**: reading the header downloads it, as opening it to play does.
  Only done on an explicit rotate, never during a folder scan.
- **Undo after switching files**: `Ctrl+Z` walks back through file switches first, so the rotated
  file is open again when its rotation is undone.
- **Batch mode**: the open video's rotate keys work in batch mode like markers, and are off while a
  job runs (the job has closed the file).
- **Debug build**: rotations stay in memory; the player shows them; nothing is written.

## Out of scope

- Re-encoding (turning pixels) and formats without a flag.
- Arbitrary angles and mirroring as user actions.
- Showing the rotation in the file list or as a filter.
- The context menu entry: #48 is not on `main` yet; its issue gets the two entries when it lands.

## Test plan

Core unit tests on small MP4 and MOV fixtures (32×16 H.264 + AAC, a few KB each, in
`crates/frename-core/tests/fixtures/`):

- the upright fixture reads 0°; after a right turn it reads 90° with the phone-style matrix; after
  a left turn 270°; 180° likewise; for MP4 and MOV;
- four turns either way give back the original bytes; so do right + left;
- only the matrix bytes change (every other byte equal), the audio track's matrix stays identity;
- comment, in/out and markers written to XMP before the turn read back the same after it;
- the file's modified time is kept;
- an ffmpeg-style (no translation) turned matrix is read and turned;
- a mirrored matrix keeps its mirror; a scaled matrix is refused; an MKV/text file is refused;
  a file of zeros is "damaged"; a read-only file fails with the write error and is unchanged;
- a movie box that claims more than the file has is "damaged"; junk after the movie is ignored;
- version-1 `tkhd` (64-bit times) is found at the right offset (synthetic boxes built in the
  test);
- the in-memory backend keeps turns in memory and leaves the file alone.

App: turning, undoing and redoing the open clip in the workspace, a failed turn pushing no undo
step, two quick reopens keeping the moment and dropping the older load, the batch action's run on a
temp copy, and a turned clip reaching the player's sink with its sides swapped. By hand: the Premiere steps in the research note; the player shows a phone portrait
clip upright; `F12` saves an upright frame.

## Decisions made without the owner

1. **Write at once, not on leaving the file.** The player must reopen the file to show the turn,
   and nothing else waits for the rotation, so it is written on the key press. Undo writes again.
2. **Keep the file's modified time**, as the issue asks and as XMP writes do. Risk: Premiere or
   Explorer may keep a cached old orientation of a file they already know; the owner's hand test
   step 5 checks Premiere. If Premiere ignores the change, the follow-up is a setting-free change
   to bump the modified time for rotations only.
3. **Shortcuts `Ctrl+Alt+←/→`** as the issue suggests. Old Intel graphics drivers used the same keys
   to rotate the whole screen, but those hotkeys are off by default on current drivers; the README
   says what to do if they are on.
4. **Phone-style translation** on every write, because it is what iPhones and exiftool write and
   Premiere demonstrably imports; an ffmpeg-style file gets it on its first turn.
5. **Mirrored matrices are turned too** (mirror kept); the player shows them with `auto`.
6. **The batch action is not undoable**, like every other batch action; running the opposite
   direction undoes it.
7. **The player follows the flag for every clip**, which also turns existing phone portrait clips
   upright in frename. That is what every other reader does; release notes say so.
8. **No context-menu entry** now (#48 is not merged).
9. **Controls bar packed closer** (spacing 8 → 4 px, ↺ ↻ one 24 px pair): with two more buttons
   the volume no longer fitted a player of the default width. Every button stays.
10. **Short notes over the picture** instead of in the controls bar, for every note (`Frame
    saved` too): the bar has no room left, and a long note there pushed buttons out. Bottom left
    in the window, top left in fullscreen (the subtitles are at the bottom there).
