# frename UI text, the source language. Every other file under i18n/ has exactly these ids,
# attributes and $arguments (checked by the tests in src/i18n.rs).
# Keys: <feature>-<element>[-<detail>], the feature being the folder under src/features/.

## Language names: the same in every file, each written in its own language.

language-name-en = English
language-name-ru = Русский

## Settings window

settings-window-title = Settings
settings-page-interface = Interface
settings-page-saving = Saving
settings-page-ai = Describe with AI
settings-close = Close
settings-apply-note = Changes apply right away. Ctrl+Tab moves between pages.
settings-language = Language
settings-language-system = System ({ $language })
settings-video = Video
settings-video-autoplay = Play videos automatically when opened
settings-tags = Tag colors
settings-tags-monochrome = Monochrome
settings-tags-monochrome-hint = Tags you added show in a darker gray, others in a lighter one — no other colors.
settings-file-names = File names
settings-tags-space-after = Space after each tag
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = Files keep their names until renamed or saved.
settings-tags-space-add = Add the space to file names…
settings-tags-space-remove = Remove the space from file names…
settings-comments = Comments
settings-comments-in-video = Inside the video file
settings-comments-in-video-hint = XMP, the Description column in Premiere Pro
settings-comments-text-file = In a text file next to the video
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = Files keep their comments where they are until moved.
settings-comments-move-into-videos = Move comments into the videos…
settings-comments-move-into-text-files = Move comments into text files…
settings-commented-tag = Tag videos with a comment
settings-commented-tag-name = Tag
settings-commented-tag-hint = Checked when you write a comment on a video, e.g. { $tag }.IMG_0424.MOV, and unchecked when you clear it (AI descriptions do not count). Otherwise it is yours to change.
settings-commented-tag-off = Videos with a comment get no tag.
settings-in-out = In/out points
settings-in-out-in-video = Inside the video file
settings-in-out-in-video-hint = A subclip marker in Premiere Pro
settings-in-out-comment = In the comment, as one line
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = Files keep their in/out points where they are until moved.
settings-in-out-move-into-videos = Move in/out points into the videos…
settings-in-out-move-into-comments = Move in/out points into the comments…
settings-markers = Markers and ranges
settings-markers-in-video = Inside the video file
settings-markers-in-video-hint = XMP, shown on the clip in Premiere Pro
settings-markers-comment = In the comment, one line each
settings-markers-comment-example = 0:41–0:47 — Lion
settings-markers-note = Files keep their markers where they are until moved.
settings-markers-move-into-videos = Move markers into the videos…
settings-markers-copy-into-comment = Copy markers into the comments…
settings-markers-hint = Points and ranges alike, the moments AI finds too.
settings-updates = Updates
settings-version = Version
settings-old-title = Settings from an older frename
settings-old-hint = Bring the settings of a frename you ran from a zip folder.
settings-old-scheduled = Settings will be imported when frename restarts
settings-old-not-found = No frename.exe with a frename.db in { $folder }
settings-old-failed = Could not import: { $reason }
settings-old-import-button = Import from an old frename folder…
settings-ai = Describe with AI
settings-ai-key-label = Anthropic API key
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Get a key at console.anthropic.com → API keys.
settings-ai-key-remove-confirm = Remove the saved Anthropic key?
settings-ai-used-by = Used by Describe with AI in batch mode.
settings-ai-model-label = Model
settings-ai-language-label = Description language
settings-ai-hint = Haiku is the cheapest; Sonnet and Opus notice more.
settings-subtitles = Subtitles
settings-subtitles-key-label = Soniox API key
settings-subtitles-key-placeholder = Paste the key
settings-subtitles-key-get = Get a key at console.soniox.com. The audio is sent to Soniox.
settings-subtitles-key-remove-confirm = Remove the saved Soniox key?
settings-subtitles-languages-label = Languages
settings-subtitles-languages-locked = Save the key to choose from all the languages Soniox knows.
settings-subtitles-languages-loading = Getting the languages from Soniox…
settings-subtitles-languages-none = None checked: detected automatically.
settings-subtitles-languages-hint = The languages spoken in the footage, as hints.
settings-subtitles-cue-length-label = Cue length
settings-subtitles-cue-short = Short
settings-subtitles-cue-short-hint = One line, up to 8 s
settings-subtitles-cue-sentence = One sentence
settings-subtitles-hint = Used by Generate subtitles in batch mode. The cue length applies to new subtitles.
settings-key-store-windows = Windows Credential Manager
settings-key-store-macos = the macOS Keychain
settings-key-store-other = the system keyring
settings-key-unavailable = The system keyring could not be opened
settings-key-unavailable-hint = It may be locked, or there is none (such as GNOME Keyring or KWallet). Settings checks again each time it opens.
settings-key-remove-confirm-hint = You will need to paste it again.
settings-key-remove-ask = Remove…
settings-key-remove = Remove key
settings-key-keep = Keep
settings-key-replace = Replace…
settings-key-hide = Hide
settings-key-show = Show
settings-key-save = Save key
settings-key-cancel = Cancel
settings-key-saved-in = Saved in { $store } on this computer
settings-key-save-into = Save key keeps it in { $store } on this computer.

