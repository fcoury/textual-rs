//! Box-drawing character lookup for keyline rendering.
//!
//! This module provides a lookup table for Unicode box-drawing characters
//! used to render connected borders (keylines) around widgets.
//!
//! ## Quad System
//!
//! Each cell's box-drawing character is determined by a "quad" - a tuple of
//! four values `(top, right, bottom, left)` where each value indicates the
//! line type on that edge:
//!
//! - `0`: No line
//! - `1`: Thin line (─, │)
//! - `2`: Heavy line (━, ┃)
//! - `3`: Double line (═, ║)
//!
//! ## Example
//!
//! ```
//! use textual::box_drawing::{get_box_char, Quad};
//!
//! // A corner with heavy lines on right and bottom
//! let quad: Quad = (0, 2, 2, 0);
//! assert_eq!(get_box_char(quad), Some('┏'));
//! ```

use std::collections::HashMap;
use std::sync::LazyLock;

/// A quad representing line types on (top, right, bottom, left) edges.
/// Values: 0=none, 1=thin, 2=heavy, 3=double
pub type Quad = (u8, u8, u8, u8);

/// Lookup table for box-drawing characters indexed by quad.
static BOX_CHARACTERS: LazyLock<HashMap<Quad, char>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Horizontal lines
    map.insert((0, 1, 0, 1), '─'); // thin horizontal
    map.insert((0, 2, 0, 2), '━'); // heavy horizontal
    map.insert((0, 3, 0, 3), '═'); // double horizontal

    // Vertical lines
    map.insert((1, 0, 1, 0), '│'); // thin vertical
    map.insert((2, 0, 2, 0), '┃'); // heavy vertical
    map.insert((3, 0, 3, 0), '║'); // double vertical

    // Thin corners
    map.insert((0, 1, 1, 0), '┌'); // top-left
    map.insert((0, 0, 1, 1), '┐'); // top-right
    map.insert((1, 1, 0, 0), '└'); // bottom-left
    map.insert((1, 0, 0, 1), '┘'); // bottom-right

    // Heavy corners
    map.insert((0, 2, 2, 0), '┏'); // top-left
    map.insert((0, 0, 2, 2), '┓'); // top-right
    map.insert((2, 2, 0, 0), '┗'); // bottom-left
    map.insert((2, 0, 0, 2), '┛'); // bottom-right

    // Double corners
    map.insert((0, 3, 3, 0), '╔'); // top-left
    map.insert((0, 0, 3, 3), '╗'); // top-right
    map.insert((3, 3, 0, 0), '╚'); // bottom-left
    map.insert((3, 0, 0, 3), '╝'); // bottom-right

    // Thin T-junctions
    map.insert((0, 1, 1, 1), '┬'); // top T
    map.insert((1, 1, 0, 1), '┴'); // bottom T
    map.insert((1, 1, 1, 0), '├'); // left T
    map.insert((1, 0, 1, 1), '┤'); // right T

    // Heavy T-junctions
    map.insert((0, 2, 2, 2), '┳'); // top T
    map.insert((2, 2, 0, 2), '┻'); // bottom T
    map.insert((2, 2, 2, 0), '┣'); // left T
    map.insert((2, 0, 2, 2), '┫'); // right T

    // Double T-junctions
    map.insert((0, 3, 3, 3), '╦'); // top T
    map.insert((3, 3, 0, 3), '╩'); // bottom T
    map.insert((3, 3, 3, 0), '╠'); // left T
    map.insert((3, 0, 3, 3), '╣'); // right T

    // Cross junctions
    map.insert((1, 1, 1, 1), '┼'); // thin cross
    map.insert((2, 2, 2, 2), '╋'); // heavy cross
    map.insert((3, 3, 3, 3), '╬'); // double cross

    // Mixed thin/heavy corners
    // Heavy horizontal, thin vertical
    map.insert((0, 2, 1, 0), '┍'); // top-left
    map.insert((0, 0, 1, 2), '┑'); // top-right
    map.insert((1, 2, 0, 0), '┕'); // bottom-left
    map.insert((1, 0, 0, 2), '┙'); // bottom-right

    // Thin horizontal, heavy vertical
    map.insert((0, 1, 2, 0), '┎'); // top-left
    map.insert((0, 0, 2, 1), '┒'); // top-right
    map.insert((2, 1, 0, 0), '┖'); // bottom-left
    map.insert((2, 0, 0, 1), '┚'); // bottom-right

    // Mixed T-junctions (thin horizontal, heavy vertical)
    map.insert((0, 1, 2, 1), '┰'); // top T
    map.insert((2, 1, 0, 1), '┸'); // bottom T
    map.insert((2, 1, 2, 0), '┠'); // left T
    map.insert((2, 0, 2, 1), '┨'); // right T

    // Mixed T-junctions (heavy horizontal, thin vertical)
    map.insert((0, 2, 1, 2), '┯'); // top T
    map.insert((1, 2, 0, 2), '┷'); // bottom T
    map.insert((1, 2, 1, 0), '┝'); // left T
    map.insert((1, 0, 1, 2), '┥'); // right T

    // Mixed cross junctions
    map.insert((1, 2, 1, 2), '┿'); // thin vertical, heavy horizontal
    map.insert((2, 1, 2, 1), '╂'); // heavy vertical, thin horizontal
    map.insert((2, 2, 1, 2), '╈'); // heavy top/horizontal, thin bottom
    map.insert((1, 2, 2, 2), '╇'); // thin top, heavy bottom/horizontal
    map.insert((2, 2, 2, 1), '╊'); // heavy top/bottom/right, thin left
    map.insert((2, 1, 2, 2), '╉'); // heavy top/bottom/left, thin right

    // Additional mixed T-junctions
    map.insert((2, 2, 1, 0), '┡'); // left T: heavy top, thin bottom, heavy right
    map.insert((1, 2, 2, 0), '┢'); // left T: thin top, heavy bottom, heavy right
    map.insert((2, 0, 1, 2), '┩'); // right T: heavy top, thin bottom, heavy left
    map.insert((1, 0, 2, 2), '┪'); // right T: thin top, heavy bottom, heavy left
    map.insert((0, 2, 2, 1), '┲'); // top T: heavy right/bottom, thin left
    map.insert((0, 1, 2, 2), '┱'); // top T: thin right, heavy bottom/left
    map.insert((2, 2, 0, 1), '┺'); // bottom T: heavy top/right, thin left
    map.insert((2, 1, 0, 2), '┹'); // bottom T: heavy top/left, thin right

    // Half lines (for edge caps)
    map.insert((0, 1, 0, 0), '╴'); // thin left half
    map.insert((0, 0, 0, 1), '╶'); // thin right half
    map.insert((1, 0, 0, 0), '╵'); // thin top half
    map.insert((0, 0, 1, 0), '╷'); // thin bottom half
    map.insert((0, 2, 0, 0), '╸'); // heavy left half
    map.insert((0, 0, 0, 2), '╺'); // heavy right half
    map.insert((2, 0, 0, 0), '╹'); // heavy top half
    map.insert((0, 0, 2, 0), '╻'); // heavy bottom half

    // Mixed half lines
    map.insert((0, 2, 0, 1), '╼'); // thin left, heavy right
    map.insert((0, 1, 0, 2), '╾'); // heavy left, thin right
    map.insert((1, 0, 2, 0), '╽'); // thin top, heavy bottom
    map.insert((2, 0, 1, 0), '╿'); // heavy top, thin bottom

    // Double/single mixed corners
    map.insert((0, 3, 1, 0), '╒'); // double right, single down
    map.insert((0, 0, 1, 3), '╕'); // double left, single down
    map.insert((1, 3, 0, 0), '╘'); // double right, single up
    map.insert((1, 0, 0, 3), '╛'); // double left, single up
    map.insert((0, 1, 3, 0), '╓'); // single right, double down
    map.insert((0, 0, 3, 1), '╖'); // single left, double down
    map.insert((3, 1, 0, 0), '╙'); // single right, double up
    map.insert((3, 0, 0, 1), '╜'); // single left, double up

    // Double/single mixed T-junctions
    map.insert((0, 3, 1, 3), '╤'); // double horizontal, single down
    map.insert((1, 3, 0, 3), '╧'); // double horizontal, single up
    map.insert((1, 3, 1, 0), '╞'); // single vertical, double right
    map.insert((1, 0, 1, 3), '╡'); // single vertical, double left
    map.insert((0, 1, 3, 1), '╥'); // single horizontal, double down
    map.insert((3, 1, 0, 1), '╨'); // single horizontal, double up
    map.insert((3, 1, 3, 0), '╟'); // double vertical, single right
    map.insert((3, 0, 3, 1), '╢'); // double vertical, single left

    // Double/single mixed cross
    map.insert((1, 3, 1, 3), '╪'); // single vertical, double horizontal
    map.insert((3, 1, 3, 1), '╫'); // double vertical, single horizontal

    map
});

