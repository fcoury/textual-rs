# Stage 9: Layout Foundations

## Summary

Convert layout coordinates from `u16` to `i32` to enable safe scrolling and off-screen positioning. Add a clipping stack to Canvas for container-based content clipping.

**Key Changes:**
1. `Region` coordinates become `i32` (allows negative positions for scrolling)
2. `Size` remains `u16` (intrinsic size is always positive)
3. Canvas gains a clipping stack with `push_clip`/`pop_clip`
4. All containers updated for signed coordinate math
5. Hit testing uses new `Region::contains_point()` method

---

## Files to Modify

| File | Changes |
|------|---------|
| `crates/textual/src/canvas.rs` | Region → i32, add clipping stack, update put_char/put_str |
| `crates/textual/src/widget.rs` | Update trait signatures for signed Region |
| `crates/textual/src/containers/vertical.rs` | i32 math, use clipping |
| `crates/textual/src/containers/horizontal.rs` | i32 math, use clipping |
| `crates/textual/src/containers.rs` | Update Center/Middle for i32 |
| `crates/textual/src/widget/switch.rs` | Update hit testing for i32 |
| `crates/textual/src/lib.rs` | Use Region::new() in event loop |

---

## Implementation

### 1. Canvas & Region (`canvas.rs`)

Preserve the existing `Cell`-based structure while adding i32 coordinates and clipping.

```rust
use crossterm::{
    cursor, execute,
    style::{Color, SetBackgroundColor, SetForegroundColor},
};
use std::io::Write;
use tcss::types::RgbaColor;

/// A signed rectangular region for layout and clipping.
///
/// Coordinates are signed (i32) to allow off-screen positioning (e.g. scrolling).
/// Width and height are signed but invariant-checked to be non-negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Region {
    /// Create a new region, clamping width and height to be non-negative.
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width: width.max(0),
            height: height.max(0),
        }
    }

    /// Helper for migration from u16 types.
    pub fn from_u16(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self::new(x as i32, y as i32, width as i32, height as i32)
    }

    /// Returns the intersection of this region with another.
    /// If there is no overlap, returns an empty region.
    pub fn intersection(&self, other: &Region) -> Region {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x2 > x1 && y2 > y1 {
            Region { x: x1, y: y1, width: x2 - x1, height: y2 - y1 }
        } else {
            Region { x: 0, y: 0, width: 0, height: 0 }
        }
    }

    /// Checks if a point is contained within the region.
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }

    /// Returns true if the region has no area.
    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

/// A size composed of unsigned dimensions (intrinsic size).
#[derive(Clone, Copy, Debug, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub symbol: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

pub struct Canvas {
    size: Size,
    cells: Vec<Cell>,
    #[allow(dead_code)]
    current_fg: Option<Color>,
    #[allow(dead_code)]
    current_bg: Option<Color>,
    /// Stack of clipping regions. The active clip is the intersection of all.
    clip_stack: Vec<Region>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            size: Size { width, height },
            cells: vec![
                Cell { symbol: ' ', fg: None, bg: None };
                (width * height) as usize
            ],
            current_fg: None,
            current_bg: None,
            clip_stack: Vec::new(),
        }
    }

    // === Clipping ===

    /// Pushes a new clipping region onto the stack.
    /// The effective clip becomes the intersection of current clip and new region.
    pub fn push_clip(&mut self, region: Region) {
        let current = self.current_clip();
        let intersection = region.intersection(&current);
        self.clip_stack.push(intersection);
    }

    /// Removes the most recent clipping region.
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Returns the current effective clipping region.
    /// If stack is empty, returns the full screen.
    fn current_clip(&self) -> Region {
        self.clip_stack.last().cloned().unwrap_or(Region {
            x: 0,
            y: 0,
            width: self.size.width as i32,
            height: self.size.height as i32,
        })
    }

    // === Drawing ===

    /// Writes a character to the canvas at (x, y).
    /// Coordinates are i32 and will be clipped if off-screen or outside clip region.
    pub fn put_char(
        &mut self,
        x: i32,
        y: i32,
        c: char,
        fg: Option<RgbaColor>,
        bg: Option<RgbaColor>,
    ) {
        let clip = self.current_clip();

        // Clip bounds check
        if x < clip.x || x >= clip.x + clip.width {
            return;
        }
        if y < clip.y || y >= clip.y + clip.height {
            return;
        }

        // Screen bounds check
        if x < 0 || x >= self.size.width as i32 || y < 0 || y >= self.size.height as i32 {
            return;
        }

        let index = (y as usize) * (self.size.width as usize) + (x as usize);
        self.cells[index] = Cell {
            symbol: c,
            fg: fg.map(to_crossterm_color),
            bg: bg.map(to_crossterm_color),
        };
    }

    /// Writes a string to the canvas at (x, y).
    /// Coordinates are i32 and will be clipped appropriately.
    pub fn put_str(
        &mut self,
        x: i32,
        y: i32,
        s: &str,
        fg: Option<RgbaColor>,
        bg: Option<RgbaColor>,
    ) {
        let clip = self.current_clip();

        // Early vertical clip check
        if y < clip.y || y >= clip.y + clip.height {
            return;
        }
        if y < 0 || y >= self.size.height as i32 {
            return;
        }

        let mut current_x = x;
        for c in s.chars() {
            // Stop if past right edge of clip
            if current_x >= clip.x + clip.width {
                break;
            }
            // Only draw if within clip region and screen
            if current_x >= clip.x && current_x >= 0 && current_x < self.size.width as i32 {
                let index = (y as usize) * (self.size.width as usize) + (current_x as usize);
                self.cells[index] = Cell {
                    symbol: c,
                    fg: fg.clone().map(to_crossterm_color),
                    bg: bg.clone().map(to_crossterm_color),
                };
            }
            current_x += 1;
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        let mut out = std::io::stdout();
        execute!(out, cursor::MoveTo(0, 0))?;

        let mut last_fg = None;
        let mut last_bg = None;

        for row in self.cells.chunks(self.size.width as usize) {
            for cell in row {
                if cell.fg != last_fg {
                    let color = cell.fg.unwrap_or(Color::Reset);
                    execute!(out, SetForegroundColor(color))?;
                    last_fg = cell.fg;
                }
                if cell.bg != last_bg {
                    let color = cell.bg.unwrap_or(Color::Reset);
                    execute!(out, SetBackgroundColor(color))?;
                    last_bg = cell.bg;
                }
                write!(out, "{}", cell.symbol)?;
            }
            write!(out, "\r\n")?;
        }
        out.flush()?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell { symbol: ' ', fg: None, bg: None });
        self.clip_stack.clear();
    }
}

fn to_crossterm_color(c: RgbaColor) -> Color {
    Color::Rgb { r: c.r, g: c.g, b: c.b }
}
```

