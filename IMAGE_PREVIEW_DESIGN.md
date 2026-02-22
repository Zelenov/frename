# Image Preview — Design

Design-only document. No implementation begins until explicitly requested.

Test specification (Gherkin): see `IMAGE_PREVIEW_TESTS.md`.

---

## Goal

Extend frename to preview static images (JPEG and HEIC/HEIF) in the left panel alongside the existing video player. The user browses image files exactly as they browse video files: tag them, navigate, rename on switch.

---

## Supported image formats

| Extension(s) | Format | Notes |
|---|---|---|
| `.jpg`, `.jpeg` | JPEG | Most common photo format |
| `.heic`, `.heif` | HEIC / HEIF | iPhone default photo format (High Efficiency Image Container) |
| `.png` | PNG | Bonus — same decoder cost as JPEG |

---

## Why NOT GStreamer for images

The app already uses GStreamer (via `iced_video_player`) for video. A natural question is whether to route images through GStreamer too.

**GStreamer can decode images** — JPEG via `jpegdec` (gst-plugins-good), HEIC via a pipeline like `filesrc ! h265parse ! libde265dec ! videoconvert` (requires `gst-plugins-bad` + `libheif` + `libde265` compiled in). But:

1. **HEIC support is not guaranteed in standard Windows GStreamer installs.** `gst-plugins-bad` is not always bundled, and even when it is, HEIF/libde265 support may not be compiled in. Users would need a custom GStreamer build.
2. **GStreamer holds a file handle on Windows.** This is why the app has the unload-before-rename dance for videos. Using GStreamer for images would force the same expensive unload/reload cycle for static pictures.
3. **Static images do not need a streaming pipeline.** GStreamer is designed for time-based media. A full pipeline for a single still image is engineering overhead with no benefit.

**Decision: Iced's native `Image` widget + `image` crate (JPEG/PNG) + `libheif-rs` (HEIC/HEIF).** Once decoded into memory as `image::Handle`, the file handle is immediately released. The file can be renamed at any time — no unload step needed.

---

## File-kind classification (frename-core)

Add a `FileKind` enum to `frename-core` based on file extension.

```rust
// crates/frename-core/src/file_kind.rs
pub enum FileKind {
    Video,  // .mp4, .mkv, .avi, .mov, .wmv, .m4v, .flv, .webm, .ts, .m2ts, .mpg, .mpeg, .3gp
    Image,  // .jpg, .jpeg, .heic, .heif, .png, .webp, .bmp
    Other,  // everything else — skipped by directory scan
}

impl FileKind {
    pub fn from_extension(ext: &str) -> Self { ... }
    pub fn is_media(&self) -> bool { matches!(self, Self::Video | Self::Image) }
}
```

Add to `File`:
```rust
pub fn kind(&self) -> FileKind {
    let ext = self.file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    FileKind::from_extension(ext)
}
```

`Directory::open()` skips `FileKind::Other` files (`.txt`, `.nfo`, `.srt`, etc.) silently.

---

## Architecture: unified `media_viewer` feature

Rather than exposing both `video_player` and `image_viewer` to `FolderWorkspace` and having `FolderWorkspace` route between them, **a single `media_viewer` feature encapsulates all media display logic**. `FolderWorkspace` only talks to `media_viewer`.

```
FolderWorkspace
  └── media_viewer: MediaViewerState      ← one field, one message arm, one view call
        ├── video: VideoPlayerState       ← internal, not visible to FolderWorkspace
        └── image: ImageViewerState       ← internal, not visible to FolderWorkspace
```

### What `FolderWorkspace` sees

```rust
// Open a file — media_viewer decides internally whether to load video or image
media_viewer.open(&file) -> Task<media_viewer::Message>

// Should we unload before renaming? True only when a video is active
media_viewer.needs_unload_before_rename() -> bool

// Unload message (triggers video pipeline teardown, or immediate image clear)
media_viewer::Message::Unload

// Emitted when media is fully unloaded and rename can proceed
media_viewer::Message::Unloaded
```

