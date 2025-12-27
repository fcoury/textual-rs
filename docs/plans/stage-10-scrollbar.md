# Stage 10: Scrollbar Implementation Plan (Revised)

## Summary

Implement scrollbars matching **Textual Python's architecture exactly**, including:
1. Dedicated `ScrollBar` and `ScrollBarCorner` widgets (not inline rendering)
2. Full scroll interactions: wheel, click-on-track, drag-thumb
3. Scroll message system (`ScrollUp`, `ScrollDown`, `ScrollLeft`, `ScrollRight`, `ScrollTo`)
4. Complete CSS property support with hover/active states
5. Proper glyph rendering with smooth gradient ends
6. ScrollableContainer widget for easy scrolling

---

## Textual Python Reference

### Architecture (from scrollbar.py)

```
ScrollBar widget
├── Handles mouse capture for drag
├── Emits: ScrollUp, ScrollDown, ScrollLeft, ScrollRight, ScrollTo messages
├── Has: position, window_size, window_virtual_size reactives
└── Uses: ScrollBarRender for actual drawing

ScrollBarCorner widget
└── Fills gap when both scrollbars present
```

**Key insight:** Textual's scrollbars are **separate widgets**, not inline chrome. The container manages scroll state and responds to scroll messages.

### Scrollbar Glyphs

| Orientation | Gradient Glyphs (smooth ends) | Body |
|-------------|-------------------------------|------|
| Vertical | `['▁', '▂', '▃', '▄', '▅', '▆', '▇', ' ']` | space (with bg color) |
| Horizontal | `['▉', '▊', '▋', '▌', '▍', '▎', '▏', ' ']` | space (with bg color) |

The gradient glyphs are used for **sub-cell precision** at thumb edges using fractional positioning.

### CSS Properties (Complete)

| Property | Type | Default | Notes |
|----------|------|---------|-------|
| `scrollbar-color` | `<color> [<percentage>]` | theme-provided | Thumb color with optional opacity |
| `scrollbar-color-hover` | `<color> [<percentage>]` | - | Thumb when mouse over scrollbar |
| `scrollbar-color-active` | `<color> [<percentage>]` | - | Thumb when dragging |
| `scrollbar-background` | `<color> [<percentage>]` | theme-provided | Track background |
| `scrollbar-background-hover` | `<color> [<percentage>]` | - | Track when mouse over |
| `scrollbar-background-active` | `<color> [<percentage>]` | - | Track when dragging |
| `scrollbar-corner-color` | `<color>` | - | Corner fill color |
| `scrollbar-size` | `<integer> <integer>` | `1 1` | Horizontal, Vertical thickness |
| `scrollbar-size-horizontal` | `<integer>` | `1` | Horizontal bar height |
| `scrollbar-size-vertical` | `<integer>` | `1` | Vertical bar width |
| `scrollbar-gutter` | `auto \| stable` | `auto` | Reserve space even when not scrolling |
| `scrollbar-visibility` | `visible \| hidden` | `visible` | Hide bars but allow scrolling |

**Size = 0** hides that scrollbar entirely (different from visibility: hidden).

### Scroll Messages

```python
class ScrollUp(ScrollMessage): ...      # Click above thumb
class ScrollDown(ScrollMessage): ...    # Click below thumb
class ScrollLeft(ScrollMessage): ...    # Click left of thumb
class ScrollRight(ScrollMessage): ...   # Click right of thumb
class ScrollTo(ScrollMessage):          # Drag thumb
    x: float | None
    y: float | None
    animate: bool
```

---

## Implementation Plan

### Step 1: Scroll Messages

**File:** `crates/textual/src/scroll.rs` (NEW)