---

### 2. Widget Trait (`widget.rs`)

Update method signatures to use signed `Region`. No changes to trait semantics.

**Key signature changes:**
```rust
fn render(&self, canvas: &mut Canvas, region: Region);
fn on_mouse(&mut self, _event: MouseEvent, _region: Region) -> Option<M>;
```

The rest of the trait remains unchanged. Update the `Box<dyn Widget<M>>` impl to match.

---

### 3. Vertical Container (`containers/vertical.rs`)

```rust
use crate::{Canvas, KeyCode, MouseEvent, Region, Size, Widget};

pub struct Vertical<M> {
    pub children: Vec<Box<dyn Widget<M>>>,
    dirty: bool,
}

impl<M> Vertical<M> {
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self { children, dirty: true }
    }
}

impl<M> Widget<M> for Vertical<M> {
    fn desired_size(&self) -> Size {
        let mut width = 0;
        let mut height = 0;
        for child in &self.children {
            if !child.is_visible() { continue; }
            let size = child.desired_size();
            width = width.max(size.width);
            height += size.height;
        }
        Size { width, height }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        canvas.push_clip(region);

        let mut current_y = region.y;
        for child in &self.children {
            if !child.is_visible() { continue; }

            let size = child.desired_size();
            let child_height = size.height as i32;

            let child_region = Region {
                x: region.x,
                y: current_y,
                width: region.width,
                height: child_height,
            };

            child.render(canvas, child_region);
            current_y += child_height;
        }

        canvas.pop_clip();
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;

        if !region.contains_point(mx, my) {
            return None;
        }

        let mut current_y = region.y;
        for child in &mut self.children {
            if !child.is_visible() { continue; }

            let size = child.desired_size();
            let child_height = size.height as i32;

            let child_region = Region {
                x: region.x,
                y: current_y,
                width: region.width,
                height: child_height,
            };

            if child_region.contains_point(mx, my) {
                if let Some(msg) = child.on_mouse(event, child_region) {
                    return Some(msg);
                }
            }
            current_y += child_height;
        }
        None
    }

    // ... rest of Widget impl unchanged (for_each_child, is_dirty, etc.)
}
```

---

### 4. Horizontal Container (`containers/horizontal.rs`)

Same pattern as Vertical, but accumulating `current_x` instead of `current_y`.