`FolderWorkspace` has **no** reference to `VideoPlayerState`, `ImageViewerState`, or `FileKind`. The routing is an internal concern of `media_viewer`.

### Benefit for future media types

Adding audio preview, RAW photo decode, or any other media type only requires changes inside `media_viewer`. `FolderWorkspace` and `apply_file_opened` stay untouched.

---

## `media_viewer` feature

Path: `src/features/media_viewer/`

The existing `src/features/video_player/` becomes an internal submodule of `media_viewer`. `ImageViewerState` is also internal.

### Module layout

```
src/features/media_viewer/
  mod.rs          // pub use MediaViewerState, Message, view
  state.rs        // MediaViewerState + open() + routing
  messages.rs     // pub enum Message { Video(...), Image(...), Unload, Unloaded }
  view.rs         // view(state) → delegates to video or image view
  video/          // VideoPlayerState (moved from src/features/video_player/)
    mod.rs
    state.rs
    messages.rs
    view.rs
  image/          // ImageViewerState (new)
    mod.rs
    state.rs
    messages.rs
    view.rs
```

### `MediaViewerState`

```rust
pub struct MediaViewerState {
    active: ActiveMedia,
    video: VideoPlayerState,
    image: ImageViewerState,
}

enum ActiveMedia { None, Video, Image }
```

### `media_viewer::Message`

```rust
pub enum Message {
    Video(video::Message),    // internal video player messages
    Image(image::Message),    // internal image viewer messages
    Unload,                   // FolderWorkspace sends this to trigger teardown
    Unloaded,                 // MediaViewer emits this when teardown is complete
}
```

`FolderWorkspace` intercepts `Unloaded` before routing to `MediaViewerState::update()` — matching the existing pattern for `VideoUnloaded`:

```rust
// In FolderWorkspace::update()
Message::MediaViewer(msg) => match msg {
    media_viewer::Message::Unloaded => self.on_media_unloaded(),
    other => self.media_viewer.update(other).map(Message::MediaViewer),
},
```

### `MediaViewerState::open()`

Routing lives here, not in `FolderWorkspace`:

```rust
pub fn open(&mut self, file: &File) -> Task<Message> {
    match file.kind() {
        FileKind::Video => {
            self.active = ActiveMedia::Video;
            self.image.unload();
            self.video.load_video(file.file_path().to_path_buf()).map(Message::Video)
        }
        FileKind::Image => {
            self.active = ActiveMedia::Image;
            // video is already unloaded (caller ensured this if needed)
            self.image.load_image(file.file_path().to_path_buf()).map(Message::Image)
        }
        FileKind::Other => Task::none(),
    }
}
```

### `MediaViewerState::update()` — translates `VideoUnloaded` → `Unloaded`

```rust
pub fn update(&mut self, msg: Message) -> Task<Message> {
    match msg {
        Message::Video(video::Message::VideoUnloaded) => {
            self.active = ActiveMedia::None;
            Task::done(Message::Unloaded)   // FolderWorkspace intercepts this
        }
        Message::Video(vm) => self.video.update(vm).map(Message::Video),
        Message::Image(im) => self.image.update(im).map(Message::Image),
        Message::Unload => match self.active {
            ActiveMedia::Video => {
                // GStreamer pipeline teardown — async, emits VideoUnloaded when done
                self.video.update(video::Message::Unload).map(Message::Video)
            }
            ActiveMedia::Image | ActiveMedia::None => {
                self.image.unload();
                self.active = ActiveMedia::None;
                Task::done(Message::Unloaded)   // images unload synchronously
            }
        },
        Message::Unloaded => Task::none(),
    }
}
```

### `needs_unload_before_rename()`

```rust
pub fn needs_unload_before_rename(&self) -> bool {
    matches!(self.active, ActiveMedia::Video) && self.video.is_active()
}
```

`is_active()` on `VideoPlayerState`: `self.current_video.is_some() || self.loading`.

### `subscription()`

