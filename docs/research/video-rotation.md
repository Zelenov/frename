# Rotating a video without re-encoding

Research notes for issue #66. Verified on this machine with ffmpeg 7, exiftool 13 and GStreamer
1.26.10 (Windows, MSVC build) where marked **tested**; Premiere Pro behaviour is from Adobe's
forums and was confirmed by the owner's hand test (2026-09-28, see "Owner's result" below).

## How rotation is stored

### MP4 and MOV: the track header matrix

Both are ISO base media / QuickTime files. Every track has a track header box,
`moov/trak/tkhd`, with a 3×3 transformation matrix that says how the decoded picture is placed on
screen (ISO/IEC 14496-12 §8.3.2; Apple QuickTime File Format, "Track header atom" and "Matrices").
Nine 32-bit big-endian values `a b u  c d v  x y w`: `a b c d x y` are 16.16 fixed point, `u v w`
are 2.30. A point `(p, q)` of the picture goes to `(a·p + c·q + x, b·p + d·q + y)`, with `y`
pointing down. Offsets from the start of the box payload: version 0 → byte 40, version 1 → byte 52
(the version-1 header has 64-bit times and duration).

| Turn (clockwise) | a | b | c | d | x | y | exiftool `Rotation` | GStreamer tag |
|---|---|---|---|---|---|---|---|---|
| 0°   | 1 | 0 | 0 | 1 | 0 | 0 | 0 | none / `rotate-0` |
| 90°  | 0 | 1 | −1 | 0 | height | 0 | 90 | `rotate-90` |
| 180° | −1 | 0 | 0 | −1 | width | height | 180 | `rotate-180` |
| 270° | 0 | −1 | 1 | 0 | 0 | width | 270 | `rotate-270` |

`width`/`height` are the track's own size from the same `tkhd` (the stored, unrotated picture).
The translation moves the turned picture back to the origin.

- **Phones** (iPhone, Android) record the sensor's landscape frame and write this matrix; an
  iPhone portrait clip is `0 1 0 -1 0 0 1080 0 1` (exiftool `MatrixStructure`, `a b u c d v x y
  w`), i.e. 90° with the translation.
- **exiftool `-Rotation=90`** writes exactly that, translation included (**tested**:
  `MatrixStructure = 0 1 0 -1 0 0 32 0 16384` on a 64×32 clip). It rewrites the whole file.
- **ffmpeg `-display_rotation N`** (ffmpeg 7; N is counter-clockwise) with `-c copy` remuxes the
  file and writes the matrix *without* the translation (**tested**: `-display_rotation 90` gave
  `0 -1 0 1 0 0 0 0 16384`, which exiftool reads as `Rotation 270`). ffmpeg, GStreamer and exiftool
  read the turn from `a b c d` only, so both styles play the same in them.
- **The old `rotate` tag.** ffmpeg used to take `-metadata:s:v rotate=90`; it was never stored as a
  tag in the file: the muxer turned it into the same matrix, and the demuxer exposed the matrix as
  a `rotate` tag. Since ffmpeg 6/7 the option is gone in favour of `-display_rotation` and the
  display-matrix side data. Nothing else to write.
- **The movie header** `moov/mvhd` has a matrix too, applied to the whole movie. Phones and ffmpeg
  leave it identity; leave it alone.
- **`tapt` (QuickTime track aperture: `clef`, `prof`, `enof`)** holds the clean/production/encoded
  sizes of the *stored* picture; iPhones keep them unrotated next to a rotated matrix. `clap`/`pasp`
  in the sample description are about cropping and pixel aspect, not orientation. None of them
  change with a rotation.
- **`moov/meta`, `udta`, XMP**: no orientation there that players read. XMP has
  `tiff:Orientation` for stills only; video readers do not use it.

Only tracks whose `mdia/hdlr` handler is `vide` are turned. Audio tracks have their own identity
matrix (and width/height 0). MOV has a second `hdlr` inside `minf` (data handler, `url `), which is
not the media handler.

The change is the 36 bytes of the matrix of each video track: the file keeps its size and every
other byte, so XMP (comment, in/out, markers), `udta`, chapters and the sample tables are
untouched. It can be written in place.

### Matroska / WebM

