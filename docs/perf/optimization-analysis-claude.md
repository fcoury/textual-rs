# Rust TUI Library Optimization Analysis

## Executive Summary

This document provides a comprehensive analysis of the textual-rs TUI library, identifying optimization opportunities across memory management, rendering efficiency, Rust-specific patterns, and architectural design. The analysis covers three crates: `textual` (main TUI framework), `tcss` (CSS parser/styling), and `rich` (text rendering).

---

## 1. Performance Analysis

### 1.1 Memory Management

#### Identified Issues

**HIGH IMPACT: Excessive Cloning in Segment/Strip Operations**

Location: `crates/textual/src/segment.rs:227-277`, `strip.rs:84-144`

```rust
// segment.rs:split_at - Current implementation
pub fn split_at(&self, cut: usize) -> (Segment, Segment) {
    // Multiple clones of style and meta HashMap
    (
        Segment {
            text: left_text.to_string(),  // Allocation
            style: self.style.clone(),     // Clone Style (contains Option<RgbaColor>)
            meta: self.meta.clone(),       // Clone HashMap
        },
        Segment {
            text: right_text.to_string(),  // Allocation
            style: self.style.clone(),     // Clone again
            meta: self.meta.clone(),       // Clone again
        },
    )
}
```

**Optimization**: Use `Cow<str>` for text or `Arc<str>` for shared text, `Arc<Style>` for shared styles.

**MEDIUM IMPACT: String Allocations in Content Parsing**

Location: `crates/textual/src/content.rs:85-105`

```rust
// from_markup allocates multiple times
pub fn from_markup(markup: &str) -> Result<Self, rich::RichParseError> {
    let parsed = rich::ParsedMarkup::parse(markup)?;
    let spans: Vec<InternalSpan> = parsed  // New Vec allocation
        .spans()
        .iter()
        .map(|s| InternalSpan {
            start: s.start,
            end: s.end,
            style: Self::convert_rich_style(&s.style),  // Style conversion
            meta: s.meta.clone(),  // HashMap clone
        })
        .collect();
    Ok(Self {
        text: parsed.text().to_string(),  // String allocation
        // ...
    })
}
```

**MEDIUM IMPACT: Canvas Cell Array**

Location: `crates/textual/src/canvas.rs:125-141`

```rust
// Canvas allocates width*height Cell structs
pub fn new(width: u16, height: u16) -> Self {
    Self {
        cells: vec![Cell { ... }; (width * height) as usize],  // Large allocation
        // ...
    }
}
```

For a 200x50 terminal = 10,000 cells × ~24 bytes = 240KB per canvas. The `clear()` method uses `.fill()` which is good, but the canvas is recreated on resize.

**LOW IMPACT: ComputedStyle Size**

Location: `crates/tcss/src/types/mod.rs:53-164`

The `ComputedStyle` struct is large (~500+ bytes) with many optional fields:
- 11 `Option<RgbaColor>` fields (each ~16 bytes)
- Multiple `Spacing`, `Border`, `GridStyle` structs
- This is cloned frequently during style cascade

#### Recommendations

1. **Implement a string interner** for repeated text patterns (widget IDs, CSS classes)
2. **Use `Arc<ComputedStyle>`** for shared styles between widgets
3. **Pool/reuse Canvas buffers** instead of reallocating on resize
4. **Consider `smallvec`** for Strip segments (most strips have 1-3 segments)

### 1.2 Computational Efficiency

#### Hot Path Analysis

**Hot Path 1: Layout Calculation** - `layouts/vertical.rs:20-163`

Current complexity: O(n) children × 2 passes = O(2n)

The two-pass approach (measure then place) is necessary but could be optimized:

```rust
// First pass: calculate space
for (i, (_child_index, child_style, desired_size)) in children.iter().enumerate() {
    // Repeated style access and margin calculation
    let margin_top = child_style.margin.top.value as i32;
    let margin_bottom = child_style.margin.bottom.value as i32;
    // ...
}

// Second pass: place children (almost identical iteration)
for (i, (child_index, child_style, desired_size)) in children.iter().enumerate() {
    // Same style access again
}
```

**Optimization**: Single-pass layout with deferred fraction resolution.

**Hot Path 2: Style Resolution** - `style_resolver.rs:39-92`

