---
name: iced-elm-architecture
description: Iced GUI framework Elm architecture patterns for Rust. Feature-based code organization, message flow, Task::done dispatch, core/UI separation, snapshots, deferred side effects, subscriptions, component independence. Persistence and app state: database initialization only in main.rs; transient store (no singleton, no factory); Directory generic over store; store only in constructor; call store.get_last_session from workspace; on folder load failure keep previous state. Use when building iced UI features, adding messages, creating views, wiring components, organizing iced code, or implementing persistence/session.
---

# Iced Elm Architecture Patterns

## Feature-Based Structure

Every feature lives in its own folder under `src/features/`:

```
src/features/feature_name/
├── mod.rs        # Re-exports: pub use messages::Message; pub use state::FeatureState;
├── messages.rs   # All messages this feature handles
├── state.rs      # State struct + update + subscription methods
└── view.rs       # UI rendering function
```

Register in `src/features/mod.rs`:
```rust
pub mod feature_name;
```

## File Patterns

### messages.rs

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // User actions
    TogglePlayPause,
    // State changes from parent or sibling
    VideoReady { duration_secs: f32 },
    SetPlaying(bool),
    // Child feature messages
    Controls(child_feature::Message),
}
```

Rules:
- Group related state changes into **aggregate messages** (e.g., `VideoReady { duration_secs }` instead of separate `SetDuration`, `SetPlaying`, `SetPosition`)
- Child feature messages wrapped in a variant: `Controls(child::Message)`

### state.rs

```rust
pub struct FeatureState {
    // Private fields - never expose directly
    field: SomeType,
    // Child feature state
    child: ChildState,
}

impl FeatureState {
    // update returns Task<Message> for dispatching follow-up messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SomeAction => {
                // Handle own state, dispatch follow-up via Task::done
                self.field = new_value;
                Task::done(Message::FollowUp)
            }
            Message::Controls(ctrl_msg) => {
                // Forward to child - this is the ONLY place child.update is called
                self.child.update(&ctrl_msg);
                // If child action needs parent response, dispatch via Task::done
                match ctrl_msg {
                    child::Message::SomeAction => Task::done(Message::ParentResponse),
                    _ => Task::none(),
                }
            }
        }
    }

    // Subscription - returns feature-specific subscriptions
    pub fn subscription(&self) -> Subscription<Message> {
        // Gate subscriptions by state - return none when feature is inactive
        if self.is_active() {
            Subscription::batch([
                // Own subscriptions (timers, etc.)
                time::every(Duration::from_millis(250)).map(|_| Message::Tick),
                // Child subscriptions wrapped with .map()
                self.child.subscription().map(Message::Controls),
            ])
        } else {
            Subscription::none()
        }
    }

    // Expose state through getters, not public fields
    pub fn field(&self) -> &SomeType { &self.field }
    pub fn child(&self) -> &ChildState { &self.child }
}
```

### view.rs

```rust
use super::{Message, FeatureState};

pub fn view(state: &FeatureState) -> Element<'_, Message> {
    // Compose child views with .map() for message translation
    // Pass live data as parameters - child reads it at view time
    let child_view = child::view::view(state.child(), live_data).map(Message::Controls);

    column![own_content, child_view]
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}
```

### mod.rs

```rust
mod state;
pub mod view;

pub use state::FeatureState;
// Omit messages if the feature has no message enum (e.g. only exposes data).
```

A feature can omit `messages.rs` and a `Message` enum if it only exposes data (e.g. `get_snapshot()`) and never emits to the parent; the parent creates its own messages from that data.

## Critical Rules

### 0. Never Duplicate Code

Reuse the function you already have. Do not introduce a second function that does the same thing as an existing one, and do not repeat the same logic in multiple places.

- If you need to "persist state" in several code paths, have **one** function that performs the persist (e.g. `write_session`), and call it from every path (e.g. `open()` and `persist_session()` both call that one function). Do not call the low-level writer (e.g. `save_last_session`) from multiple places with different argument construction; centralize construction and the single call in one helper.
- Before adding a new helper or "convenience" function, check whether existing code already does the same thing. If it does, call that. If the existing function has the wrong shape, refactor it once and reuse it everywhere.
- Duplication leads to drift and bugs when one path is updated and the other is not.

### 1. Never Call update Directly Across Components

Use `Task::done()` to dispatch messages through the iced message loop.

```rust
// WRONG - direct cross-component call
Message::VideoReady { duration_secs } => {
    self.controls.update(&controls::Message::SetDuration(duration_secs));
    Task::none()
}

