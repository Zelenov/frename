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
