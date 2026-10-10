# frename

![frename turns camera file names like IMG_2072.MOV into names made of tags](docs/frename-poster.jpg)

frename is a desktop app for the hour before you start editing. You open a folder of raw video,
watch each clip, and tag what you see. When you move to the next clip, frename renames the one you
left with its tags, so `MVI_0410.mp4` becomes `pick.wide.night.city.MVI_0410.mp4`. Your markers,
in and out points and comments go into the video file, where Premiere Pro reads them.

I made frename after a trip to Africa, when I came home with 2,000 clips named `MVI_0001.MP4` to
`MVI_2000.MP4`. With frename I watched and tagged every clip in a few seconds each, so the project
was already organized when I opened Premiere.

**Download:** [Windows installer](https://github.com/Zelenov/frename/releases/latest/download/frename-win-Setup.exe),
[all downloads (Windows zip, Linux AppImage)](https://github.com/Zelenov/frename/releases/latest).
frename is free and open source (MIT license).

## What you can do with it

- **Tag clips in seconds.** Click a tag, or move through the tag grid with the arrow keys and press
  `Shift+Space`. Copy one clip's tags to the next with `Ctrl+C` and `Ctrl+V`. The tags become part
  of the file name, so your file manager, your editing app and your backup tools all see them.
- **Mark the moments that matter.** Press `F2` to drop a marker and name it, hold `F2` to mark a
  range, and press `[` and `]` to set the usable part of the clip. Premiere Pro shows the markers on
  the clip, with their names, lengths and colors, and turns the in and out points into a subclip.
  Describe with AI can suggest in and out points: its AI pill sets them in one click, rounded to
  whole seconds.
- **Write a comment for each clip.** By default the comment is saved inside the video, where
  Premiere Pro shows it in the Description column and finds it by search.
- **Let AI describe your footage.** Describe with AI writes a short summary of each clip and marks
  what happens when, even in clips with no speech. It can also suggest an in and out point and
  name a single marker, or every unnamed marker in one click. You use your own Anthropic API key,
  and Haiku costs about $10 per 1,000 one-minute clips.
- **Get subtitles from speech.** Generate subtitles sends the audio to Soniox, a paid
  speech-to-text service, and saves an `.srt` file, a Premiere Pro transcript, or both next to
  the video.
- **Find any clip later.** Search the list by name, tag or comment, and filter it to untagged clips
  or clips with subtitles, comments or markers.
- **Work on many clips at once.** Batch mode runs one action on all the clips you check, for
  example rotating them, moving comments between the video and text files, or turning comment
  lines into markers.
- **Never lose your work.** `Ctrl+Z` undoes almost everything, and frename keeps a recovery file
  of the open clip a second after each change, so a crash or a power cut does not cost you your
  tags, comments or markers. A line under the file name tells you when your changes are safe.

![The frename window, with labels for each part](docs/frename-screenshot.jpg)

The window has three columns. On the left is the video player with the subtitle and marker lists.
In the middle is the list of files in the folder. On the right are the tag search, your starred
tags, all the folder's tags, the new file name (drag its tags to reorder them) and the clip's
comment.

## How a session goes

1. Open a folder with the **Open a folder** button under the file list, or drag a folder onto the
   window. The installed Windows version also adds "Open in frename" to the right-click menu of
   folders in Explorer. The arrow next to the button (or `Ctrl+R`) lists the last ten folders
   you worked in, each with the clip you had open, and so does the empty start screen.
2. The first clip starts playing.
3. Tag what you see.
4. Press `[` and `]` to mark the usable part, and `F2` at a moment you want to find again. Press
   `F2` again right away to name the marker, then `Enter`.
5. Type a comment for the clip, press `Esc` to leave the comment box, and press `PgDn` to go
   to the next clip. frename renames the clip you left and saves its markers.
6. On Windows, drag finished clips from the list into Premiere Pro. frename saves each clip
   first, so it arrives with its new name and everything you marked. Nothing is moved.

The next time you start frename, it opens the same folder at the same clip, two seconds before
where you stopped watching. The [guide](docs/guide.md) has more ways to open a folder or a
single clip.

## Keyboard shortcuts

### Files
| Key | Action |
|---|---|
| `PgDn` | Next file (renames the file you leave) |
| `PgUp` | Previous file (renames the file you leave) |
| `Ctrl+R` | Recent folders: `↑` `↓` walk the list, `Enter` opens the folder with its last clip, `Esc` closes it |
| Double-click a file | Rename it by hand: `Enter` renames, `Esc` cancels |
| `Ctrl+click` a file | Add it (and the open one) to a selection, or drop it if already selected |
| `Shift+click` a file | Select every file between it and the last `Ctrl`-clicked one |
| Right-click a file or the open file's name | Menu: show it in Explorer, copy its full path or its name (`Esc` or a click outside closes it) |
| `F11` | Show the open file in Explorer, with the file selected (Linux: in the file manager) |
| `Shift+F11` | Copy the open file's full path (`D:\footage\clip.mp4`) |
| `Ctrl+F11` | Copy the open file's name (`clip.mp4`) |
| `Escape` | Leaves a selection made with `Ctrl`/`Shift`+click (batch mode); otherwise clears the file and tag searches |

### Tags
| Key | Action |
|---|---|
| `↑` `↓` `←` `→` | Move the selection in the tag grid |
| `Shift+Space` | Tag or untag the file with the selected tag |
| Any letter, `Backspace` | Type into the tag search |
| `Enter` | Add the typed tag, or the selected unsaved (○) tag, to the folder's tags |
| `Delete` | Delete the selected tag from the folder's tags |
| `Escape` | Clear the tag and file searches, or leave a `Ctrl`/`Shift`+click selection first if one is running (see Files above) |
| `Ctrl+C` | Copy the file's tags (and its new name to the clipboard) |
| `Ctrl+V` | Replace the file's tags with the copied ones |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` or `Ctrl+Shift+Z` | Redo |
| Middle-click a chip in the file name | Untag the file |
| `F6` / `Shift+F6` | Add the tag Describe with AI suggests most, or every one it suggests |

### Video
| Key | Action |
|---|---|
| `Space` | Play / pause (or click the picture) |
| `F1` | Back 10 seconds |
| `F3` | Forward 10 seconds |
| `Alt+←` / `Alt+→` | One frame back / forward (pauses; hold to keep stepping) |
| `Home` | Go to the start of the clip |
| `[` | Set the in point |
| `]` | Set the out point |
| `F2` | Add a marker; `F2` again within a second and a half, or on a marker, names it |
| Hold `F2` | Mark a range: from where you pressed to where you let go |
| `Shift+F2` | Delete the marker under the playhead |
| `Ctrl+F2` | Describe the marker under the playhead with AI |
| `Shift+F1` / `Shift+F3` | Jump to the previous / next marker |
| `Shift` + drag the progress bar | Snap to the nearest marker |
| `Alt` + drag the progress bar (paused) | Mark a range |
| `F12` | Save the current frame as a JPEG next to the video |
| `F5` | Fullscreen on / off (or double-click the picture) |
| `Ctrl+Alt+←` / `Ctrl+Alt+→` | Rotate the clip 90° left / right |
| `Escape` | Leave fullscreen |

### Settings

| Key | Action |
|---|---|
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Next / previous page |
| `Escape` | Close Settings (while you remove or replace a key, Esc cancels that first) |

The [guide](docs/guide.md) explains which keys a text box takes while you type, and everything
else in detail: tags, markers, comments, search, subtitles, batch mode, Describe with AI and
Settings.

## Your files and your data

- Tags live in the file names, and each folder's tag list lives in a `.frename` file inside that
  folder, so both travel with the footage.
- Markers, in and out points and comments are saved inside mp4 and mov files. Settings can keep
  comments in a `.comment.txt` file next to the video instead. Formats such as mkv cannot hold
  them, so for those frename keeps the comment and the in and out points in the text file, and
  markers are not available.
- frename sends your footage nowhere unless you ask. Describe with AI sends frames and
  subtitles to Anthropic, and Generate subtitles sends the audio to Soniox, both with your own API
  key. Your keys are kept in the system's password store. On Windows that is Credential Manager.
- On Windows, frename checks GitHub once a day for a new version. You can turn that off in
  Settings, and nothing is downloaded until you click **Update and restart**. The Linux AppImage
  does not update itself.

## Install

**Windows 10 or 11 (64-bit):** download
[frename-win-Setup.exe](https://github.com/Zelenov/frename/releases/latest/download/frename-win-Setup.exe)
and run it. It installs without questions and starts frename. Video playback is built in, so you
don't need to install anything else. Windows may warn about an unknown publisher the first time.
If it does, click **More info**, then **Run anyway**. frename keeps its settings in
`%LocalAppData%\frename`. Uninstalling removes them, but your tags and comments stay with your
files.

If you prefer not to install, use the Windows zip from the
[latest release](https://github.com/Zelenov/frename/releases/latest). Right-click it, open
**Properties**, tick **Unblock**, unzip it anywhere and run `frename.exe`. The zip keeps its
settings in its own folder and updates itself too.

If you used the zip of version 0.66 or older, the new frename looks for its folder in Downloads,
Desktop and Documents on the first start and offers to import your settings and recent folders.
If the folder was somewhere else, use **Settings → Updates → Import from an old frename
folder…**. After that you can delete the old folder, and uninstall GStreamer if you installed it
only for frename.

**Linux (64-bit, Ubuntu 24.04, Linux Mint 22, Fedora 40, Debian 13 or newer):** download the
`.AppImage` from the [latest release](https://github.com/Zelenov/frename/releases/latest), make it
executable with `chmod +x frename-*.AppImage`, and run it. Video playback is built in, so you
don't need to install anything else. If it says FUSE is missing, run it with
`--appimage-extract-and-run`. frename keeps its settings in `~/.local/share/frename`.

The interface is in English and Russian.

## Building from source

```sh
cargo build
cargo run
```

You need Rust and the GStreamer development files. See [GSTREAMER_SETUP.md](GSTREAMER_SETUP.md).

frename is written in Rust. The interface uses [iced](https://iced.rs), and video plays through
[GStreamer](https://gstreamer.freedesktop.org). The AI descriptions come from
[clipscribe](https://github.com/Zelenov/clipscribe) and the subtitles from
[sonisub](https://github.com/Zelenov/sonisub), two small libraries I wrote for frename. Every
change is tested on Windows and Linux in GitHub Actions, and every release is built and published
from there.
