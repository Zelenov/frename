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
settings-tags-monochrome-hint = Every tag chip in one gray.
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

batch-title = Batch actions
batch-on-checked = on { $count ->
    [one] { $count } file
   *[other] { $count } files
} checked
batch-run = Run on { $count ->
    [one] { $count } file
   *[other] { $count } files
}
batch-done-label-changed = changed
batch-done-label-subtitled = subtitled
batch-counts = ✓ { $done } { $done_label }   – { $skipped } unchanged   ✗ { $failed } failed
batch-ai-at-least = at least
batch-ai-spend-line = AI: { $spend }
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
batch-failed-subtitles = Not subtitled:
batch-failed-plain = Failed:
batch-failed-with-log = Failed (the log says why):
batch-retry = Retry
batch-add-credit = Add credit
batch-open-log = Open log

batch-action-move-comments = Move comments
batch-action-move-comments-hint = Moves the comment of each checked file to the chosen place, with the in/out points it holds. Tags and in/out points kept in the video stay where they are.
batch-action-move-comments-into-videos = From text files into the videos (XMP)
batch-action-move-comments-into-text-files = From the videos (XMP) into text files

batch-action-move-in-out = In/out points: comment ⇄ video (XMP)
batch-action-move-in-out-hint = Moves the in/out points of each checked file to the chosen place. The rest of the comment stays where it is.
batch-action-move-in-out-into-videos = From the comments into the videos (Adobe XMP marker)
batch-action-move-in-out-into-comments = From the videos (XMP marker) into the comments

batch-action-in-out-from-names = Move in/out points out of file names
batch-action-in-out-from-names-hint = Older versions could keep in/out points in the file name (clip.in_00_01_05.mp4). This takes them out of the name of each checked file and saves them where in/out points are kept now. A file that already has in/out points stored keeps those; the report lists it.
batch-action-in-out-from-names-status-comment = Goes to: the comment
batch-action-in-out-from-names-status-video = Goes to: inside the video (XMP); for mkv, webm and other formats that cannot hold it, the comment
batch-action-in-out-from-names-settings = In/out settings…
batch-action-in-out-from-names-kept = kept the stored { $stored }, dropped the name's { $name }
batch-action-in-out-from-names-not-renamed = could not be renamed (the log says why)
batch-action-in-out-from-names-empty = left as it is: the name would be empty
batch-action-in-out-from-names-taken = left as it is: { $name } already exists
batch-in-out-from-names-listed = Files that failed or kept their stored in/out points:

batch-action-markers-comment = Markers ⇄ comment
batch-action-markers-comment-to-markers = Comment lines with a time into markers
batch-action-markers-to-comment = Markers into the comment (a copy: the markers stay)
batch-action-markers-comment-hint = A line like "03:24 — Take 3 — nice light" is a marker at 3:24 named "Take 3" with the comment "nice light"; "0:41-0:47 — Lion" is a marker from 0:41 to 0:47. The name and the comment are split at the first " — " or " -- ", not at a plain " - ". The moments of an AI description ("0:00–0:14 Street.") become white markers and stay in the description. Running either way again adds nothing twice.

batch-action-tag-commented = Tag commented videos
batch-action-tag-commented-hint = Adds the "{ $tag }" tag to each checked video with a comment of yours (AI descriptions do not count) and removes it from those without one. Files whose tag changes are renamed.
batch-action-tag-commented-hint-off = Adds the tag for videos with a comment to each checked video with a comment of yours and removes it from those without one. The tag is turned off in the settings.
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

batch-action-describe-ai = Describe with AI
batch-action-describe-ai-run = Describe { $videos }
batch-action-describe-ai-estimating = Estimating… { $known } / { $total }
batch-action-describe-ai-none = No videos to describe.
batch-action-describe-ai-plan = { $videos }, { $minutes } · about { $dollars } with { $model }
batch-action-describe-ai-hint = Takes about { $duration }. The folder is locked until it ends. Cancel keeps the videos already described; running it again skips them.
batch-action-describe-ai-no-subtitles = Without subtitles (only the picture is described): { $videos }.
batch-action-describe-ai-redo = Redo videos that already have an AI description
batch-action-describe-ai-hint-panel = Describes what happens in each checked video, and when: a summary and time-ranged segments go into the AI description of its comment; your own text is kept. Frames and subtitles are sent to Anthropic.
batch-ai-change = Change
batch-ai-open-settings = Open Settings
batch-ai-key-missing = Set an Anthropic API key in Settings
batch-ai-key-unavailable = The system keyring could not be opened: it may be locked, or there is none (such as GNOME Keyring or KWallet).
batch-ai-language-same-as-subtitles = Descriptions in the subtitles' language (English if none)
batch-ai-language = Descriptions in { $language }

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

## Controls bar under the file list

folder-controls-filter = Filter
folder-controls-filter-active = Filter ({ $count })
folder-controls-filter-untagged = Untagged
folder-controls-filter-subtitles = Subtitles
folder-controls-filter-comments = Comments
folder-controls-filter-markers = Markers
folder-controls-scroll = Scroll to file
folder-controls-open = Open folder (right-click: open a file)
folder-controls-batch = Batch actions on checked files
folder-controls-batch-back = Back to the open file
folder-controls-update-available = Update available: { $version }

## Video

video-controls-set-in = [  Set In
video-controls-set-out = ]  Set Out
video-controls-screenshot = Save this frame as a JPEG (F12)
video-controls-add-marker = Add marker (F2, hold for a range; again to name it)
video-controls-cannot-hold-markers = This file cannot hold markers
video-controls-add-a-name = Add a name
video-controls-rotate-left = Rotate left (Ctrl+Alt+←)
video-controls-rotate-right = Rotate right (Ctrl+Alt+→)
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
media-viewer-video-show-subtitles = Show subtitle list
media-viewer-video-hide-subtitles = Hide subtitle list
media-viewer-video-markers-hint = Markers (Shift+F1 / Shift+F3 to jump, Shift+drag to snap)
media-viewer-video-tab-markers = Markers

## Markers list

markers-empty = No markers yet
markers-add = 📍 Add a marker (F2)
markers-ai-hint = AI marker: replaced when the AI describes this clip again
markers-keep-color = Keep the color
markers-done-enter = Done (Enter)
markers-delete = Delete the marker
markers-read-only = read-only
markers-name-placeholder = Name

## File workspace

file-workspace-comment-placeholder = Comment...
file-workspace-comment-collapse = Back to the tags
file-workspace-comment-expand = Expand the comment

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