## Batch mode

batch-back-while-running = Cancel the action first
batch-checked-count = { $count ->
    [one] { $count } file checked
   *[other] { $count } files checked
}
batch-no-key = No key
batch-group-move = Move between places
batch-group-fix = Fix names and videos
batch-group-paid = Paid services
batch-reason-none-checked = No files checked
batch-reason-reading = Reading clip lengths…
batch-check-all = Check all { $count ->
    [one] { $count } file
   *[other] { $count } files
}
batch-job-running = running
batch-job-finished = finished
batch-job-stopped = stopped
batch-progress-files = { $finished } of { $total ->
    [one] { $total } file
   *[other] { $total } files
}
batch-time-left = about { $time } left
batch-time-estimating = estimating time…
batch-time-spent = { $time } so far
batch-locked-until-end = The folder is locked until it ends
batch-run-again = Run again on { $count ->
    [one] { $count } file
   *[other] { $count } files
}
batch-result-done = Done: { $total ->
    [one] { $total } file
   *[other] { $total } files
}
batch-result-done-detail = { $changed } changed, { $unchanged } had nothing to change
batch-result-problems = Done with problems: { $failed } of { $total ->
    [one] { $total } file
   *[other] { $total } files
} not done
batch-figure-unchanged = unchanged
batch-figure-not-done = not done
batch-figure-not-reached = not reached
batch-files-not-done = Files not done
batch-table-file = File
batch-table-why = Why

batch-run-move-comments = Move { $count ->
    [one] { $count } comment
   *[other] { $count } comments
}
batch-run-move-in-out = Move in/out of { $count ->
    [one] { $count } file
   *[other] { $count } files
}
batch-run-convert = Convert { $count ->
    [one] { $count } file
   *[other] { $count } files
}
batch-run-rotate = Rotate { $count ->
    [one] { $count } video
   *[other] { $count } videos
}
batch-run-tag = Tag { $count ->
    [one] { $count } file
   *[other] { $count } files
}
batch-run-fix-tags = Reorder tags in { $count ->
    [one] { $count } name
   *[other] { $count } names
}
batch-run-rename = Rename { $count ->
    [one] { $count } file
   *[other] { $count } files
}
batch-run-reload = Reload { $count ->
    [one] { $count } file
   *[other] { $count } files
}
batch-service-anthropic = Anthropic
batch-service-soniox = Soniox
batch-change-renames = Renames files
batch-change-videos = Writes into the videos
batch-change-text-files = Writes text files next to the videos
batch-change-comments = Writes into the comments
batch-change-subtitles = Writes subtitle files next to the videos
batch-change-records = Changes only frename's own records
batch-option-direction = Direction
batch-option-language = Language
batch-option-described = Described videos
batch-option-subtitled = Subtitled videos
batch-option-spacing = Spacing
batch-option-tag = Tag
batch-option-turn = Turn
batch-plan-videos = Videos
batch-plan-length = Length
batch-plan-cost = Cost
batch-plan-time = Time
batch-reason-estimate = Waiting for the estimate
batch-set-key = Set the key…
batch-check-key = Check the key…
batch-subtitles-languages-auto = Detected in each video
batch-action-describe-ai-run-waiting = Describe videos
batch-action-fix-tags-order = Order: as in the tag list, unknown tags first.
batch-action-markers-comment-hint-short = Turns comment lines with a time into markers, or copies the markers into the comment.
batch-action-markers-to-comment-hint = The markers stay; running it again adds nothing twice.
batch-action-rotate-hint-short = Turns MP4 and MOV clips by their rotation flag, without re-encoding.
batch-action-tag-commented-off = Tagging commented videos is off
batch-action-tag-commented-choose = Choose the tag…

