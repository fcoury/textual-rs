//! Border character definitions for terminal rendering.
//!
//! This module provides the character grids used to render borders around
//! widgets. Each border type is defined as a 3×3 grid of characters:
//!
//! ```text
//! ╭─╮  <- top: corners and horizontal line
//! │ │  <- middle: vertical lines
//! ╰─╯  <- bottom: corners and horizontal line
//! ```
//!
//! The grid positions are:
//! ```text
//! [0][0] [0][1] [0][2]  <- top-left, top, top-right
//! [1][0] [1][1] [1][2]  <- left, center (unused), right
//! [2][0] [2][1] [2][2]  <- bottom-left, bottom, bottom-right
//! ```

use phf::phf_map;

/// A 3×3 character grid for border rendering.
///
/// Row indices: 0=top, 1=middle, 2=bottom
/// Column indices: 0=left, 1=center, 2=right
pub type BorderChars = [[char; 3]; 3];

/// Style zone mappings for border rendering.
///
/// Each position in the 3×3 grid indicates which style to use:
/// - 0: Use the widget's style (inner)
/// - 1: Use the outer/parent style
/// - 2: Reversed outer bg with inner fg (for block characters like ▊)
/// - 3: Reversed inner bg with outer fg
///
/// This is used for border types like "inner" where the border characters
/// should use the outer style rather than the widget's style.
pub type BorderLocations = [[u8; 3]; 3];

/// Standard border location: all positions use widget style.
const STANDARD_LOCATIONS: BorderLocations = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];

/// Inner border location: border uses outer style, center uses widget style.
const INNER_LOCATIONS: BorderLocations = [[1, 1, 1], [1, 0, 1], [1, 1, 1]];

/// Outer border location: all positions use outer style.
const OUTER_LOCATIONS: BorderLocations = [[1, 1, 1], [1, 1, 1], [1, 1, 1]];

/// Tall border location: left=reversed outer/inner, center=inner, right=outer.
const TALL_LOCATIONS: BorderLocations = [[2, 0, 1], [2, 0, 1], [2, 0, 1]];

/// Panel border location: same as tall (left=reversed, center=inner, right=outer).
const PANEL_LOCATIONS: BorderLocations = [[2, 0, 1], [2, 0, 1], [2, 0, 1]];

/// Wide border location: special mixed zones for horizontal emphasis.
const WIDE_LOCATIONS: BorderLocations = [[1, 1, 1], [0, 1, 3], [1, 1, 1]];

/// Map of border type names to their character grids.
pub static BORDER_CHARS: phf::Map<&'static str, BorderChars> = phf_map! {
    "none" => [
        [' ', ' ', ' '],
        [' ', ' ', ' '],
        [' ', ' ', ' '],
    ],
    "hidden" => [
        [' ', ' ', ' '],
        [' ', ' ', ' '],
        [' ', ' ', ' '],
    ],
    "blank" => [
        [' ', ' ', ' '],
        [' ', ' ', ' '],
        [' ', ' ', ' '],
    ],
    "round" => [
        ['╭', '─', '╮'],
        ['│', ' ', '│'],
        ['╰', '─', '╯'],
    ],
    "solid" => [
        ['┌', '─', '┐'],
        ['│', ' ', '│'],
        ['└', '─', '┘'],
    ],
    "double" => [
        ['╔', '═', '╗'],
        ['║', ' ', '║'],
        ['╚', '═', '╝'],
    ],
    "heavy" => [
        ['┏', '━', '┓'],
        ['┃', ' ', '┃'],
        ['┗', '━', '┛'],
    ],
    "ascii" => [
        ['+', '-', '+'],
        ['|', ' ', '|'],
        ['+', '-', '+'],
    ],
    "dashed" => [
        ['┏', '╍', '┓'],
        ['╏', ' ', '╏'],
        ['┗', '╍', '┛'],
    ],
    "thick" => [
        ['█', '▀', '█'],
        ['█', ' ', '█'],
        ['█', '▄', '█'],
    ],
    "tall" => [
        ['▊', '▔', '▎'],
        ['▊', ' ', '▎'],
        ['▊', '▁', '▎'],
    ],
    "panel" => [
        ['▊', '█', '▎'],
        ['▊', ' ', '▎'],
        ['▊', '▁', '▎'],
    ],
    "wide" => [
        ['▁', '▁', '▁'],
        ['▎', ' ', '▊'],
        ['▔', '▔', '▔'],
    ],
    "inner" => [
        ['▗', '▄', '▖'],
        ['▐', ' ', '▌'],
        ['▝', '▀', '▘'],
    ],
    "outer" => [
        ['▛', '▀', '▜'],
        ['▌', ' ', '▐'],
        ['▙', '▄', '▟'],
    ],
    "hkey" => [
        ['▔', '▔', '▔'],
        [' ', ' ', ' '],
        ['▁', '▁', '▁'],
    ],
    "vkey" => [
        ['▏', ' ', '▕'],
        ['▏', ' ', '▕'],
        ['▏', ' ', '▕'],
    ],
};

