use crossterm::{
    cursor, execute,
    style::{Attribute, Color, SetAttribute, SetBackgroundColor, SetForegroundColor},
};
use std::io::Write;
use tcss::types::RgbaColor;

use crate::strip::Strip;

/// Text styling attributes (bold, italic, etc.)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextAttributes {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub reverse: bool,
}

/// The physical dimensions of a widget or terminal.
#[derive(Clone, Copy, Debug, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    /// Create a new Size with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

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
    /// Uses saturating arithmetic to prevent overflow with large coordinates.
    pub fn intersection(&self, other: &Region) -> Region {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = self.x.saturating_add(self.width).min(other.x.saturating_add(other.width));
        let y2 = self.y.saturating_add(self.height).min(other.y.saturating_add(other.height));

        if x2 > x1 && y2 > y1 {
            Region {
                x: x1,
                y: y1,
                width: x2 - x1,
                height: y2 - y1,
            }
        } else {
            Region {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            }
        }
    }

    /// Checks if a point is contained within the region.
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x.saturating_add(self.width)
            && y >= self.y
            && y < self.y.saturating_add(self.height)
    }

    /// Returns true if the region has no area.
    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub symbol: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub attrs: TextAttributes,
}

pub struct Canvas {
    size: Size,
    cells: Vec<Cell>,
    // Track current active colors to minimize ANSI escape codes
    // TODO: Use these for optimization in flush()
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
                Cell {
                    symbol: ' ',
                    fg: None,
                    bg: None,
                    attrs: TextAttributes::default(),
                };
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
        attrs: TextAttributes,
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
            attrs,
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
        attrs: TextAttributes,
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
                    attrs,
                };
            }
            current_x += 1;
        }
    }

    /// Renders a Strip at the given position.
    ///
    /// Iterates through each segment in the strip and renders its text
    /// with the appropriate styling. The strip is rendered left-to-right
    /// starting at (x, y).
    pub fn render_strip(&mut self, strip: &Strip, x: i32, y: i32) {
        let mut current_x = x;

        for segment in strip.segments() {
            let fg = segment.fg().cloned();
            let bg = segment.bg().cloned();
            let attrs = segment
                .style()
                .map(|s| TextAttributes {
                    bold: s.bold,
                    dim: s.dim,
                    italic: s.italic,
                    underline: s.underline,
                    strike: s.strike,
                    reverse: s.reverse,
                })
                .unwrap_or_default();
            self.put_str(current_x, y, segment.text(), fg, bg, attrs);
            current_x += segment.cell_length() as i32;
        }
    }

    /// Renders multiple Strips starting at the given position.
    ///
    /// Each strip is rendered on a successive line, starting at `start_y`.
    pub fn render_strips(&mut self, strips: &[Strip], x: i32, start_y: i32) {
        for (i, strip) in strips.iter().enumerate() {
            self.render_strip(strip, x, start_y + i as i32);
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        let mut out = std::io::stdout();
        execute!(out, cursor::MoveTo(0, 0))?;

        // Reset colors and attributes at start of each render to prevent bleeding
        // between frames. Without this, the terminal keeps colors/attributes from the
        // previous render when cells have None for fg/bg.
        execute!(out, SetForegroundColor(Color::Reset))?;
        execute!(out, SetBackgroundColor(Color::Reset))?;
        execute!(out, SetAttribute(Attribute::Reset))?;

        let mut last_fg = Some(Color::Reset);
        let mut last_bg = Some(Color::Reset);
        let mut last_attrs = TextAttributes::default();

        let rows: Vec<_> = self.cells.chunks(self.size.width as usize).collect();
        let num_rows = rows.len();

        for (row_idx, row) in rows.into_iter().enumerate() {
            for cell in row {
                // Handle attribute changes
                if cell.attrs != last_attrs {
                    // Reset all attributes first, then set the new ones
                    execute!(out, SetAttribute(Attribute::Reset))?;
                    if cell.attrs.bold {
                        execute!(out, SetAttribute(Attribute::Bold))?;
                    }
                    if cell.attrs.dim {
                        execute!(out, SetAttribute(Attribute::Dim))?;
                    }
                    if cell.attrs.italic {
                        execute!(out, SetAttribute(Attribute::Italic))?;
                    }
                    if cell.attrs.underline {
                        execute!(out, SetAttribute(Attribute::Underlined))?;
                    }
                    if cell.attrs.strike {
                        execute!(out, SetAttribute(Attribute::CrossedOut))?;
                    }
                    if cell.attrs.reverse {
                        execute!(out, SetAttribute(Attribute::Reverse))?;
                    }
                    last_attrs = cell.attrs;
                    // Reset color tracking since attribute reset clears colors
                    last_fg = None;
                    last_bg = None;
                }

                // Only send escape code if the color actually changed
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
            // Don't print newline after the last row to prevent terminal scrolling
            if row_idx < num_rows - 1 {
                write!(out, "\r\n")?;
            }
        }
        out.flush()?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell {
            symbol: ' ',
            fg: None,
            bg: None,
            attrs: TextAttributes::default(),
        });
        self.clip_stack.clear();
    }

    // === Test helpers ===

    /// Get the character at (x, y) for testing.
    pub fn get_char(&self, x: i32, y: i32) -> char {
        if x < 0 || x >= self.size.width as i32 || y < 0 || y >= self.size.height as i32 {
            return '\0';
        }
        let index = (y as usize) * (self.size.width as usize) + (x as usize);
        self.cells[index].symbol
    }

    /// Get all characters in a row as a string for testing.
    pub fn row_str(&self, y: i32) -> String {
        if y < 0 || y >= self.size.height as i32 {
            return String::new();
        }
        let start = (y as usize) * (self.size.width as usize);
        let end = start + (self.size.width as usize);
        self.cells[start..end].iter().map(|c| c.symbol).collect()
    }

    /// Check if a cell has a background color set (for testing scrollbar presence).
    pub fn has_bg_at(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.size.width as i32 || y < 0 || y >= self.size.height as i32 {
            return false;
        }
        let index = (y as usize) * (self.size.width as usize) + (x as usize);
        self.cells[index].bg.is_some()
    }
}

