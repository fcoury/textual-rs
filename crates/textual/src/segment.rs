//! Segment: The fundamental rendering unit in Textual's pipeline.
//!
//! A Segment is a styled chunk of text - the atomic unit that gets rendered
//! to the terminal. Segments are immutable and can be combined into Strips.
//!
//! ## Pipeline Position
//! ```text
//! Content → Strip[] → Segment[] → Canvas
//!                         ↑
//!                    You are here
//! ```

use tcss::types::RgbaColor;
use unicode_width::UnicodeWidthStr;

/// Rendering style for a segment.
///
/// Contains all visual attributes that can be applied to text in a terminal.
/// Styles are layered during rendering - later styles override earlier ones.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Style {
    /// Foreground (text) color.
    pub fg: Option<RgbaColor>,
    /// Background color.
    pub bg: Option<RgbaColor>,
    /// Bold/increased intensity.
    pub bold: bool,
    /// Dim/decreased intensity.
    pub dim: bool,
    /// Italic text.
    pub italic: bool,
    /// Underlined text.
    pub underline: bool,
    /// Strikethrough text.
    pub strike: bool,
    /// Reverse video (swap fg/bg).
    pub reverse: bool,
}

impl Style {
    /// Creates a new style with the specified foreground color.
    pub fn with_fg(fg: RgbaColor) -> Self {
        Self {
            fg: Some(fg),
            ..Default::default()
        }
    }

    /// Creates a new style with the specified background color.
    pub fn with_bg(bg: RgbaColor) -> Self {
        Self {
            bg: Some(bg),
            ..Default::default()
        }
    }

    /// Creates a new style with both foreground and background colors.
    pub fn with_colors(fg: RgbaColor, bg: RgbaColor) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            ..Default::default()
        }
    }

    /// Returns true if no style properties are set.
    pub fn is_empty(&self) -> bool {
        self.fg.is_none()
            && self.bg.is_none()
            && !self.bold
            && !self.dim
            && !self.italic
            && !self.underline
            && !self.strike
            && !self.reverse
    }

    /// Applies another style on top of this one.
    ///
    /// Non-None values in `other` override values in `self`.
    /// Boolean attributes are OR'd together.
    pub fn apply(&self, other: &Style) -> Style {
        Style {
            fg: other.fg.clone().or_else(|| self.fg.clone()),
            bg: other.bg.clone().or_else(|| self.bg.clone()),
            bold: self.bold || other.bold,
            dim: self.dim || other.dim,
            italic: self.italic || other.italic,
            underline: self.underline || other.underline,
            strike: self.strike || other.strike,
            reverse: self.reverse || other.reverse,
        }
    }
}

/// A styled text chunk - Textual's fundamental rendering unit.
///
/// Segments are immutable building blocks that combine text with styling.
/// They are collected into Strips (horizontal lines) for rendering.
///
/// # Examples
///
/// ```
/// use textual::segment::{Segment, Style};
/// use tcss::types::RgbaColor;
///
/// // Create a plain text segment
/// let seg = Segment::new("Hello");
/// assert_eq!(seg.cell_length(), 5);
///
/// // Create a styled segment
/// let styled = Segment::new("World").with_style(Style::with_fg(RgbaColor::parse("red").unwrap()));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    /// The text content.
    text: String,
    /// Optional styling for this segment.
    style: Option<Style>,
}