```rust
//! Scroll-related messages and types.

/// Messages emitted by scrollbars for container handling.
#[derive(Debug, Clone)]
pub enum ScrollMessage {
    /// Scroll up by one "click" (click above thumb or wheel up)
    ScrollUp,
    /// Scroll down by one "click" (click below thumb or wheel down)
    ScrollDown,
    /// Scroll left by one "click"
    ScrollLeft,
    /// Scroll right by one "click"
    ScrollRight,
    /// Scroll to absolute position (drag or programmatic)
    ScrollTo {
        x: Option<f32>,
        y: Option<f32>,
        animate: bool,
    },
}

/// Scroll state for a scrollable container.
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    /// Current scroll offset (pixels/cells from top)
    pub offset_x: i32,
    pub offset_y: i32,
    /// Virtual content size
    pub virtual_width: i32,
    pub virtual_height: i32,
    /// Viewport size
    pub viewport_width: i32,
    pub viewport_height: i32,
}

impl ScrollState {
    pub fn max_scroll_x(&self) -> i32 {
        (self.virtual_width - self.viewport_width).max(0)
    }

    pub fn max_scroll_y(&self) -> i32 {
        (self.virtual_height - self.viewport_height).max(0)
    }

    pub fn can_scroll_x(&self) -> bool {
        self.virtual_width > self.viewport_width
    }

    pub fn can_scroll_y(&self) -> bool {
        self.virtual_height > self.viewport_height
    }

    pub fn scroll_percent_x(&self) -> f32 {
        if self.max_scroll_x() == 0 { 0.0 }
        else { self.offset_x as f32 / self.max_scroll_x() as f32 }
    }

    pub fn scroll_percent_y(&self) -> f32 {
        if self.max_scroll_y() == 0 { 0.0 }
        else { self.offset_y as f32 / self.max_scroll_y() as f32 }
    }
}
```

---

### Step 2: CSS Properties in TCSS

**File:** `crates/tcss/src/types/scrollbar.rs` (NEW)

```rust
//! Scrollbar CSS types.

use super::RgbaColor;

/// Scrollbar size configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollbarSize {
    /// Horizontal scrollbar height (0 = hidden)
    pub horizontal: u16,
    /// Vertical scrollbar width (0 = hidden)
    pub vertical: u16,
}

impl ScrollbarSize {
    pub const DEFAULT: Self = Self { horizontal: 1, vertical: 1 };
}

/// Scrollbar gutter behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScrollbarGutter {
    /// Only show scrollbar space when scrolling is possible
    #[default]
    Auto,
    /// Always reserve space for scrollbar
    Stable,
}

/// Scrollbar visibility (independent of overflow).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    #[default]
    Visible,
    /// Hide scrollbar but still allow scrolling
    Hidden,
}

/// Complete scrollbar style configuration.
#[derive(Debug, Clone, Default)]
pub struct ScrollbarStyle {
    // Thumb colors
    pub color: Option<RgbaColor>,
    pub color_hover: Option<RgbaColor>,
    pub color_active: Option<RgbaColor>,

    // Track colors
    pub background: Option<RgbaColor>,
    pub background_hover: Option<RgbaColor>,
    pub background_active: Option<RgbaColor>,

    // Corner
    pub corner_color: Option<RgbaColor>,

    // Size & visibility
    pub size: ScrollbarSize,
    pub gutter: ScrollbarGutter,
    pub visibility: ScrollbarVisibility,
}

impl ScrollbarStyle {
    /// Fallback colors (used when theme doesn't provide defaults)
    pub const FALLBACK_THUMB: RgbaColor = RgbaColor::rgb(255, 0, 255); // bright_magenta
    pub const FALLBACK_TRACK: RgbaColor = RgbaColor::rgb(85, 85, 85);  // #555555

    pub fn effective_color(&self) -> RgbaColor {
        self.color.clone().unwrap_or(Self::FALLBACK_THUMB)
    }

    pub fn effective_background(&self) -> RgbaColor {
        self.background.clone().unwrap_or(Self::FALLBACK_TRACK)
    }
}
```

**File:** `crates/tcss/src/types/mod.rs`

Add to `ComputedStyle`:
```rust
pub scrollbar: ScrollbarStyle,
```

**File:** `crates/tcss/src/parser.rs`

Add parsing for:
- `scrollbar-color: <color> [<percentage>]`
- `scrollbar-color-hover: <color> [<percentage>]`
- `scrollbar-color-active: <color> [<percentage>]`
- `scrollbar-background: <color> [<percentage>]`
- `scrollbar-background-hover: <color> [<percentage>]`
- `scrollbar-background-active: <color> [<percentage>]`
- `scrollbar-corner-color: <color>`
- `scrollbar-size: <int> <int>` or `scrollbar-size: <int>` (applies to both)
- `scrollbar-size-horizontal: <int>`
- `scrollbar-size-vertical: <int>`
- `scrollbar-gutter: auto | stable`
- `scrollbar-visibility: visible | hidden`

