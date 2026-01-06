# Plan: Fix Grid row-span and column-span Support

## Problem Summary

The Grid container's `row-span` and `column-span` CSS properties are **parsed correctly but never applied during layout**. Children that should span multiple rows/columns are rendered as single cells.

### Visual Comparison

**Expected (Python Textual):**
```
┌──────────────────────┬───────────┐
│                      │  Cell 2   │
│      Cell 1          ├───────────┤
│   (row-span: 3       │  Cell 3   │
│    column-span: 2)   ├───────────┤
│                      │  Cell 4   │
├──────────┬───────────┼───────────┤
│  Cell 5  │  Cell 6   │  Cell 7   │
└──────────┴───────────┴───────────┘
```

**Actual (Current Rust Implementation):**
```
┌──────────┐
│  Cell 1  │  (all cells stacked vertically,
├──────────┤   spans completely ignored)
│  Cell 2  │
├──────────┤
│  Cell 3  │
...
```

---

## Python Textual Reference Analysis

### How Python Textual Implements Grid Spans

Python Textual uses a **two-pass algorithm** with pre-computed data structures:

#### Data Structures

```python
# Maps (col, row) -> (widget, is_primary_cell)
cell_map: dict[tuple[int, int], tuple[Widget, bool]]

# Maps widget -> (col, row, col_span-1, row_span-1)
# NOTE: Stores span-1, not span!
cell_size_map: dict[Widget, tuple[int, int, int, int]]
```

#### Size Resolution

Python's `_resolve.py` returns `[(offset, width), ...]` with **pre-computed offsets**:

```python
columns = [(0, 26), (27, 26), (54, 26)]  # (offset, width) pairs
rows = [(0, 5), (6, 5), (12, 5), (18, 5)]
```

#### Span Width Calculation (Key Insight!)

Python uses `span-1` indexing for elegant offset math:

```python
# cell_size_map stores span-1, so for static1 with column_span=2:
# stored value is column_span=1

x = columns[column][0]                    # Start offset
x2, cell_width = columns[column + column_span]  # End column (span-1 indexing!)
width = cell_width + x2 - x               # Total span width

# Example for static1 (col=0, stored_span=1):
# x = columns[0][0] = 0
# columns[0+1] = (27, 26)
# width = 26 + 27 - 0 = 53 ✓ (includes gutter!)
```

#### Cell Mapping (Tetris Algorithm)

```python
for child in children:
    column_span = child.styles.column_span or 1
    row_span = child.styles.row_span or 1

    # Find first slot where widget fits
    while True:
        coords = widget_coords(column, row, column_span, row_span)
        if cell_map.keys().isdisjoint(coords):  # All cells free?
            for coord in coords:
                cell_map[coord] = (child, coord == primary_cell)
            break
        else:
            advance_to_next_cell()
```

### Visual Cell Mapping Example

For the grid example with 3 columns × 4 rows:

```
Column:    0         1         2
        ┌─────────────────┬─────────┐
Row 0:  │    static1      │ static2 │
        │  (spans 2×3)    │         │
        ├                 ┼─────────┤
Row 1:  │                 │ static3 │
        ├                 ┼─────────┤
Row 2:  │                 │ static4 │
        ├─────────┬───────┼─────────┤
Row 3:  │ static5 │static6│ static7 │
        └─────────┴───────┴─────────┘

cell_map = {
    (0,0): (static1, True),   # Primary cell
    (1,0): (static1, False),  # Span cell
    (0,1): (static1, False),
    (1,1): (static1, False),
    (0,2): (static1, False),
    (1,2): (static1, False),
    (2,0): (static2, True),
    (2,1): (static3, True),
    (2,2): (static4, True),
    (0,3): (static5, True),
    (1,3): (static6, True),
    (2,3): (static7, True),
}
```

---

## Our Implementation vs Python: Comparison

