# Image Preview — Gherkin Test Cases

Test specification for the image preview feature described in `IMAGE_PREVIEW_DESIGN.md`.

---

## Feature: FileKind classification (frename-core)

```gherkin
Feature: FileKind classifies files by media type based on extension

  Scenario Outline: Video extensions are classified as Video
    When FileKind::from_extension("<ext>") is called
    Then the result is FileKind::Video

    Examples:
      | ext  |
      | mp4  |
      | mkv  |
      | avi  |
      | mov  |
      | wmv  |
      | m4v  |
      | flv  |
      | webm |
      | ts   |
      | mpg  |

  Scenario Outline: Image extensions are classified as Image
    When FileKind::from_extension("<ext>") is called
    Then the result is FileKind::Image

    Examples:
      | ext  |
      | jpg  |
      | jpeg |
      | heic |
      | heif |
      | png  |

  Scenario Outline: Non-media extensions are classified as Other
    When FileKind::from_extension("<ext>") is called
    Then the result is FileKind::Other

    Examples:
      | ext  |
      | txt  |
      | nfo  |
      | srt  |
      | pdf  |

  Scenario: Extension matching is case-insensitive
    When FileKind::from_extension("JPEG") is called
    Then the result is FileKind::Image
    When FileKind::from_extension("MP4") is called
    Then the result is FileKind::Video

  Scenario: File::kind() returns the correct kind based on its path extension
    Given a File at path "holiday.summer.vacation.jpg"
    When file.kind() is called
    Then the result is FileKind::Image

  Scenario: is_media() returns true for Video and Image, false for Other
    Then FileKind::Video.is_media() is true
      And FileKind::Image.is_media() is true
      And FileKind::Other.is_media() is false
```

---

## Feature: Directory scanning filters out non-media files

```gherkin
Feature: Directory::open() skips FileKind::Other files

  Scenario: Only video and image files appear in the file list
    Given a folder contains:
      | file name       | kind    |
      | vacation.mp4    | Video   |
      | portrait.jpg    | Image   |
      | notes.txt       | Other   |
      | subtitles.srt   | Other   |
      | photo.heic      | Image   |
    When the folder is scanned via Directory::open()
    Then the file list contains exactly:
      | vacation.mp4  |
      | portrait.jpg  |
      | photo.heic    |
    And "notes.txt" is not in the file list
    And "subtitles.srt" is not in the file list

  Scenario: A folder with only non-media files loads as empty
    Given a folder contains only ".nfo" and ".txt" files
    When the folder is scanned
    Then the file list is empty
    And no file is selected
```

---

## Feature: Opening an image file — image viewer loads, video player stays idle

```gherkin
Feature: Image files are routed to the image viewer, not the video player

  Background:
    Given a folder is open containing:
      | index | name          | kind  |
      | 0     | clip.mp4      | Video |
      | 1     | portrait.jpg  | Image |
      | 2     | photo.heic    | Image |

  Scenario: Selecting a JPEG file shows the image viewer, not the video player
    Given file at index 0 (video) is currently shown
    When the user navigates to file at index 1 (portrait.jpg)
    Then the image viewer loads "portrait.jpg"
      And the image viewer shows the decoded JPEG
      And the video player is idle (no video loaded)
      And the video controls are hidden

  Scenario: Selecting a HEIC file shows the image viewer
    When the user navigates to file at index 2 (photo.heic)
    Then the image viewer loads "photo.heic"
      And the image viewer shows the decoded HEIC image
      And the video player is idle

  Scenario: Image shows loading placeholder while decoding
    When the user navigates to an image file
      And decoding has not yet completed
    Then the image viewer shows a loading placeholder (e.g. ⏳)
      And the left panel is not empty

  Scenario: Image shows error placeholder when decoding fails
    Given "corrupt.jpg" is a file with invalid JPEG data
    When the user navigates to "corrupt.jpg"
      And decoding completes with an error
    Then the image viewer shows an error placeholder (e.g. ✕)
      And no crash occurs

  Scenario: Empty placeholder is shown when no file is selected
    Given no file is selected
    Then the image viewer shows the empty placeholder (e.g. 🖼)
```

