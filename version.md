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
