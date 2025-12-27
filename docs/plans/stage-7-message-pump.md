# Message Pump Implementation Plan

## Summary

Implement an async message system with tokio, enabling:
- Background tasks (timers, network) to trigger UI updates
- Message bubbling from child widgets to parents
- Reactive styling integration (messages → dirty → restyle → render)

**User Requirements:**
- Full async (tokio) runtime
- Bubbling-only routing (child → parent)
- Rust-idiomatic API

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Message Queue (mpsc)                      │
│   Sources: User Input | Timers | Async Tasks | Widgets       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       Main Loop                              │
│  1. Drain message queue                                      │
│  2. Bubble messages UP from focused widget (O(d) not O(n×d)) │
│  3. resolve_dirty_styles() for changed widgets               │
│  4. Render                                                   │
│  5. Poll terminal events + async message channel             │
└─────────────────────────────────────────────────────────────┘
```

### Key Architectural Decision: Focus-Targeted Dispatch

**Problem:** Top-down container search is O(n×d) per event.

**Solution:** Events go directly to focused widget, then bubble UP the cached path.

```
KeyEvent arrives
      │
      ▼
┌─────────────────┐
│ Focused Widget  │  ← Direct dispatch (no tree search)
│  on_event(key)  │
└────────┬────────┘
         │ returns Option<M>
         ▼
┌─────────────────┐
│ Bubble UP path  │  ← [Switch, Vertical, Center, Root]
│ handle_message  │     Each ancestor can intercept/stop
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ App::handle_msg │  ← Final handler (if still bubbling)
└─────────────────┘
```

**Performance:** O(d) where d = tree depth (typically <10)

---

## Implementation Stages

### Stage 1: Dependencies & Infrastructure

**Files:**
- `crates/textual/Cargo.toml` - Add dependencies
- `crates/textual/src/context.rs` - New: AppContext
- `crates/textual/src/message.rs` - New: MessageEnvelope

**Changes:**

```toml
# Cargo.toml additions
crossterm = { version = "0.27", features = ["event-stream"] }
tokio = { version = "1.0", features = ["rt", "rt-multi-thread", "sync", "time", "macros"] }
futures = "0.3"
```

**New Types:**

```rust
// context.rs
pub struct AppContext<M> {
    sender: mpsc::UnboundedSender<MessageEnvelope<M>>,
}

impl<M> AppContext<M> {
    pub fn post(&self, message: M, sender_id: Option<String>, sender_type: String);
    pub fn sender(&self) -> mpsc::UnboundedSender<MessageEnvelope<M>>;
    pub fn set_timer(&self, delay: Duration, message: M);
    pub fn set_interval<F>(&self, interval: Duration, f: F) -> IntervalHandle;
}

// message.rs
pub struct MessageEnvelope<M> {
    pub message: M,
    pub sender_id: Option<String>,
    pub sender_type: String,
    bubbling: bool,
}

impl<M> MessageEnvelope<M> {
    pub fn stop(&mut self);      // Stop bubbling
    pub fn is_bubbling(&self) -> bool;
}
```

**Success Criteria:** Types compile, unit tests pass

---

### Stage 2: Async Event Loop

**Files:**
- `crates/textual/src/lib.rs` - Convert run() to async

**Changes:**

1. Keep `run()` synchronous (backwards compatible) - calls `block_on(run_async())`
2. Add `run_async()` that uses `tokio::select!` with:
   - `crossterm::event::EventStream` for terminal events
   - `mpsc::UnboundedReceiver` for messages
3. Add `on_mount(&mut self, ctx: &AppContext<Self::Message>)` lifecycle hook

**New Loop Structure:**

```rust
async fn event_loop_async(&mut self) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let ctx = AppContext::new(tx);

    self.on_mount(&ctx);  // User can start timers here

    let mut event_stream = EventStream::new();

    while !self.should_quit() {
        // Focus handling (unchanged)

        // Render if needed
        if needs_render {
            resolve_dirty_styles(...);
            render(...);
            needs_render = false;
        }

        tokio::select! {
            biased;

            // Terminal events
            Some(Ok(event)) = event_stream.next() => {
                match event {
                    Event::Key(k) => {
                        if let Some(envelope) = root.dispatch_event(k.code) {
                            self.handle_message(envelope);
                        }
                        needs_render = true;
                    }
                    // ... mouse, resize
                }
            }

            // Async messages
            Some(envelope) = rx.recv() => {
                self.handle_message(envelope);
                needs_render = true;
            }
        }
    }
}
```

**Success Criteria:** Existing examples work unchanged

---

### Stage 3: Message Bubbling

**Files:**
- `crates/textual/src/widget.rs` - Add dispatch methods
- `crates/textual/src/containers/vertical.rs` - Implement bubbling
- `crates/textual/src/containers/horizontal.rs` - Implement bubbling
- `crates/textual/src/containers.rs` - Update Center/Middle

**Widget Trait Additions:**

```rust
pub trait Widget<M> {
    // Existing methods...

    /// Widget's optional ID for message tracking
    fn id(&self) -> Option<&str> { None }

    /// Handle a message bubbling from a descendant
    fn handle_message(&mut self, _envelope: &mut MessageEnvelope<M>) -> Option<M> {
        None  // Default: don't intercept
    }