```rust
pub fn subscription(&self) -> Subscription<Message> {
    match self.active {
        ActiveMedia::Video => self.video.subscription().map(Message::Video),
        _ => Subscription::none(),   // images and idle state need no subscription
    }
}
```

Video controls keyboard shortcuts (Space, F1–F3) are automatically suppressed when an image is displayed.

### `view()`

```rust
pub fn view(state: &MediaViewerState) -> Element<'_, Message> {
    match state.active() {
        ActiveMedia::Video => video::view::view(&state.video).map(Message::Video),
        ActiveMedia::Image => image::view::view(&state.image).map(Message::Image),
        ActiveMedia::None  => empty_placeholder(),  // 🎬 icon, same as today
    }
}
```

---

## `FolderWorkspace` changes

### Struct field change

```rust
// Before
video_player: VideoPlayerState,

// After
media_viewer: MediaViewerState,
```

### `apply_file_opened` — simplified

```rust
fn apply_file_opened(&mut self, file: File) -> Task<Message> {
    let snapshot = self.file_workspace.get_snapshot();
    self.file_workspace.set_file(Some(file.clone()));

    let Some(snapshot) = snapshot else {
        // First open — no previous file to rename
        return self.media_viewer.open(&file).map(Message::MediaViewer);
    };

    if self.media_viewer.needs_unload_before_rename() {
        // Video is active — must unload GStreamer before rename (file handle on Windows)
        self.pending_file_updated = Some(snapshot);
        Task::done(Message::MediaViewer(media_viewer::Message::Unload))
    } else {
        // Image was active (or nothing) — no file handle, rename immediately
        let (path, snap) = snapshot;
        Task::batch([
            Task::done(Message::FileUpdated { path, snapshot: snap }),
            self.media_viewer.open(&file).map(Message::MediaViewer),
        ])
    }
}
```

`FolderWorkspace` no longer contains any `FileKind` switch, any `video_player.is_active()` call, or any reference to `image_viewer`. The entire "what to load" decision is hidden behind `media_viewer.open()`.

### `on_media_unloaded` (renamed from `on_video_unloaded`)

```rust
fn on_media_unloaded(&mut self) -> Task<Message> {
    let Some((path, snapshot)) = self.pending_file_updated.take() else {
        return Task::none();
    };
    let Some(file) = self.directory.as_ref().and_then(|d| d.selected_file()).cloned() else {
        return Task::none();
    };
    Task::batch([
        Task::done(Message::FileUpdated { path, snapshot }),
        self.media_viewer.open(&file).map(Message::MediaViewer),
    ])
}
```

Same logic as today's `on_video_unloaded`, but now calls `media_viewer.open()` instead of `video_player.load_video()`. Works correctly regardless of whether the new file is a video or image.

### `subscription()` — simplified

```rust
pub fn subscription(&self) -> Subscription<Message> {
    Subscription::batch([
        self.media_viewer.subscription().map(Message::MediaViewer),
        self.file_name_panel.subscription().map(Message::FileNamePanel),
    ])
}
```

Replaces `self.video_player.subscription().map(Message::VideoPlayer)`.

### Message arm change

```rust
// Before (two arms)
Message::VideoPlayer(msg) => match msg {
    video_player::Message::VideoUnloaded => self.on_video_unloaded(),
    other => self.video_player.update(other).map(Message::VideoPlayer),
},

// After (one arm; Unloaded interception moves inside MediaViewerState)
Message::MediaViewer(msg) => match msg {
    media_viewer::Message::Unloaded => self.on_media_unloaded(),
    other => self.media_viewer.update(other).map(Message::MediaViewer),
},
```

### View — left panel

```rust
// Before
video_player::view::view(video_state).map(Message::VideoPlayer)

// After
media_viewer::view::view(media_state).map(Message::MediaViewer)
```

No routing logic in the workspace view at all.

---

## `ImageViewerState` (internal to `media_viewer/image/`)

```rust
pub struct ImageViewerState {
    current_handle: Option<image::Handle>,
    loading: bool,
    load_failed: bool,
}
```

