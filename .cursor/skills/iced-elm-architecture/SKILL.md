---
name: iced-elm-architecture
description: Iced GUI framework Elm architecture patterns for Rust. Feature-based code organization, message flow, Task::done dispatch, subscriptions, and component independence. Use when building iced UI features, adding messages, creating views, wiring components, or organizing iced application code.
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
mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub use state::FeatureState;
```

## Critical Rules

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
