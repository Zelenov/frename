---
name: image-preview
description: >
  Image preview implementation guide for frename. Use when: adding or changing the
  media_viewer feature, working with FileKind classification, changing how FolderWorkspace
  opens files, adding JPEG or HEIC/HEIF decoding, understanding the no-unload path for
  images, or moving video_player into media_viewer.
disable-model-invocation: false
---

# Image Preview Implementation Guide — frename

## WHEN to use this skill
- Implementing or modifying `src/features/media_viewer/`
- Working with `FileKind` in `frename-core`
- Changing `apply_file_opened` or `on_media_unloaded` in `FolderWorkspace`
- Adding JPEG or HEIC decode logic in `media_viewer/image/`
- Debugging image load failures or the no-unload path

Read `IMAGE_PREVIEW_DESIGN.md` for rationale and decisions.
Read `IMAGE_PREVIEW_TESTS.md` for acceptance criteria (Gherkin).

---

## Module layout

```
crates/frename-core/src/
  file_kind.rs              // FileKind enum
  file.rs                   // + kind() method
  directory.rs              // + skip Other in open()

src/features/
  media_viewer/             // unified feature — FolderWorkspace only knows this
    mod.rs
    state.rs                // MediaViewerState + open() + routing
    messages.rs             // Message enum
    view.rs                 // delegates to video/ or image/ view
    video/                  // VideoPlayerState (moved from features/video_player/)
      mod.rs, state.rs, messages.rs, view.rs
    image/                  // ImageViewerState (new)
      mod.rs, state.rs, messages.rs, view.rs
```

`src/features/video_player/` is deleted; its contents move to `media_viewer/video/`.

---

## Core: FileKind

```rust
// crates/frename-core/src/file_kind.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind { Video, Image, Other }

impl FileKind {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "mp4"|"mkv"|"avi"|"mov"|"wmv"|"m4v"|"flv"|"webm"|"ts"|"m2ts"|"mpg"|"mpeg"|"3gp"
                => Self::Video,
            "jpg"|"jpeg"|"heic"|"heif"|"png"|"webp"|"bmp"
                => Self::Image,
            _   => Self::Other,
        }
    }
    pub fn is_media(&self) -> bool { !matches!(self, Self::Other) }
}
```

`File::kind()`:
```rust
pub fn kind(&self) -> FileKind {
    let ext = self.file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    FileKind::from_extension(ext)
}
```

`Directory::open()` — add after `File::open()`:
```rust
let file = File::open(&path).await?;
if !file.kind().is_media() { continue; }
files.push(file);
```

---

## MediaViewerState

```rust
pub struct MediaViewerState {
    active: ActiveMedia,
    video: VideoPlayerState,
    image: ImageViewerState,
}

#[derive(Default)]
enum ActiveMedia { #[default] None, Video, Image }
```

### Key methods

```rust
// Called by FolderWorkspace — all routing is internal
pub fn open(&mut self, file: &File) -> Task<Message> {
    match file.kind() {
        FileKind::Video => {
            self.active = ActiveMedia::Video;
            self.image.unload();
            self.video.load_video(file.file_path().to_path_buf()).map(Message::Video)
        }
        FileKind::Image => {
            self.active = ActiveMedia::Image;
            self.image.load_image(file.file_path().to_path_buf()).map(Message::Image)
        }
        FileKind::Other => Task::none(),
    }
}

// Used by FolderWorkspace to decide unload-before-rename
pub fn needs_unload_before_rename(&self) -> bool {
    matches!(self.active, ActiveMedia::Video) && self.video.is_active()
}

pub fn subscription(&self) -> Subscription<Message> {
    match self.active {
        ActiveMedia::Video => self.video.subscription().map(Message::Video),
        _ => Subscription::none(),
    }
}
```

### update() — translates VideoUnloaded → Unloaded