    /// Dispatch event with bubbling support
    fn dispatch_event(&mut self, key: KeyCode) -> Option<MessageEnvelope<M>> {
        self.on_event(key).map(|msg| MessageEnvelope::new(msg, self.id(), self.type_name()))
    }

    /// Dispatch mouse event with bubbling support
    fn dispatch_mouse(&mut self, event: MouseEvent, region: Region) -> Option<MessageEnvelope<M>> {
        self.on_mouse(event, region).map(|msg| MessageEnvelope::new(msg, self.id(), self.type_name()))
    }
}
```

**Container Bubbling Pattern (Vertical example):**

```rust
fn dispatch_event(&mut self, key: KeyCode) -> Option<MessageEnvelope<M>> {
    for child in &mut self.children {
        if let Some(mut envelope) = child.dispatch_event(key) {
            // Give container a chance to intercept
            if envelope.is_bubbling() {
                if let Some(new_msg) = self.handle_message(&mut envelope) {
                    envelope.message = new_msg;
                }
            }
            return Some(envelope);  // Continue bubbling up
        }
    }
    // No child handled, try own handler
    self.on_event(key).map(|msg| MessageEnvelope::new(msg, None, "Vertical"))
}
```

**Success Criteria:** Messages bubble up, containers can intercept

---

### Stage 4: App Trait Update

**Files:**
- `crates/textual/src/lib.rs` - Update handle_message signature

**Breaking Change:**

```rust
// Before
fn handle_message(&mut self, message: Self::Message);

// After
fn handle_message(&mut self, envelope: MessageEnvelope<Self::Message>);
```

**Migration:** Users access `envelope.message` for the actual message.

```rust
fn handle_message(&mut self, envelope: MessageEnvelope<Message>) {
    match envelope.message {
        Message::WifiToggled(on) => { /* ... */ }
    }
}
```

**Success Criteria:** Examples updated and working

---

### Stage 5: Timer Support

**Files:**
- `crates/textual/src/timer.rs` - New: timer utilities

**API:**

```rust
// One-shot timer
ctx.set_timer(Duration::from_secs(5), Message::Timeout);

// Repeating interval (returns handle to cancel)
let handle = ctx.set_interval(Duration::from_millis(100), || Message::Tick);
handle.cancel();  // Stop the interval
```

**Implementation:**

```rust
pub fn set_timer<M: Send + 'static>(
    sender: mpsc::UnboundedSender<MessageEnvelope<M>>,
    delay: Duration,
    message: M,
) {
    tokio::spawn(async move {
        tokio::time::sleep(delay).await;
        let envelope = MessageEnvelope::new(message, None, "Timer");
        let _ = sender.send(envelope);
    });
}
```

**Success Criteria:** Timer example works

---

### Stage 6: Widget ID Support

**Files:**
- `crates/textual/src/widget/switch.rs` - Add ID field

**Changes:**

```rust
pub struct Switch<M, F> {
    // existing fields...
    id: Option<String>,
}

impl<M, F> Switch<M, F> {
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl<M, F> Widget<M> for Switch<M, F> {
    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}
```

**Success Criteria:** MessageEnvelope contains sender_id

---

### Stage 7: Example & Documentation

**Files:**
- `examples/async_timer.rs` - New: demo async features
- Update existing examples for new `handle_message` signature

**Example:**

```rust
struct TimerApp {
    tick_count: u32,
    running: bool,
}

impl App for TimerApp {
    fn on_mount(&mut self, ctx: &AppContext<Message>) {
        ctx.set_interval(Duration::from_secs(1), || Message::Tick);
    }

    fn handle_message(&mut self, envelope: MessageEnvelope<Message>) {
        match envelope.message {
            Message::Tick => self.tick_count += 1,
        }
    }
}
```

---

## Critical Files

| File | Action | Purpose |
|------|--------|---------|
| `crates/textual/Cargo.toml` | Modify | Add tokio, futures, event-stream |
| `crates/textual/src/lib.rs` | Modify | Async event loop, on_mount hook |
| `crates/textual/src/widget.rs` | Modify | dispatch_*, handle_message, id() |
| `crates/textual/src/context.rs` | Create | AppContext for message posting |
| `crates/textual/src/message.rs` | Create | MessageEnvelope with bubbling |
| `crates/textual/src/timer.rs` | Create | Timer/interval helpers |
| `crates/textual/src/containers/vertical.rs` | Modify | Bubbling implementation |
| `crates/textual/src/containers/horizontal.rs` | Modify | Bubbling implementation |
| `examples/switch.rs` | Modify | Update for MessageEnvelope |
| `examples/async_timer.rs` | Create | Demo async features |

---

## Breaking Changes

1. **`App::handle_message`** - Now takes `MessageEnvelope<M>` instead of `M`
2. **New dependency** - tokio adds ~2-3MB to release binary

---

## Backwards Compatibility

- `run()` remains synchronous (wraps `run_async()`)
- Existing `on_event()`/`on_mouse()` signatures unchanged
- `on_mount()` has default no-op implementation
- Widgets without IDs work fine (sender_id = None)