batch-title = Batch actions
batch-done-label-changed = changed
batch-done-label-subtitled = subtitled
batch-ai-at-least = at least
batch-cancel = Cancel
batch-stopping = Stopping…
batch-stopped = Stopped after { $finished } of { $total ->
    [one] { $total } file
   *[other] { $total } files
}.
batch-close = Close
batch-failed-subtitles = Not subtitled:
batch-add-credit = Add credit
batch-open-log = Open the log

batch-action-move-comments = Move comments
batch-action-move-comments-hint = Moves the comment of each checked file to the chosen place, with the in/out points it holds. Tags and in/out points kept in the video stay where they are.
batch-action-move-comments-into-videos = From text files into the videos (XMP)
batch-action-move-comments-into-text-files = From the videos (XMP) into text files

batch-action-move-in-out = In/out points: comment ⇄ video (XMP)
batch-action-move-in-out-hint = Moves the in/out points of each checked file to the chosen place. The rest of the comment stays where it is.
batch-action-move-in-out-into-videos = From the comments into the videos (Adobe XMP marker)
batch-action-move-in-out-into-comments = From the videos (XMP marker) into the comments

batch-action-markers-comment = Markers ⇄ comment
batch-action-markers-comment-to-markers = Comment lines with a time into markers
batch-action-markers-to-comment = Markers into the comment (a copy: the markers stay)
batch-action-markers-comment-hint = A line like "03:24 — Take 3 — nice light" is a marker at 3:24 named "Take 3" with the comment "nice light"; "0:41-0:47 — Lion" is a marker from 0:41 to 0:47. The name and the comment are split at the first " — " or " -- ", not at a plain " - ". The moments of an AI description ("0:00–0:14 Street.") become white markers and stay in the description. Running either way again adds nothing twice.

batch-action-tag-commented = Tag commented videos
batch-action-tag-commented-hint = Adds the "{ $tag }" tag to each checked video with a comment of yours (AI descriptions do not count) and removes it from those without one. Files whose tag changes are renamed.
batch-action-tag-commented-hint-off = Adds the tag for videos with a comment to each checked video with a comment of yours and removes it from those without one. The tag is turned off in the settings.
batch-action-fix-tags = Fix tags by priority
batch-action-fix-tags-hint = Puts the tags in the name of each checked file in the order of the tag panel, so the higher a tag is there, the earlier it comes in the name. Tags the folder does not know yet come first, as in the tag panel. Files whose order changes are renamed.

batch-action-respace-tags = Apply tag spacing
batch-action-respace-tags-hint-space = Renames each checked file to put a space after each tag, as set in the settings: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Renames each checked file to have no space after its tags, as set in the settings: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Spacing: a space after each tag
batch-action-respace-tags-status-no-space = Spacing: no space after tags
batch-action-reload-files = Reset cache and reload
batch-action-reload-files-hint = Reads the comment and in/out points of each checked file from the file itself again and replaces what the folder remembered for it. Use it after the files were changed in another program. Files whose remembered values were missing or out of date count as changed.

