---
name: iced-elm-architecture
description: Iced GUI framework Elm architecture patterns for Rust. Feature-based code organization, message flow, Task::done dispatch, and component independence. Use when building iced UI features, adding messages, creating views, wiring components, or organizing iced application code.
---

# Iced Elm Architecture Patterns

## Feature-Based Structure

Every feature lives in its own folder under `src/features/`:

```
src/features/feature_name/
├── mod.rs        # Re-exports: pub use messages::Message; pub use state::FeatureState;
├── messages.rs   # All messages this feature handles
├── state.rs      # State struct + update method
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
    let child_view = child::view::view(state.child()).map(Message::Controls);

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

### 5. Avoid Redundant Redraws

Don't use `on_new_frame` or similar per-frame callbacks unless you need per-frame UI updates (e.g., a progress slider). Each message triggers a re-render in iced.

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
```

The `.map(Message::Feature)` wraps child messages into the app-level enum for proper routing.