```rust
pub fn resolve_dirty_styles<M>(
    widget: &mut dyn Widget<M>,
    stylesheet: &StyleSheet,
    // ...
) {
    // Creates WidgetMeta on every call (allocates String for type_name)
    let meta = widget.get_meta();  // WidgetMeta { type_name: String::new(), ... }

    // CSS cascade involves matching rules
    let mut style = compute_style(&meta, ancestors, stylesheet, theme);

    // Recursion into children
    widget.for_each_child(&mut |child| {
        resolve_dirty_styles(child, ...);
    });
}
```

**Issue**: `get_meta()` creates a new `WidgetMeta` with `type_name: String` each time.

**Hot Path 3: Rendering** - `containers/container.rs:179-275`

```rust
fn render(&self, canvas: &mut Canvas, region: Region) {
    // Creates RenderCache every render
    let cache = RenderCache::new(&self.style);

    // For each line, constructs Strip
    for y in 0..height {
        let strip = cache.render_line(y, height, width, ...);  // Allocates Strip
        canvas.render_strip(&strip, ...);
    }

    // Compute placements every render
    for placement in self.compute_child_placements(inner_region) {
        // ...
    }
}
```

**Issue**: Layout placements are recomputed every render even when nothing changed.

#### Recommendations

1. **Cache layout placements** in Container, invalidate on resize/style change
2. **Make `type_name` return `&'static str`** instead of `String`
3. **Pool RenderCache** or make it part of Container state
4. **Implement dirty rectangles** - only re-render changed regions

### 1.3 Rendering Optimization

#### Current Terminal I/O

Location: `canvas.rs:285-353`

```rust
pub fn flush(&mut self) -> std::io::Result<()> {
    let mut out = std::io::stdout();
    execute!(out, cursor::MoveTo(0, 0))?;

    // Per-cell loop with potential escape codes
    for (row_idx, row) in rows.into_iter().enumerate() {
        for cell in row {
            // Check and potentially emit fg/bg/attrs escape codes
            if cell.fg != last_fg { execute!(out, SetForegroundColor(...))?; }
            if cell.bg != last_bg { execute!(out, SetBackgroundColor(...))?; }
            if cell.attrs != last_attrs { /* multiple execute! calls */ }
            write!(out, "{}", cell.symbol)?;
        }
    }
    out.flush()?;
}
```

**Issues**:
1. Individual `write!` calls for each character
2. No buffering strategy beyond stdout's default
3. Attribute reset resets colors unnecessarily

**Optimization**:

```rust
// Batch characters with same style
fn flush_optimized(&mut self) -> std::io::Result<()> {
    let mut buffer = String::with_capacity(self.cells.len() * 2);
    let mut pending_chars = String::new();
    let mut current_style = StyleState::default();

    for cell in &self.cells {
        let cell_style = StyleState::from(cell);
        if cell_style == current_style {
            pending_chars.push(cell.symbol);
        } else {
            // Flush pending, emit style change
            buffer.push_str(&pending_chars);
            buffer.push_str(&cell_style.escape_sequence());
            pending_chars.clear();
            pending_chars.push(cell.symbol);
            current_style = cell_style;
        }
    }
    // Single write
    stdout().write_all(buffer.as_bytes())
}
```

#### Differential Rendering

Currently missing. The library redraws entire screen every frame:

```rust
// lib.rs:389
canvas.clear();
let region = Region::from_u16(0, 0, cols, rows);
tree.root().render(&mut canvas, region);
canvas.flush()?;
```

**Recommendation**: Implement double buffering with diff detection:

```rust
struct DoubleBufferedCanvas {
    front: Vec<Cell>,
    back: Vec<Cell>,
}

fn flush_diff(&mut self) -> io::Result<()> {
    for (i, (front, back)) in self.front.iter().zip(&self.back).enumerate() {
        if front != back {
            // Move cursor and render only changed cells
            let (x, y) = index_to_coords(i);
            execute!(stdout(), cursor::MoveTo(x, y))?;
            // render cell
        }
    }
    std::mem::swap(&mut self.front, &mut self.back);
}
```

---

## 2. Rust-Specific Optimizations

### 2.1 Ownership and Borrowing

**Unnecessary Clones**

