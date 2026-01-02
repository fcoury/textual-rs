//! Keyline canvas for rendering connected box-drawing borders.
//!
//! This module provides [`KeylineCanvas`], a 2D grid that accumulates
//! box-drawing quads from multiple widgets and renders them as connected
//! lines.
//!
//! ## Usage
//!
//! 1. Create a `KeylineCanvas` with the container dimensions
//! 2. Call `add_widget_box()` for each widget to register its edges
//! 3. Call `render()` to draw the keylines to the main canvas

use crate::{
    Canvas, Region,
    box_drawing::{Quad, combine_quads, get_box_char},
    canvas::TextAttributes,
};
use tcss::types::RgbaColor;

/// A 2D grid for accumulating and rendering keylines.
///
/// The canvas stores a quad at each cell position. When multiple widgets
/// share an edge, their quads are combined (using max line type per edge).
#[derive(Debug)]
pub struct KeylineCanvas {
    /// Width in cells
    width: usize,
    /// Height in cells
    height: usize,
    /// The quad grid (width * height entries)
    quads: Vec<Quad>,
    /// Line type for this canvas (1=thin, 2=heavy, 3=double)
    line_type: u8,
    /// Color for the keylines
    color: RgbaColor,
}

impl KeylineCanvas {
    /// Create a new keyline canvas.
    ///
    /// # Arguments
    /// * `width` - Width in cells
    /// * `height` - Height in cells
    /// * `line_type` - Line type (1=thin, 2=heavy, 3=double)
    /// * `color` - Color for the keylines
    pub fn new(width: usize, height: usize, line_type: u8, color: RgbaColor) -> Self {
        Self {
            width,
            height,
            quads: vec![(0, 0, 0, 0); width * height],
            line_type,
            color,
        }
    }

    /// Get the quad at a position, or None if out of bounds.
    fn get(&self, x: usize, y: usize) -> Option<Quad> {
        if x < self.width && y < self.height {
            Some(self.quads[y * self.width + x])
        } else {
            None
        }
    }