---

## Feature: No video-unload delay when renaming image files

```gherkin
Feature: Image files do not require video unload before rename

  Background:
    Given a folder contains:
      | index | name          | kind  |
      | 0     | portrait.jpg  | Image |
      | 1     | clip.mp4      | Video |

  Scenario: Navigating from one image to another renames immediately — no unload step
    Given file at index 0 (portrait.jpg) is open
      And the "Holiday" tag is checked
      And the file name preview shows "holiday.portrait.jpg"
    When the user navigates to file at index 1 (clip.mp4)
    Then "portrait.jpg" is renamed to "holiday.portrait.jpg" on disk immediately
      # No VideoPlayer(Unload) message is emitted
      And the video player loads "clip.mp4"

  Scenario: Navigating from an image to a video does NOT trigger media unload
    Given file at index 0 (portrait.jpg) is open (image viewer active, video player idle)
      And media_viewer.needs_unload_before_rename() returns false
    When the user navigates to file at index 1 (clip.mp4)
    Then Message::MediaViewer(Unload) is NOT emitted
      And Message::FileUpdated is emitted directly
      And media_viewer.open(clip.mp4) is called, which internally loads the video

  Scenario: Navigating from a video to an image still triggers media unload (GStreamer file handle)
    Given file at index 1 (clip.mp4) is open and playing
      And media_viewer.needs_unload_before_rename() returns true
      And the "Action" tag is checked
    When the user navigates back to file at index 0 (portrait.jpg)
    Then Message::MediaViewer(Unload) IS emitted
      And MediaViewerState internally issues VideoPlayer(Unload)
      And after the video unloads, MediaViewer emits Message::Unloaded
      And FolderWorkspace calls on_media_unloaded: renames "clip.mp4" → "action.clip.mp4"
      And media_viewer.open(portrait.jpg) is called, which internally loads the image
```

---

## Feature: JPEG image decoding

```gherkin
Feature: JPEG images are decoded correctly by the image viewer

  Scenario: A valid JPEG file is decoded and displayed
    Given a valid JPEG file "landscape.jpg" with dimensions 1920×1080
    When the image viewer loads "landscape.jpg"
    Then the decoded image has width 1920 and height 1080
      And the image handle contains RGBA pixel data
      And the image is displayed in the left panel at fit-to-panel size

  Scenario: A JPEG with no tags in the file name is parsed correctly
    Given a file "photo.jpg" (no tags in the name)
    When the file is opened
    Then the tag panel shows no checked tags
      And the file name preview shows "photo.jpg"
      And the image viewer shows the decoded photo

  Scenario: After JPEG is loaded into memory, the file handle is released
    Given the image viewer has loaded "portrait.jpg" into memory
    Then the file "portrait.jpg" can be renamed on disk without error
      # No GStreamer file handle is held
```

---

## Feature: HEIC / HEIF image decoding

```gherkin
Feature: HEIC and HEIF images are decoded correctly by the image viewer

  Scenario: A valid HEIC file is decoded and displayed
    Given a valid HEIC file "iphone_photo.heic"
    When the image viewer loads "iphone_photo.heic"
    Then the image viewer shows the decoded image (no crash, no error placeholder)
      And the left panel displays the image content

  Scenario: A valid HEIF file (.heif extension) is decoded and displayed
    Given a valid HEIF file "capture.heif"
    When the image viewer loads "capture.heif"
    Then the image viewer shows the decoded image

  Scenario: A corrupt HEIC file shows the error placeholder gracefully
    Given a file "broken.heic" with invalid HEIF data
    When the image viewer tries to load "broken.heic"
    Then the image viewer shows the error placeholder (✕)
      And no crash occurs
      And an error is logged

  Scenario: HEIC file can be renamed immediately after loading into memory
    Given "iphone_photo.heic" has been decoded and displayed
      And the "Vacation" tag is checked
    When the user navigates to the next file
    Then "iphone_photo.heic" is renamed to "vacation.iphone_photo.heic" on disk
      And no file-handle error occurs
```