Matroska has `Video/Projection/ProjectionPoseRoll` (a float, degrees). ffmpeg reads it into its
display matrix, but at the time of Hoppenheit's test "no viewer supports Matroska's
ProjectionPoseRoll element yet" (mpv, VLC and most players ignore it), and Premiere Pro does not
import MKV at all. Changing it also means rewriting EBML element sizes. **Not worth doing: "cannot
rotate".**

### MTS / M2TS (AVCHD), AVI, MPG

No orientation field in the container (MPEG-TS, RIFF AVI, MPEG-PS). H.264/HEVC have an optional
"display orientation" SEI message, but it sits in the picture data, most decoders ignore it, and
writing it means rewriting the stream. **"Cannot rotate".**

## Who honours the matrix

### Premiere Pro

- Premiere turns phone footage upright on import: "On MacOS and in Premiere these files
  automatically come in as vertical" (Logik forums, about iPhone MP4); "Resolve also handles this
  correctly".
- Adobe community expert Stan Jones fixed wrongly flagged phone clips by editing the flag with
  exiftool before import: `exiftool -rotation=0 file.mp4` → "MediaInfo confirms there is no longer
  a rotation flag. Open in PR, the video is landscape." (Adobe community, "Fix clips rotated 90
  degrees", August 2023). So Premiere reads the `tkhd` matrix of an MP4 on import, and a changed
  matrix changes what it shows. The same thread confirms there is **no** rotation override in
  Interpret Footage; the alternatives are Transform/Motion effects, which keep the clip's 16:9
  frame (letterboxed portrait).