| Aspect | Python Textual | Our Approach | Parity? |
|--------|---------------|--------------|---------|
| Cell placement order | Left-to-right, top-to-bottom | Same | ✓ |
| Span collision detection | Check all spanned cells | Same | ✓ |
| Gutter handling | Between cells only | Same | ✓ |
| Fr distribution | Fraction-based remainder | Our Fraction type | ✓ |
| Offset storage | Pre-computed `[(offset, size), ...]` | Computed on-demand | ✓ (same result) |
| Widget tracking | `cell_map` stores widget refs | Boolean occupancy | Functional ✓ |

### Width Calculation Verification

Both approaches compute **identical span widths**:

**Python:** `cell_width + x2 - x` with span-1 indexing
```
For 2-column span: 26 + 27 - 0 = 53
```

**Ours:** Sum widths + gutters
```rust
for c in col..(col + col_span) {
    width += col_widths[c];
    if c > col { width += gutter_h; }
}
// For col=0, col_span=2: 26 + 1 + 26 = 53 ✓
```

---

## Root Cause Analysis

### What Works ✓

1. **CSS Declaration Parsing** (`crates/tcss/src/parser/mod.rs:184-185`)
   ```rust
   "column-span" => map(units::parse_u16, Declaration::ColumnSpan)(input)?,
   "row-span" => map(units::parse_u16, Declaration::RowSpan)(input)?,
   ```

2. **Declaration Types** (`crates/tcss/src/parser/stylesheet.rs:254-256`)
   ```rust
   ColumnSpan(u16),
   RowSpan(u16),
   ```

3. **Cascade Application** (`crates/tcss/src/parser/cascade.rs:314-319`)
   ```rust
   Declaration::ColumnSpan(n) => {
       style.grid_placement.column_span = *n;
   }
   Declaration::RowSpan(n) => {
       style.grid_placement.row_span = *n;
   }
   ```

4. **Storage in ComputedStyle** (`crates/tcss/src/types/grid.rs:59-75`)
   ```rust
   pub struct GridPlacement {
       pub column_span: u16,  // default: 1
       pub row_span: u16,     // default: 1
   }
   ```

### What's Missing ✗

**File:** `crates/textual/src/containers/grid.rs`

1. **`child_region()` function (lines 195-231)** - Only handles single cells:
   ```rust
   let width = col_widths.get(col).copied().unwrap_or(1);  // Single column!
   let height = row_heights.get(row).copied().unwrap_or(1); // Single row!
   ```

2. **Render loop (lines 260-291)** - Never reads span values:
   ```rust
   for child in &self.children {
       // MISSING: let style = child.get_style();
       // MISSING: let col_span = style.grid_placement.column_span;
       // MISSING: let row_span = style.grid_placement.row_span;

       let cell_region = child_region(col, row, ...);  // No span params!
       child.render(canvas, cell_region);

       col += 1;  // Always advances by 1, ignoring spans
       if col >= cols { col = 0; row += 1; }
   }
   ```

3. **No occupancy tracking** - No way to skip cells already occupied by spanning widgets

4. **`on_mouse()` (lines 361-421)** - Same issues as render loop

---

## Implementation Plan

### Step 1: Add ResolvedTrack Type (Performance Enhancement)

Pre-compute offsets like Python does, for efficient repeated lookups:

```rust
/// Pre-computed track (column or row) with offset and size.
#[derive(Debug, Clone, Copy)]
struct ResolvedTrack {
    offset: i32,
    size: i32,
}

impl Grid<M> {
    /// Distribute space and pre-compute offsets (matches Python's resolve()).
    fn resolve_tracks(
        &self,
        specs: &[Scalar],
        count: usize,
        available: i32,
        gutter: i32,
    ) -> Vec<ResolvedTrack> {
        let sizes = self.distribute_space(specs, count, available, gutter);
        let mut offset = 0;
        sizes
            .into_iter()
            .map(|size| {
                let track = ResolvedTrack { offset, size };
                offset += size + gutter;
                track
            })
            .collect()
    }
}
```

### Step 2: Create Occupancy Grid Helper

Track which cells are occupied (Tetris-style placement):