---

## Feature: Mixed folder — video and image files coexist

```gherkin
Feature: A folder with both video and image files navigates correctly

  Background:
    Given a folder contains:
      | index | name             | kind  |
      | 0     | clip.mp4         | Video |
      | 1     | portrait.jpg     | Image |
      | 2     | holiday.heic     | Image |
      | 3     | action_scene.mkv | Video |

  Scenario: Navigating through a mixed folder switches between viewer types
    Given file at index 0 is selected (clip.mp4 plays via media_viewer internally)
    When the user navigates to index 1 (portrait.jpg)
    Then media_viewer receives Unload (video was active), then Unloaded fires
      And "clip.mp4" is renamed (if tags changed) by on_media_unloaded
      And media_viewer.open(portrait.jpg) loads the JPEG internally
    When the user navigates to index 2 (holiday.heic)
    Then needs_unload_before_rename() is false (image was active, no file handle)
      And "portrait.jpg" is renamed immediately
      And media_viewer.open(holiday.heic) loads the HEIC internally
    When the user navigates to index 3 (action_scene.mkv)
    Then needs_unload_before_rename() is false (image was active)
      And "holiday.heic" is renamed immediately
      And media_viewer.open(action_scene.mkv) loads the video internally

  Scenario: Tag panel and file name panel work identically for image and video files
    Given "portrait.jpg" is open in the image viewer
    When the user checks the "Wedding" and "2024" tags
    Then the file name panel shows chips: ["Wedding", "2024"]
      And the file name preview shows "wedding.2024.portrait.jpg"
    When the user navigates to the next file
    Then "portrait.jpg" is renamed to "wedding.2024.portrait.jpg" on disk

  Scenario: PageUp / PageDown navigation works across image and video files
    Given file at index 1 (portrait.jpg) is selected
    When the user presses PageDown
    Then file at index 2 (holiday.heic) is selected
      And the image viewer shows "holiday.heic"
    When the user presses PageDown
    Then file at index 3 (action_scene.mkv) is selected
      And the video player loads "action_scene.mkv"
    When the user presses PageUp
    Then file at index 2 (holiday.heic) is selected again
      And the image viewer shows "holiday.heic"

  Scenario: Drag-and-drop opening an image file works
    Given no folder is open
    When the user drops "portrait.jpg" onto the window
    Then the parent folder is scanned
      And "portrait.jpg" is selected
      And the image viewer shows the JPEG
```

---

## Feature: Image aspect ratio and layout

```gherkin
Feature: Images are displayed with correct aspect ratio in the left panel

  Scenario: A landscape image fills the panel width without cropping
    Given a JPEG with dimensions 1920×1080 (16:9 landscape)
    When the image viewer shows it in a 460×400 panel
    Then the image is scaled to fit within 460×400
      And the image is not cropped
      And the image is centred with letterboxing if needed (ContentFit::Contain behaviour)

  Scenario: A portrait image fills the panel height without cropping
    Given a HEIC with dimensions 3024×4032 (3:4 portrait)
    When the image viewer shows it in a 460×800 panel
    Then the image is scaled to fit within 460×800
      And the image is not cropped
```

---

## Feature: Session persistence with image files

```gherkin
Feature: Last opened image file is restored on next app launch

  Scenario: Opening an image file persists the session correctly
    Given the user opens a folder and selects "portrait.jpg"
    When the app is closed and re-opened
    Then the same folder is opened
      And "portrait.jpg" is selected
      And the image viewer shows "portrait.jpg"

  Scenario: A folder with mixed files restores the last selected file
    Given the user was on "holiday.heic" (index 2) when they closed the app
    When the app reopens
    Then "holiday.heic" is selected and shown in the image viewer
```