- MOV from iPhones imports upright as well (the Adobe thread "Premiere Pro rotation bug with iPhone
  video after file renaming" shows the rotated `IMG_E…MOV` upright). The same thread reports that
  after renaming `IMG_E0001.MOV` to `IMG_0001.MOV` Premiere showed it in the old orientation. No
  explanation was given; the likely cause is Premiere's media cache matching the file by name
  (the unedited `IMG_0001.MOV` had been imported before), not the matrix. It matters for us:
- **Footage already in a project** (confirmed by the owner, 2026-09-28: Premiere does not pick up
  a new rotation of a clip it has already imported until its media cache is cleared). No Adobe
  document says when Premiere re-reads a changed file
  header. Premiere keeps per-file data (conformed audio, peak files, indexes) in its media cache
  database and checks files it has open for changes; it does not say which properties it compares.
  frename keeps the file's modified time on XMP writes, and a rotation keeps the file's size too,
  so Premiere may not notice. A clip that was already imported may need re-importing (or Media ▸
  "Replace Footage", or deleting the clip's cache files); this is a guess until the hand test
  below confirms it. A clip imported after the rotation is read fresh. This is the main thing to
  confirm by hand.
- 90° and 270° are the same kind of matrix for Premiere; nothing suggests one works and the other
  does not, but the hand test covers both.
- XMP survives: the rotation touches only the matrix bytes, so the comment, in/out subclip marker
  and clip markers are byte for byte the same (unit-tested).

### GStreamer and frename's own player (**tested**)

`qtdemux` reads the matrix and posts an `image-orientation` tag (`rotate-90`, `rotate-180`,
`rotate-270`, `flip-rotate-*` for mirrored ones); `gst-launch-1.0 -t filesrc ! qtdemux` on the
ffmpeg-rotated file printed `image orientation: rotate-270`. But **`playbin` does not apply it**:
with frename's pipeline (`playbin … video-sink="videoscale ! videoconvert ! appsink"`) the sink
negotiated `width=64, height=32` for the rotated 64×32 clip, i.e. the picture stays as stored.
With `videoflip video-direction=auto` in front of the sink it negotiated `32×64`.

So **frename today shows every phone portrait clip lying on its side**, and would show a rotation
it writes only if its pipeline gets a `videoflip`. `videoflip` is in gst-plugins-good's
`videofilter` plugin (`gstvideofilter.dll`), which the Windows bundle does not carry yet
(`packaging/windows/gstreamer-plugins.txt`). The Linux AppImage takes every installed plugin.

Setting the direction explicitly (`video-direction=90r|180|90l`) instead of `auto` lets frename
show a rotation that exists only in memory (debug builds write nothing to disk) and makes the
player independent of the tag. `iced_video_player` reads the frame size from the sink caps when the
video opens, so a new orientation needs the video to be opened again; screenshots (`F12`) take the
sink's last frame, so they come out turned, and fullscreen shows the same frames.

### clipscribe ("Describe with AI")

clipscribe reads the same `image-orientation` tag from GStreamer and turns frames upright before
sending them (`src/frames.rs`, `orient`: `rotate-N` turns N° clockwise, `flip-rotate-N` mirrors
first; test `a_rotated_clip_comes_out_upright`). A rotation written by frename is therefore
followed with no change there.

### Windows Explorer thumbnails, Films & TV / Media Player

Windows' Media Foundation MP4/MOV source reads the `tkhd` matrix (it exposes it as
`MF_MT_VIDEO_ROTATION`), and Explorer's thumbnails of phone clips are upright. Explorer caches
thumbnails; with the modified time kept, a cached thumbnail can stay in the old orientation until
the cache is cleared (Disk Cleanup ▸ Thumbnails). Cosmetic only.

### VLC

VLC 3 applies the orientation of MP4/MOV (the "auto rotate" of phone clips has worked since 3.0,
`--no-autorotate` to turn it off).

## Hand test for the owner (Premiere Pro)

1. Take two short clips shot landscape: one `.mp4`, one `.mov` (an iPhone `.MOV` is ideal).
   Give each a comment, an in/out and a marker in frename first.
2. In frename rotate the MP4 **right** once (90° clockwise) and the MOV **left** once (270°).
3. In a **new** Premiere project import both. Expected: both in the Project panel and Source
   Monitor as 9:16 portrait, turned the right way; the comment in the Description column, the
   in/out subclip and the marker still there.
4. Drop each on a 16:9 sequence: Premiere should show a pillar-boxed portrait picture (not a
   stretched or letterboxed landscape one).
5. Back in frename rotate the MP4 right again (now 180°), with the project open. Check in
   Premiere: does the clip change by itself? After File ▸ Close Project and reopen? After
   right-click ▸ "Replace Footage…" with the same file? Note which one it takes.
6. Rotate the MOV while Premiere has it imported and open: frename should show
   "Not rotated: the file is read-only or in use" if Premiere locks it.
7. Rotate a clip four times in frename: it is back to how it was, and plays the same in Premiere.

### Owner's result (2026-09-28)

Rotation works in frename and a turned clip imports turned into Premiere Pro. Premiere does **not**
update the rotation of a clip it imported before the turn: the old orientation stays until the
media cache is cleared (Media Cache ▸ Delete in its preferences) and the clip is imported again. This is
Premiere's cache, not frename; the README says so next to the rotation keys.

## Sources

- ISO/IEC 14496-12, ISO base media file format, §8.3.2 Track Header Box (matrix, width/height).
- Apple, QuickTime File Format Specification: "Track header atoms", "Matrices" —
  <https://developer.apple.com/documentation/quicktime-file-format/track_header_atom>
- ffmpeg `-display_rotation` and the removal of the `rotate` metadata option; lossless rotation
  notes — <https://gist.github.com/ViktorNova/1dd68a2ec99781fd9adca49507c73ee2>,
  <https://ffmpeg.org/pipermail/ffmpeg-user/2024-July/058428.html>
- Martin Hoppenheit, "Rotating Matroska video" (2022) —
  <https://martin.hoppenheit.info/blog/2022/rotating-matroska-video/>
- Adobe community, "Fix clips rotated 90 degrees" (Aug 2023, exiftool fix confirmed in Premiere) —
  <https://community.adobe.com/t5/premiere-pro-discussions/fix-clips-rotated-90-degrees/m-p/13983843>
- Adobe community, "Premiere Pro rotation bug with iPhone video after file renaming" —
  <https://community.adobe.com/questions-729/premiere-pro-rotation-bug-with-iphone-video-after-file-renaming-1419480>
- Logik forums, "Rotation metadata flag" (Premiere and Resolve honour it, Flame does not) —
  <https://forum.logik.tv/t/rotation-metadata-flag/11184>
- Adobe, "Manage media cache files in Premiere" —
  <https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/manage-media-cache.html>
- GStreamer `videoflip` (`video-direction`, `auto` follows `image-orientation`) and `qtdemux`
  (matrix → `image-orientation`): `gst-inspect-1.0 videoflip`, GStreamer 1.26 sources.
- clipscribe `src/frames.rs` (`orient`), pinned commit in `Cargo.toml`.
