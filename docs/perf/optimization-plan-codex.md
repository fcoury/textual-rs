# Rust TUI Library Optimization Plan

This document captures a comprehensive optimization analysis and a concrete roadmap for the TUI library.

## 1) Performance Analysis

### Memory Management
- Repeated allocations during render: `Content::from_markup` and `Content::wrap` are rebuilt every render in `Static` and `Container`.
- `Segment` always owns a `String` and `HashMap` meta even when empty (`Segment` is heavy for common paths).
- `BOX_CACHE` in `border_box.rs` can grow unbounded across many style combinations.
- `Canvas::cells` reinitialized on `Canvas::new` for every resize; the cell vector is fully rewritten in `clear()`.

### Computational Efficiency
- `Segment::cell_length()` recomputes Unicode width on every call; `split_at` rescans per call.
- `Content::render_line_with_spans` filters spans per line, per line (O(lines * spans)).
- `Content::wrap_line_with_spans` repeatedly searches for word positions.
- Layout placements are recomputed for every render even if sizes/styles did not change.

### Rendering Optimization
- Full-screen redraw on every render; no diffing or partial updates in `Canvas::flush`.
- Per-cell `execute!` for colors/attributes; expensive on Windows and slow terminals.
- Color conversion (`RgbaColor` -> `crossterm::Color`) happens per cell and per char.

## 2) Rust-Specific Optimizations

### Ownership and Borrowing
- `get_style()` and `get_meta()` return owned values, cloning data during layout/style resolution.
- `MessageEnvelope::new` allocates `String` each event.
- `WidgetMeta` and `ComputedStyle` cloning frequently in `style_resolver.rs`.

### Zero-Cost Abstractions
- Heavy use of trait objects (`Box<dyn Widget<M>>`) and dynamic dispatch; consider generic containers or enums for hot path widgets.
- Frequent iterator chains in hot rendering paths; manual loops could reduce overhead in tight loops.

### Concurrency
- Unbounded message channel in `AppContext`; can grow without backpressure if producers outpace render loop.
- Event loop currently single-threaded; parallelization is limited to background tasks only.

## 3) Architecture and Code Quality

### Data Structures
- `Segment.meta` uses `HashMap`; expensive for small metadata. A small, optional structure would be cheaper.
- Repeated style cloning when merging spans and rendering.
- `RenderCache` is rebuilt per render; border state could be cached on style change.

### API Design
- `get_style()` returning owned `ComputedStyle` is easy but expensive. A `get_style_ref()` would be faster but is a breaking change.
- Consider a per-widget render cache with explicit invalidation.

## 4) TUI-Specific Considerations

### Event Handling
- `needs_render = true` for every key/mouse event; mouse motion can trigger redundant renders.
- Hover/active updates should be debounced or coalesced.

### Layout and Styling
- Layout algorithms re-run every frame; cache placements when region/style unchanged.
- Title/subtitle parsing in `Static`/`Container` happens every frame.

### Text Measurement and Wrapping
- Unicode width is recomputed for every wrap; cache widths per line and per segment.
- Span-aware wrapping does repeated searches and span scans per line.

## 5) Dependencies and Ecosystem

### Crate Analysis
- `tokio` is heavyweight for simple apps; consider feature-gating async/event-loop.
- `crossterm` with `event-stream` may be optional for sync-only apps.

### Cross-Platform
- Windows console performance is sensitive to output size; diffed/batched output is highest leverage.

---

## Priority Matrix (Impact vs Effort)

| Priority | Optimization | Impact | Effort | Notes |
| --- | --- | --- | --- | --- |
| P0 | Diff rendering + output batching (`Canvas::flush`) | High | Medium | Biggest runtime win; reduces I/O dramatically |
| P1 | Cache parsed markup + wrapped strips in `Static`/`Container` | High | Medium | Cuts allocations and parsing per frame |
| P2 | Cache `Segment` cell width | Medium | Low | Reduces Unicode width recomputation |
| P3 | Reduce `Segment.meta` allocations | Medium | Medium | Use optional small collection or shared map |
| P4 | Reduce `ComputedStyle`/`WidgetMeta` cloning | Medium | High | API change risk |
| P5 | Bound/clear `BOX_CACHE` | Low | Low | Prevent unbounded growth |

---

## Benchmarking Plan

### Metrics
- Frame time (ms), CPU%, allocations/frame, bytes written to stdout, number of ANSI ops per frame.
- Memory growth under sustained input and resizing.

### Tools
- `cargo flamegraph` to identify hot paths.
- `perf stat` for syscalls/instructions.
- Optional: `tokio-console` or tracing for event-loop latency.

### Scenarios
- Large text with markup and wrapping.
- Rapid mouse movement and scroll wheel.
- High-frequency timers/intervals.
- Resizing the terminal repeatedly.

### Targets
- 50%+ reduction in bytes written per frame.
- 30%+ reduction in allocations per frame.
- Stable 60 FPS with input spam on typical terminals.

---

## Implementation Roadmap

1. Instrumentation: add frame timer + bytes-written counters for baseline.
2. `Canvas` diffing + batching: track previous frame, emit only changed cells, buffer output.
3. `Static`/`Container` render cache: store parsed content + wrapped strips keyed by `(text, markup, style, width)`.
4. Cache segment widths: store width in `Segment` to avoid recomputation.
5. Optional API improvements: reduce style/meta cloning via references or cached `WidgetMeta`.
6. Dependency trimming: feature-gate async runtime or event-stream.

---

## Risk Assessment

- Diff rendering: incorrect cursor positioning or missing resets can corrupt output; needs integration tests.
- Caches: stale caches can produce incorrect styles/layout if invalidation is wrong.
- API changes: altering `Widget` signatures is a breaking change.
- Concurrency: caching and shared state can introduce contention if not careful.

---

## Code Examples

### Cache `Segment` cell width
```rust
// Before
pub fn cell_length(&self) -> usize {
    self.text.width()
}

// After
pub struct Segment {
    text: String,
    cell_len: usize,
    // ...
}

impl Segment {
    pub fn new<S: Into<String>>(text: S) -> Self {
        let text = text.into();
        let cell_len = text.width();
        Self { text, cell_len, /* ... */ }
    }

    pub fn cell_length(&self) -> usize { self.cell_len }
}
```

### Cache parsed/wrapped content in `Static`
```rust
// Before: parse + wrap every render
let content = Content::from_markup(self.text()).unwrap_or_else(|_| Content::new(self.text()));
let lines = content.wrap(inner_width);

// After: cache by (text, style, width)
if cache.is_stale(self.text(), &style, inner_width) {
    cache.rebuild(self.text(), self.markup, &style, inner_width);
}
let lines = cache.lines();
```

### Batch + diff output in `Canvas::flush`
```rust
// Before: per-cell execute!/write!
for cell in row {
    execute!(out, SetForegroundColor(...))?;
    write!(out, "{}", cell.symbol)?;
}

// After: diff + buffered queue
let mut buf = Vec::with_capacity(estimate);
for (idx, (cell, prev)) in self.cells.iter().zip(self.prev_cells.iter()).enumerate() {
    if cell != prev { queue_cell(&mut buf, idx, cell)?; }
}
out.write_all(&buf)?;
self.prev_cells.copy_from_slice(&self.cells);
```