---

### Step 3: ScrollBarRender (Rendering Logic)

**File:** `crates/textual/src/scrollbar.rs` (NEW)

```rust
//! Scrollbar rendering with Textual-compatible glyphs.

use crate::{Canvas, Region};
use tcss::types::RgbaColor;

/// Glyphs for smooth scrollbar thumb edges.
pub struct ScrollbarGlyphs;

impl ScrollbarGlyphs {
    /// Vertical: bottom-to-top gradient for smooth edges
    pub const VERTICAL: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', ' '];
    /// Horizontal: right-to-left gradient for smooth edges
    pub const HORIZONTAL: [char; 8] = ['▉', '▊', '▋', '▌', '▍', '▎', '▏', ' '];
    /// Body glyph (space with background)
    pub const BODY: char = ' ';
}

/// Renders scrollbar visuals (used by ScrollBar widget).
pub struct ScrollBarRender;

impl ScrollBarRender {
    /// Render a vertical scrollbar with proper glyph gradients.
    ///
    /// # Arguments
    /// * `canvas` - Target canvas
    /// * `region` - Region for the scrollbar (width = thickness)
    /// * `virtual_size` - Total content height
    /// * `window_size` - Visible viewport height
    /// * `position` - Current scroll position (0.0 to virtual_size - window_size)
    /// * `thumb_color` - Thumb/grabber color
    /// * `track_color` - Track background color
    pub fn render_vertical(
        canvas: &mut Canvas,
        region: Region,
        virtual_size: f32,
        window_size: f32,
        position: f32,
        thumb_color: RgbaColor,
        track_color: RgbaColor,
    ) {
        let size = region.height as f32;
        let thickness = region.width;

        // Draw track background
        for y in 0..region.height {
            for x in 0..thickness {
                canvas.put_char(
                    region.x + x,
                    region.y + y,
                    ScrollbarGlyphs::BODY,
                    None,
                    Some(track_color.clone()),
                );
            }
        }

        // No thumb if content fits
        if window_size >= virtual_size || size == 0.0 {
            return;
        }

        let len_bars = ScrollbarGlyphs::VERTICAL.len() as f32;

        // Calculate thumb size and position (Textual's algorithm)
        let bar_ratio = virtual_size / size;
        let thumb_size = (window_size / bar_ratio).max(1.0);

        let position_ratio = position / (virtual_size - window_size);
        let thumb_position = (size - thumb_size) * position_ratio;

        // Convert to sub-cell precision
        let start = (thumb_position * len_bars) as i32;
        let end = start + (thumb_size * len_bars).ceil() as i32;

        let start_index = (start / len_bars as i32).max(0);
        let start_bar = (start % len_bars as i32).max(0) as usize;
        let end_index = (end / len_bars as i32).max(0);
        let end_bar = (end % len_bars as i32).max(0) as usize;

        // Draw thumb body
        for y in start_index..=end_index.min(region.height - 1) {
            let screen_y = region.y + y;
            if screen_y >= region.y + region.height {
                break;
            }

            for x in 0..thickness {
                let screen_x = region.x + x;

                // Determine glyph based on position
                let (glyph, fg, bg) = if y == start_index && start_bar > 0 {
                    // Top edge with gradient
                    let bar_char = ScrollbarGlyphs::VERTICAL[len_bars as usize - 1 - start_bar];
                    if bar_char != ' ' {
                        (bar_char, Some(thumb_color.clone()), Some(track_color.clone()))
                    } else {
                        (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                    }
                } else if y == end_index && end_bar > 0 && y > start_index {
                    // Bottom edge with gradient
                    let bar_char = ScrollbarGlyphs::VERTICAL[len_bars as usize - 1 - end_bar];
                    if bar_char != ' ' {
                        // Reverse colors for bottom edge
                        (bar_char, Some(track_color.clone()), Some(thumb_color.clone()))
                    } else {
                        (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                    }
                } else {
                    // Solid thumb body
                    (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                };

                canvas.put_char(screen_x, screen_y, glyph, fg, bg);
            }
        }
    }

    /// Render a horizontal scrollbar with proper glyph gradients.
    pub fn render_horizontal(
        canvas: &mut Canvas,
        region: Region,
        virtual_size: f32,
        window_size: f32,
        position: f32,
        thumb_color: RgbaColor,
        track_color: RgbaColor,
    ) {
        let size = region.width as f32;
        let thickness = region.height;

        // Draw track background
        for y in 0..thickness {
            for x in 0..region.width {
                canvas.put_char(
                    region.x + x,
                    region.y + y,
                    ScrollbarGlyphs::BODY,
                    None,
                    Some(track_color.clone()),
                );
            }
        }

        // No thumb if content fits
        if window_size >= virtual_size || size == 0.0 {
            return;
        }

        let len_bars = ScrollbarGlyphs::HORIZONTAL.len() as f32;

        // Calculate thumb size and position
        let bar_ratio = virtual_size / size;
        let thumb_size = (window_size / bar_ratio).max(1.0);

        let position_ratio = position / (virtual_size - window_size);
        let thumb_position = (size - thumb_size) * position_ratio;

        // Convert to sub-cell precision
        let start = (thumb_position * len_bars) as i32;
        let end = start + (thumb_size * len_bars).ceil() as i32;

        let start_index = (start / len_bars as i32).max(0);
        let start_bar = (start % len_bars as i32).max(0) as usize;
        let end_index = (end / len_bars as i32).max(0);
        let end_bar = (end % len_bars as i32).max(0) as usize;

        // Draw thumb body
        for x in start_index..=end_index.min(region.width - 1) {
            let screen_x = region.x + x;
            if screen_x >= region.x + region.width {
                break;
            }

            for y in 0..thickness {
                let screen_y = region.y + y;

                // Determine glyph based on position
                let (glyph, fg, bg) = if x == start_index && start_bar > 0 {
                    // Left edge with gradient
                    let bar_char = ScrollbarGlyphs::HORIZONTAL[len_bars as usize - 1 - start_bar];
                    if bar_char != ' ' {
                        (bar_char, Some(track_color.clone()), Some(thumb_color.clone()))
                    } else {
                        (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                    }
                } else if x == end_index && end_bar > 0 && x > start_index {
                    // Right edge with gradient
                    let bar_char = ScrollbarGlyphs::HORIZONTAL[len_bars as usize - 1 - end_bar];
                    if bar_char != ' ' {
                        (bar_char, Some(thumb_color.clone()), Some(track_color.clone()))
                    } else {
                        (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                    }
                } else {
                    // Solid thumb body
                    (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                };

                canvas.put_char(screen_x, screen_y, glyph, fg, bg);
            }
        }
    }
}
```