/// Get the box-drawing character for a given quad.
///
/// Returns `None` if no character exists for the given combination.
pub fn get_box_char(quad: Quad) -> Option<char> {
    BOX_CHARACTERS.get(&quad).copied()
}

/// Combine two quads by taking the maximum line type for each edge.
///
/// This is used when multiple widgets share an edge and we want the
/// "strongest" line to win.
pub fn combine_quads(a: Quad, b: Quad) -> Quad {
    (a.0.max(b.0), a.1.max(b.1), a.2.max(b.2), a.3.max(b.3))
}

/// Get the quad for a widget's position in a grid.
///
/// # Arguments
/// * `row` - Widget's row (0-indexed)
/// * `col` - Widget's column (0-indexed)
/// * `rows` - Total rows in grid
/// * `cols` - Total columns in grid
/// * `line_type` - Line type (1=thin, 2=heavy, 3=double)
///
/// # Returns
/// A quad representing which edges should have lines.
pub fn widget_edge_quad(row: usize, col: usize, rows: usize, cols: usize, line_type: u8) -> Quad {
    let top = if row == 0 { line_type } else { 0 };
    let bottom = if row == rows - 1 { line_type } else { 0 };
    let left = if col == 0 { line_type } else { 0 };
    let right = if col == cols - 1 { line_type } else { 0 };
    (top, right, bottom, left)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thin_corners() {
        assert_eq!(get_box_char((0, 1, 1, 0)), Some('┌'));
        assert_eq!(get_box_char((0, 0, 1, 1)), Some('┐'));
        assert_eq!(get_box_char((1, 1, 0, 0)), Some('└'));
        assert_eq!(get_box_char((1, 0, 0, 1)), Some('┘'));
    }

    #[test]
    fn test_heavy_corners() {
        assert_eq!(get_box_char((0, 2, 2, 0)), Some('┏'));
        assert_eq!(get_box_char((0, 0, 2, 2)), Some('┓'));
        assert_eq!(get_box_char((2, 2, 0, 0)), Some('┗'));
        assert_eq!(get_box_char((2, 0, 0, 2)), Some('┛'));
    }

    #[test]
    fn test_double_corners() {
        assert_eq!(get_box_char((0, 3, 3, 0)), Some('╔'));
        assert_eq!(get_box_char((0, 0, 3, 3)), Some('╗'));
        assert_eq!(get_box_char((3, 3, 0, 0)), Some('╚'));
        assert_eq!(get_box_char((3, 0, 0, 3)), Some('╝'));
    }

    #[test]
    fn test_lines() {
        assert_eq!(get_box_char((0, 1, 0, 1)), Some('─'));
        assert_eq!(get_box_char((1, 0, 1, 0)), Some('│'));
        assert_eq!(get_box_char((0, 2, 0, 2)), Some('━'));
        assert_eq!(get_box_char((2, 0, 2, 0)), Some('┃'));
        assert_eq!(get_box_char((0, 3, 0, 3)), Some('═'));
        assert_eq!(get_box_char((3, 0, 3, 0)), Some('║'));
    }

    #[test]
    fn test_t_junctions() {
        // Thin
        assert_eq!(get_box_char((0, 1, 1, 1)), Some('┬'));
        assert_eq!(get_box_char((1, 1, 0, 1)), Some('┴'));
        assert_eq!(get_box_char((1, 1, 1, 0)), Some('├'));
        assert_eq!(get_box_char((1, 0, 1, 1)), Some('┤'));
        // Heavy
        assert_eq!(get_box_char((0, 2, 2, 2)), Some('┳'));
        assert_eq!(get_box_char((2, 2, 0, 2)), Some('┻'));
        assert_eq!(get_box_char((2, 2, 2, 0)), Some('┣'));
        assert_eq!(get_box_char((2, 0, 2, 2)), Some('┫'));
    }

    #[test]
    fn test_crosses() {
        assert_eq!(get_box_char((1, 1, 1, 1)), Some('┼'));
        assert_eq!(get_box_char((2, 2, 2, 2)), Some('╋'));
        assert_eq!(get_box_char((3, 3, 3, 3)), Some('╬'));
    }

    #[test]
    fn test_combine_quads() {
        assert_eq!(combine_quads((0, 1, 0, 0), (0, 0, 1, 0)), (0, 1, 1, 0));
        assert_eq!(combine_quads((1, 1, 0, 0), (0, 0, 2, 2)), (1, 1, 2, 2));
        assert_eq!(combine_quads((2, 2, 2, 2), (1, 1, 1, 1)), (2, 2, 2, 2));
    }

    #[test]
    fn test_widget_edge_quad() {
        // Top-left corner in 2x2 grid
        assert_eq!(widget_edge_quad(0, 0, 2, 2, 2), (2, 0, 0, 2));
        // Top-right corner in 2x2 grid
        assert_eq!(widget_edge_quad(0, 1, 2, 2, 2), (2, 2, 0, 0));
        // Bottom-left corner in 2x2 grid
        assert_eq!(widget_edge_quad(1, 0, 2, 2, 2), (0, 0, 2, 2));
        // Bottom-right corner in 2x2 grid
        assert_eq!(widget_edge_quad(1, 1, 2, 2, 2), (0, 2, 2, 0));
    }

    #[test]
    fn test_invalid_quad() {
        // Non-existent combination
        assert_eq!(get_box_char((1, 2, 3, 1)), None);
    }
}
