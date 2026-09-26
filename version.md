# 0.70
## Added
- Markers: `F2` (or 📍) marks a moment of the video, `F2` again names it. Each marker has a name and one of Premiere's colors, and is saved inside the video, where Premiere Pro shows it on the clip after import.
- The marker list (◆, in fullscreen too) shares the side of the picture with the subtitle list, with a tab for each (CC Subtitles, ◆ Markers): click a marker to jump to it or edit it. An empty list has an Add button.
- `Shift+F1` / `Shift+F3` jump to the previous / next marker, `Shift+F2` deletes the marker under the playhead, and dragging the progress bar with `Shift` snaps to markers. Markers show as pins in their colors on the progress bar.
- While the playhead is on a marker, its name becomes the head of its pin above the progress bar, the way a subtitle line shows. Click it to rename the marker.
- The batch report has an Open log button next to the failed files: the log says why each one failed.
- The file list shows 📍 and the number of markers of each file, and the Filter dropdown can show only files with markers.
- Batch action "Markers ⇄ comment" turns comment lines like `03:24 — Take 3` into markers, or copies markers into the comment.

## Changed
- `F12` just saves the current frame as a JPEG next to the video, like VLC: no mark on the progress bar, no time in the comment, and the picture is no longer renamed with the video. Frames saved before stay on disk.
- `F1`, `F3` and `F12` no longer react while `Shift` is held.
- The Filter dropdown moved from the bottom bar into the file search bar.
- In fullscreen the CC button is shown too and opens or closes the subtitle list; before, fullscreen always showed the list.
- A video whose content is not a video (all zeros, as a download that did not finish leaves it) is reported as damaged in the log, not as a format that cannot hold markers.
- Long button labels and file names in the batch panel no longer run past its edge.
- When the video panel is narrow, the progress bar gets a row of its own above the buttons, so it stays long enough to seek.

# 0.69
## Added
- A Windows installer: download `frename-win-Setup.exe` from the release page and run it. It installs without questions, and video plays with nothing else to install.
- Updates: Settings → Updates checks for a new version and installs it with **Update and restart**. frename also checks once a day by itself and puts a dot on ⚙ when an update is ready.
- On first start, the installed frename finds the folder of an older zip version and offers to import your settings and recent folders; Settings → **Import from an old frename folder…** does it for a folder kept elsewhere.

## Changed
- The Windows zip is now a portable frename with video playback built in: unzip and run, no GStreamer to install. It keeps its settings in its own folder and updates itself too.
- The installed frename keeps its settings in `%LocalAppData%\frename`.

# 0.68
## Added
- A setting puts a space after each tag in file names (`Food. Goat. clip.mp4`); names are read the same either way.
- Batch action "Apply tag spacing" renames checked files to the chosen spacing; Settings offer it after the setting changes.

# 0.67
## Added
- A Linux version: one AppImage file from the release page, with video playback built in.

# 0.66
## Added
- Batch mode (☑ in the controls bar): check files in the list (All / Invert) and run an action on all of them, with progress, Cancel and a per-file result — a green or red check box.
- Batch actions: move comments between the video and text files, move in/out points between the file name and the video, tag commented videos, fix tags by priority (tag panel order), and reset cache and reload.
- After switching comment or in/out storage, Settings offer the matching batch action for files already stored the other way.
- A check box turns the "Commented" tag off.

## Changed
- The "Commented" tag is checked when a video gets its first comment and unchecked when the comment is cleared; otherwise it is yours to change, and it is no longer forced back on save. A new one goes last.
- Settings no longer convert the whole folder; batch actions do that.
- Starred tags are filtered by the tag search too.
- The progress bar and subtitle highlight no longer freeze after seeking.

# 0.65
## Added
- While comments are stored inside the video, every video with a comment gets a "Commented" tag in its name, removed again with the comment. The tag's wording is a setting.
- The file list shows the first line of each file's comment under its name.
- Converting a folder shows progress and can be cancelled.

## Changed
- Folders open instantly again with comments stored inside the videos: comments load in the background, with a spinner in the list, and are remembered in the folder's `.frename` file. A file you open always shows its comment right away.
- `.frename` is now TOML, with multi-line comments as plain text. The older JSON files are no longer read.
- The subtitles marker sits on the file name line.

# 0.64
## Added
- Comments can be saved inside the video file (XMP), where Premiere Pro shows them in its Description column and finds them by search. Settings choose this (the default) or a `.comment.txt` next to the video.
- In/out points can be saved inside the video as an Adobe marker, which Premiere Pro turns into a subclip; the file name then carries no in/out. Settings choose this or the file name (the default).
- Settings show which files in the open folder still keep comments or in/out the other way, and convert the whole folder in one click, in either direction.
- Filter the file list to files with subtitles or with a comment. All filters are in one Filter dropdown, with counts.
- Double-click a file in the list to rename it in place: Enter renames, Esc cancels. A name another file already has is refused.

## Changed
- Writing comments or in/out into a video keeps its modified and created dates.
- A video no longer fails to play after it was renamed while open.
- The file list scrollbar no longer covers long names.

# 0.63
## Added
- Subtitles: a `.srt` next to a video is shown under the picture, and over it in fullscreen with a list of every line. Click a line to jump to it; the list follows playback.
- The CC button opens the subtitle list outside fullscreen too.
- The file list marks videos that have subtitles.
- A settings window (gear button): start videos playing when opened, and draw every tag in one neutral color.

## Changed
- The file list shows only videos; subtitles, text files and images are hidden.
- Subtitles are renamed together with their video, undo included.
- Comments are saved as UTF-8 with a BOM, so Windows editors no longer show Cyrillic as garbage.
- The per-file "Paste tags" button is gone; Ctrl+C / Ctrl+V still copy tags.

# 0.62
## Added
- Show only the files that have no tags yet, so a folder can be worked through to the end.
- Each folder now keeps its own tags in a `.frename` text file next to the footage. It travels with the folder and can be edited by hand or by an AI: one tag per line, and the line order is the tag order.
- Search the file list by name, level with the tag search. It matches the name as it is on disk, so the tags already in a name are searchable too, and it narrows the list together with the untagged filter.

## Changed
- Tags belong to the folder instead of being shared by every folder. A folder without a tag file starts from the built-in set.
- iPhone clips recorded at a variable frame rate now play instead of showing an error.
- Tags saved in earlier versions are not carried over: the app database no longer stores tags.

# 0.61
## Changed
- Dragging the panel splitters, moving the window and the volume slider no longer stutter.
- Folders with thousands of files open almost instantly.
- Scrolling and typing stay smooth in large folders.
- A paused video no longer keeps redrawing the window.

# 0.60
## Added
- Initial release. Tag-based video file reviewer for editors: play clips, assign tags, copy tags from previous file, add comments, mark screenshots.