Location: `segment.rs:84-95` (Style::apply)

```rust
pub fn apply(&self, other: &Style) -> Style {
    Style {
        fg: other.fg.clone().or_else(|| self.fg.clone()),  // Two potential clones
        bg: other.bg.clone().or_else(|| self.bg.clone()),
        // ...
    }
}
```

**Better Pattern**:
```rust
pub fn apply(&self, other: &Style) -> Style {
    Style {
        fg: other.fg.as_ref().or(self.fg.as_ref()).cloned(),  // Single clone
        // ...
    }
}
```

**Missing Cow<str> Opportunities**

Location: `segment.rs:117-124`

```rust
pub struct Segment {
    text: String,  // Always owned
    // ...
}

impl Segment {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self { text: text.into(), ... }  // Always allocates
    }
}
```

**Better Pattern**:
```rust
use std::borrow::Cow;

pub struct Segment<'a> {
    text: Cow<'a, str>,
    // ...
}

impl<'a> Segment<'a> {
    pub fn borrowed(text: &'a str) -> Self {
        Self { text: Cow::Borrowed(text), ... }
    }

    pub fn into_owned(self) -> Segment<'static> {
        Segment { text: Cow::Owned(self.text.into_owned()), ... }
    }
}
```

### 2.2 Zero-Cost Abstractions

**Trait Object vs Generics**

Location: `widget.rs` - Widget trait

```rust
pub trait Widget<M> {
    fn render(&self, canvas: &mut Canvas, region: Region);
    // ... 30+ methods
}

// Used as Box<dyn Widget<M>> everywhere
children: Vec<Box<dyn Widget<M>>>
```

**Trade-off Analysis**:
- Current approach: Dynamic dispatch, flexible composition
- Generic approach: Static dispatch, monomorphization bloat

**Recommendation**: Keep trait objects for flexibility but consider:
1. Mark hot-path methods with `#[inline]`
2. Use `enum_dispatch` for common widget types to eliminate vtable calls
3. Consider a sealed `WidgetImpl` for internal hot paths

**Iterator Efficiency**

Location: `strip.rs:175-178`

```rust
pub fn apply_style(&self, style: &Style) -> Strip {
    let segments: Vec<_> = self.segments.iter()
        .map(|s| s.apply_style(style))
        .collect();  // Allocation
    Strip::from_segments(segments)
}
```

**Better Pattern** (when strip is consumed):
```rust
pub fn apply_style_in_place(&mut self, style: &Style) {
    for segment in &mut self.segments {
        *segment = segment.apply_style(style);
    }
}
```

### 2.3 Concurrency

**Current State**: Single-threaded with async event loop (tokio).

**Opportunities**:

1. **Parallel layout calculation** for independent subtrees:
```rust
use rayon::prelude::*;

fn compute_child_placements_parallel(&self, region: Region) -> Vec<WidgetPlacement> {
    let children_data: Vec<_> = self.children.par_iter()
        .enumerate()
        .filter(|(_, c)| c.is_visible())
        .map(|(i, c)| (i, c.get_style(), c.desired_size()))
        .collect();
    // ...
}
```

2. **Background style resolution**:
```rust
// Resolve styles in background while user interacts
tokio::spawn(async move {
    let resolved_styles = resolve_subtree_styles(&widget_tree);
    style_channel.send(resolved_styles).await;
});
```

**Recommendation**: Low priority - TUI rendering is typically fast enough single-threaded.

---

## 3. Architecture and Code Quality

### 3.1 Data Structures

**Vec vs SmallVec**

Location: `strip.rs:23-28`

```rust
pub struct Strip {
    segments: Vec<Segment>,  // Heap allocation even for 1 segment
    cell_length: usize,
}
```

**Recommendation**:
```rust
use smallvec::SmallVec;

pub struct Strip {
    segments: SmallVec<[Segment; 4]>,  // Inline storage for ≤4 segments
    cell_length: usize,
}
```

Most strips have 1-3 segments. This eliminates heap allocation for the common case.

**HashMap vs FxHashMap**

Location: `segment.rs:123`

```rust
meta: HashMap<String, String>,  // Standard HashMap with SipHash
```