impl Segment {
    /// Creates a new segment with the given text and no style.
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: None,
        }
    }

    /// Creates a new segment with the given text and style.
    pub fn styled<S: Into<String>>(text: S, style: Style) -> Self {
        Self {
            text: text.into(),
            style: Some(style),
        }
    }

    /// Creates a blank segment of spaces with the given width and style.
    pub fn blank(width: usize, style: Option<Style>) -> Self {
        Self {
            text: " ".repeat(width),
            style,
        }
    }

    /// Returns a builder-style method to add a style.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Returns the text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the style, if any.
    pub fn style(&self) -> Option<&Style> {
        self.style.as_ref()
    }

    /// Returns the foreground color, if any.
    pub fn fg(&self) -> Option<&RgbaColor> {
        self.style.as_ref().and_then(|s| s.fg.as_ref())
    }

    /// Returns the background color, if any.
    pub fn bg(&self) -> Option<&RgbaColor> {
        self.style.as_ref().and_then(|s| s.bg.as_ref())
    }

    /// Returns the terminal cell width of this segment.
    ///
    /// Uses Unicode width calculation to handle wide characters (CJK, emoji, etc.)
    /// correctly. Each terminal cell is one unit.
    pub fn cell_length(&self) -> usize {
        self.text.width()
    }

    /// Returns true if the segment is empty (no text).
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Splits the segment at a cell position.
    ///
    /// Returns two segments: one containing cells 0..cut, another containing cut..end.
    /// The split respects character boundaries, so splitting in the middle of a
    /// wide character will include the full character in the left segment.
    ///
    /// # Examples
    ///
    /// ```
    /// use textual::segment::Segment;
    ///
    /// let seg = Segment::new("Hello");
    /// let (left, right) = seg.split_at(2);
    /// assert_eq!(left.text(), "He");
    /// assert_eq!(right.text(), "llo");
    /// ```
    pub fn split_at(&self, cut: usize) -> (Segment, Segment) {
        if cut == 0 {
            return (
                Segment {
                    text: String::new(),
                    style: self.style.clone(),
                },
                self.clone(),
            );
        }

        if cut >= self.cell_length() {
            return (
                self.clone(),
                Segment {
                    text: String::new(),
                    style: self.style.clone(),
                },
            );
        }

        // Find the byte position corresponding to the cell position
        let mut cell_pos = 0;
        let mut byte_pos = 0;

        for (idx, ch) in self.text.char_indices() {
            if cell_pos >= cut {
                byte_pos = idx;
                break;
            }
            cell_pos += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            byte_pos = idx + ch.len_utf8();
        }

        let (left_text, right_text) = self.text.split_at(byte_pos);

        (
            Segment {
                text: left_text.to_string(),
                style: self.style.clone(),
            },
            Segment {
                text: right_text.to_string(),
                style: self.style.clone(),
            },
        )
    }

    /// Applies a style layer on top of this segment's style.
    ///
    /// Creates a new segment with the combined styles. The new style's
    /// non-None values override the current style.
    pub fn apply_style(&self, style: &Style) -> Segment {
        let new_style = match &self.style {
            Some(current) => current.apply(style),
            None => style.clone(),
        };

        Segment {
            text: self.text.clone(),
            style: Some(new_style),
        }
    }

    /// Sets the style, replacing any existing style.
    pub fn set_style(&self, style: Option<Style>) -> Segment {
        Segment {
            text: self.text.clone(),
            style,
        }
    }

    /// Applies a tint color to this segment's fg and bg colors.
    ///
    /// Uses Textual's linear interpolation formula:
    /// `result = base + (overlay - base) * alpha`
    ///
    /// Returns a new segment with tinted colors.
    pub fn apply_tint(&self, tint: &RgbaColor) -> Segment {
        // Skip if tint is fully transparent
        if tint.a == 0.0 {
            return self.clone();
        }

        let new_style = self.style.as_ref().map(|s| {
            Style {
                fg: s.fg.as_ref().map(|c| c.tint(tint)),
                bg: s.bg.as_ref().map(|c| c.tint(tint)),
                bold: s.bold,
                dim: s.dim,
                italic: s.italic,
                underline: s.underline,
                strike: s.strike,
                reverse: s.reverse,
            }
        });

        Segment {
            text: self.text.clone(),
            style: new_style,
        }
    }
}

impl Default for Segment {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_new_creates_unstyled() {
        let seg = Segment::new("hello");
        assert_eq!(seg.text(), "hello");
        assert!(seg.style().is_none());
    }

    #[test]
    fn segment_styled_creates_with_style() {
        let style = Style {
            bold: true,
            ..Default::default()
        };
        let seg = Segment::styled("hello", style);
        assert_eq!(seg.text(), "hello");
        assert!(seg.style().unwrap().bold);
    }