// CORRECT - dispatch through message loop
Message::VideoReady { duration_secs } => {
    Task::done(Message::Controls(controls::Message::VideoReady { duration_secs }))
}
```

The ONLY acceptable direct `update` call is parent forwarding to its own child when the child's message arrives:
```rust
Message::Controls(ctrl_msg) => {
    self.controls.update(&ctrl_msg);  // OK - child handling its own message
    // ...
}
```

### 2. Each Component Owns Its Own State

Components change their own state through their own messages. Never mutate a child's fields from the parent.

```rust
// WRONG
self.controls.is_playing = true;
self.controls.position = 0.0;

// CORRECT - send a message, let controls handle it
Task::done(Message::Controls(controls::Message::VideoReady { duration_secs }))
```

### 3. Separate Concerns Into Messages

Each message should represent one coherent action. When a message causes a follow-up, return `Task::done()`.

```rust
Message::VideoLoaded(success) => {
    self.loading = false;
    if success {
        // ... load video ...
        // Don't update controls here - dispatch a follow-up
        return Task::done(Message::VideoReady { duration_secs });
    }
    Task::none()
}
Message::VideoReady { duration_secs } => {
    // This message notifies child controls
    Task::done(Message::Controls(controls::Message::VideoReady { duration_secs }))
}
```

### 4. update Returns Task<Message>

Always return `Task<Message>` from `update`. Use:
- `Task::none()` - no follow-up
- `Task::done(Message::X)` - dispatch a follow-up message through the loop
- `Task::batch([...])` - dispatch multiple follow-ups

### 5. Flatten With Early Returns

Use `let ... else { return }` guard clauses instead of nested `if let` / `match`. Keep nesting shallow.

```rust
// WRONG - deep nesting
Message::VideoLoaded(success) => {
    if success {
        if let Some(path) = &self.video_path {
            match url::Url::from_file_path(path) {
                Ok(url) => match Video::new(&url) {
                    Ok(video) => { /* finally the logic */ }
                    Err(e) => { log::error!("..."); }
                },
                Err(()) => { log::error!("..."); }
            }
        }
    }
    Task::none()
}

// CORRECT - flat with early returns
Message::VideoLoaded(success) => {
    let Some(path) = self.video_path.as_ref().filter(|_| success) else {
        return Task::none();
    };
    let Ok(url) = url::Url::from_file_path(path) else {
        log::error!("Failed to create URL from path: {}", path.display());
        return Task::none();
    };
    let Ok(video) = Video::new(&url) else {
        log::error!("Failed to reload video from: {url}");
        return Task::none();
    };
    let duration_secs = video.duration().as_secs_f32();
    self.current_video = Some(video);
    Task::done(Message::VideoReady { duration_secs })
}
```

### 6. Read High-Frequency Data at View Time

Don't pipe high-frequency data (e.g., playback position) through messages. Instead, read it directly from the source in `view()` and pass as a parameter to child views.

```rust
// WRONG - pushing position through messages on every frame
Message::NewFrame => {
    let pos = video.position().as_secs_f32();
    Task::done(Message::Controls(controls::Message::UpdatePosition(pos)))
    // Creates 2 message cycles per frame → layout invalidation warnings
}

// CORRECT - read position at view time, pass to child view
pub fn view(state: &VideoPlayerState) -> Element<'_, Message> {
    if let Some(video) = state.current_video() {
        let position_secs = video.position().as_secs_f32();
        let controls = controls::view::view(state.controls(), position_secs)
            .map(Message::Controls);
        // ...
    }
}
```

The child view receives live data as a parameter and uses it directly (or falls back to local state during user interaction like seeking):
```rust
pub fn view(state: &ControlsState, position_secs: f32) -> Element<'_, Message> {
    let current_pos = if state.is_seeking() {
        state.seek_position_secs()  // user is dragging - show drag position
    } else {
        position_secs               // normal playback - show live position
    };
    // ...
}
```

### 7. Use Time Subscriptions Instead of Per-Frame Callbacks

Never use `on_new_frame` or similar per-frame widget callbacks to drive UI updates. They fire at video framerate (30-60fps), causing "consecutive RedrawRequested layout invalidation" warnings.

Instead, use `iced::time::every()` at a controlled rate. The underlying widget (e.g., VideoPlayer) renders at full framerate internally. The subscription just triggers periodic `view()` refreshes to pick up fresh data.

```rust
// WRONG - fires 30-60x/sec, causes layout invalidation warnings
VideoPlayer::new(video)
    .on_new_frame(Message::NewFrame)  // don't do this for progress bar updates