```rust
pub fn update(&mut self, msg: Message) -> Task<Message> {
    match msg {
        // Intercept: video pipeline finished teardown → tell FolderWorkspace to proceed
        Message::Video(video::Message::VideoUnloaded) => {
            self.active = ActiveMedia::None;
            Task::done(Message::Unloaded)
        }
        Message::Video(vm) => self.video.update(vm).map(Message::Video),
        Message::Image(im) => self.image.update(im).map(Message::Image),
        Message::Unload => match self.active {
            ActiveMedia::Video => {
                // Async teardown — VideoUnloaded will follow
                self.video.update(video::Message::Unload).map(Message::Video)
            }
            _ => {
                // Images and idle unload synchronously
                self.image.unload();
                self.active = ActiveMedia::None;
                Task::done(Message::Unloaded)
            }
        },
        Message::Unloaded => Task::none(),
    }
}
```

### view()

```rust
pub fn view(state: &MediaViewerState) -> Element<'_, Message> {
    match state.active {
        ActiveMedia::Video => video::view::view(&state.video).map(Message::Video),
        ActiveMedia::Image => image::view::view(&state.image).map(Message::Image),
        ActiveMedia::None  => center_placeholder("🎬", theme::TEXT_MUTED),
    }
}
```

---

## FolderWorkspace changes

### Field
```rust
// Before: video_player: VideoPlayerState,
media_viewer: MediaViewerState,
```

### apply_file_opened — simplified
```rust
fn apply_file_opened(&mut self, file: File) -> Task<Message> {
    let snapshot = self.file_workspace.get_snapshot();
    self.file_workspace.set_file(Some(file.clone()));

    let Some(snapshot) = snapshot else {
        return self.media_viewer.open(&file).map(Message::MediaViewer);
    };

    if self.media_viewer.needs_unload_before_rename() {
        self.pending_file_updated = Some(snapshot);
        Task::done(Message::MediaViewer(media_viewer::Message::Unload))
    } else {
        let (path, snap) = snapshot;
        Task::batch([
            Task::done(Message::FileUpdated { path, snapshot: snap }),
            self.media_viewer.open(&file).map(Message::MediaViewer),
        ])
    }
}
```

No `FileKind` switch here — that's inside `media_viewer.open()`.

### on_media_unloaded (renamed from on_video_unloaded)
```rust
fn on_media_unloaded(&mut self) -> Task<Message> {
    let Some((path, snapshot)) = self.pending_file_updated.take() else { return Task::none(); };
    let Some(file) = self.directory.as_ref().and_then(|d| d.selected_file()).cloned() else {
        return Task::none();
    };
    Task::batch([
        Task::done(Message::FileUpdated { path, snapshot }),
        self.media_viewer.open(&file).map(Message::MediaViewer),
    ])
}
```

### Message arm
```rust
// Before
Message::VideoPlayer(msg) => match msg {
    video_player::Message::VideoUnloaded => self.on_video_unloaded(),
    other => self.video_player.update(other).map(Message::VideoPlayer),
},
// After
Message::MediaViewer(msg) => match msg {
    media_viewer::Message::Unloaded => self.on_media_unloaded(),
    other => self.media_viewer.update(other).map(Message::MediaViewer),
},
```

### subscription
```rust
self.media_viewer.subscription().map(Message::MediaViewer)
// replaces: self.video_player.subscription().map(Message::VideoPlayer)
```

### view
```rust
media_viewer::view::view(&state.media_viewer).map(Message::MediaViewer)
// replaces: video_player::view::view(&state.video_player).map(Message::VideoPlayer)
```

---

## VideoPlayerState — add is_active()

```rust
// In media_viewer/video/state.rs
pub fn is_active(&self) -> bool {
    self.current_video.is_some() || self.loading
}
```

---

## ImageViewerState (media_viewer/image/state.rs)

```rust
pub struct ImageViewerState {
    current_handle: Option<image::Handle>,
    loading: bool,
    load_failed: bool,
}

pub enum Message {
    ImageLoaded(Result<image::Handle, String>),
}
```