`load_image(path) -> Task<image::Message>` spawns a blocking `decode_image` task:

1. **JPEG / PNG**: `std::fs::read` → `image::load_from_memory` → `.into_rgba8()` → `image::Handle::from_rgba(w, h, pixels)`.
2. **HEIC / HEIF**: `libheif_rs::LibHeif::new()` → `read_from_file` → `decode` (RGBA) → `image::Handle::from_rgba(w, h, pixels)`.

File handle is released after decode. `unload()` is synchronous (drops the `Handle`).

---

## Dependencies

### `Cargo.toml` (binary)

```toml
# Enable image widget support in Iced
iced = { version = "0.14.0-dev", features = ["tokio", "image"] }

# Pure-Rust JPEG / PNG / WebP / BMP decoding
image = "0.25"

# HEIC / HEIF decoding — statically links libheif (requires CMake at build time)
libheif-rs = { version = "2", features = ["embedded-libheif"] }
```

`embedded-libheif` compiles libheif from bundled C sources at build time. No runtime DLL for users. Requires CMake in `PATH` during build.

---

## File layout

```
crates/frename-core/src/
  file_kind.rs                        // FileKind enum + from_extension()
  file.rs                             // + kind() method
  directory.rs                        // + skip Other in open()
  lib.rs                              // + pub use FileKind

src/features/
  media_viewer/                       // ← new unified feature (replaces video_player at top level)
    mod.rs                            // pub use MediaViewerState, Message, view
    state.rs                          // MediaViewerState, ActiveMedia, open(), update()
    messages.rs                       // pub enum Message
    view.rs                           // view() delegates to video/ or image/ view
    video/                            // VideoPlayerState (moved from features/video_player/)
      mod.rs
      state.rs
      messages.rs
      view.rs
    image/                            // ImageViewerState (new)
      mod.rs
      state.rs
      messages.rs
      view.rs

src/features/folder_workspace/
  state.rs                            // media_viewer field; simplified apply_file_opened; on_media_unloaded
  messages.rs                         // MediaViewer(media_viewer::Message) replaces VideoPlayer(...)
```

`src/features/video_player/` is deleted — its contents move into `media_viewer/video/`.

---

## Binary-side changes summary

| File | Change |
|---|---|
| `crates/frename-core/src/file_kind.rs` | New — `FileKind` enum |
| `crates/frename-core/src/file.rs` | Add `kind() -> FileKind` |
| `crates/frename-core/src/directory.rs` | Skip `Other` in `open()` |
| `crates/frename-core/src/lib.rs` | `pub mod file_kind; pub use file_kind::FileKind` |
| `src/features/media_viewer/` | New unified feature |
| `src/features/media_viewer/video/` | VideoPlayerState moved here from `features/video_player/` |
| `src/features/media_viewer/image/` | ImageViewerState — new |
| `src/features/video_player/` | **Deleted** — replaced by `media_viewer/video/` |
| `src/features/folder_workspace/state.rs` | `media_viewer` field; `apply_file_opened` simplified; `on_media_unloaded` |
| `src/features/folder_workspace/messages.rs` | `MediaViewer(media_viewer::Message)` replaces `VideoPlayer(...)` |
| `src/features/folder_workspace/view.rs` | Single `media_viewer::view::view(...)` call |
| `src/features/folder_workspace/state.rs` (subscription) | `media_viewer.subscription()` replaces `video_player.subscription()` |
| `Cargo.toml` | Add `image` feature to iced; add `image` crate; add `libheif-rs` |

---

## Out of scope

- EXIF orientation auto-rotation — future enhancement.
- Thumbnail caching — decode on demand only in v1.
- RAW camera formats (`.cr2`, `.nef`, `.arw`) — no pure-Rust decoder available.
- Animated HEIF / Live Photos — first frame only.
- Image zoom / pan — fit-to-panel only (same as video player today).

---

After implementation: record the `libheif-rs` `embedded-libheif` decision and CMake requirement in `docs/DECISIONS.md`.