    /// Set the quad at a position, combining with existing quad.
    fn set(&mut self, x: usize, y: usize, quad: Quad) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.quads[idx] = combine_quads(self.quads[idx], quad);
        }
    }

    /// Add a widget's box to the canvas.
    ///
    /// This registers the edges of a widget at the given position,
    /// using the canvas's line type.
    ///
    /// # Arguments
    /// * `x` - Left edge x-coordinate
    /// * `y` - Top edge y-coordinate
    /// * `width` - Width of the widget in cells
    /// * `height` - Height of the widget in cells
    pub fn add_widget_box(&mut self, x: usize, y: usize, width: usize, height: usize) {
        if width == 0 || height == 0 {
            return;
        }

        let lt = self.line_type;

        // Top-left corner
        self.set(x, y, (0, lt, lt, 0));

        // Top edge (excluding corners)
        for i in 1..width.saturating_sub(1) {
            self.set(x + i, y, (0, lt, 0, lt));
        }

        // Top-right corner (if width > 1)
        if width > 1 {
            self.set(x + width - 1, y, (0, 0, lt, lt));
        }

        // Left edge (excluding corners)
        for j in 1..height.saturating_sub(1) {
            self.set(x, y + j, (lt, 0, lt, 0));
        }

        // Right edge (excluding corners)
        if width > 1 {
            for j in 1..height.saturating_sub(1) {
                self.set(x + width - 1, y + j, (lt, 0, lt, 0));
            }
        }

        // Bottom-left corner (if height > 1)
        if height > 1 {
            self.set(x, y + height - 1, (lt, lt, 0, 0));
        }

        // Bottom edge (excluding corners)
        if height > 1 {
            for i in 1..width.saturating_sub(1) {
                self.set(x + i, y + height - 1, (0, lt, 0, lt));
            }
        }

        // Bottom-right corner (if width > 1 and height > 1)
        if width > 1 && height > 1 {
            self.set(x + width - 1, y + height - 1, (lt, 0, 0, lt));
        }
    }

    /// Add a grid structure to the canvas.
    ///
    /// This draws keylines at column and row boundaries, creating a connected
    /// table-like structure. Column and row positions are the START of each
    /// track (not including the outer border).
    ///
    /// # Arguments
    /// * `col_positions` - X positions where columns start (and end+1 for last column)
    /// * `row_positions` - Y positions where rows start (and end+1 for last row)
    ///
    /// For a 2-column grid spanning x=0 to x=20:
    /// - col_positions might be [0, 10, 20] (start of col 0, start of col 1, end of col 1)
    pub fn add_grid(&mut self, col_positions: &[usize], row_positions: &[usize]) {
        if col_positions.len() < 2 || row_positions.len() < 2 {
            return;
        }

        let lt = self.line_type;
        let last_col = col_positions.len() - 1;
        let last_row = row_positions.len() - 1;

        // Draw horizontal lines at each row boundary
        for (row_idx, &y) in row_positions.iter().enumerate() {
            if y >= self.height {
                continue;
            }

            let is_top = row_idx == 0;
            let is_bottom = row_idx == last_row;

            for col_idx in 0..col_positions.len() {
                let x = col_positions[col_idx];
                if x >= self.width {
                    continue;
                }

                let is_left = col_idx == 0;
                let is_right = col_idx == last_col;

                // Determine the quad based on position
                let top = if is_top { 0 } else { lt };
                let bottom = if is_bottom { 0 } else { lt };
                let left = if is_left { 0 } else { lt };
                let right = if is_right { 0 } else { lt };

                self.set(x, y, (top, right, bottom, left));

                // Draw horizontal line between column positions
                if col_idx < last_col {
                    let next_x = col_positions[col_idx + 1];
                    for fill_x in (x + 1)..next_x.min(self.width) {
                        self.set(fill_x, y, (0, lt, 0, lt));
                    }
                }
            }
        }

        // Draw vertical lines at each column boundary
        for (_col_idx, &x) in col_positions.iter().enumerate() {
            if x >= self.width {
                continue;
            }

            for row_idx in 0..row_positions.len() {
                let y = row_positions[row_idx];
                if y >= self.height {
                    continue;
                }

                // Draw vertical line between row positions
                if row_idx < last_row {
                    let next_y = row_positions[row_idx + 1];
                    for fill_y in (y + 1)..next_y.min(self.height) {
                        self.set(x, fill_y, (lt, 0, lt, 0));
                    }
                }
            }
        }
    }

    /// Add a grid structure with span-aware keylines.
    ///
    /// This draws keylines only where adjacent cells contain different widgets,
    /// respecting column-span and row-span CSS properties.
    ///
    /// # Arguments
    /// * `col_positions` - X positions where columns start (and end+1 for last column)
    /// * `row_positions` - Y positions where rows start (and end+1 for last row)
    /// * `cell_occupancy` - Grid of widget indices: `[row][col] = Some(widget_index)` or None
    pub fn add_grid_with_occupancy(
        &mut self,
        col_positions: &[usize],
        row_positions: &[usize],
        cell_occupancy: &[Vec<Option<usize>>],
    ) {
        if col_positions.len() < 2 || row_positions.len() < 2 {
            return;
        }

        let lt = self.line_type;
        let num_cols = col_positions.len() - 1; // Number of actual columns
        let num_rows = row_positions.len() - 1; // Number of actual rows

        let x_min = col_positions[0];
        let x_max = col_positions[num_cols];
        let y_min = row_positions[0];
        let y_max = row_positions[num_rows];

        // Helper: check if two cells have different occupants (meaning we draw a line between them)
        let different_occupant = |r1: usize, c1: usize, r2: usize, c2: usize| -> bool {
            let occ1 = cell_occupancy
                .get(r1)
                .and_then(|row| row.get(c1))
                .copied()
                .flatten();
            let occ2 = cell_occupancy
                .get(r2)
                .and_then(|row| row.get(c2))
                .copied()
                .flatten();
            occ1 != occ2
        };

        // 1. Draw outer border as a complete rectangle
        // Top horizontal line
        if y_min < self.height {
            for x in x_min..(x_max + 1).min(self.width) {
                let left = if x > x_min { lt } else { 0 };
                let right = if x < x_max { lt } else { 0 };
                self.set(x, y_min, (0, right, 0, left));
            }
        }

        // Bottom horizontal line
        if y_max < self.height {
            for x in x_min..(x_max + 1).min(self.width) {
                let left = if x > x_min { lt } else { 0 };
                let right = if x < x_max { lt } else { 0 };
                self.set(x, y_max, (0, right, 0, left));
            }
        }

        // Left vertical line
        if x_min < self.width {
            for y in y_min..(y_max + 1).min(self.height) {
                let top = if y > y_min { lt } else { 0 };
                let bottom = if y < y_max { lt } else { 0 };
                self.set(x_min, y, (top, 0, bottom, 0));
            }
        }

        // Right vertical line
        if x_max < self.width {
            for y in y_min..(y_max + 1).min(self.height) {
                let top = if y > y_min { lt } else { 0 };
                let bottom = if y < y_max { lt } else { 0 };
                self.set(x_max, y, (top, 0, bottom, 0));
            }
        }

        // 2. Draw internal vertical lines (between columns) only where adjacent cells differ
        for col_idx in 1..num_cols {
            let x = col_positions[col_idx];
            if x >= self.width {
                continue;
            }

            for row_idx in 0..num_rows {
                // Check if cells to the left and right of this boundary have different occupants
                if !different_occupant(row_idx, col_idx - 1, row_idx, col_idx) {
                    continue; // Same widget spans these cells, skip the line
                }

                let y_start = row_positions[row_idx];
                let y_end = row_positions[row_idx + 1];
                for y in y_start..(y_end + 1).min(self.height) {
                    let top = if y > y_start { lt } else { 0 };
                    let bottom = if y < y_end { lt } else { 0 };
                    self.set(x, y, (top, 0, bottom, 0));
                }
            }
        }

        // 3. Draw internal horizontal lines (between rows) only where adjacent cells differ
        for row_idx in 1..num_rows {
            let y = row_positions[row_idx];
            if y >= self.height {
                continue;
            }

            for col_idx in 0..num_cols {
                // Check if cells above and below this boundary have different occupants
                if !different_occupant(row_idx - 1, col_idx, row_idx, col_idx) {
                    continue; // Same widget spans these cells, skip the line
                }

                let x_start = col_positions[col_idx];
                let x_end = col_positions[col_idx + 1];
                for x in x_start..(x_end + 1).min(self.width) {
                    let left = if x > x_start { lt } else { 0 };
                    let right = if x < x_end { lt } else { 0 };
                    self.set(x, y, (0, right, 0, left));
                }
            }
        }

        // 4. Draw junctions at all intersection points
        // This ensures proper corner/T-junction/cross characters
        for row_idx in 0..=num_rows {
            let y = row_positions[row_idx];
            if y >= self.height {
                continue;
            }

            for col_idx in 0..=num_cols {
                let x = col_positions[col_idx];
                if x >= self.width {
                    continue;
                }

                let is_top_edge = row_idx == 0;
                let is_bottom_edge = row_idx == num_rows;
                let is_left_edge = col_idx == 0;
                let is_right_edge = col_idx == num_cols;

                // Check for vertical line above this junction
                let has_line_up = if is_top_edge {
                    false
                } else if is_left_edge || is_right_edge {
                    true // Outer border always has vertical lines
                } else {
                    // Internal junction: check if vertical line exists in the row above
                    different_occupant(row_idx - 1, col_idx - 1, row_idx - 1, col_idx)
                };

                // Check for vertical line below this junction
                let has_line_down = if is_bottom_edge {
                    false
                } else if is_left_edge || is_right_edge {
                    true // Outer border always has vertical lines
                } else {
                    // Internal junction: check if vertical line exists in the row below
                    different_occupant(row_idx, col_idx - 1, row_idx, col_idx)
                };

                // Check for horizontal line to the left of this junction
                let has_line_left = if is_left_edge {
                    false
                } else if is_top_edge || is_bottom_edge {
                    true // Outer border always has horizontal lines
                } else {
                    // Internal junction: check if horizontal line exists in the column to the left
                    different_occupant(row_idx - 1, col_idx - 1, row_idx, col_idx - 1)
                };

                // Check for horizontal line to the right of this junction
                let has_line_right = if is_right_edge {
                    false
                } else if is_top_edge || is_bottom_edge {
                    true // Outer border always has horizontal lines
                } else {
                    // Internal junction: check if horizontal line exists in the column to the right
                    different_occupant(row_idx - 1, col_idx, row_idx, col_idx)
                };

                let top = if has_line_up { lt } else { 0 };
                let bottom = if has_line_down { lt } else { 0 };
                let left = if has_line_left { lt } else { 0 };
                let right = if has_line_right { lt } else { 0 };

                self.set(x, y, (top, right, bottom, left));
            }
        }
    }

    /// Render the keylines to a canvas.
    ///
    /// # Arguments
    /// * `canvas` - The canvas to render to
    /// * `region` - The region within the canvas to render
    pub fn render(&self, canvas: &mut Canvas, region: Region) {
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(quad) = self.get(x, y) {
                    // Skip empty quads
                    if quad == (0, 0, 0, 0) {
                        continue;
                    }

                    if let Some(ch) = get_box_char(quad) {
                        let canvas_x = region.x + x as i32;
                        let canvas_y = region.y + y as i32;

                        // put_char handles bounds checking internally
                        canvas.put_char(
                            canvas_x,
                            canvas_y,
                            ch,
                            Some(self.color.clone()),
                            None,
                            TextAttributes::default(),
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_widget_box() {
        let mut kc = KeylineCanvas::new(10, 5, 2, RgbaColor::white());
        kc.add_widget_box(0, 0, 5, 3);

        // Check corners
        assert_eq!(kc.get(0, 0), Some((0, 2, 2, 0))); // top-left
        assert_eq!(kc.get(4, 0), Some((0, 0, 2, 2))); // top-right
        assert_eq!(kc.get(0, 2), Some((2, 2, 0, 0))); // bottom-left
        assert_eq!(kc.get(4, 2), Some((2, 0, 0, 2))); // bottom-right

        // Check edges
        assert_eq!(kc.get(2, 0), Some((0, 2, 0, 2))); // top edge
        assert_eq!(kc.get(0, 1), Some((2, 0, 2, 0))); // left edge
        assert_eq!(kc.get(4, 1), Some((2, 0, 2, 0))); // right edge
        assert_eq!(kc.get(2, 2), Some((0, 2, 0, 2))); // bottom edge
    }

    #[test]
    fn test_adjacent_widgets_merge() {
        let mut kc = KeylineCanvas::new(8, 3, 2, RgbaColor::white());

        // Two widgets side by side, OVERLAPPING by 1 cell at x=3
        // Widget 1: 0,0 to 3,2 (width=4, covers x=0,1,2,3)
        // Widget 2: 3,0 to 6,2 (width=4, covers x=3,4,5,6)
        // They share column x=3
        kc.add_widget_box(0, 0, 4, 3);
        kc.add_widget_box(3, 0, 4, 3);

        // The shared edge (x=3) should form T-junctions
        // At (3, 0) - top-right of first + top-left of second
        // First contributes (0, 0, 2, 2), second contributes (0, 2, 2, 0)
        // Combined: (0, 2, 2, 2) which is ┳
        assert_eq!(kc.get(3, 0), Some((0, 2, 2, 2)));

        // At (3, 1) - right edge of first + left edge of second
        // Both contribute (2, 0, 2, 0)
        // Combined: (2, 0, 2, 0) which is ┃
        assert_eq!(kc.get(3, 1), Some((2, 0, 2, 0)));

        // At (3, 2) - bottom-right of first + bottom-left of second
        // First contributes (2, 0, 0, 2), second contributes (2, 2, 0, 0)
        // Combined: (2, 2, 0, 2) which is ┻
        assert_eq!(kc.get(3, 2), Some((2, 2, 0, 2)));
    }

    #[test]
    fn test_stacked_widgets_merge() {
        let mut kc = KeylineCanvas::new(5, 5, 2, RgbaColor::white());

        // Two widgets stacked vertically, OVERLAPPING by 1 row at y=2
        // Widget 1: 0,0 to 4,2 (height=3, covers y=0,1,2)
        // Widget 2: 0,2 to 4,4 (height=3, covers y=2,3,4)
        // They share row y=2
        kc.add_widget_box(0, 0, 5, 3);
        kc.add_widget_box(0, 2, 5, 3);

        // The shared horizontal edge (y=2) should form T-junctions
        // At (0, 2) - bottom-left of first + top-left of second
        // First contributes (2, 2, 0, 0), second contributes (0, 2, 2, 0)
        // Combined: (2, 2, 2, 0) which is ┣
        assert_eq!(kc.get(0, 2), Some((2, 2, 2, 0)));

        // At (2, 2) - bottom edge of first + top edge of second
        // Both contribute (0, 2, 0, 2)
        // Combined: (0, 2, 0, 2) which is ━
        assert_eq!(kc.get(2, 2), Some((0, 2, 0, 2)));

        // At (4, 2) - bottom-right of first + top-right of second
        // First contributes (2, 0, 0, 2), second contributes (0, 0, 2, 2)
        // Combined: (2, 0, 2, 2) which is ┫
        assert_eq!(kc.get(4, 2), Some((2, 0, 2, 2)));
    }

    #[test]
    fn test_four_widgets_cross() {
        let mut kc = KeylineCanvas::new(7, 5, 2, RgbaColor::white());

        // 2x2 grid of widgets, OVERLAPPING at shared edges
        // Top-left:     (0,0) to (3,2) - width=4, height=3
        // Top-right:    (3,0) to (6,2) - width=4, height=3, overlaps at x=3
        // Bottom-left:  (0,2) to (3,4) - width=4, height=3, overlaps at y=2
        // Bottom-right: (3,2) to (6,4) - width=4, height=3, overlaps at (3,2)
        kc.add_widget_box(0, 0, 4, 3);
        kc.add_widget_box(3, 0, 4, 3);
        kc.add_widget_box(0, 2, 4, 3);
        kc.add_widget_box(3, 2, 4, 3);

        // The center point (3, 2) should be a cross ╋
        // All four widgets contribute to this point:
        // Top-left widget: bottom-right (2, 0, 0, 2)
        // Top-right widget: bottom-left (2, 2, 0, 0)
        // Bottom-left widget: top-right (0, 0, 2, 2)
        // Bottom-right widget: top-left (0, 2, 2, 0)
        // Combined: (2, 2, 2, 2)
        assert_eq!(kc.get(3, 2), Some((2, 2, 2, 2)));
    }

    #[test]
    fn test_1x1_widget() {
        let mut kc = KeylineCanvas::new(3, 3, 1, RgbaColor::white());
        kc.add_widget_box(1, 1, 1, 1);

        // A 1x1 widget should just be a corner (but with all edges)
        // This is a degenerate case - we get top-left corner only
        assert_eq!(kc.get(1, 1), Some((0, 1, 1, 0)));
    }

    #[test]
    fn test_add_grid_2x2() {
        let mut kc = KeylineCanvas::new(7, 5, 2, RgbaColor::white());

        // A 2x2 grid with columns at [0, 3, 6] and rows at [0, 2, 4]
        // This creates a grid like:
        // ┏━━┳━━┓
        // ┃  ┃  ┃
        // ┣━━╋━━┫
        // ┃  ┃  ┃
        // ┗━━┻━━┛
        kc.add_grid(&[0, 3, 6], &[0, 2, 4]);

        // Check corners
        assert_eq!(kc.get(0, 0), Some((0, 2, 2, 0))); // ┏ top-left
        assert_eq!(kc.get(6, 0), Some((0, 0, 2, 2))); // ┓ top-right
        assert_eq!(kc.get(0, 4), Some((2, 2, 0, 0))); // ┗ bottom-left
        assert_eq!(kc.get(6, 4), Some((2, 0, 0, 2))); // ┛ bottom-right

        // Check T-junctions on edges
        assert_eq!(kc.get(3, 0), Some((0, 2, 2, 2))); // ┳ top middle
        assert_eq!(kc.get(3, 4), Some((2, 2, 0, 2))); // ┻ bottom middle
        assert_eq!(kc.get(0, 2), Some((2, 2, 2, 0))); // ┣ left middle
        assert_eq!(kc.get(6, 2), Some((2, 0, 2, 2))); // ┫ right middle

        // Check center cross
        assert_eq!(kc.get(3, 2), Some((2, 2, 2, 2))); // ╋ center

        // Check horizontal lines
        assert_eq!(kc.get(1, 0), Some((0, 2, 0, 2))); // ━ top
        assert_eq!(kc.get(2, 0), Some((0, 2, 0, 2))); // ━ top
        assert_eq!(kc.get(1, 2), Some((0, 2, 0, 2))); // ━ middle
        assert_eq!(kc.get(1, 4), Some((0, 2, 0, 2))); // ━ bottom

        // Check vertical lines
        assert_eq!(kc.get(0, 1), Some((2, 0, 2, 0))); // ┃ left
        assert_eq!(kc.get(3, 1), Some((2, 0, 2, 0))); // ┃ middle
        assert_eq!(kc.get(6, 1), Some((2, 0, 2, 0))); // ┃ right
    }
}
