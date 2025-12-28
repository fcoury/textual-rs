//! Text styling and alignment types.
//!
//! This module provides types for controlling text appearance:
//!
//! - [`TextStyle`]: Terminal text modifiers (bold, italic, underline, etc.)
//! - [`TextAlign`]: Horizontal text alignment within a line
//! - [`AlignHorizontal`], [`AlignVertical`]: Content alignment within a container
//!
//! ## Terminal Text Modifiers
//!
//! Text styles map to ANSI SGR (Select Graphic Rendition) codes:
//!
//! | Modifier     | ANSI Code | Description                    |
//! |--------------|-----------|--------------------------------|
//! | `bold`       | 1         | Bold/increased intensity       |
//! | `dim`        | 2         | Faint/decreased intensity      |
//! | `italic`     | 3         | Italic text                    |
//! | `underline`  | 4         | Single underline               |
//! | `blink`      | 5         | Slow blink                     |
//! | `reverse`    | 7         | Swap foreground/background     |
//! | `strike`     | 9         | Strikethrough                  |
//!
//! ## CSS Syntax
//!
//! ```css
//! Label {
//!     text-style: bold italic;
//!     text-align: center;
//! }
//! ```

/// Text style modifiers for terminal rendering.
///
/// Combines multiple ANSI text attributes that can be applied
/// together. Styles are merged during the CSS cascade.
///
/// # Examples
///
/// ```
/// use tcss::types::TextStyle;
///
/// let mut style = TextStyle::default();
/// style.bold = true;
/// style.italic = true;
///
/// assert!(!style.is_none()); // Has active styles
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextStyle {
    /// Bold/increased intensity.
    pub bold: bool,
    /// Dim/faint intensity.
    pub dim: bool,
    /// Italic text.
    pub italic: bool,
    /// Single underline.
    pub underline: bool,
    /// Double underline (extended attribute).
    pub underline2: bool,
    /// Slow blink.
    pub blink: bool,
    /// Rapid blink (extended attribute).
    pub blink2: bool,
    /// Swap foreground and background colors.
    pub reverse: bool,
    /// Strikethrough text.
    pub strike: bool,
    /// Overline text (extended attribute).
    pub overline: bool,
    /// Theme variable name (e.g., "link-style") for runtime resolution.
    pub theme_var: Option<String>,
}

impl TextStyle {
    /// Creates a default style with all modifiers disabled.
    pub fn none() -> Self {
        Self::default()
    }

    /// Creates a theme variable reference.
    ///
    /// The variable will be resolved from the active theme at cascade time.
    ///
    /// # Example
    ///
    /// ```
    /// use tcss::types::TextStyle;
    ///
    /// let style = TextStyle::theme_variable("link-style");
    /// assert_eq!(style.theme_var, Some("link-style".to_string()));
    /// ```
    pub fn theme_variable(name: &str) -> Self {
        Self {
            theme_var: Some(name.to_string()),
            ..Default::default()
        }
    }

    /// Check if no styles are applied (and no theme variable reference).
    pub fn is_none(&self) -> bool {
        self.theme_var.is_none()
            && !self.bold
            && !self.dim
            && !self.italic
            && !self.underline
            && !self.underline2
            && !self.blink
            && !self.blink2
            && !self.reverse
            && !self.strike
            && !self.overline
    }

    /// Merges another style into this one.
    /// Used during the CSS cascade where multiple rules apply to one widget.
    ///
    /// Note: If `other` has a theme_var, it replaces this style's theme_var.
    pub fn merge(&mut self, other: &TextStyle) {
        if other.bold {
            self.bold = true;
        }
        if other.dim {
            self.dim = true;
        }
        if other.italic {
            self.italic = true;
        }
        if other.underline {
            self.underline = true;
        }
        if other.underline2 {
            self.underline2 = true;
        }
        if other.blink {
            self.blink = true;
        }
        if other.blink2 {
            self.blink2 = true;
        }
        if other.reverse {
            self.reverse = true;
        }
        if other.strike {
            self.strike = true;
        }
        if other.overline {
            self.overline = true;
        }
        // Theme variable from the other style takes precedence
        if other.theme_var.is_some() {
            self.theme_var = other.theme_var.clone();
        }
    }
}

/// Horizontal text alignment within a line.
///
/// Controls how text flows within its container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    /// Align to the start (locale-aware, typically left).
    #[default]
    Start,
    /// Align to the end (locale-aware, typically right).
    End,
    /// Align to the left edge.
    Left,
    /// Center the text.
    Center,
    /// Align to the right edge.
    Right,
    /// Justify text to fill the line width.
    Justify,
}

/// Horizontal alignment for content within a container.
///
/// Used for `content-align-horizontal` to position child elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignHorizontal {
    /// Align content to the left (default).
    #[default]
    Left,
    /// Center content horizontally.
    Center,
    /// Align content to the right.
    Right,
}

/// Vertical alignment for content within a container.
///
/// Used for `content-align-vertical` to position child elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignVertical {
    /// Align content to the top (default).
    #[default]
    Top,
    /// Center content vertically.
    Middle,
    /// Align content to the bottom.
    Bottom,
}