**Recommendation**: Use `rustc_hash::FxHashMap` for small string keys:
```rust
use rustc_hash::FxHashMap;
meta: FxHashMap<String, String>,  // ~2x faster for small keys
```

**Focus Path Optimization**

Location: `tree.rs:29-32`

```rust
pub struct FocusPath {
    indices: Vec<usize>,  // Heap allocated
}
```

Widget trees rarely exceed 10 levels deep.

**Recommendation**:
```rust
pub struct FocusPath {
    indices: SmallVec<[usize; 8]>,  // Inline for depth ≤ 8
}
```

### 3.2 API Design

**Builder Pattern Efficiency**

Location: `container.rs:50-69`

```rust
pub fn with_id(mut self, id: impl Into<String>) -> Self {
    self.id = Some(id.into());  // Moves self, returns owned
    self
}
```

This is good! But could be even better with lazy evaluation:

```rust
pub fn with_id<S: Into<String>>(mut self, id: S) -> Self {
    self.id = Some(id.into());
    self
}

// Also provide set_ variant for mutation without ownership
pub fn set_id(&mut self, id: impl Into<String>) {
    self.id = Some(id.into());
}
```

**Widget Trait Bounds**

Location: `widget.rs:19-326`

The Widget trait has ~30 methods with default implementations. This is clean but:
1. Large vtable per widget type
2. No way to specialize hot paths

**Recommendation**: Split into smaller traits:
```rust
trait WidgetCore<M> {
    fn render(&self, canvas: &mut Canvas, region: Region);
    fn desired_size(&self) -> Size;
}

trait WidgetEvents<M>: WidgetCore<M> {
    fn on_event(&mut self, key: KeyCode) -> Option<M>;
    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M>;
}

trait WidgetFocus<M>: WidgetCore<M> {
    fn is_focusable(&self) -> bool;
    fn focus_nth(&mut self, n: usize) -> bool;
    // ...
}
```

---

## 4. TUI-Specific Considerations

### 4.1 Event Handling

**Current Implementation**

Location: `lib.rs:400-463`

```rust
tokio::select! {
    maybe_event = event_stream.next() => {
        match maybe_event {
            Some(Ok(Event::Key(key_event))) => {
                if let Some(msg) = tree.dispatch_key(key_event.code) {
                    // ...
                }
                self.on_key(key_event.code);
                needs_render = true;  // Always re-render on key
            }
            // ...
        }
    }
}
```

**Issues**:
1. No key event batching (rapid typing causes multiple renders)
2. `needs_render = true` even for no-op keys

**Recommendation**:
```rust
// Batch events within a time window
let batch_duration = Duration::from_millis(8);  // ~120fps max
let mut pending_events = Vec::new();

loop {
    tokio::select! {
        event = event_stream.next() => {
            pending_events.push(event);
        }
        _ = tokio::time::sleep(batch_duration) => {
            if !pending_events.is_empty() {
                process_event_batch(&mut pending_events);
                if events_caused_changes {
                    render_frame();
                }
            }
        }
    }
}
```

### 4.2 Layout and Styling

**Layout Recalculation**

Current: Layout is recalculated every render in `compute_child_placements()`.

**Recommendation**: Cache placements, invalidate on:
- Parent resize
- Child visibility change
- Style change affecting layout (margin, padding, width/height)

```rust
pub struct Container<M> {
    // ...
    cached_placements: Option<Vec<WidgetPlacement>>,
    layout_dirty: bool,
}

impl Container<M> {
    fn get_placements(&mut self, region: Region) -> &[WidgetPlacement] {
        if self.layout_dirty || self.cached_region != Some(region) {
            self.cached_placements = Some(self.compute_child_placements(region));
            self.cached_region = Some(region);
            self.layout_dirty = false;
        }
        self.cached_placements.as_ref().unwrap()
    }
}
```

**Text Measurement**

Location: `segment.rs:202-204`

```rust
pub fn cell_length(&self) -> usize {
    self.text.width()  // unicode_width crate, iterates entire string
}
```

**Recommendation**: Cache cell_length in Segment:
```rust
pub struct Segment {
    text: String,
    cell_length: usize,  // Cached on construction
    // ...
}
```

---

## 5. Dependencies and Ecosystem

### 5.1 Current Dependencies