---

### Step 4: ScrollBar Widget

**File:** `crates/textual/src/widget/scrollbar.rs` (NEW)

```rust
//! ScrollBar widget with full mouse interaction support.

use crate::{Canvas, KeyCode, MouseEvent, MouseEventKind, Region, Size, Widget};
use crate::scroll::ScrollMessage;
use crate::scrollbar::ScrollBarRender;
use tcss::types::{RgbaColor, ScrollbarStyle};

/// Scrollbar interaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ScrollBarState {
    #[default]
    Normal,
    Hover,
    /// Dragging with grab position
    Grabbed { grab_position: i32 },
}

/// A scrollbar widget that emits scroll messages.
pub struct ScrollBar<M, F>
where
    F: Fn(ScrollMessage) -> M,
{
    /// Vertical (true) or horizontal (false)
    vertical: bool,
    /// Scrollbar thickness
    thickness: u16,
    /// Current scroll position (0 to virtual_size - window_size)
    position: f32,
    /// Position when grab started
    grabbed_position: f32,
    /// Total virtual content size
    virtual_size: f32,
    /// Visible window size
    window_size: f32,
    /// Current interaction state
    state: ScrollBarState,
    /// Styling
    style: ScrollbarStyle,
    /// Callback to convert ScrollMessage to app message
    on_scroll: F,
    /// Dirty flag
    dirty: bool,
}

impl<M, F> ScrollBar<M, F>
where
    F: Fn(ScrollMessage) -> M,
{
    pub fn new(vertical: bool, on_scroll: F) -> Self {
        Self {
            vertical,
            thickness: 1,
            position: 0.0,
            grabbed_position: 0.0,
            virtual_size: 100.0,
            window_size: 100.0,
            state: ScrollBarState::Normal,
            style: ScrollbarStyle::default(),
            on_scroll,
            dirty: true,
        }
    }

    pub fn with_thickness(mut self, thickness: u16) -> Self {
        self.thickness = thickness;
        self
    }

    /// Update scroll parameters from container.
    pub fn update(&mut self, virtual_size: f32, window_size: f32, position: f32) {
        if self.virtual_size != virtual_size
            || self.window_size != window_size
            || self.position != position
        {
            self.virtual_size = virtual_size;
            self.window_size = window_size;
            self.position = position;
            self.dirty = true;
        }
    }

    fn thumb_bounds(&self, region: &Region) -> (i32, i32) {
        let size = if self.vertical { region.height } else { region.width } as f32;

        if self.window_size >= self.virtual_size || size == 0.0 {
            return (0, 0);
        }

        let bar_ratio = self.virtual_size / size;
        let thumb_size = (self.window_size / bar_ratio).max(1.0);
        let position_ratio = self.position / (self.virtual_size - self.window_size).max(1.0);
        let thumb_position = ((size - thumb_size) * position_ratio) as i32;
        let thumb_end = thumb_position + thumb_size.ceil() as i32;

        (thumb_position, thumb_end)
    }

    fn current_colors(&self) -> (RgbaColor, RgbaColor) {
        match self.state {
            ScrollBarState::Grabbed { .. } => (
                self.style.color_active.clone().unwrap_or_else(|| self.style.effective_color()),
                self.style.background_active.clone().unwrap_or_else(|| self.style.effective_background()),
            ),
            ScrollBarState::Hover => (
                self.style.color_hover.clone().unwrap_or_else(|| self.style.effective_color()),
                self.style.background_hover.clone().unwrap_or_else(|| self.style.effective_background()),
            ),
            ScrollBarState::Normal => (
                self.style.effective_color(),
                self.style.effective_background(),
            ),
        }
    }
}

impl<M, F> Widget<M> for ScrollBar<M, F>
where
    F: Fn(ScrollMessage) -> M,
{
    fn desired_size(&self) -> Size {
        if self.vertical {
            Size { width: self.thickness, height: 1 }
        } else {
            Size { width: 1, height: self.thickness }
        }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let (thumb_color, track_color) = self.current_colors();

        if self.vertical {
            ScrollBarRender::render_vertical(
                canvas,
                region,
                self.virtual_size,
                self.window_size,
                self.position,
                thumb_color,
                track_color,
            );
        } else {
            ScrollBarRender::render_horizontal(
                canvas,
                region,
                self.virtual_size,
                self.window_size,
                self.position,
                thumb_color,
                track_color,
            );
        }
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;

        if !region.contains_point(mx, my) {
            // Mouse left - reset hover
            if self.state == ScrollBarState::Hover {
                self.state = ScrollBarState::Normal;
                self.dirty = true;
            }
            return None;
        }

        let pos_in_bar = if self.vertical {
            my - region.y
        } else {
            mx - region.x
        };

        let (thumb_start, thumb_end) = self.thumb_bounds(&region);
        let on_thumb = pos_in_bar >= thumb_start && pos_in_bar < thumb_end;

        match event.kind {
            MouseEventKind::Moved => {
                if !matches!(self.state, ScrollBarState::Grabbed { .. }) {
                    if self.state != ScrollBarState::Hover {
                        self.state = ScrollBarState::Hover;
                        self.dirty = true;
                    }
                }
                None
            }

            MouseEventKind::Down(_) => {
                if on_thumb {
                    // Start drag
                    self.state = ScrollBarState::Grabbed { grab_position: pos_in_bar };
                    self.grabbed_position = self.position;
                    self.dirty = true;
                    None
                } else if pos_in_bar < thumb_start {
                    // Click above/left of thumb
                    self.dirty = true;
                    let msg = if self.vertical {
                        ScrollMessage::ScrollUp
                    } else {
                        ScrollMessage::ScrollLeft
                    };
                    Some((self.on_scroll)(msg))
                } else {
                    // Click below/right of thumb
                    self.dirty = true;
                    let msg = if self.vertical {
                        ScrollMessage::ScrollDown
                    } else {
                        ScrollMessage::ScrollRight
                    };
                    Some((self.on_scroll)(msg))
                }
            }

            MouseEventKind::Drag(_) => {
                if let ScrollBarState::Grabbed { grab_position } = self.state {
                    let delta = pos_in_bar - grab_position;
                    let size = if self.vertical { region.height } else { region.width } as f32;
                    let scroll_delta = delta as f32 * (self.virtual_size / size);

                    let (x, y) = if self.vertical {
                        (None, Some(self.grabbed_position + scroll_delta))
                    } else {
                        (Some(self.grabbed_position + scroll_delta), None)
                    };

                    Some((self.on_scroll)(ScrollMessage::ScrollTo { x, y, animate: false }))
                } else {
                    None
                }
            }

            MouseEventKind::Up(_) => {
                if matches!(self.state, ScrollBarState::Grabbed { .. }) {
                    self.state = ScrollBarState::Hover;
                    self.dirty = true;
                }
                None
            }

            _ => None,
        }
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }
}
```