```rust
/// Tracks which grid cells are occupied by widgets.
struct OccupancyGrid {
    cells: Vec<Vec<bool>>,  // [row][col]
    rows: usize,
    cols: usize,
}

impl OccupancyGrid {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            cells: vec![vec![false; cols]; rows],
            rows,
            cols,
        }
    }

    /// Find the next unoccupied cell starting from (row, col).
    /// Scans left-to-right, top-to-bottom (matches Python).
    fn find_next_free(&self, mut row: usize, mut col: usize) -> Option<(usize, usize)> {
        while row < self.rows {
            while col < self.cols {
                if !self.cells[row][col] {
                    return Some((row, col));
                }
                col += 1;
            }
            col = 0;
            row += 1;
        }
        None
    }

    /// Mark cells as occupied for a widget spanning from (row, col).
    fn occupy(&mut self, row: usize, col: usize, row_span: usize, col_span: usize) {
        for r in row..(row + row_span).min(self.rows) {
            for c in col..(col + col_span).min(self.cols) {
                self.cells[r][c] = true;
            }
        }
    }

    /// Check if a widget can fit at (row, col) with given spans.
    fn can_fit(&self, row: usize, col: usize, row_span: usize, col_span: usize) -> bool {
        if row + row_span > self.rows || col + col_span > self.cols {
            return false;
        }
        for r in row..(row + row_span) {
            for c in col..(col + col_span) {
                if self.cells[r][c] {
                    return false;
                }
            }
        }
        true
    }
}
```

### Step 3: Modify `child_region()` Function

Use pre-computed tracks for efficient span calculation:

```rust
/// Calculate the region for a child at the given grid position with spans.
fn child_region(
    col: usize,
    row: usize,
    col_span: usize,
    row_span: usize,
    columns: &[ResolvedTrack],
    rows: &[ResolvedTrack],
    region: Region,
    gutter_h: i32,
    gutter_v: i32,
) -> Region {
    // Start position from pre-computed offsets
    let x = region.x + columns.get(col).map(|t| t.offset).unwrap_or(0);
    let y = region.y + rows.get(row).map(|t| t.offset).unwrap_or(0);

    // Calculate span width: sum of cell sizes + gutters between them
    let end_col = (col + col_span).min(columns.len());
    let width = if col < columns.len() {
        let start_offset = columns[col].offset;
        if end_col < columns.len() {
            // Width = next column's offset - our offset - trailing gutter
            columns[end_col].offset - start_offset - gutter_h
        } else {
            // Spans to edge: sum remaining sizes + gutters
            columns[col..].iter().map(|t| t.size).sum::<i32>()
                + (end_col - col).saturating_sub(1) as i32 * gutter_h
        }
    } else {
        0
    };

    // Calculate span height (same logic)
    let end_row = (row + row_span).min(rows.len());
    let height = if row < rows.len() {
        let start_offset = rows[row].offset;
        if end_row < rows.len() {
            rows[end_row].offset - start_offset - gutter_v
        } else {
            rows[row..].iter().map(|t| t.size).sum::<i32>()
                + (end_row - row).saturating_sub(1) as i32 * gutter_v
        }
    } else {
        0
    };

    Region { x, y, width, height }
}
```

### Step 4: Update `render()` Method

Replace simple linear placement with span-aware Tetris algorithm:

```rust
fn render(&self, canvas: &mut Canvas, region: Region) {
    if region.width <= 0 || region.height <= 0 {
        return;
    }

    canvas.push_clip(region);

    let visible_count = self.visible_children();
    if visible_count == 0 {
        canvas.pop_clip();
        return;
    }

    let cols = self.column_count();
    let rows = self.row_count(visible_count);

    // Get gutter values
    let gutter_v = self.resolve_scalar(&self.style.grid.gutter.0, region.height);
    let gutter_h = self.resolve_scalar(&self.style.grid.gutter.1, region.width);

    // Resolve tracks with pre-computed offsets
    let columns = self.resolve_tracks(
        &self.style.grid.column_widths,
        cols,
        region.width,
        gutter_h,
    );
    let row_tracks = self.resolve_tracks(
        &self.style.grid.row_heights,
        rows,
        region.height,
        gutter_v,
    );

    // Create occupancy grid for Tetris-style placement
    let mut occupancy = OccupancyGrid::new(rows, cols);
    let mut current_row = 0;
    let mut current_col = 0;

    for child in &self.children {
        if !child.is_visible() {
            continue;
        }

        // Get span values from child's computed style
        let child_style = child.get_style();
        let col_span = (child_style.grid_placement.column_span as usize).max(1);
        let row_span = (child_style.grid_placement.row_span as usize).max(1);

        // Find next position where this widget fits (Tetris algorithm)
        let placed = loop {
            match occupancy.find_next_free(current_row, current_col) {
                Some((r, c)) => {
                    current_row = r;
                    current_col = c;

                    // Clamp spans to grid bounds
                    let effective_col_span = col_span.min(cols - current_col);
                    let effective_row_span = row_span.min(rows - current_row);

                    if occupancy.can_fit(current_row, current_col, effective_row_span, effective_col_span) {
                        // Mark cells as occupied
                        occupancy.occupy(current_row, current_col, effective_row_span, effective_col_span);

                        // Calculate spanning region
                        let cell_region = child_region(
                            current_col,
                            current_row,
                            effective_col_span,
                            effective_row_span,
                            &columns,
                            &row_tracks,
                            region,
                            gutter_h,
                            gutter_v,
                        );

                        child.render(canvas, cell_region);

                        // Advance to next column for next widget
                        current_col += 1;
                        if current_col >= cols {
                            current_col = 0;
                            current_row += 1;
                        }
                        break true;
                    } else {
                        // Can't fit here, try next cell
                        current_col += 1;
                        if current_col >= cols {
                            current_col = 0;
                            current_row += 1;
                        }
                    }
                }
                None => break false, // No more space in grid
            }
        };

        if !placed || current_row >= rows {
            break; // Grid is full
        }
    }

    canvas.pop_clip();
}
```

### Step 5: Update `on_mouse()` Method

Apply the same span-aware logic for mouse hit-testing:

```rust
fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
    let mx = event.column as i32;
    let my = event.row as i32;

    if !region.contains_point(mx, my) {
        return None;
    }

    let visible_count = self.visible_children();
    if visible_count == 0 {
        return None;
    }

    let cols = self.column_count();
    let rows = self.row_count(visible_count);

    let gutter_v = self.resolve_scalar(&self.style.grid.gutter.0, region.height);
    let gutter_h = self.resolve_scalar(&self.style.grid.gutter.1, region.width);

    let columns = self.resolve_tracks(
        &self.style.grid.column_widths,
        cols,
        region.width,
        gutter_h,
    );
    let row_tracks = self.resolve_tracks(
        &self.style.grid.row_heights,
        rows,
        region.height,
        gutter_v,
    );

    // Mirror the render placement algorithm
    let mut occupancy = OccupancyGrid::new(rows, cols);
    let mut current_row = 0;
    let mut current_col = 0;

    for child in &mut self.children {
        if !child.is_visible() {
            continue;
        }

        let child_style = child.get_style();
        let col_span = (child_style.grid_placement.column_span as usize).max(1);
        let row_span = (child_style.grid_placement.row_span as usize).max(1);

        // Same placement logic as render()
        loop {
            match occupancy.find_next_free(current_row, current_col) {
                Some((r, c)) => {
                    current_row = r;
                    current_col = c;

                    let effective_col_span = col_span.min(cols - current_col);
                    let effective_row_span = row_span.min(rows - current_row);

                    if occupancy.can_fit(current_row, current_col, effective_row_span, effective_col_span) {
                        occupancy.occupy(current_row, current_col, effective_row_span, effective_col_span);

                        let cell_region = child_region(
                            current_col,
                            current_row,
                            effective_col_span,
                            effective_row_span,
                            &columns,
                            &row_tracks,
                            region,
                            gutter_h,
                            gutter_v,
                        );

                        // Check if mouse is in this cell
                        if cell_region.contains_point(mx, my) {
                            if let Some(msg) = child.on_mouse(event, cell_region) {
                                return Some(msg);
                            }
                        }

                        current_col += 1;
                        if current_col >= cols {
                            current_col = 0;
                            current_row += 1;
                        }
                        break;
                    } else {
                        current_col += 1;
                        if current_col >= cols {
                            current_col = 0;
                            current_row += 1;
                        }
                    }
                }
                None => break,
            }
        }

        if current_row >= rows {
            break;
        }
    }

    None
}
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `crates/textual/src/containers/grid.rs` | Add ResolvedTrack, OccupancyGrid, modify child_region, update render/on_mouse |

---

## Testing

### Unit Tests to Add

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occupancy_grid_basic() {
        let mut grid = OccupancyGrid::new(3, 3);
        assert!(grid.can_fit(0, 0, 2, 2));
        grid.occupy(0, 0, 2, 2);
        assert!(!grid.can_fit(0, 0, 1, 1));
        assert!(!grid.can_fit(1, 1, 1, 1));
        assert!(grid.can_fit(0, 2, 1, 1));
        assert!(grid.can_fit(2, 0, 1, 1));
    }

    #[test]
    fn test_find_next_free() {
        let mut grid = OccupancyGrid::new(2, 3);
        grid.occupy(0, 0, 1, 2);
        assert_eq!(grid.find_next_free(0, 0), Some((0, 2)));
        grid.occupy(0, 2, 1, 1);
        assert_eq!(grid.find_next_free(0, 0), Some((1, 0)));
    }

    #[test]
    fn test_resolved_track_offsets() {
        // Simulating 3 columns of 26 each with gutter 1
        let tracks = vec![
            ResolvedTrack { offset: 0, size: 26 },
            ResolvedTrack { offset: 27, size: 26 },
            ResolvedTrack { offset: 54, size: 26 },
        ];

        // Verify offset progression includes gutter
        assert_eq!(tracks[0].offset, 0);
        assert_eq!(tracks[1].offset, 27);  // 0 + 26 + 1 gutter
        assert_eq!(tracks[2].offset, 54);  // 27 + 26 + 1 gutter
    }

    #[test]
    fn test_span_width_calculation() {
        let tracks = vec![
            ResolvedTrack { offset: 0, size: 26 },
            ResolvedTrack { offset: 27, size: 26 },
            ResolvedTrack { offset: 54, size: 26 },
        ];
        let gutter = 1;

        // 2-column span starting at col 0
        // Width should be: col0 + gutter + col1 = 26 + 1 + 26 = 53
        let col = 0;
        let col_span = 2;
        let end_col = col + col_span;

        let width = tracks[end_col].offset - tracks[col].offset - gutter;
        assert_eq!(width, 53);
    }
}
```

### Manual Testing

```bash
# Run the grid example
cargo run --example grid

# Expected: Cell 1 spans 3 rows × 2 columns (magenta)
# Cells 2, 3, 4 stacked on right
# Cells 5, 6, 7 in bottom row
```

---

## Edge Cases to Handle

1. **Span exceeds grid bounds** - Clamp to available space
2. **Widget can't fit anywhere** - Skip widget (matches Python behavior)
3. **All cells occupied** - Stop placing children
4. **Zero or negative spans** - Treat as 1
5. **Dynamic row count** - For auto rows with spans, may need simulation

---

## Implementation Order

1. Add `ResolvedTrack` struct
2. Add `resolve_tracks()` method
3. Add `OccupancyGrid` struct and methods
4. Modify `child_region()` signature and implementation
5. Update `render()` with span-aware placement
6. Update `on_mouse()` with matching logic
7. Add unit tests
8. Test with `examples/grid.rs`

---

## Verification Checklist

- [ ] `cargo test` passes
- [ ] `cargo run --example grid` shows correct spanning layout
- [ ] Cell 1 spans 3 rows × 2 columns with magenta tint
- [ ] Cells 2, 3, 4 stack correctly in rightmost column
- [ ] Cells 5, 6, 7 appear in bottom row
- [ ] Mouse events route to correct spanning cells
- [ ] Span width matches Python calculation: 26 + 1 + 26 = 53 for 2-col span
