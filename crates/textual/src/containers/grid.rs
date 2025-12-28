//! Grid container for CSS Grid-like layouts.
//!
//! Implements a 2D grid layout with support for:
//! - Fixed column/row counts via `grid-size`
//! - Flexible column/row sizes via `grid-columns` and `grid-rows`
//! - Gutter spacing via `grid-gutter`
//! - Column/row spanning for children via `row-span` and `column-span`
//!
//! ## CSS Properties
//!
//! ```css
//! Grid {
//!     grid-size: 3;              /* 3 columns, auto rows */
//!     grid-size: 3 2;            /* 3 columns, 2 rows */
//!     grid-columns: 1fr 2fr 1fr; /* flexible widths */
//!     grid-rows: 5 auto;         /* fixed + auto heights */
//!     grid-gutter: 1;            /* 1 cell spacing */
//!     grid-gutter: 1 2;          /* vertical horizontal */
//! }
//!
//! /* Child spanning */
//! #my-widget {
//!     row-span: 2;               /* span 2 rows */
//!     column-span: 3;            /* span 3 columns */
//! }
//! ```

use tcss::types::{Scalar, Unit};
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

use crate::fraction::Fraction;
use crate::{Canvas, KeyCode, MouseEvent, Region, Size, Widget};

/// Pre-computed track (column or row) with offset and size.
///
/// Used to efficiently calculate spanning regions without re-computing
/// offsets for each cell. Matches Python Textual's `_resolve.py` output.
#[derive(Debug, Clone, Copy)]
struct ResolvedTrack {
    /// Offset from the start of the grid region (in cells).
    offset: i32,
    /// Size of this track (in cells).
    size: i32,
}

/// Tracks which grid cells are occupied by widgets.
///
/// Implements Tetris-style placement: widgets are placed left-to-right,
/// top-to-bottom, skipping cells that are already occupied by spanning widgets.
struct OccupancyGrid {
    cells: Vec<Vec<bool>>, // [row][col]
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
    /// Scans left-to-right, top-to-bottom (matches Python Textual).
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

/// A grid container that arranges children in a 2D grid.
///
/// Children are placed left-to-right, top-to-bottom.
/// Grid size and layout are controlled via CSS.
pub struct Grid<M> {
    children: Vec<Box<dyn Widget<M>>>,
    style: ComputedStyle,
    dirty: bool,
    id: Option<String>,
}

impl<M> Grid<M> {
    /// Create a new Grid with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            style: ComputedStyle::default(),
            dirty: true,
            id: None,
        }
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Get the number of columns from style, defaulting to 1.
    fn column_count(&self) -> usize {
        self.style.grid.columns.unwrap_or(1) as usize
    }

    /// Get the number of rows, either from style or calculated.
    fn row_count(&self, visible_children: usize) -> usize {
        if let Some(rows) = self.style.grid.rows {
            rows as usize
        } else {
            let cols = self.column_count();
            if cols == 0 {
                1
            } else {
                (visible_children + cols - 1) / cols
            }
        }
    }

    /// Count visible children.
    fn visible_children(&self) -> usize {
        self.children.iter().filter(|c| c.is_visible()).count()
    }

    /// Resolve a scalar value to cells given the available space.
    fn resolve_scalar(&self, scalar: &Scalar, available: i32) -> i32 {
        match scalar.unit {
            Unit::Cells => scalar.value as i32,
            Unit::Percent => ((scalar.value / 100.0) * available as f64) as i32,
            Unit::Auto => available,     // Auto fills available space
            Unit::Fraction => available, // fr handled specially in distribution
            _ => scalar.value as i32,    // Default to treating as cells
        }
    }