```rust
fn render(&self, canvas: &mut Canvas, region: Region) {
    canvas.push_clip(region);

    let mut current_x = region.x;
    for child in &self.children {
        if !child.is_visible() { continue; }

        let size = child.desired_size();
        let child_width = size.width as i32;

        let child_region = Region {
            x: current_x,
            y: region.y,
            width: child_width,
            height: region.height,
        };

        child.render(canvas, child_region);
        current_x += child_width;
    }

    canvas.pop_clip();
}

fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
    let mx = event.column as i32;
    let my = event.row as i32;

    if !region.contains_point(mx, my) {
        return None;
    }

    let mut current_x = region.x;
    for child in &mut self.children {
        if !child.is_visible() { continue; }

        let size = child.desired_size();
        let child_width = size.width as i32;

        let child_region = Region {
            x: current_x,
            y: region.y,
            width: child_width,
            height: region.height,
        };

        if child_region.contains_point(mx, my) {
            if let Some(msg) = child.on_mouse(event, child_region) {
                return Some(msg);
            }
        }
        current_x += child_width;
    }
    None
}
```

---

### 5. Center & Middle (`containers.rs`)

Update for i32 math:

```rust
// Center
fn render(&self, canvas: &mut Canvas, region: Region) {
    if !self.child.is_visible() { return; }

    let child_size = self.child.desired_size();
    let child_width = child_size.width as i32;
    let x_offset = (region.width - child_width).max(0) / 2;

    let centered_region = Region {
        x: region.x + x_offset,
        y: region.y,
        width: child_width,
        height: region.height,
    };

    self.child.render(canvas, centered_region);
}

// Middle
fn render(&self, canvas: &mut Canvas, region: Region) {
    if !self.child.is_visible() { return; }

    let child_size = self.child.desired_size();
    let child_height = child_size.height as i32;
    let y_offset = (region.height - child_height).max(0) / 2;

    let middled_region = Region {
        x: region.x,
        y: region.y + y_offset,
        width: region.width,
        height: child_height,
    };

    self.child.render(canvas, middled_region);
}
```

---

### 6. Switch Widget (`widget/switch.rs`)

Update hit testing to use `contains_point`:

```rust
fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
    let mx = event.column as i32;
    let my = event.row as i32;
    let in_bounds = region.contains_point(mx, my);

    match event.kind {
        MouseEventKind::Moved => {
            if in_bounds != self.hovered {
                self.hovered = in_bounds;
                self.dirty = true;
            }
            None
        }
        MouseEventKind::Down(_) if in_bounds && !self.disabled && !self.loading => {
            if !self.active {
                self.active = true;
                self.dirty = true;
            }
            None
        }
        MouseEventKind::Up(_) if in_bounds && self.active && !self.disabled && !self.loading => {
            self.active = false;
            self.value = !self.value;
            self.dirty = true;
            Some((self.on_change)(self.value))
        }
        MouseEventKind::Up(_) => {
            if self.active {
                self.active = false;
                self.dirty = true;
            }
            None
        }
        _ => None,
    }
}
```

Also update `render` signature:
```rust
fn render(&self, canvas: &mut Canvas, region: Region) {
    // Use region.x and region.y (now i32) directly with put_str
    canvas.put_str(region.x, region.y, &display, ...);
}
```

---

### 7. Main Event Loop (`lib.rs`)

Update region creation:

```rust
// In event_loop_async, replace:
let region = Region { x: 0, y: 0, width: cols, height: rows };

// With:
let region = Region::new(0, 0, cols as i32, rows as i32);
```

Both places: render and mouse event handling.

---

## Testing Strategy

1. **Compile check** - Ensure all type changes propagate correctly
2. **Basic rendering** - Run existing examples, verify display unchanged
3. **Mouse interaction** - Verify Switch click/hover still works
4. **Focus navigation** - Verify Tab/Enter still works
5. **Edge cases**:
   - Very small terminal sizes
   - Large widget trees
   - Nested containers

---

## Why This Matters

This stage lays the foundation for Stage 10 (Scrolling):

| Feature | Without i32 | With i32 |
|---------|-------------|----------|
| Scroll offset | Wraps at 0 (u16 underflow) | Works correctly |
| Off-screen widgets | Undefined behavior | Safe clipping |
| Hit testing | Complex bounds math | Simple `contains_point()` |
| Nested clipping | Manual tracking | Automatic via stack |

---

## Checklist

- [ ] Update Region to i32 with new methods
- [ ] Add clip_stack to Canvas
- [ ] Update put_char/put_str for i32 + clipping
- [ ] Update Widget trait signatures
- [ ] Update Vertical container
- [ ] Update Horizontal container
- [ ] Update Center/Middle wrappers
- [ ] Update Switch widget
- [ ] Update lib.rs event loop
- [ ] Run `cargo check`
- [ ] Test with existing examples