---

### Step 5: ScrollBarCorner Widget

**File:** `crates/textual/src/widget/scrollbar_corner.rs` (NEW)

```rust
//! Corner widget for when both scrollbars are visible.

use crate::{Canvas, Region, Size, Widget};
use tcss::types::RgbaColor;

/// Fills the corner gap between horizontal and vertical scrollbars.
pub struct ScrollBarCorner {
    color: Option<RgbaColor>,
    width: u16,
    height: u16,
}

impl ScrollBarCorner {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            color: None,
            width,
            height,
        }
    }

    pub fn with_color(mut self, color: RgbaColor) -> Self {
        self.color = Some(color);
        self
    }
}

impl<M> Widget<M> for ScrollBarCorner {
    fn desired_size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let bg = self.color.clone();
        for y in 0..region.height.min(self.height as i32) {
            for x in 0..region.width.min(self.width as i32) {
                canvas.put_char(region.x + x, region.y + y, ' ', None, bg.clone());
            }
        }
    }

    fn is_dirty(&self) -> bool { false }
    fn mark_dirty(&mut self) {}
    fn mark_clean(&mut self) {}
}
```

---

### Step 6: ScrollableContainer

**File:** `crates/textual/src/containers/scrollable.rs` (NEW)

A container that:
- Owns ScrollBar widgets for each axis
- Manages scroll state
- Handles scroll messages
- Reserves gutter space based on CSS
- Shows/hides scrollbars based on overflow and visibility settings

