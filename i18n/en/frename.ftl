# frename UI text, the source language. Every other file under i18n/ has exactly these ids,
# attributes and $arguments (checked by the tests in src/i18n.rs).
# Keys: <feature>-<element>[-<detail>], the feature being the folder under src/features/.

## Language names: the same in every file, each written in its own language.

language-name-en = English
language-name-ru = Русский

## Settings window

settings-window-title = Settings
settings-language = Language
settings-language-system = System ({ $language })
settings-video = Video
settings-video-autoplay = Play videos automatically when opened
settings-tags = Tags
settings-tags-monochrome = Monochrome tags
settings-tags-space-after = Space after each tag in file names (Food. Goat. clip.mp4)
settings-tags-space-note = Files keep their names until renamed or saved.
settings-tags-space-add = Add the space to existing file names…
settings-tags-space-remove = Remove the space from existing file names…
settings-comments = Comments
settings-comments-in-video = Inside the video file (XMP, Premiere Pro's Description column)
settings-comments-text-file = In a .comment.txt file next to the video
settings-comments-note = Files keep their comments where they are until moved.
settings-comments-move-into-videos = Move existing comments from text files into the videos…
settings-comments-move-into-text-files = Move existing comments from the videos into text files…
settings-commented-tag = Tag videos with a comment
settings-commented-tag-hint = Checked when a video gets a comment, e.g. { $tag }.IMG_0424.MOV, and unchecked when the comment is cleared. Otherwise it is yours to change.
settings-commented-tag-off = Videos with a comment get no tag.
settings-in-out = In/out points
settings-in-out-in-video = Adobe: a marker inside the video file (XMP, a subclip in Premiere Pro)
settings-in-out-file-name = In the file name (in_HH_MM_SS / out_HH_MM_SS)
settings-in-out-note = Files keep their in/out points where they are until moved.
settings-in-out-move-into-videos = Move existing in/out points from file names into the videos…
settings-in-out-move-into-file-names = Move existing in/out points from the videos into file names…

## Batch mode

batch-title = Batch actions
batch-on-checked = on { $count ->
    [one] { $count } file
   *[other] { $count } files
} checked
batch-back = Back to the open file
batch-run = Run on { $count ->
    [one] { $count } file
   *[other] { $count } files
}
batch-counts = ✓ { $done } changed   – { $skipped } unchanged   ✗ { $failed } failed
batch-cancel = Cancel
batch-stopping = Stopping…
batch-stopped = Stopped after { $finished } of { $total ->
    [one] { $total } file
   *[other] { $total } files
}.
batch-finished = Finished { $total ->
    [one] { $total } file
   *[other] { $total } files
}.
batch-close = Close
batch-failed = Failed (see the log for why):

batch-action-move-comments = Move comments
batch-action-move-comments-hint = Moves the comment of each checked file to the chosen place. Tags and in/out points stay where they are.
batch-action-move-comments-into-videos = From text files into the videos (XMP)
batch-action-move-comments-into-text-files = From the videos (XMP) into text files

batch-action-move-in-out = Move in/out points
batch-action-move-in-out-hint = Moves the in/out points of each checked file to the chosen place, renaming the files whose name gains or loses them. Comments stay where they are.
batch-action-move-in-out-into-videos = From file names into the videos (Adobe XMP marker)
batch-action-move-in-out-into-file-names = From the videos (XMP marker) into file names

batch-action-tag-commented = Tag commented videos
batch-action-tag-commented-hint = Adds the “{ $tag }” tag to each checked video that has a comment and removes it from those without one. Files whose tag changes are renamed.
batch-action-tag-commented-hint-off = Adds the tag for videos with a comment to each checked video that has one and removes it from those without one. The tag is turned off in the settings.
batch-action-tag-commented-status = Tag: { $tag }
batch-action-tag-commented-status-off = Tag: off
batch-action-tag-commented-settings = Tag settings…

batch-action-fix-tags = Fix tags by priority
batch-action-fix-tags-hint = Puts the tags in the name of each checked file in the order of the tag panel, so the higher a tag is there, the earlier it comes in the name. Tags the folder does not know yet come first, as in the tag panel. Files whose order changes are renamed.

batch-action-respace-tags = Apply tag spacing
batch-action-respace-tags-hint-space = Renames each checked file to put a space after each tag, as set in the settings: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Renames each checked file to have no space after its tags, as set in the settings: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Spacing: a space after each tag
batch-action-respace-tags-status-no-space = Spacing: no space after tags
batch-action-respace-tags-settings = Spacing settings…

batch-action-reload-files = Reset cache and reload
batch-action-reload-files-hint = Reads the comment and in/out points of each checked file from the file itself again and replaces what the folder remembered for it. Use it after the files were changed in another program. Files whose remembered values were missing or out of date count as changed.

## File list

folder-all = All
folder-invert = Invert
folder-checked = { $count } checked
folder-outcome-changed = Changed
folder-outcome-unchanged = Nothing to change
folder-outcome-failed = Failed, see the log
folder-rename-error-empty = Name is empty
folder-rename-error-bad-character = Not allowed: \ / : * ? " < > |
folder-rename-error-trailing = Cannot end with a dot or space
folder-rename-error-exists = A file with this name exists

## Controls bar under the file list

folder-controls-filter = Filter
folder-controls-filter-active = Filter ({ $count })
folder-controls-filter-untagged = Untagged
folder-controls-filter-subtitles = Subtitles
folder-controls-filter-comments = Comments
folder-controls-scroll = Scroll to file
folder-controls-open = Open file
folder-controls-settings = Settings
folder-controls-batch = Batch actions on checked files
folder-controls-batch-back = Back to the open file

## Video

video-controls-set-in = [  Set In
video-controls-set-out = ]  Set Out
media-viewer-video-show-subtitles = Show subtitle list
media-viewer-video-hide-subtitles = Hide subtitle list
file-workspace-comment-placeholder = Comment...
