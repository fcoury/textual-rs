//! Grid layout algorithm for CSS Grid-like layouts.
//!
//! Implements a 2D grid layout with support for:
//! - Fixed column/row counts via `grid-size`
//! - Flexible column/row sizes via `grid-columns` and `grid-rows`
//! - Gutter spacing via `grid-gutter`
//! - Column/row spanning for children via `row-span` and `column-span`

use crate::canvas::{Region, Size};
use crate::fraction::Fraction;
use tcss::types::{ComputedStyle, GridStyle, Scalar, Unit};

use super::size_resolver::{resolve_height_with_intrinsic, resolve_width_with_intrinsic};
use super::{Layout, Viewport, WidgetPlacement};

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

/// Grid layout algorithm with configurable properties.
///
/// This struct holds runtime-configurable properties that can be set via
/// the `pre_layout` hook. CSS properties are read from `ComputedStyle`.
#[derive(Debug, Clone, Default)]
pub struct GridLayout {
    /// Minimum column width - used to auto-calculate column count.
    pub min_column_width: Option<u16>,
    /// Maximum column width - used to limit column widths.
    pub max_column_width: Option<u16>,
    /// Whether to stretch cell height to row height.
    pub stretch_height: bool,
    /// Whether the grid should be regular (no partial rows).
    pub regular: bool,
}

impl GridLayout {
    /// Create a new GridLayout with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of columns, considering min_column_width if set.
    fn column_count(&self, grid: &GridStyle, available_width: i32) -> usize {
        // If min_column_width is set, calculate columns to fit
        if let Some(min_width) = self.min_column_width {
            let min_width = min_width as i32;
            if min_width > 0 && available_width > 0 {
                return (available_width / min_width).max(1) as usize;
            }
        }
        // Otherwise use CSS grid-size-columns
        grid.columns.unwrap_or(1) as usize
    }

    /// Get the number of rows, either from CSS or calculated.
    fn row_count(&self, grid: &GridStyle, cols: usize, child_count: usize) -> usize {
        if let Some(rows) = grid.rows {
            rows as usize
        } else if cols == 0 {
            1
        } else {
            (child_count + cols - 1) / cols
        }
    }