### load_image
```rust
pub fn load_image(&mut self, path: PathBuf) -> Task<Message> {
    self.loading = true; self.load_failed = false; self.current_handle = None;
    Task::future(async move {
        let result = tokio::task::spawn_blocking(move || decode_image(&path))
            .await.unwrap_or_else(|e| Err(e.to_string()));
        Message::ImageLoaded(result)
    })
}
```

### decode_image (blocking)
```rust
fn decode_image(path: &Path) -> Result<image::Handle, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "jpg"|"jpeg"|"png"|"webp"|"bmp" => decode_via_image_crate(path),
        "heic"|"heif"                   => decode_heic(path),
        _                               => Err(format!("Unsupported extension: {ext}")),
    }
}

fn decode_via_image_crate(path: &Path) -> Result<image::Handle, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let img   = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let rgba  = img.into_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(image::Handle::from_rgba(w, h, rgba.into_raw()))
}

fn decode_heic(path: &Path) -> Result<image::Handle, String> {
    use libheif_rs::{ColorSpace, LibHeif, RgbChroma};
    let ctx    = LibHeif::new();
    let handle = ctx.read_from_file(path.to_str().ok_or("bad path")?).map_err(|e| e.to_string())?;
    let image  = ctx.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None).map_err(|e| e.to_string())?;
    let interleaved = image.planes().interleaved.ok_or("no interleaved plane")?;
    Ok(image::Handle::from_rgba(image.width(), image.height(), interleaved.data.to_vec()))
}
```

### update
```rust
pub fn update(&mut self, msg: Message) -> Task<Message> {
    match msg {
        Message::ImageLoaded(Ok(handle)) => { self.loading = false; self.current_handle = Some(handle); }
        Message::ImageLoaded(Err(e))     => { self.loading = false; self.load_failed = true; log::error!("Image load failed: {e}"); }
    }
    Task::none()
}

pub fn unload(&mut self) { *self = Self::default(); }
```

### view (image/)
```rust
pub fn view(state: &ImageViewerState) -> Element<'_, Message> {
    if let Some(h) = state.current_handle.as_ref() {
        Image::new(h.clone()).width(Fill).height(Fill).content_fit(ContentFit::Contain).into()
    } else if state.loading   { center_text("⏳", 48, theme::TEXT_MUTED) }
    else if state.load_failed { center_text("✕", 80, theme::ERROR) }
    else                      { center_text("🖼", 48, theme::TEXT_MUTED) }
}
```

---

## Cargo.toml additions

```toml
iced = { version = "0.14.0-dev", features = ["tokio", "image"] }  # add "image" feature
image = "0.25"                                                      # pure-Rust decoder
libheif-rs = { version = "2", features = ["embedded-libheif"] }    # HEIC/HEIF (needs CMake)
```

---

## Core tests (FileKind)

In `crates/frename-core/src/file_kind.rs`:

```rust
#[test] fn jpeg_is_image()   { assert_eq!(FileKind::from_extension("jpg"),  FileKind::Image); }
#[test] fn heic_is_image()   { assert_eq!(FileKind::from_extension("heic"), FileKind::Image); }
#[test] fn heif_is_image()   { assert_eq!(FileKind::from_extension("heif"), FileKind::Image); }
#[test] fn mp4_is_video()    { assert_eq!(FileKind::from_extension("mp4"),  FileKind::Video); }
#[test] fn txt_is_other()    { assert_eq!(FileKind::from_extension("txt"),  FileKind::Other); }
#[test] fn uppercase_jpeg()  { assert_eq!(FileKind::from_extension("JPEG"), FileKind::Image); }
#[test] fn is_media_video()  { assert!(FileKind::Video.is_media()); }
#[test] fn is_media_other()  { assert!(!FileKind::Other.is_media()); }
```
