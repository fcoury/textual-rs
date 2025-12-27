# Stage 8: Reactive Attributes (Loading, Visibility & Disabled)

## Overview

Add standard reactive flags (`visible`, `loading`, `disabled`) to the Widget trait, enabling apps to control widget presence and visual state without rebuilding the tree. This preserves focus, styles, and eliminates flicker.

Includes animated spinner support and `set_value()` for external state control.

## Files to Modify

| File | Action |
|------|--------|
| `crates/textual/src/widget.rs` | Add trait methods + update Box impl |
| `crates/textual/src/containers/vertical.rs` | Skip invisible children in render/layout/events |
| `crates/textual/src/containers/horizontal.rs` | Skip invisible children in render/layout/events |
| `crates/textual/src/containers.rs` | Update Center/Middle wrappers |
| `crates/textual/src/widget/switch.rs` | Add loading state + spinner rendering |
| `crates/textual/src/style_resolver.rs` | Skip invisible widgets (optimization) |
| `examples/api_fetch.rs` | **New** - Demo async loading workflow |

---

## Implementation Details

### 1. Widget Trait Extensions (`widget.rs`)

Add these methods to `Widget<M>`:

```rust
/// If false, widget is excluded from layout, rendering, and events.
fn is_visible(&self) -> bool { true }
fn set_visible(&mut self, _visible: bool) {}

/// If true, widget renders loading state instead of normal content.
fn is_loading(&self) -> bool { false }
fn set_loading(&mut self, _loading: bool) {}

/// If true, widget is visible but non-interactive (grayed out).
fn is_disabled(&self) -> bool { false }
fn set_disabled(&mut self, _disabled: bool) {}
```

**Also update:**
- `is_focusable()` - return `false` if not visible or disabled
- `get_state()` - include DISABLED flag for CSS `:disabled` selector
- `Box<dyn Widget<M>>` impl - delegate all new methods

### 2. Container Updates (`vertical.rs`, `horizontal.rs`)

Update these methods to skip invisible children:

**`render()`:**
```rust
for child in &self.children {
    if !child.is_visible() { continue; }
    // ... existing layout logic
}
```

**`desired_size()`:**
```rust
for child in &self.children {
    if !child.is_visible() { continue; }
    let size = child.desired_size();
    // accumulate...
}
```

**`on_mouse()`:**
```rust
for child in &mut self.children {
    if !child.is_visible() { continue; }
    // ... hit testing
}
```

**`count_focusable()`:**
```rust
for child in &self.children {
    if !child.is_visible() { continue; }
    count += child.count_focusable();
}
```

**`focus_nth()`:**
```rust
for child in &mut self.children {
    if !child.is_visible() { continue; }
    // ... focus navigation
}
```

**`clear_focus()` / `clear_hover()`:**
```rust
for child in &mut self.children {
    if !child.is_visible() { continue; }
    child.clear_focus();
}
```

**`child_count()` / `get_child_mut()`:**
- These should still return all children (for tree traversal)
- Visibility filtering happens in the iteration logic above

### 3. Center/Middle Wrappers (`containers.rs`)

Same pattern - check `child.is_visible()` before:
- `render()`
- `desired_size()` - return (0,0) if child invisible
- `on_mouse()`
- Focus methods

### 4. Switch Reactive State (`switch.rs`)

Add fields:
```rust
pub struct Switch<M, F> {
    // ... existing fields
    visible: bool,
    loading: bool,
    disabled: bool,
    spinner_frame: usize,  // For animated spinner
}
```

**New public methods:**
```rust
/// Set the switch value externally (e.g., from API response)
pub fn set_value(&mut self, value: bool) {
    if self.value != value {
        self.value = value;
        self.dirty = true;
    }
}

/// Advance spinner animation frame (call from timer)
pub fn tick_spinner(&mut self) {
    self.spinner_frame = (self.spinner_frame + 1) % 10;
    if self.loading {
        self.dirty = true;
    }
}
```