/// Map of border type names to their style zone mappings.
pub static BORDER_LOCATIONS: phf::Map<&'static str, BorderLocations> = phf_map! {
    "none" => STANDARD_LOCATIONS,
    "hidden" => STANDARD_LOCATIONS,
    "blank" => STANDARD_LOCATIONS,
    "round" => STANDARD_LOCATIONS,
    "solid" => STANDARD_LOCATIONS,
    "double" => STANDARD_LOCATIONS,
    "heavy" => STANDARD_LOCATIONS,
    "ascii" => STANDARD_LOCATIONS,
    "dashed" => STANDARD_LOCATIONS,
    "thick" => STANDARD_LOCATIONS,
    "tall" => TALL_LOCATIONS,
    "panel" => PANEL_LOCATIONS,
    "wide" => WIDE_LOCATIONS,
    "inner" => INNER_LOCATIONS,
    "outer" => OUTER_LOCATIONS,
    "hkey" => STANDARD_LOCATIONS,
    "vkey" => STANDARD_LOCATIONS,
};

/// Gets the border characters for a given border type.
///
/// Returns the "solid" border if the type is not found.
pub fn get_border_chars(border_type: &str) -> &'static BorderChars {
    BORDER_CHARS.get(border_type).unwrap_or_else(|| {
        BORDER_CHARS
            .get("solid")
            .expect("solid border must exist")
    })
}

/// Gets the border locations for a given border type.
///
/// Returns standard locations if the type is not found.
pub fn get_border_locations(border_type: &str) -> &'static BorderLocations {
    BORDER_LOCATIONS
        .get(border_type)
        .unwrap_or(&STANDARD_LOCATIONS)
}

/// Returns true if the border type is "none", "hidden", or "blank".
pub fn is_invisible_border(border_type: &str) -> bool {
    matches!(border_type, "none" | "hidden" | "blank")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn border_chars_round_exists() {
        let chars = get_border_chars("round");
        assert_eq!(chars[0][0], '╭');
        assert_eq!(chars[0][1], '─');
        assert_eq!(chars[0][2], '╮');
        assert_eq!(chars[1][0], '│');
        assert_eq!(chars[1][2], '│');
        assert_eq!(chars[2][0], '╰');
        assert_eq!(chars[2][2], '╯');
    }

    #[test]
    fn border_chars_solid_exists() {
        let chars = get_border_chars("solid");
        assert_eq!(chars[0][0], '┌');
        assert_eq!(chars[0][2], '┐');
        assert_eq!(chars[2][0], '└');
        assert_eq!(chars[2][2], '┘');
    }

    #[test]
    fn border_chars_unknown_returns_solid() {
        let chars = get_border_chars("nonexistent");
        assert_eq!(chars[0][0], '┌'); // solid's top-left
    }

    #[test]
    fn border_locations_standard() {
        let locs = get_border_locations("round");
        // All positions should use widget style (0)
        for row in locs {
            for &loc in row {
                assert_eq!(loc, 0);
            }
        }
    }

    #[test]
    fn border_locations_inner() {
        let locs = get_border_locations("inner");
        // Center should use widget style, edges use outer
        assert_eq!(locs[1][1], 0); // center
        assert_eq!(locs[0][0], 1); // top-left uses outer
        assert_eq!(locs[0][1], 1); // top uses outer
    }

    #[test]
    fn is_invisible_border_check() {
        assert!(is_invisible_border("none"));
        assert!(is_invisible_border("hidden"));
        assert!(is_invisible_border("blank"));
        assert!(!is_invisible_border("solid"));
        assert!(!is_invisible_border("round"));
    }
}