```toml
[dependencies]
crossterm = { version = "0.27", features = ["event-stream"] }
tokio = { version = "1.0", features = ["rt", "rt-multi-thread", "sync", "time", "macros"] }
futures = "0.3"
unicode-width = "0.2"
phf = { version = "0.11", features = ["macros"] }
once_cell = "1.19"
nom = "7.1"  # In tcss
```

### 5.2 Dependency Analysis

**crossterm (0.27)**
- Good choice for cross-platform terminal handling
- `event-stream` feature adds async support
- **No changes recommended**

**tokio**
- Full async runtime is heavy (~1MB compile time contribution)
- Consider: `tokio = { features = ["rt", "sync", "time"] }` (remove `rt-multi-thread` if single-threaded is sufficient)
- For simpler apps: Consider `async-std` or `smol` as lighter alternatives

**futures (0.3)**
- Only using `StreamExt` for event stream
- Could use `futures-lite` instead (~5x smaller compile time)

**nom (7.1)** - in tcss
- Full parser combinator library
- Consider: For CSS parsing, `pest` or hand-written parser might be faster
- **Low priority** - parsing happens once at startup

**unicode-width (0.2)**
- Essential for correct CJK/emoji handling
- **No changes recommended**

### 5.3 Recommended Additions

```toml
# Performance
smallvec = "1.11"       # Inline small vectors
rustc-hash = "1.1"      # Fast HashMap for strings
bumpalo = "3.14"        # Arena allocation for per-frame data

# Development
criterion = "0.5"       # Benchmarking
tracing = "0.1"         # Performance tracing
```

### 5.4 Compilation Time

Current: Full workspace clean build ~15-20 seconds (estimated)

**Recommendations**:
1. Split tcss parser into separate compilation unit (changes less frequently)
2. Use `cargo build --release -j 1` for benchmarking (more reproducible)
3. Consider `cargo-chef` for Docker builds

---

## 6. Deliverables

### 6.1 Priority Matrix

| Optimization | Impact | Effort | Risk | Priority |
|-------------|--------|--------|------|----------|
| Differential rendering | HIGH | HIGH | MEDIUM | P1 |
| Cache layout placements | HIGH | MEDIUM | LOW | P1 |
| SmallVec for Strip/FocusPath | MEDIUM | LOW | LOW | P1 |
| Cache cell_length in Segment | MEDIUM | LOW | LOW | P1 |
| Batch terminal writes | MEDIUM | MEDIUM | LOW | P2 |
| Cow<str> for Segment text | MEDIUM | HIGH | MEDIUM | P2 |
| Arc<ComputedStyle> | MEDIUM | MEDIUM | MEDIUM | P2 |
| FxHashMap for meta | LOW | LOW | LOW | P2 |
| Static type_name | LOW | LOW | LOW | P3 |
| Event batching | LOW | MEDIUM | LOW | P3 |
| Parallel layout | LOW | HIGH | HIGH | P4 |

### 6.2 Benchmarking Plan

**Metrics to Measure**:

1. **Frame render time** (target: <5ms for 200x50 terminal)
   - Canvas clear → widget render → flush

2. **Memory per widget** (target: <1KB baseline)
   - Measure with `std::mem::size_of`

3. **Event latency** (target: <1ms from keypress to handler)
   - Time from crossterm event to `on_event` call

4. **Style resolution time** (target: <100μs per dirty widget)
   - Time in `resolve_dirty_styles`

**Benchmark Setup**:

```rust
// benches/render.rs
use criterion::{criterion_group, Criterion};

fn bench_canvas_render(c: &mut Criterion) {
    c.bench_function("canvas_200x50_full_render", |b| {
        let mut canvas = Canvas::new(200, 50);
        b.iter(|| {
            canvas.clear();
            // render typical widget tree
            canvas.flush().unwrap();
        });
    });
}

fn bench_strip_operations(c: &mut Criterion) {
    c.bench_function("strip_crop_1000_chars", |b| {
        let strip = Strip::from_segment(Segment::new("x".repeat(1000)));
        b.iter(|| strip.crop(100, 500));
    });
}
```

### 6.3 Implementation Roadmap

**Phase 1: Quick Wins (1-2 days)**
1. Add `cell_length` cache to Segment
2. Replace `Vec` with `SmallVec` in Strip and FocusPath
3. Make `type_name()` return `&'static str`
4. Add benchmarks with criterion