// CORRECT - tick at controlled rate via subscription
pub fn subscription(&self) -> Subscription<Message> {
    if self.current_video.is_some() {
        time::every(Duration::from_millis(250)).map(|_| Message::NewFrame)
    } else {
        Subscription::none()
    }
}
// NewFrame handler is a no-op - just triggers view() refresh
Message::NewFrame => Task::none(),
```

**Requires** `tokio` feature for iced: `iced = { version = "...", features = ["tokio"] }`

### 8. Subscriptions Belong to Features

Each feature owns its subscriptions. Features return `Subscription::none()` when inactive. Parent features batch child subscriptions with `.map()`. The app batches all feature subscriptions. `main.rs` just delegates.

```
Subscription flow:

  ControlsState::subscription()         → Subscription<controls::Message>
       ↓ .map(Message::Controls)
  VideoPlayerState::subscription()      → Subscription<video_player::Message>
       ↓ .map(Message::VideoPlayer)
  DragDropState::subscription()         → Subscription<drag_drop::Message>
       ↓ .map(Message::DragDrop)
  FrenameApp::subscription()            → Subscription<app::Message>
       ↓
  main.rs: .subscription(FrenameApp::subscription)
```

Feature subscription pattern:
```rust
// Feature gates its subscription by state
pub fn subscription(&self) -> Subscription<Message> {
    if self.is_active() {
        Subscription::batch([
            time::every(Duration::from_millis(250)).map(|_| Message::Tick),
            self.child.subscription().map(Message::Controls),
        ])
    } else {
        Subscription::none()
    }
}
```

App batches all feature subscriptions:
```rust
pub fn subscription(&self) -> Subscription<Message> {
    Subscription::batch([
        self.drag_drop_state.subscription().map(Message::DragDrop),
        self.video_player_state.subscription().map(Message::VideoPlayer),
    ])
}
```

`main.rs` stays clean:
```rust
.subscription(FrenameApp::subscription)
```

### 9. Custom Widgets Use Range + Value API

Custom widgets (like progress bars) should accept a range and value, not a pre-computed fraction. The widget handles conversion internally. This mirrors iced's built-in `Slider` API.

```rust
// WRONG - caller computes fraction, widget works with 0.0..1.0
let progress = position / duration;
ProgressBar::new(progress, |fraction| Message::Seek(fraction * duration))

// CORRECT - widget accepts range + value, handles math internally
ProgressBar::new(0.0..=duration, position, Message::Seek)
```

## app.rs Pattern

Root app delegates to features. Minimal coordination logic.

```rust
pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::FeatureA(msg) => {
            self.feature_a.update(msg).map(Message::FeatureA)
        }
        Message::FeatureB(msg) => {
            self.feature_b.update(msg).map(Message::FeatureB)
        }
    }
}

pub fn view(&self) -> Element<'_, Message> {
    feature::view::view(&self.feature_state).map(Message::Feature)
}