fn to_crossterm_color(c: RgbaColor) -> Color {
    // Terminals don't support true alpha transparency, so we pre-composite
    // semi-transparent colors against black (terminal default background).
    // Formula: result = base + (color - base) * alpha, where base = black (0,0,0)
    // Simplified: result = color * alpha
    let alpha = c.a;
    if alpha >= 1.0 {
        Color::Rgb {
            r: c.r,
            g: c.g,
            b: c.b,
        }
    } else {
        Color::Rgb {
            r: (c.r as f32 * alpha).round() as u8,
            g: (c.g as f32 * alpha).round() as u8,
            b: (c.b as f32 * alpha).round() as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Region::new tests
    // =========================================================================

    #[test]
    fn region_new_basic() {
        let r = Region::new(10, 20, 100, 50);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 50);
    }

    #[test]
    fn region_new_clamps_negative_width() {
        let r = Region::new(0, 0, -10, 50);
        assert_eq!(r.width, 0);
        assert_eq!(r.height, 50);
    }

    #[test]
    fn region_new_clamps_negative_height() {
        let r = Region::new(0, 0, 50, -20);
        assert_eq!(r.width, 50);
        assert_eq!(r.height, 0);
    }

    #[test]
    fn region_new_clamps_both_negative() {
        let r = Region::new(0, 0, -10, -20);
        assert_eq!(r.width, 0);
        assert_eq!(r.height, 0);
    }

    #[test]
    fn region_new_allows_negative_position() {
        let r = Region::new(-10, -20, 100, 50);
        assert_eq!(r.x, -10);
        assert_eq!(r.y, -20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 50);
    }

    // =========================================================================
    // Region::from_u16 tests
    // =========================================================================

    #[test]
    fn region_from_u16_converts_correctly() {
        let r = Region::from_u16(10, 20, 100, 50);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 50);
    }

    #[test]
    fn region_from_u16_max_values() {
        let r = Region::from_u16(u16::MAX, u16::MAX, u16::MAX, u16::MAX);
        assert_eq!(r.x, u16::MAX as i32);
        assert_eq!(r.y, u16::MAX as i32);
        assert_eq!(r.width, u16::MAX as i32);
        assert_eq!(r.height, u16::MAX as i32);
    }

    // =========================================================================
    // Region::intersection tests
    // =========================================================================

    #[test]
    fn intersection_overlapping_regions() {
        let a = Region::new(0, 0, 100, 100);
        let b = Region::new(50, 50, 100, 100);
        let i = a.intersection(&b);
        assert_eq!(i, Region::new(50, 50, 50, 50));
    }

    #[test]
    fn intersection_inner_contained_in_outer() {
        let outer = Region::new(0, 0, 100, 100);
        let inner = Region::new(25, 25, 50, 50);
        let i = outer.intersection(&inner);
        assert_eq!(i, inner);
    }

    #[test]
    fn intersection_outer_contains_inner() {
        let outer = Region::new(0, 0, 100, 100);
        let inner = Region::new(25, 25, 50, 50);
        let i = inner.intersection(&outer);
        assert_eq!(i, inner);
    }

    #[test]
    fn intersection_no_overlap_horizontal() {
        let a = Region::new(0, 0, 50, 50);
        let b = Region::new(100, 0, 50, 50);
        let i = a.intersection(&b);
        assert!(i.is_empty());
    }

    #[test]
    fn intersection_no_overlap_vertical() {
        let a = Region::new(0, 0, 50, 50);
        let b = Region::new(0, 100, 50, 50);
        let i = a.intersection(&b);
        assert!(i.is_empty());
    }

    #[test]
    fn intersection_no_overlap_diagonal() {
        let a = Region::new(0, 0, 50, 50);
        let b = Region::new(100, 100, 50, 50);
        let i = a.intersection(&b);
        assert!(i.is_empty());
    }

    #[test]
    fn intersection_touching_edge_horizontal() {
        let a = Region::new(0, 0, 50, 50);
        let b = Region::new(50, 0, 50, 50);
        let i = a.intersection(&b);
        assert!(i.is_empty()); // Touching but not overlapping
    }

    #[test]
    fn intersection_touching_edge_vertical() {
        let a = Region::new(0, 0, 50, 50);
        let b = Region::new(0, 50, 50, 50);
        let i = a.intersection(&b);
        assert!(i.is_empty());
    }

    #[test]
    fn intersection_with_negative_coords() {
        let a = Region::new(-50, -50, 100, 100);
        let b = Region::new(0, 0, 100, 100);
        let i = a.intersection(&b);
        assert_eq!(i, Region::new(0, 0, 50, 50));
    }

    #[test]
    fn intersection_both_negative() {
        let a = Region::new(-100, -100, 100, 100);
        let b = Region::new(-75, -75, 100, 100);
        let i = a.intersection(&b);
        assert_eq!(i, Region::new(-75, -75, 75, 75));
    }

    #[test]
    fn intersection_is_commutative() {
        let a = Region::new(10, 20, 100, 80);
        let b = Region::new(50, 40, 80, 100);
        assert_eq!(a.intersection(&b), b.intersection(&a));
    }

    // =========================================================================
    // Region::contains_point tests
    // =========================================================================

    #[test]
    fn contains_point_inside() {
        let r = Region::new(10, 10, 50, 50);
        assert!(r.contains_point(30, 30));
        assert!(r.contains_point(25, 35));
    }

    #[test]
    fn contains_point_top_left_corner() {
        let r = Region::new(10, 10, 50, 50);
        assert!(r.contains_point(10, 10)); // Inclusive
    }

    #[test]
    fn contains_point_bottom_right_corner() {
        let r = Region::new(10, 10, 50, 50);
        assert!(!r.contains_point(60, 60)); // Exclusive
        assert!(r.contains_point(59, 59)); // Just inside
    }

    #[test]
    fn contains_point_on_right_edge() {
        let r = Region::new(10, 10, 50, 50);
        assert!(!r.contains_point(60, 30)); // Right edge is exclusive
    }

    #[test]
    fn contains_point_on_bottom_edge() {
        let r = Region::new(10, 10, 50, 50);
        assert!(!r.contains_point(30, 60)); // Bottom edge is exclusive
    }

    #[test]
    fn contains_point_outside_left() {
        let r = Region::new(10, 10, 50, 50);
        assert!(!r.contains_point(5, 30));
    }

    #[test]
    fn contains_point_outside_above() {
        let r = Region::new(10, 10, 50, 50);
        assert!(!r.contains_point(30, 5));
    }

    #[test]
    fn contains_point_negative_region() {
        let r = Region::new(-50, -50, 100, 100);
        assert!(r.contains_point(-25, -25));
        assert!(r.contains_point(0, 0));
        assert!(r.contains_point(49, 49));
        assert!(!r.contains_point(50, 50)); // Just outside
        assert!(!r.contains_point(-51, 0)); // Outside left
    }

    #[test]
    fn contains_point_empty_region() {
        let r = Region::new(10, 10, 0, 0);
        assert!(!r.contains_point(10, 10));
    }

    // =========================================================================
    // Region::is_empty tests
    // =========================================================================

    #[test]
    fn is_empty_zero_width() {
        let r = Region::new(0, 0, 0, 100);
        assert!(r.is_empty());
    }

    #[test]
    fn is_empty_zero_height() {
        let r = Region::new(0, 0, 100, 0);
        assert!(r.is_empty());
    }

    #[test]
    fn is_empty_both_zero() {
        let r = Region::new(0, 0, 0, 0);
        assert!(r.is_empty());
    }

    #[test]
    fn is_empty_has_area() {
        let r = Region::new(0, 0, 1, 1);
        assert!(!r.is_empty());
    }

    #[test]
    fn is_empty_large_region() {
        let r = Region::new(0, 0, 1000, 1000);
        assert!(!r.is_empty());
    }

    // =========================================================================
    // Canvas clipping tests
    // =========================================================================

    // Helper to get a cell from canvas
    impl Canvas {
        #[cfg(test)]
        fn get_cell(&self, x: i32, y: i32) -> Option<&Cell> {
            if x < 0 || y < 0 || x >= self.size.width as i32 || y >= self.size.height as i32 {
                return None;
            }
            let index = (y as usize) * (self.size.width as usize) + (x as usize);
            self.cells.get(index)
        }
    }

    #[test]
    fn canvas_put_char_within_bounds() {
        let mut canvas = Canvas::new(80, 24);
        canvas.put_char(10, 5, 'X', None, None, TextAttributes::default());

        let cell = canvas.get_cell(10, 5).unwrap();
        assert_eq!(cell.symbol, 'X');
    }

    #[test]
    fn canvas_put_char_at_origin() {
        let mut canvas = Canvas::new(80, 24);
        canvas.put_char(0, 0, 'A', None, None, TextAttributes::default());

        let cell = canvas.get_cell(0, 0).unwrap();
        assert_eq!(cell.symbol, 'A');
    }

    #[test]
    fn canvas_put_char_at_max_corner() {
        let mut canvas = Canvas::new(80, 24);
        canvas.put_char(79, 23, 'Z', None, None, TextAttributes::default());

        let cell = canvas.get_cell(79, 23).unwrap();
        assert_eq!(cell.symbol, 'Z');
    }

    #[test]
    fn canvas_put_char_outside_right() {
        let mut canvas = Canvas::new(80, 24);
        canvas.put_char(80, 10, 'X', None, None, TextAttributes::default());
        // Should not panic, just no-op
    }

    #[test]
    fn canvas_put_char_outside_bottom() {
        let mut canvas = Canvas::new(80, 24);
        canvas.put_char(10, 24, 'X', None, None, TextAttributes::default());
        // Should not panic, just no-op
    }

    #[test]
    fn canvas_put_char_negative_x() {
        let mut canvas = Canvas::new(80, 24);
        canvas.put_char(-5, 10, 'X', None, None, TextAttributes::default());
        // Should not panic, just no-op
    }

    #[test]
    fn canvas_put_char_negative_y() {
        let mut canvas = Canvas::new(80, 24);
        canvas.put_char(10, -5, 'X', None, None, TextAttributes::default());
        // Should not panic, just no-op
    }

    #[test]
    fn canvas_push_clip_restricts_drawing() {
        let mut canvas = Canvas::new(80, 24);
        canvas.push_clip(Region::new(10, 10, 20, 10));

        // Inside clip - should draw
        canvas.put_char(15, 15, 'A', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(15, 15).unwrap().symbol, 'A');

        // Outside clip left - should NOT draw
        canvas.put_char(5, 15, 'B', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(5, 15).unwrap().symbol, ' ');

        // Outside clip right - should NOT draw
        canvas.put_char(35, 15, 'C', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(35, 15).unwrap().symbol, ' ');

        // Outside clip above - should NOT draw
        canvas.put_char(15, 5, 'D', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(15, 5).unwrap().symbol, ' ');

        // Outside clip below - should NOT draw
        canvas.put_char(15, 25, 'E', None, None, TextAttributes::default());
        // y=25 is outside canvas anyway
    }

    #[test]
    fn canvas_nested_clips_intersect() {
        let mut canvas = Canvas::new(80, 24);

        // First clip: 0,0 to 50,50
        canvas.push_clip(Region::new(0, 0, 50, 20));

        // Second clip: 25,10 to 75,30 (but intersected with first = 25,10 to 50,20)
        canvas.push_clip(Region::new(25, 10, 50, 20));

        // Inside intersection (25-49, 10-19)
        canvas.put_char(30, 15, 'A', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(30, 15).unwrap().symbol, 'A');

        // Inside first clip but outside intersection
        canvas.put_char(10, 5, 'B', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(10, 5).unwrap().symbol, ' ');

        // Outside both
        canvas.put_char(60, 15, 'C', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(60, 15).unwrap().symbol, ' ');
    }

    #[test]
    fn canvas_pop_clip_restores_previous() {
        let mut canvas = Canvas::new(80, 24);

        // First clip
        canvas.push_clip(Region::new(0, 0, 50, 20));

        // Second (smaller) clip
        canvas.push_clip(Region::new(10, 10, 10, 5));

        // Can only draw in small region
        canvas.put_char(5, 5, 'A', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(5, 5).unwrap().symbol, ' ');

        // Pop back to first clip
        canvas.pop_clip();

        // Now can draw in larger region
        canvas.put_char(5, 5, 'B', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(5, 5).unwrap().symbol, 'B');
    }

    #[test]
    fn canvas_pop_all_clips_restores_full_screen() {
        let mut canvas = Canvas::new(80, 24);

        canvas.push_clip(Region::new(10, 10, 10, 10));
        canvas.pop_clip();

        // Should be able to draw anywhere
        canvas.put_char(0, 0, 'A', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(0, 0).unwrap().symbol, 'A');

        canvas.put_char(79, 23, 'Z', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(79, 23).unwrap().symbol, 'Z');
    }

    #[test]
    fn canvas_clear_resets_clip_stack() {
        let mut canvas = Canvas::new(80, 24);

        canvas.push_clip(Region::new(10, 10, 10, 10));
        canvas.push_clip(Region::new(15, 15, 5, 5));

        canvas.clear();

        // Clip stack should be empty, full screen available
        canvas.put_char(0, 0, 'A', None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(0, 0).unwrap().symbol, 'A');
    }

    #[test]
    fn canvas_put_str_basic() {
        let mut canvas = Canvas::new(80, 24);
        canvas.put_str(5, 10, "Hello", None, None, TextAttributes::default());

        assert_eq!(canvas.get_cell(5, 10).unwrap().symbol, 'H');
        assert_eq!(canvas.get_cell(6, 10).unwrap().symbol, 'e');
        assert_eq!(canvas.get_cell(7, 10).unwrap().symbol, 'l');
        assert_eq!(canvas.get_cell(8, 10).unwrap().symbol, 'l');
        assert_eq!(canvas.get_cell(9, 10).unwrap().symbol, 'o');
    }

    #[test]
    fn canvas_put_str_clips_left() {
        let mut canvas = Canvas::new(80, 24);
        canvas.push_clip(Region::new(5, 0, 70, 24));

        // String starts before clip region
        canvas.put_str(2, 10, "Hello", None, None, TextAttributes::default());

        // First 3 chars (at x=2,3,4) should be clipped
        assert_eq!(canvas.get_cell(2, 10).unwrap().symbol, ' ');
        assert_eq!(canvas.get_cell(3, 10).unwrap().symbol, ' ');
        assert_eq!(canvas.get_cell(4, 10).unwrap().symbol, ' ');
        // Last 2 chars (at x=5,6) should be drawn
        assert_eq!(canvas.get_cell(5, 10).unwrap().symbol, 'l');
        assert_eq!(canvas.get_cell(6, 10).unwrap().symbol, 'o');
    }

    #[test]
    fn canvas_put_str_clips_right() {
        let mut canvas = Canvas::new(80, 24);
        canvas.push_clip(Region::new(0, 0, 8, 24));

        canvas.put_str(5, 10, "Hello", None, None, TextAttributes::default());

        // First 3 chars should be drawn
        assert_eq!(canvas.get_cell(5, 10).unwrap().symbol, 'H');
        assert_eq!(canvas.get_cell(6, 10).unwrap().symbol, 'e');
        assert_eq!(canvas.get_cell(7, 10).unwrap().symbol, 'l');
        // Last 2 chars should be clipped
        assert_eq!(canvas.get_cell(8, 10).unwrap().symbol, ' ');
        assert_eq!(canvas.get_cell(9, 10).unwrap().symbol, ' ');
    }

    #[test]
    fn canvas_put_str_clips_vertically() {
        let mut canvas = Canvas::new(80, 24);
        canvas.push_clip(Region::new(0, 5, 80, 10));

        // String above clip
        canvas.put_str(10, 3, "Above", None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(10, 3).unwrap().symbol, ' ');

        // String below clip
        canvas.put_str(10, 16, "Below", None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(10, 16).unwrap().symbol, ' ');

        // String inside clip
        canvas.put_str(10, 10, "Inside", None, None, TextAttributes::default());
        assert_eq!(canvas.get_cell(10, 10).unwrap().symbol, 'I');
    }
}