**Implement trait methods:**
```rust
fn is_visible(&self) -> bool { self.visible }
fn set_visible(&mut self, visible: bool) {
    if self.visible != visible {
        self.visible = visible;
        self.dirty = true;
    }
}

fn is_loading(&self) -> bool { self.loading }
fn set_loading(&mut self, loading: bool) {
    if self.loading != loading {
        self.loading = loading;
        self.dirty = true;
    }
}

fn is_disabled(&self) -> bool { self.disabled }
fn set_disabled(&mut self, disabled: bool) {
    if self.disabled != disabled {
        self.disabled = disabled;
        self.dirty = true;
    }
}

fn is_focusable(&self) -> bool {
    self.visible && !self.disabled  // Can't focus if invisible or disabled
}
```

**Update `render()`:**
```rust
const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

fn render(&self, canvas: &mut Canvas, region: Region) {
    if self.loading {
        let frame = SPINNER_FRAMES[self.spinner_frame];
        let display = format!("  [ {} ]  ", frame);
        canvas.put_str(region.x, region.y, &display, ...);
        return;
    }
    // ... existing switch rendering (with dimmed colors if disabled)
}
```

**Update `on_event()` and `on_mouse()`:**
```rust
fn on_event(&mut self, key: KeyCode) -> Option<M> {
    if self.disabled { return None; }  // Ignore input when disabled
    // ... existing logic
}
```

**Builder methods:**
```rust
pub fn with_loading(mut self, loading: bool) -> Self { ... }
pub fn with_disabled(mut self, disabled: bool) -> Self { ... }
pub fn with_visible(mut self, visible: bool) -> Self { ... }
```

### 5. Style Resolver Optimization (`style_resolver.rs`)

In `resolve_dirty_styles()`, skip invisible widgets:
```rust
pub fn resolve_dirty_styles<M>(...) {
    if !widget.is_visible() {
        return; // Skip invisible subtrees
    }
    // ... existing logic
}
```

### 6. Example: `api_fetch.rs`

Demonstrates the full reactive workflow with animated spinner:

```rust
use std::time::Duration;
use textual::{App, AppContext, Compose, MessageEnvelope, /* ... */};

enum Message {
    SpinnerTick,           // Animate loading spinners
    WifiStatusLoaded(bool),
    WifiToggled(bool),
}

struct ApiApp {
    quit: bool,
    wifi_loading: bool,
    wifi_status: bool,
}

impl App for ApiApp {
    fn on_mount(&mut self, ctx: &AppContext<Message>) {
        // 1. Start spinner animation timer (100ms = smooth animation)
        ctx.set_interval(Duration::from_millis(100), || Message::SpinnerTick);

        // 2. Simulate API fetch with 2-second delay
        ctx.spawn_task(async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            Message::WifiStatusLoaded(true)
        });
    }

    fn handle_message(&mut self, envelope: MessageEnvelope<Message>) {
        match envelope.message {
            Message::SpinnerTick => {
                // Tick all loading widgets (via tree access - see note below)
            }
            Message::WifiStatusLoaded(status) => {
                self.wifi_status = status;
                self.wifi_loading = false;
            }
            Message::WifiToggled(on) => {
                self.wifi_status = on;
            }
        }
    }
}
```

**Key workflow:**
1. App starts with `wifi_loading = true`
2. `on_mount` starts spinner timer + async API fetch
3. Every 100ms, `SpinnerTick` advances spinner frame
4. After 2s, `WifiStatusLoaded` sets loading=false
5. Switch re-renders showing actual "ON" state

**Note on tree access:** The App needs to access widgets to call `tick_spinner()`. Options:
- Store widget references in App struct (complex lifetime management)
- Add a `tick_all_spinners(&mut root)` helper that walks tree
- Have widgets self-animate via internal timers (more complex)

For simplicity, example will use approach 2 with a helper function.

---

## Implementation Order

1. **Widget trait** (`widget.rs`) - Add `is_visible`, `set_visible`, `is_loading`, `set_loading`, `is_disabled`, `set_disabled` with defaults
2. **Switch widget** (`switch.rs`) - Implement all reactive flags + `set_value()` + spinner rendering
3. **Containers** (`vertical.rs`, `horizontal.rs`, `containers.rs`) - Add visibility checks
4. **Style resolver** (`style_resolver.rs`) - Skip invisible widgets
5. **Example** (`api_fetch.rs`) - Demo the full async loading workflow

## Design Decisions

| Decision | Choice |
|----------|--------|
| Spinner animation | Animated (Braille frames via timer) |
| Switch value control | Add `set_value()` for external control |
| Disabled state | Include in this stage |
| Spinner timing | App-controlled via `SpinnerTick` message |