This is the main user-facing API for scrolling content.

---

### Step 7: Update Vertical/Horizontal Containers

Instead of adding scroll directly to Vertical/Horizontal, they should:
- Remain simple layout containers (no scrolling)
- Be used inside ScrollableContainer when scrolling is needed

This matches Textual's architecture where layout and scrolling are separate concerns.

---

## Files Summary

| File | Type | Purpose |
|------|------|---------|
| `scroll.rs` | NEW | ScrollMessage enum, ScrollState |
| `scrollbar.rs` | NEW | ScrollBarRender with proper glyphs |
| `widget/scrollbar.rs` | NEW | ScrollBar widget with click/drag |
| `widget/scrollbar_corner.rs` | NEW | ScrollBarCorner for gap fill |
| `containers/scrollable.rs` | NEW | ScrollableContainer |
| `tcss/types/scrollbar.rs` | NEW | ScrollbarStyle, ScrollbarSize, etc. |
| `tcss/parser.rs` | MODIFY | Parse all scrollbar-* properties |

---

## Issues Addressed

| Finding | Resolution |
|---------|------------|
| Only wheel scrolling | Added click-on-track and drag-to-scroll via ScrollBar widget |
| scrollbar-size is single u16 | Now `ScrollbarSize { horizontal, vertical }` |
| Hover/active colors unused | ScrollBar tracks state and uses appropriate colors |
| Missing scrollbar-visibility/gutter | Added to ScrollbarStyle |
| No corner rendering | Added ScrollBarCorner widget |
| Incomplete glyph usage | Full gradient algorithm in ScrollBarRender |
| No color opacity | Parser will support `<color> [<percentage>]` |
| Hardcoded defaults | Moved to fallbacks, theme provides defaults |

---

## Testing Strategy

1. **Unit tests:** ScrollState calculations, thumb bounds, glyph selection
2. **Integration tests:** Scroll message flow, container state updates
3. **Visual example:** `scroll_demo.rs` with:
   - Tall content that needs vertical scroll
   - Wide content that needs horizontal scroll
   - Both scrollbars with corner
   - Custom CSS colors
   - Click and drag interactions