pub fn subscription(&self) -> Subscription<Message> {
    Subscription::batch([
        self.feature_a.subscription().map(Message::FeatureA),
        self.feature_b.subscription().map(Message::FeatureB),
    ])
}
```

The `.map(Message::Feature)` wraps child messages into the app-level enum for proper routing.

## Core vs UI (View model vs Model)

Keep **domain logic in a core crate** (e.g. `frename-core`); the UI only orchestrates via messages, Task dispatch, and view.

- **Core (model)**: types and operations (Directory, File, …). All checks and decisions live here: "does this exist?", "is this already open?", "did selection change?". Core functions return **coherent results** (e.g. `Option<File>`) so the UI never has to inspect internals or duplicate logic.
- **UI state (view model)**: holds core types, **only** dispatches messages and calls core. No conditionals that mirror core logic (no "if current file == target" in the UI). Call core; if it returns `None` / "no-op", return `Task::none()` or send nothing; if it returns `Some(x)`, send a message (e.g. `FileOpened(x)`). Messages stay in the feature; all the "ifs" and checks live in Directory / File.
- **Directory is the only source of truth for selection.** The folder (Directory) knows what file is selected. The UI must **not** pass "current workspace file" or "current selection" into Directory. Directory uses its own state (e.g. `selected_file()`) to decide "already selected?" or "open this path". One code path for persistence; no duplicate helpers.

### Database initialization: only in main.rs

**Rule: Database initialization must only be called in main.rs. Nowhere else.**

- In `main.rs`, before starting the UI (e.g. before `iced::application`), call **once**: `AppDatabase::new().initialize()` (or equivalent). This runs migrations and prepares the DB file.
- **Do not** call `.initialize()` in the workspace, in features, in core when creating a store for normal use, or in tests. Creating a store (e.g. `AppDatabase::new()`) is fine; only the one-time migration/setup belongs in main.
- Rationale: one place owns “the app has started, DB is ready”; no scattered init, no duplicate migration runs.

### App state store: transient, no singleton, no factory

- **Store is transient.** Create as many stores as you need. It is not a singleton and not scoped; use the **constructor** where a store is needed (e.g. `AppDatabase::new()`).
- **No factory, no helper.** Do not add a “create app state store” function or factory trait. Where the UI or workspace needs a store (e.g. to load last session or to pass into `Directory::open`), call the store constructor directly (e.g. `AppDatabase::new()`).
- **No Arc for passing the store around.** If a type (e.g. Directory) needs to hold the store and must be `Clone` (e.g. for messages), make that type **generic over the store** (e.g. `Directory<S: AppStateStore + Clone>`) and store `S` by value, not `Arc<dyn AppStateStore>`.

### Directory generic over the store; store only in the constructor

- **`Directory<S>`** where `S: AppStateStore + Clone`. The store is passed **only** to the constructor; no store parameter on other methods.
- **Constructors**: `Directory::open(path, store: S)` and `Directory::with_files(path, files, store: S)`. Directory keeps `S` and uses it internally for persistence (e.g. on `select_index`, `clear_selection`).
- **No store parameter** on `set_selection`, `open_path`, `select_index`, `select_previous`, `select_next`. Those methods use the store held by Directory.
- In the app, use a **type alias** for the concrete directory type (e.g. `type Directory = frename_core::Directory<frename_core::AppDatabase>`) so messages and workspace stay readable.

### Where the store is used

- **Load last session**: The **workspace** (or feature that owns “load last”) creates a store with the constructor and calls **store.get_last_session()** directly. Do not add a “get_last_session(store)” helper in the directory (or core) module; call the store method from the workspace.
- **Opening a folder**: The workspace creates a store with the constructor and passes it into `Directory::open(path, store)`. On success, the returned `Directory<S>` holds that store for future persistence. On failure, do not create an empty directory (see below).

### Folder load failure: keep previous state

- When **Directory::open** fails (e.g. async scan errors), do **not** create or use an “empty” directory. Do not replace the current directory with an empty one.
- Send a dedicated message (e.g. `FolderLoadFailed`). The handler sets `loading = false` and **leaves everything else unchanged** (same directory as before, if any). So: no `Directory::empty()`; on failure, stay in the same state as before the attempt.

### Session shape and who touches it

- **Session shape**: folder (required) + optional file, e.g. `FolderAndFile { folder, file: Option<PathBuf> }`. Only Directory (and the store it holds) reads/writes this for persistence.
- **Directory’s public surface**: the UI calls `Directory::open(path, store)`, `dir.set_selection(Some(path) | None)`, `dir.open_path(path)`, `dir.select_index(i)`, `dir.select_previous()`, `dir.select_next()`. No inner persistence helpers are exposed. Directory persists inside these methods using its stored `S`.

### Core module layout and style

- **File structure (e.g. `directory.rs`, `file.rs`)**: Put **types and data** first (struct and fields, with section comment). Then **constructors** (private helper if any, then `open` / `from_path` etc.). Then **accessors** (read-only getters). Then **mutation / commands** (set_selection, select_index, update_file, etc.). Last: **private helpers** (find_by_path, persist_session, etc.). Use section comments so the file is easy to scan.
- **If/else: short branch first.** In conditionals, put the **short** branch first and the **long** branch second. Prefer early return for the short path (e.g. `if index >= len { return None; }` then the main logic) so the reader sees the guard first and then the "happy" path.
- **Logging**: Log **inside the module that owns the operation**. Directory logs scan start/complete/failure and file updates; callers do not log directory results. Avoid duplicate or "caller reports core result" logs.
- **Public API**: Make functions **private** if only used within the same module. **Remove** unused methods. For test-only or programmatic-only APIs (e.g. `with_files`, `from_path`), either keep public with a doc like "For testing and programmatic use only" or use `pub(crate)` when all callers are in the same crate. Use `#[allow(dead_code)]` only when a method is used only by tests (same crate).

Prefer **extension-style APIs** in core using traits so call sites read as receiver-first:

```rust
// In core: trait + impl
pub trait SaveAndReparse { fn save_and_reparse(&self, path: &Path) -> (PathBuf, Vec<FileTag>); }
impl SaveAndReparse for [FileTag] { ... }

// In UI: one import, clear call
use frename_core::SaveAndReparse;
let (path, tags) = tags.file_tags().save_and_reparse(&path);
```

## Snapshots and Data, Not Messages

When a child provides "data to be applied later" (e.g. file tags to save), use a **plain data type** (e.g. `FileTagSnapshot { path, tags }`) in core, not a message.

- **Child feature**: exposes something like `get_snapshot() -> Option<FileTagSnapshot>`. It does **not** save and does **not** create a message.
- **Parent**: when it needs to persist, it creates **its own** message (e.g. `Message::FileUpdated`) from that snapshot and sends it to itself. The message type lives in the **parent** (the one that performs the side effect).

So: snapshot = data in core; message = owned by the component that runs the side effect.

## Defer Side Effects Until Safe

When a side effect (e.g. saving a file) must **not** run until another resource is released (e.g. video stopped), defer the side effect until you get a "released" signal.

1. **Store pending data** in parent state (e.g. `pending_file_updated: Option<FileTagSnapshot>`).
2. **Request release** by sending a message to the child (e.g. `Message::VideoPlayer(Unload)`).
3. **Child** clears the resource and returns a "released" message (e.g. `VideoUnloaded`).
4. **Parent** handles the "released" message: take pending data, dispatch the side-effect message to itself (e.g. `FileUpdated`), then start the next action (e.g. load new video). Clear pending.

Example flow: switch file → get snapshot, switch UI to new file, store snapshot, send Unload → on VideoUnloaded → send FileUpdated (so save runs), then load new video. Save never runs while the previous video is still playing.

## Parent Intercepts Child Messages

When the parent must **react** to a specific child message (e.g. `VideoUnloaded`) before or instead of forwarding, match on the child message and handle that variant; forward the rest.

```rust
Message::VideoPlayer(msg) => match msg {
    video_player::Message::VideoUnloaded => self.on_video_unloaded(),
    other => self.video_player.update(other).map(Message::VideoPlayer),
}
```

Do not forward the "special" message to the child; handle it in the parent and return the appropriate `Task`.

### 10. Single Message for "Apply This Selection"

When multiple code paths lead to the same UI update (e.g. "file was selected → apply snapshot, set file workspace, load/unload video"), use **one message** and **one handler** instead of calling a shared function from many places.

- **Define one message** (e.g. `FileOpened(File)`) meaning "this file is now selected; apply the usual logic."
- **Call sites** (open by path, folder loaded, select by index, prev/next) get a `File` from core and **send** the message: `Task::done(Message::FileOpened(file))`. They do **not** call an "apply file opened" function directly.
- **One handler** for that message does all the work (snapshot, set file workspace, load or unload video). This keeps a single place for the logic and avoids duplication.

In tests, the iced runtime does not run tasks; simulate by sending the same message (e.g. get selected file from directory and call `update(Message::FileOpened(file))`) when a task would have produced it.

### 11. Icons Over Text for UI

Prefer icons over informational text. Keep the screen free of labels except for user content.

- **No text on buttons** – use a single icon (e.g. `◀` `▶` `⏸` for prev/next/play-pause).
- **No standalone informational text** – replace placeholders and status text with one clear icon:
  - Loading: `⏳`
  - Drop / open folder: `📂`
  - Drop video: `🎬`
  - Error: `✕` or `❌`
  - Empty: `📭`
  - Select file / document: `📄`
  - No tags: `🏷`
- **Keep as text** – file names, tag names, and any other user-defined or user-visible content.
- Use a larger size for placeholder icons (e.g. 48–120) so they read as a single visual, not body text.

### 12. One Big Panel When Empty

When the workspace has no data yet (e.g. no folder opened), show **one full-window panel** (e.g. drop zone with icon), not multiple empty panels side by side.

- **Where**: Implement this in the **workspace view** (e.g. `folder_workspace/view.rs`), not in `app.rs`.
- **Condition**: If `state.directory().is_none()` (and similar “no data” checks), render a single centered container (icon + optional loading state). Otherwise render the normal multi-panel layout.
- **Loading**: While loading after a drop, the same big panel can show a loading icon (e.g. `⏳`) until the workspace has real data.
- App stays minimal: it only delegates view to the workspace; the workspace decides one-panel vs. multi-panel from its own state.