    /// Distribute available space among tracks (columns or rows).
    ///
    /// Handles fr units, fixed sizes, and auto sizing.
    fn distribute_space(
        &self,
        specs: &[Scalar],
        count: usize,
        available: i32,
        gutter: i32,
    ) -> Vec<i32> {
        if count == 0 {
            return vec![];
        }

        // Total gutter space
        let total_gutter = (count.saturating_sub(1)) as i32 * gutter;
        let available_for_tracks = (available - total_gutter).max(0);

        // Get specs for each track (cycling if needed)
        let track_specs: Vec<Scalar> = (0..count)
            .map(|i| {
                if specs.is_empty() {
                    Scalar::cells(1.0) // Default: equal distribution
                } else {
                    specs[i % specs.len()]
                }
            })
            .collect();

        // First pass: calculate fixed sizes and sum fr values
        let mut sizes = vec![0i32; count];
        let mut total_fr = 0.0;
        let mut remaining = available_for_tracks;

        for (i, spec) in track_specs.iter().enumerate() {
            match spec.unit {
                Unit::Fraction => {
                    total_fr += spec.value;
                }
                Unit::Auto => {
                    // Auto gets equal share of remaining space
                    // Will be calculated after fixed sizes
                }
                _ => {
                    let size = self.resolve_scalar(spec, available_for_tracks);
                    sizes[i] = size;
                    remaining -= size;
                }
            }
        }

        // Count auto tracks
        let auto_count = track_specs.iter().filter(|s| s.unit == Unit::Auto).count();

        // Second pass: distribute remaining space to fr and auto using Fraction
        // to avoid floating-point accumulation errors. Remainder is carried forward
        // so extra pixels naturally go to LATER tracks (matching Textual behavior).
        if total_fr > 0.0 {
            // Scale fr values to integers (multiply by 1000 to preserve precision)
            let total_fr_scaled = (total_fr * 1000.0) as i64;
            let mut remainder = Fraction::ZERO;

            for (i, spec) in track_specs.iter().enumerate() {
                if spec.unit == Unit::Fraction {
                    let fr_scaled = (spec.value * 1000.0) as i64;
                    let raw = Fraction::new(remaining as i64 * fr_scaled, total_fr_scaled) + remainder;
                    sizes[i] = raw.floor() as i32;
                    remainder = raw.fract();
                }
            }
        } else if auto_count > 0 {
            // No fr units, distribute equally to auto tracks
            let mut remainder = Fraction::ZERO;

            for (i, spec) in track_specs.iter().enumerate() {
                if spec.unit == Unit::Auto {
                    let raw = Fraction::new(remaining as i64, auto_count as i64) + remainder;
                    sizes[i] = raw.floor() as i32;
                    remainder = raw.fract();
                }
            }
        } else if specs.is_empty() {
            // No specs at all: equal distribution
            let mut remainder = Fraction::ZERO;

            for size in &mut sizes {
                let raw = Fraction::new(available_for_tracks as i64, count as i64) + remainder;
                *size = raw.floor() as i32;
                remainder = raw.fract();
            }
        }

        // Ensure minimum size of 1 for each track
        for size in &mut sizes {
            if *size < 1 {
                *size = 1;
            }
        }

        sizes
    }

    /// Distribute space and pre-compute offsets for tracks.
    ///
    /// Returns a vector of `ResolvedTrack` with pre-computed offsets,
    /// matching Python Textual's `_resolve.py` output format.
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

/// Calculate the region for a child at the given grid position with span support.
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
            columns[col..]
                .iter()
                .map(|t| t.size)
                .sum::<i32>()
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

    Region {
        x,
        y,
        width,
        height,
    }
}

impl<M> Widget<M> for Grid<M> {
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

        // Resolve tracks with pre-computed offsets (matches Python Textual)
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

                        if occupancy.can_fit(
                            current_row,
                            current_col,
                            effective_row_span,
                            effective_col_span,
                        ) {
                            // Mark cells as occupied
                            occupancy.occupy(
                                current_row,
                                current_col,
                                effective_row_span,
                                effective_col_span,
                            );

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

    fn desired_size(&self) -> Size {
        // Grid fills available space; return reasonable minimum
        Size::new(
            self.column_count() as u16 * 10,
            self.row_count(self.visible_children()) as u16 * 3,
        )
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Grid".to_string(),
            id: self.id.clone(),
            classes: Vec::new(),
            states: WidgetStates::empty(),
        }
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
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

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        for child in &mut self.children {
            f(child.as_mut());
        }
    }

    fn on_resize(&mut self, size: Size) {
        for child in &mut self.children {
            child.on_resize(size);
        }
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        for child in &mut self.children {
            if !child.is_visible() {
                continue;
            }
            if let Some(msg) = child.on_event(key) {
                return Some(msg);
            }
        }
        None
    }

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