batch-action-describe-ai = Describe with AI
batch-action-describe-ai-run = Describe { $videos } · about { $dollars }
batch-action-describe-ai-estimating = Estimating… { $known } / { $total }
batch-action-describe-ai-none = No videos to describe.
batch-action-describe-ai-hint = The folder is locked until it ends. Cancel keeps the videos already described; running it again skips them.
batch-action-describe-ai-no-subtitles = Without subtitles (only the picture is described): { $videos }.
batch-action-describe-ai-redo = Redo videos that already have an AI description
batch-action-describe-ai-hint-panel = Describes what happens in each checked video, and when: a summary and time-ranged segments go into the AI description of its comment; your own text is kept. Frames and subtitles are sent to Anthropic.
batch-ai-change = Change
batch-ai-key-missing = Set an Anthropic API key in Settings
batch-ai-key-unavailable = The system keyring could not be opened: it may be locked, or there is none (such as GNOME Keyring or KWallet).
## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Same as subtitles
ai-language-english = English
ai-language-russian = Russian
ai-language-ukrainian = Ukrainian
ai-language-german = German
ai-language-spanish = Spanish
ai-language-french = French
batch-videos-count = { $n ->
    [one] { $n } video
   *[other] { $n } videos
}
batch-minutes = { $n } min
batch-ai-minutes-under = < 1 min
batch-ai-duration-under-minute = under a minute
batch-ai-duration-hours = { $h } h
batch-ai-duration-hours-minutes = { $h } h { $m } min
batch-ai-dollars-under = under $0.01
batch-ai-skip-described = already described
batch-ai-skip-too-long = over 30 min
batch-ai-skip-unreadable = unreadable
batch-ai-skip-photos = { $n ->
    [one] { $n } photo
   *[other] { $n } photos
}
batch-ai-skipped = Skipped: { $parts }.
batch-ai-progress-frame = frame { $done } of { $total }
batch-ai-progress-waiting = waiting for Claude
batch-ai-progress-saving = saving
batch-ai-stop-no-key = Stopped: no Anthropic API key. Set one in Settings.
batch-ai-fail-no-key = No API key
batch-ai-fail-unreadable = Video could not be read
batch-ai-fail-not-saved = The description could not be saved
batch-ai-stop-offline = Stopped: no connection to Anthropic. Run it again to describe the rest.

batch-action-generate-subtitles = Generate subtitles
batch-action-generate-subtitles-install-ffmpeg = to read .mkv, .m2ts, .avi …, install ffmpeg from ffmpeg.org, add it to PATH, restart frename
batch-action-generate-subtitles-replace = Replace existing subtitles
batch-action-generate-subtitles-replace-hint = Transcribes again; costs as shown.
batch-action-generate-subtitles-privacy = The audio of these videos is sent to Soniox and deleted there afterwards.
batch-action-generate-subtitles-duration-hint = Takes a few minutes per hour of audio; the folder is locked until it ends. Closing frename stops the run; finished subtitles are kept.
batch-action-generate-subtitles-hint = Transcribes the speech of each checked video with Soniox and saves the subtitles next to it (clip.srt), where frename shows them.
batch-subtitles-transcribe = Transcribe
batch-subtitles-transcribe-count = Transcribe { $videos }
batch-subtitles-build-free = Build { $count } (free)
batch-subtitles-nothing = Nothing to transcribe
batch-subtitles-estimating = Estimating…
batch-subtitles-key-missing = Set a Soniox API key in Settings
batch-subtitles-key-rejected = Soniox rejected the key
batch-subtitles-cost-unknown = cost unknown
batch-subtitles-typical-price = { $usd } (typical price)
batch-subtitles-unknown-length = + { $n } of unknown length
batch-subtitles-plan-line = { $videos } to transcribe, { $duration } of audio{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } already has subtitles
   *[other] { $n } already have subtitles
}
batch-subtitles-rebuilt-free = { $n } rebuilt free from a saved transcript
batch-subtitles-no-speech-before = { $n } had no speech last time
batch-subtitles-no-audio = { $n ->
    [one] { $n } has no audio
   *[other] { $n } have no audio
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } shares its subtitle name with another video
   *[other] { $n } share their subtitle name with another video
}
batch-subtitles-unreadable = { $n } cannot be read ({ $how_to })
batch-subtitles-shared-name-reason = shares its subtitle name with another video
batch-subtitles-unsupported-reason = audio format not supported
batch-subtitles-progress-transcribing = transcribing
batch-subtitles-reason-already = already had subtitles
batch-subtitles-reason-no-audio = no audio
batch-subtitles-reason-no-speech = no speech
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = cannot reach Soniox
batch-subtitles-stop-key-rejected = Stopped: Soniox rejected the key. Check it in Settings → Subtitles.
batch-subtitles-stop-balance-empty = Stopped: the Soniox balance is empty. Top it up at console.soniox.com.
batch-subtitles-stop-budget-used = Stopped: the Soniox monthly budget is used up. Raise it at console.soniox.com.
batch-subtitles-stop-other = Stopped: Soniox refused to go on ({ $error_type }).
batch-subtitles-stop-unreachable = Stopped: Soniox cannot be reached. Check the internet connection.
batch-subtitles-report-spend = Soniox: at least { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } upload
   *[other] { $n } uploads
} could not be deleted on Soniox (see the log)
batch-subtitles-languages-need-key = Save a Soniox key to choose from all its languages.
batch-subtitles-languages-rejected = Soniox rejected the key.
batch-subtitles-languages-failed = Could not get the languages from Soniox.
batch-subtitles-count = { $n ->
    [one] { $n } subtitle
   *[other] { $n } subtitles
}
batch-subtitles-duration-seconds = { $s } s
batch-subtitles-usd-under = less than $0.01
batch-subtitles-usd-about = about ${ $amount }

