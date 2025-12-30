//! Hatch pattern types for filling widget areas.
//!
//! Hatch patterns are used to fill areas with repeating characters,
//! creating visual textures similar to cross-hatching in art.

use crate::types::RgbaColor;

/// Hatch pattern character mapping (matches Python Textual exactly).
///
/// These Unicode box-drawing characters create clean visual patterns:
/// - `left`: `╲` (U+2572) - diagonal backslash
/// - `right`: `╱` (U+2571) - diagonal slash
/// - `cross`: `╳` (U+2573) - diagonal cross
/// - `horizontal`: `─` (U+2500) - horizontal line
/// - `vertical`: `│` (U+2502) - vertical line
pub const HATCH_CHARS: &[(&str, char)] = &[
    ("left", '╲'),       // U+2572 - Box Drawings Light Diagonal Upper Left to Lower Right
    ("right", '╱'),      // U+2571 - Box Drawings Light Diagonal Upper Right to Lower Left
    ("cross", '╳'),      // U+2573 - Box Drawings Light Diagonal Cross
    ("horizontal", '─'), // U+2500 - Box Drawings Light Horizontal
    ("vertical", '│'),   // U+2502 - Box Drawings Light Vertical
];

/// A hatch pattern type.
///
/// Predefined patterns use Unicode box-drawing characters, while custom
/// patterns can use any character.
#[derive(Debug, Clone, PartialEq)]
pub enum HatchPattern {
    /// Diagonal backslash: `╲`
    Left,
    /// Diagonal slash: `╱`
    Right,
    /// Diagonal cross: `╳`
    Cross,
    /// Horizontal lines: `─`
    Horizontal,
    /// Vertical lines: `│`
    Vertical,
    /// Custom character
    Custom(char),
}

impl HatchPattern {
    /// Get the character used to render this pattern.
    pub fn char(&self) -> char {
        match self {
            HatchPattern::Left => '╲',
            HatchPattern::Right => '╱',
            HatchPattern::Cross => '╳',
            HatchPattern::Horizontal => '─',
            HatchPattern::Vertical => '│',
            HatchPattern::Custom(c) => *c,
        }
    }

    /// Parse a pattern from a string.
    ///
    /// Accepts pattern names ("left", "right", "cross", "horizontal", "vertical")
    /// or a single character for custom patterns.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "left" => Some(HatchPattern::Left),
            "right" => Some(HatchPattern::Right),
            "cross" => Some(HatchPattern::Cross),
            "horizontal" => Some(HatchPattern::Horizontal),
            "vertical" => Some(HatchPattern::Vertical),
            _ => {
                // Try to parse as a single character (possibly quoted)
                let trimmed = s.trim_matches('"').trim_matches('\'');
                let mut chars = trimmed.chars();
                if let Some(c) = chars.next() {
                    if chars.next().is_none() {
                        return Some(HatchPattern::Custom(c));
                    }
                }
                None
            }
        }
    }
}

impl Default for HatchPattern {
    fn default() -> Self {
        HatchPattern::Cross
    }
}

/// A hatch fill specification.
///
/// Defines a pattern, color, and opacity for filling a widget area.
#[derive(Debug, Clone, PartialEq)]
pub struct Hatch {
    /// The pattern to use for filling.
    pub pattern: HatchPattern,
    /// The color of the hatch pattern.
    pub color: RgbaColor,
    /// The opacity of the hatch (0.0 - 1.0, default 1.0).
    /// This creates a "see-through" effect where the background shows through.
    pub opacity: f32,
}

impl Hatch {
    /// Create a new hatch with the given pattern and color, full opacity.
    pub fn new(pattern: HatchPattern, color: RgbaColor) -> Self {
        Self {
            pattern,
            color,
            opacity: 1.0,
        }
    }

    /// Set the opacity (0.0 - 1.0).
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Default for Hatch {
    fn default() -> Self {
        Self {
            pattern: HatchPattern::default(),
            color: RgbaColor::default(),
            opacity: 1.0,
        }
    }
}