        // Resolve tracks with pre-computed offsets (matches Python Textual)
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

        // Mirror the render placement algorithm (Tetris-style)
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

                        if occupancy.can_fit(
                            current_row,
                            current_col,
                            effective_row_span,
                            effective_col_span,
                        ) {
                            occupancy.occupy(
                                current_row,
                                current_col,
                                effective_row_span,
                                effective_col_span,
                            );

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

    fn count_focusable(&self) -> usize {
        self.children
            .iter()
            .filter(|c| c.is_visible())
            .map(|c| c.count_focusable())
            .sum()
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            if child.is_visible() {
                child.clear_focus();
            }
        }
    }

    fn focus_nth(&mut self, mut n: usize) -> bool {
        for child in &mut self.children {
            if !child.is_visible() {
                continue;
            }
            let count = child.count_focusable();
            if n < count {
                return child.focus_nth(n);
            }
            n -= count;
        }
        false
    }

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            if child.is_visible() {
                child.clear_hover();
            }
        }
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        if index < self.children.len() {
            Some(self.children[index].as_mut())
        } else {
            None
        }
    }
}

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
        assert_eq!(tracks[1].offset, 27); // 0 + 26 + 1 gutter
        assert_eq!(tracks[2].offset, 54); // 27 + 26 + 1 gutter
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

    #[test]
    fn test_occupancy_grid_out_of_bounds() {
        let grid = OccupancyGrid::new(2, 2);
        // Can't fit something that exceeds grid bounds
        assert!(!grid.can_fit(0, 0, 3, 1)); // 3 rows but only 2 available
        assert!(!grid.can_fit(0, 0, 1, 3)); // 3 cols but only 2 available
        assert!(!grid.can_fit(1, 1, 2, 2)); // Starting at (1,1), needs 2x2 but only 1x1 available
    }

    #[test]
    fn test_find_next_free_filled_grid() {
        let mut grid = OccupancyGrid::new(2, 2);
        grid.occupy(0, 0, 2, 2); // Fill entire grid
        assert_eq!(grid.find_next_free(0, 0), None);
    }

    #[test]
    fn test_child_region_single_cell() {
        let columns = vec![
            ResolvedTrack { offset: 0, size: 10 },
            ResolvedTrack { offset: 11, size: 10 },
        ];
        let rows = vec![
            ResolvedTrack { offset: 0, size: 5 },
            ResolvedTrack { offset: 6, size: 5 },
        ];
        let region = Region {
            x: 0,
            y: 0,
            width: 21,
            height: 11,
        };

        // Single cell at (0, 0)
        let r = child_region(0, 0, 1, 1, &columns, &rows, region, 1, 1);
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
        assert_eq!(r.width, 10);
        assert_eq!(r.height, 5);

        // Single cell at (1, 1)
        let r = child_region(1, 1, 1, 1, &columns, &rows, region, 1, 1);
        assert_eq!(r.x, 11);
        assert_eq!(r.y, 6);
        assert_eq!(r.width, 10);
        assert_eq!(r.height, 5);
    }

    #[test]
    fn test_child_region_spanning() {
        let columns = vec![
            ResolvedTrack { offset: 0, size: 10 },
            ResolvedTrack { offset: 11, size: 10 },
            ResolvedTrack { offset: 22, size: 10 },
        ];
        let rows = vec![
            ResolvedTrack { offset: 0, size: 5 },
            ResolvedTrack { offset: 6, size: 5 },
            ResolvedTrack { offset: 12, size: 5 },
        ];
        let region = Region {
            x: 0,
            y: 0,
            width: 32,
            height: 17,
        };

        // 2x2 span starting at (0, 0)
        let r = child_region(0, 0, 2, 2, &columns, &rows, region, 1, 1);
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
        // Width: columns[2].offset - columns[0].offset - gutter = 22 - 0 - 1 = 21
        assert_eq!(r.width, 21);
        // Height: rows[2].offset - rows[0].offset - gutter = 12 - 0 - 1 = 11
        assert_eq!(r.height, 11);
    }
}