**Phase 2: Layout Caching (2-3 days)**
1. Add `cached_placements` to Container
2. Implement invalidation logic
3. Add dirty tracking for layout-affecting properties

**Phase 3: Rendering Optimization (3-5 days)**
1. Implement double-buffered Canvas
2. Add differential rendering (only redraw changed cells)
3. Batch terminal writes into single buffer

**Phase 4: Memory Optimization (3-5 days)**
1. Investigate `Cow<str>` for Segment
2. Add Arc<ComputedStyle> for style sharing
3. Implement string interning for CSS identifiers

### 6.4 Risk Assessment

| Change | Breaking Change? | Compatibility Risk |
|--------|-----------------|-------------------|
| SmallVec internals | No | None |
| Cache cell_length | No | None |
| Differential rendering | No | Low (visual glitches possible) |
| Layout caching | No | Medium (stale layouts if invalidation wrong) |
| Cow<str> Segment | Yes (lifetime) | High (API change) |
| Arc<ComputedStyle> | No | Low |
| Event batching | No | Medium (timing-sensitive code) |

**Migration Strategy for Breaking Changes**:
1. Add new API alongside old
2. Deprecate old API
3. Provide migration guide
4. Remove after 2 minor versions

### 6.5 Code Examples

**Before: Segment split_at (current)**
```rust
pub fn split_at(&self, cut: usize) -> (Segment, Segment) {
    // ... find byte_pos ...
    (
        Segment {
            text: left_text.to_string(),
            style: self.style.clone(),
            meta: self.meta.clone(),
        },
        Segment {
            text: right_text.to_string(),
            style: self.style.clone(),
            meta: self.meta.clone(),
        },
    )
}
```

**After: Segment split_at (optimized)**
```rust
pub fn split_at(&self, cut: usize) -> (Segment, Segment) {
    // ... find byte_pos ...
    let left_text = &self.text[..byte_pos];
    let right_text = &self.text[byte_pos..];

    // Share style via Rc (or keep clone if style is small)
    let shared_style = self.style.clone();
    let shared_meta = if self.meta.is_empty() {
        HashMap::new()
    } else {
        self.meta.clone()
    };

    (
        Segment {
            text: left_text.to_string(),
            cell_length: left_text.width(),  // Cache!
            style: shared_style.clone(),
            meta: shared_meta.clone(),
        },
        Segment {
            text: right_text.to_string(),
            cell_length: right_text.width(),  // Cache!
            style: shared_style,
            meta: shared_meta,
        },
    )
}
```

**Before: Canvas flush (current)**
```rust
for cell in row {
    if cell.fg != last_fg {
        execute!(out, SetForegroundColor(...))?;
    }
    if cell.bg != last_bg {
        execute!(out, SetBackgroundColor(...))?;
    }
    write!(out, "{}", cell.symbol)?;  // Per-character write
}
```

**After: Canvas flush (batched)**
```rust
let mut buffer = String::with_capacity(self.cells.len() * 2);
let mut run_start = 0;
let mut current_style = CellStyle::default();

for (i, cell) in self.cells.iter().enumerate() {
    let cell_style = CellStyle::from(cell);
    if cell_style != current_style || i == self.cells.len() - 1 {
        // Emit escape codes for style change
        if i > run_start {
            buffer.push_str(&build_style_escape(&current_style));
            for j in run_start..i {
                buffer.push(self.cells[j].symbol);
            }
        }
        current_style = cell_style;
        run_start = i;
    }
}
stdout().write_all(buffer.as_bytes())?;
stdout().flush()
```

---

## Conclusion

This TUI library has a solid foundation with good architectural decisions (persistent widget tree, dirty-flag styling, focus path caching). The main optimization opportunities are:

1. **Differential rendering** - Biggest impact, moderate effort
2. **Layout caching** - High impact, prevents redundant work
3. **Memory optimizations** - SmallVec and caching cell_length are quick wins
4. **Terminal I/O batching** - Medium impact, reduces syscall overhead

The recommended approach is to implement Phase 1 (quick wins) first to establish benchmarks, then tackle differential rendering which will provide the most visible performance improvement for users.