## File list
folder-has-subtitles = Has subtitles
folder-filter-tip = Show only…

folder-search-placeholder = Find a file
folder-search-clear = Clear
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
folder-markers-not-saved = Markers not saved: the file is read-only or in use (close it in Premiere, then open the file and leave it again)
drag-out-not-saved = Not dragged: can't save the file (read-only, or open in Premiere)
folder-opening = Opening the folder…
folder-empty-title = No videos in this folder
folder-empty-line = frename shows MP4, MOV, MKV and other video files.
folder-open-another = Open another folder…
folder-no-match = No files match
folder-show-all = Show all
folder-locked = Locked while { $action } runs
folder-checked-hidden = { $count } checked · { $hidden } hidden
folder-checked-hidden-tip = { $hidden ->
    [one] 1 checked file is hidden by the search or a filter; the action runs on it too
   *[other] { $hidden } checked files are hidden by the search or a filter; the action runs on them too
}
folder-outcome-working = Working on it
folder-outcome-not-reached = Not reached
folder-window-empty-title = Open a folder of clips
folder-window-empty-line = Or drop a folder on the window. A right-click on the button opens one file.
folder-window-open = Open a folder…

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Show in Explorer
file-menu-show-in-file-manager = Show in file manager
file-menu-copy-path = Copy full path
file-menu-copy-name = Copy file name
file-menu-copied = Copied
file-menu-not-copied = Not copied: the clipboard is not available
file-menu-not-shown = Could not open the file manager

## Controls bar under the file list

folder-controls-filter-untagged = Untagged
folder-controls-filter-subtitles = Subtitles
folder-controls-filter-comments = Comments
folder-controls-filter-markers = Markers
folder-controls-scroll = Show the open file in the list
folder-controls-open = Open a folder
folder-controls-batch = Batch actions on checked files
folder-controls-batch-back = Back to the open file
folder-controls-update-available = Update available: { $version }
folder-controls-previous = Previous file
folder-controls-next = Next file
folder-controls-open-file = Right-click: open one file

## Video