    #[test]
    fn segment_cell_length_ascii() {
        let seg = Segment::new("hello");
        assert_eq!(seg.cell_length(), 5);
    }

    #[test]
    fn segment_cell_length_wide_chars() {
        // CJK characters are 2 cells wide
        let seg = Segment::new("日本");
        assert_eq!(seg.cell_length(), 4); // 2 chars × 2 cells
    }

    #[test]
    fn segment_cell_length_mixed() {
        let seg = Segment::new("a日b");
        assert_eq!(seg.cell_length(), 4); // 1 + 2 + 1
    }

    #[test]
    fn segment_split_at_middle() {
        let seg = Segment::new("Hello");
        let (left, right) = seg.split_at(2);
        assert_eq!(left.text(), "He");
        assert_eq!(right.text(), "llo");
    }

    #[test]
    fn segment_split_at_zero() {
        let seg = Segment::new("Hello");
        let (left, right) = seg.split_at(0);
        assert_eq!(left.text(), "");
        assert_eq!(right.text(), "Hello");
    }

    #[test]
    fn segment_split_at_end() {
        let seg = Segment::new("Hello");
        let (left, right) = seg.split_at(5);
        assert_eq!(left.text(), "Hello");
        assert_eq!(right.text(), "");
    }

    #[test]
    fn segment_split_at_beyond_end() {
        let seg = Segment::new("Hello");
        let (left, right) = seg.split_at(10);
        assert_eq!(left.text(), "Hello");
        assert_eq!(right.text(), "");
    }

    #[test]
    fn segment_split_preserves_style() {
        let style = Style {
            bold: true,
            ..Default::default()
        };
        let seg = Segment::styled("Hello", style);
        let (left, right) = seg.split_at(2);
        assert!(left.style().unwrap().bold);
        assert!(right.style().unwrap().bold);
    }

    #[test]
    fn segment_apply_style_layers() {
        let base = Style {
            fg: Some(RgbaColor::rgb(255, 0, 0)),
            bold: true,
            ..Default::default()
        };
        let seg = Segment::styled("hello", base);

        let overlay = Style {
            bg: Some(RgbaColor::rgb(0, 0, 255)),
            italic: true,
            ..Default::default()
        };

        let result = seg.apply_style(&overlay);
        let style = result.style().unwrap();

        // Original fg preserved
        assert!(style.fg.is_some());
        // New bg applied
        assert!(style.bg.is_some());
        // Both bold and italic active
        assert!(style.bold);
        assert!(style.italic);
    }

    #[test]
    fn segment_apply_style_overrides() {
        let base = Style {
            fg: Some(RgbaColor::rgb(255, 0, 0)), // Red
            ..Default::default()
        };
        let seg = Segment::styled("hello", base);

        let overlay = Style {
            fg: Some(RgbaColor::rgb(0, 255, 0)), // Green overrides
            ..Default::default()
        };

        let result = seg.apply_style(&overlay);
        let style = result.style().unwrap();

        // Green should override red
        assert_eq!(style.fg.as_ref().unwrap().g, 255);
    }

    #[test]
    fn segment_blank_creates_spaces() {
        let seg = Segment::blank(5, None);
        assert_eq!(seg.text(), "     ");
        assert_eq!(seg.cell_length(), 5);
    }

    #[test]
    fn segment_blank_with_style() {
        let style = Style::with_bg(RgbaColor::rgb(0, 0, 255));
        let seg = Segment::blank(3, Some(style));
        assert_eq!(seg.text(), "   ");
        assert!(seg.style().unwrap().bg.is_some());
    }

    #[test]
    fn style_is_empty() {
        let empty = Style::default();
        assert!(empty.is_empty());

        let with_fg = Style::with_fg(RgbaColor::rgb(255, 0, 0));
        assert!(!with_fg.is_empty());

        let with_bold = Style {
            bold: true,
            ..Default::default()
        };
        assert!(!with_bold.is_empty());
    }
}