    /// Resolve a scalar value to cells given the available space.
    fn resolve_scalar(scalar: &Scalar, available: i32) -> i32 {
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
    fn distribute_space(specs: &[Scalar], count: usize, available: i32, gutter: i32) -> Vec<i32> {
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
                    Scalar::fr(1.0) // Default: 1fr (equal distribution)
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
                    let size = Self::resolve_scalar(spec, available_for_tracks);
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
                    let raw =
                        Fraction::new(remaining as i64 * fr_scaled, total_fr_scaled) + remainder;
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
    fn resolve_tracks(specs: &[Scalar], count: usize, available: i32, gutter: i32) -> Vec<ResolvedTrack> {
        let sizes = Self::distribute_space(specs, count, available, gutter);
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

    /// Resolve tracks with content-based sizing when specs are empty.
    ///
    /// When grid-rows/grid-columns are not specified, tracks are auto-sized
    /// based on the maximum content size of children in each track.
    fn resolve_tracks_with_content(
        specs: &[Scalar],
        count: usize,
        available: i32,
        gutter: i32,
        content_sizes: &[i32], // Max content size per track
    ) -> Vec<ResolvedTrack> {
        if specs.is_empty() && !content_sizes.is_empty() {
            // Auto-sizing: use content sizes
            let total_gutter = (count.saturating_sub(1)) as i32 * gutter;
            let total_content: i32 = content_sizes.iter().sum();
            let available_for_tracks = available - total_gutter;

            // If content fits, use content sizes; otherwise scale proportionally
            let sizes: Vec<i32> = if total_content <= available_for_tracks {
                content_sizes.to_vec()
            } else {
                // Scale down proportionally
                content_sizes
                    .iter()
                    .map(|&size| {
                        if total_content > 0 {
                            (size as i64 * available_for_tracks as i64 / total_content as i64) as i32
                        } else {
                            1
                        }
                    })
                    .collect()
            };

            let mut offset = 0;
            sizes
                .into_iter()
                .map(|size| {
                    let track = ResolvedTrack { offset, size: size.max(1) };
                    offset += size.max(1) + gutter;
                    track
                })
                .collect()
        } else {
            // Use standard distribution
            Self::resolve_tracks(specs, count, available, gutter)
        }
    }

    /// Compute which cell each child occupies (Tetris placement) without actually placing them.
    /// Returns (row, col, row_span, col_span) for each child.
    fn compute_cell_assignments(
        children: &[(usize, ComputedStyle, Size)],
        rows: usize,
        cols: usize,
    ) -> Vec<(usize, usize, usize, usize)> {
        let mut occupancy = OccupancyGrid::new(rows, cols);
        let mut current_row = 0;
        let mut current_col = 0;
        let mut assignments = Vec::with_capacity(children.len());

        for (_child_index, child_style, _desired_size) in children {
            let col_span = (child_style.grid_placement.column_span as usize).max(1);
            let row_span = (child_style.grid_placement.row_span as usize).max(1);

            loop {
                match occupancy.find_next_free(current_row, current_col) {
                    Some((r, c)) => {
                        current_row = r;
                        current_col = c;

                        let effective_col_span = col_span.min(cols - current_col);
                        let effective_row_span = row_span.min(rows - current_row);

                        if occupancy.can_fit(current_row, current_col, effective_row_span, effective_col_span) {
                            occupancy.occupy(current_row, current_col, effective_row_span, effective_col_span);
                            assignments.push((current_row, current_col, effective_row_span, effective_col_span));

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
                    None => {
                        // No space left
                        assignments.push((0, 0, 0, 0)); // Placeholder for children that don't fit
                        break;
                    }
                }
            }

            if current_row >= rows {
                break;
            }
        }

        assignments
    }

    /// Compute max content sizes per track based on children.
    fn compute_content_sizes(
        children: &[(usize, ComputedStyle, Size)],
        assignments: &[(usize, usize, usize, usize)],
        rows: usize,
        cols: usize,
    ) -> (Vec<i32>, Vec<i32>) {
        let mut col_sizes = vec![0i32; cols];
        let mut row_sizes = vec![0i32; rows];

        for (i, (_child_index, _child_style, desired_size)) in children.iter().enumerate() {
            if i >= assignments.len() {
                break;
            }
            let (row, col, row_span, col_span) = assignments[i];
            if row_span == 0 || col_span == 0 {
                continue; // Didn't fit
            }

            // For single-span cells, use content size directly
            // For multi-span cells, distribute evenly (simplified)
            if col_span == 1 {
                col_sizes[col] = col_sizes[col].max(desired_size.width as i32);
            }
            if row_span == 1 {
                row_sizes[row] = row_sizes[row].max(desired_size.height as i32);
            }
        }

        (col_sizes, row_sizes)
    }
}

impl Layout for GridLayout {
    fn arrange(
        &mut self,
        parent_style: &ComputedStyle,
        children: &[(usize, ComputedStyle, Size)],
        available: Region,
        _viewport: Viewport,
    ) -> Vec<WidgetPlacement> {
        if children.is_empty() {
            return Vec::new();
        }

        let grid = &parent_style.grid;
        let cols = self.column_count(grid, available.width);
        let rows = self.row_count(grid, cols, children.len());

        // Get gutter values
        let gutter_v = Self::resolve_scalar(&grid.gutter.0, available.height);
        let gutter_h = Self::resolve_scalar(&grid.gutter.1, available.width);

        // Python Textual logic for default grid-rows/columns:
        // - If grid-rows/columns specified → use those
        // - If NOT specified AND parent has auto height/width → use auto (content-sized)
        // - If NOT specified AND parent has fixed/fr height/width → use 1fr (expand)
        //
        // Check if parent has auto width/height
        let is_auto_width = parent_style.width.as_ref().map_or(false, |w| w.unit == Unit::Auto);
        let is_auto_height = parent_style.height.as_ref().map_or(false, |h| h.unit == Unit::Auto);

        // Determine if we should use auto (content) sizing for tracks
        // Only use auto when specs are empty AND parent has auto dimension
        let use_auto_cols = grid.column_widths.is_empty() && is_auto_width;
        let use_auto_rows = grid.row_heights.is_empty() && is_auto_height;

        let (col_content_sizes, row_content_sizes) = if use_auto_cols || use_auto_rows {
            let assignments = Self::compute_cell_assignments(children, rows, cols);
            Self::compute_content_sizes(children, &assignments, rows, cols)
        } else {
            (vec![], vec![])
        };

        // Resolve tracks:
        // - If specs provided → use them
        // - If specs empty + auto dimension → use content sizes
        // - If specs empty + fixed/fr dimension → use 1fr (equal distribution)
        let columns = if use_auto_cols {
            Self::resolve_tracks_with_content(
                &grid.column_widths,
                cols,
                available.width,
                gutter_h,
                &col_content_sizes,
            )
        } else {
            // Use 1fr distribution when no specs and not auto width
            Self::resolve_tracks(&grid.column_widths, cols, available.width, gutter_h)
        };
        let row_tracks = if use_auto_rows {
            Self::resolve_tracks_with_content(
                &grid.row_heights,
                rows,
                available.height,
                gutter_v,
                &row_content_sizes,
            )
        } else {
            // Use 1fr distribution when no specs and not auto height
            Self::resolve_tracks(&grid.row_heights, rows, available.height, gutter_v)
        };

        // Create occupancy grid for Tetris-style placement
        let mut occupancy = OccupancyGrid::new(rows, cols);
        let mut current_row = 0;
        let mut current_col = 0;
        let mut result = Vec::new();

        for (child_index, child_style, desired_size) in children {
            // Get span values from child's computed style
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

                            // Calculate spanning region (the full cell area)
                            let cell_region = child_region(
                                current_col,
                                current_row,
                                effective_col_span,
                                effective_row_span,
                                &columns,
                                &row_tracks,
                                available,
                                gutter_h,
                                gutter_v,
                            );

                            // Resolve child's actual size based on CSS width/height
                            // Behavior depends on parent's alignment:
                            // - Default alignment (left/top): children fill their cells
                            // - Non-default alignment (center/right/middle/bottom): children use
                            //   intrinsic size and get positioned within the cell
                            use tcss::types::text::{AlignHorizontal, AlignVertical};

                            let has_h_align = !matches!(parent_style.align_horizontal, AlignHorizontal::Left);
                            let has_v_align = !matches!(parent_style.align_vertical, AlignVertical::Top);

                            let child_width = if child_style.width.is_none() && has_h_align {
                                // No width + non-default h-align: use intrinsic size
                                desired_size.width as i32
                            } else {
                                resolve_width_with_intrinsic(
                                    child_style,
                                    desired_size.width,
                                    cell_region.width,
                                )
                            };
                            let child_height = if child_style.height.is_none() && has_v_align {
                                // No height + non-default v-align: use intrinsic size
                                desired_size.height as i32
                            } else {
                                resolve_height_with_intrinsic(
                                    child_style,
                                    desired_size.height,
                                    cell_region.height,
                                )
                            };

                            // Apply alignment within the cell
                            let x_offset = match parent_style.align_horizontal {
                                AlignHorizontal::Left => 0,
                                AlignHorizontal::Center => {
                                    (cell_region.width - child_width) / 2
                                }
                                AlignHorizontal::Right => cell_region.width - child_width,
                            };

                            let y_offset = match parent_style.align_vertical {
                                AlignVertical::Top => 0,
                                AlignVertical::Middle => {
                                    (cell_region.height - child_height) / 2
                                }
                                AlignVertical::Bottom => cell_region.height - child_height,
                            };

                            let final_region = Region {
                                x: cell_region.x + x_offset,
                                y: cell_region.y + y_offset,
                                width: child_width,
                                height: child_height,
                            };

                            result.push(WidgetPlacement {
                                child_index: *child_index,
                                region: final_region,
                            });

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

        result
    }

    fn as_grid_mut(&mut self) -> Option<&mut GridLayout> {
        Some(self)
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

    #[test]
    fn test_child_region_span_to_grid_edge() {
        // Test case: span reaches exactly to the grid edge
        // This exercises the summation-based calculation branch
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

        // 2-column span starting at col 1 (reaches to edge: 1 + 2 = 3 = columns.len())
        let r = child_region(1, 0, 2, 1, &columns, &rows, region, 1, 1);
        assert_eq!(r.x, 11); // Starting at column 1's offset
        assert_eq!(r.y, 0);
        // Width should sum remaining columns: 10 + 10 = 20, plus 1 internal gutter = 21
        // (end_col - col).saturating_sub(1) = (3 - 1).saturating_sub(1) = 1 gutter
        assert_eq!(r.width, 21);
        assert_eq!(r.height, 5);

        // 3-column span starting at col 0 (full width)
        let r = child_region(0, 0, 3, 1, &columns, &rows, region, 1, 1);
        assert_eq!(r.x, 0);
        // Width should sum all columns: 10 + 10 + 10 = 30, plus 2 internal gutters = 32
        assert_eq!(r.width, 32);

        // 2-row span starting at row 1 (reaches to edge: 1 + 2 = 3 = rows.len())
        let r = child_region(0, 1, 1, 2, &columns, &rows, region, 1, 1);
        assert_eq!(r.y, 6); // Starting at row 1's offset
        // Height should sum remaining rows: 5 + 5 = 10, plus 1 internal gutter = 11
        assert_eq!(r.height, 11);

        // 3-row span starting at row 0 (full height)
        let r = child_region(0, 0, 1, 3, &columns, &rows, region, 1, 1);
        assert_eq!(r.y, 0);
        // Height should sum all rows: 5 + 5 + 5 = 15, plus 2 internal gutters = 17
        assert_eq!(r.height, 17);
    }

    #[test]
    fn test_child_region_span_exceeds_grid() {
        // Test case: span would exceed grid boundary (gets clamped)
        let columns = vec![
            ResolvedTrack { offset: 0, size: 10 },
            ResolvedTrack { offset: 11, size: 10 },
            ResolvedTrack { offset: 22, size: 10 },
        ];
        let rows = vec![
            ResolvedTrack { offset: 0, size: 5 },
            ResolvedTrack { offset: 6, size: 5 },
        ];
        let region = Region {
            x: 0,
            y: 0,
            width: 32,
            height: 11,
        };

        // 3-column span starting at col 2 (2 + 3 > 3, clamped to 1 column)
        // end_col = min(2 + 3, 3) = 3 = columns.len(), so uses summation branch
        let r = child_region(2, 0, 3, 1, &columns, &rows, region, 1, 1);
        assert_eq!(r.x, 22); // Starting at column 2's offset
        // Only 1 column remaining (column 2), width = 10, no internal gutters
        assert_eq!(r.width, 10);

        // 3-row span starting at row 1 (1 + 3 > 2, clamped to 1 row)
        let r = child_region(0, 1, 1, 3, &columns, &rows, region, 1, 1);
        assert_eq!(r.y, 6); // Starting at row 1's offset
        // Only 1 row remaining (row 1), height = 5, no internal gutters
        assert_eq!(r.height, 5);

        // Full grid span from corner (both dimensions exceed)
        let r = child_region(1, 1, 5, 5, &columns, &rows, region, 1, 1);
        assert_eq!(r.x, 11);
        assert_eq!(r.y, 6);
        // 2 columns remaining (col 1, 2): 10 + 10 + 1 gutter = 21
        assert_eq!(r.width, 21);
        // 1 row remaining (row 1): 5, no internal gutters
        assert_eq!(r.height, 5);
    }
}