video-controls-back = Back 10 s
video-controls-play = Play
video-controls-pause = Pause
video-controls-forward = Forward 10 s
video-controls-set-in = Set the in point
video-controls-set-out = Set the out point
video-controls-screenshot = Save this frame
video-controls-add-marker = Add a marker
video-controls-add-marker-hold = Hold for a range; press again to name it
video-controls-cannot-hold-markers = This file cannot hold markers
video-controls-add-a-name = Add a name
video-controls-rotate-left = Rotate left
video-controls-rotate-right = Rotate right
video-controls-more = More
media-viewer-no-clip = No clip open
media-viewer-loading-slow = Waiting for the file… (a cloud file may take a while)
media-viewer-cannot-play = This clip cannot be played
media-viewer-no-picture = This file has no video picture
video-controls-volume-scroll = Volume — scroll to change
rotate-cannot = Cannot rotate: { $reason }
rotate-reason-missing = the file is no longer there
rotate-flag-right = 90° right
rotate-flag-left = 90° left
rotate-flag-half = 180°
rotate-flag-none = none
rotate-now = Rotation: { $flag }
rotate-turned = Turned { $turn } · rotation now { $flag }
rotate-failed = Not rotated: { $reason }
rotate-reason-in-use = the file is read-only or in use
rotate-reason-format = this format has no rotation flag
rotate-reason-damaged = the file is damaged
rotate-reason-no-video = the file has no video track
rotate-reason-matrix = the video has an unusual display matrix
batch-action-rotate = Rotate videos
batch-action-rotate-hint = Changes the rotation flag of each checked MP4 and MOV file; the picture is not re-encoded, and comments, in/out points and markers stay. Premiere Pro is expected to show the clip turned when it imports it. Other formats have no rotation flag and fail. Running it again turns the files again; “Reset” removes any turn, including one a phone recorded, so a phone's portrait clip then plays on its side.
batch-action-rotate-right = 90° right (clockwise)
batch-action-rotate-left = 90° left (counter-clockwise)
batch-action-rotate-half = 180°
batch-action-rotate-reset = Reset: no rotation (0°)
media-viewer-video-subtitle-list = Subtitle list
media-viewer-video-marker-list = Marker list
media-viewer-video-markers-hint = Shift+F1 / Shift+F3 jump between markers; Shift+drag snaps
media-viewer-video-tab-markers = Markers
media-viewer-video-fullscreen = Full screen
media-viewer-video-close-list = Close the list

## Markers list
markers-in-out = In/out points

markers-empty = No markers yet
markers-add = Add a marker
markers-ai-hint = AI marker: replaced when the AI describes this clip again
markers-keep-color = Keep the color
markers-done = Done
markers-delete = Delete the marker
markers-cannot-hold-hint = Premiere reads markers from MP4 and MOV files.
markers-color-green = Green
markers-color-red = Red
markers-color-orange = Orange
markers-color-yellow = Yellow
markers-color-white = White
markers-color-blue = Blue
markers-color-cyan = Cyan
markers-color-lavender = Lavender
markers-color-magenta = Magenta
markers-color-other = Another color
markers-read-only = read-only
markers-name-placeholder = Name

## File workspace

file-workspace-search-placeholder = Find a tag — or just type
file-workspace-search-clear = Clear
file-workspace-comment-placeholder = Comment...
file-workspace-comment-collapse = Back to the tags
file-workspace-comment-expand = Expand the comment

## Tag grid

tag-grid-star = Star: keep it at the top
tag-grid-unstar = Unstar
tag-grid-save = Add to the folder's tags
tag-grid-delete = Delete “{ $tag }” from the folder's tags
tag-grid-create = Create “{ $tag }”
tag-grid-no-file = Open a clip to tag it
tag-grid-no-tags = No tags yet
tag-grid-no-tags-hint = Type a name and press Enter to create the first one.
tag-grid-group-unsaved = Not in the folder's tags
tag-grid-group-folder = Folder tags
tag-grid-more = +{ $count } more

## Order strip

sync-panel-locked = Reordering below reorders the folder
sync-panel-unlocked = Reordering below changes this clip only
sync-panel-unlock = Unlock
sync-panel-lock = Lock
sync-panel-differs = Order differs from the folder
sync-panel-use-for-folder = Use for the folder
sync-panel-sort-like-folder = Sort like the folder
sync-panel-no-undo = This cannot be undone yet

## File name card

file-name-panel-no-tags = No tags on this clip
file-name-panel-untag = Untag

## Updates (Settings)

updates-check = Check for updates
updates-not-installed = Updates work in the installed version
updates-checking = Checking…
updates-downloading-named = Downloading { $version }… { $percent }%
updates-downloading = Downloading… { $percent }%
updates-restarting = Restarting…
updates-check-failed = Could not check for updates: { $reason }
updates-update-failed = Could not update: { $reason }
updates-version-available = Version { $version } is available
updates-up-to-date = frename is up to date
updates-update-and-restart = Update and restart
updates-wait-for-batch = Wait for the batch to finish
updates-current-version = frename { $version }
updates-check-on-start = Check for updates when frename starts
