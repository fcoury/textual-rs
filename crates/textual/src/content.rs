//! Content: Text parser that produces Strips.
//!
//! Content is the entry point of the rendering pipeline. It takes plain text
//! (or eventually markup) and converts it into a series of Strips that can
//! be rendered to the canvas.
//!
//! ## Pipeline Position
//! ```text
//! Content → Strip[] → Segment[] → Canvas
//!    ↑
//! You are here
//! ```

use crate::segment::{Segment, Style};
use crate::strip::Strip;
use unicode_width::UnicodeWidthStr;

/// Text content that can be converted to Strips for rendering.
///
/// Content handles text parsing, line splitting, and word wrapping.
/// It produces Strips that can be rendered to the canvas.
#[derive(Clone, Debug)]
pub struct Content {
    /// The raw text content.
    text: String,
    /// Style to apply to all text.
    style: Option<Style>,
}

impl Content {
    /// Creates new content from text.
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: None,
        }
    }

    /// Sets the style for this content.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Returns the raw text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the width of the longest line in cells.
    pub fn cell_length(&self) -> usize {
        self.text
            .lines()
            .map(|line| line.width())
            .max()
            .unwrap_or(0)
    }

    /// Returns the number of lines.
    pub fn height(&self) -> usize {
        self.text.lines().count().max(1)
    }

    /// Splits content into lines, returning one Strip per line.
    pub fn lines(&self) -> Vec<Strip> {
        if self.text.is_empty() {
            return vec![Strip::new()];
        }

        self.text
            .lines()
            .map(|line| {
                let segment = match &self.style {
                    Some(s) => Segment::styled(line, s.clone()),
                    None => Segment::new(line),
                };
                Strip::from_segment(segment)
            })
            .collect()
    }

    /// Word-wraps the content to fit within the given width.
    ///
    /// Returns a vector of Strips, one per wrapped line.
    pub fn wrap(&self, width: usize) -> Vec<Strip> {
        if width == 0 {
            return vec![Strip::new()];
        }

        let mut result = Vec::new();

        for line in self.text.lines() {
            if line.is_empty() {
                result.push(Strip::new());
                continue;
            }

            let line_width = line.width();
            if line_width <= width {
                // Line fits, no wrapping needed
                let segment = match &self.style {
                    Some(s) => Segment::styled(line, s.clone()),
                    None => Segment::new(line),
                };
                result.push(Strip::from_segment(segment));
            } else {
                // Need to wrap
                result.extend(self.wrap_line(line, width));
            }
        }

        if result.is_empty() {
            result.push(Strip::new());
        }

        result
    }

    /// Wraps a single line to the given width.
    fn wrap_line(&self, line: &str, width: usize) -> Vec<Strip> {
        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in line.split_whitespace() {
            let word_width = word.width();

            if current_width == 0 {
                // Start of line
                if word_width <= width {
                    current_line = word.to_string();
                    current_width = word_width;
                } else {
                    // Word is too long, need to break it
                    result.extend(self.break_word(word, width));
                }
            } else if current_width + 1 + word_width <= width {
                // Word fits with space
                current_line.push(' ');
                current_line.push_str(word);
                current_width += 1 + word_width;
            } else {
                // Word doesn't fit, emit current line and start new
                let segment = match &self.style {
                    Some(s) => Segment::styled(&current_line, s.clone()),
                    None => Segment::new(&current_line),
                };
                result.push(Strip::from_segment(segment));

                if word_width <= width {
                    current_line = word.to_string();
                    current_width = word_width;
                } else {
                    current_line.clear();
                    current_width = 0;
                    result.extend(self.break_word(word, width));
                }
            }
        }

        // Emit final line
        if !current_line.is_empty() {
            let segment = match &self.style {
                Some(s) => Segment::styled(&current_line, s.clone()),
                None => Segment::new(&current_line),
            };
            result.push(Strip::from_segment(segment));
        }

        result
    }

    /// Breaks a word that is longer than the available width.
    fn break_word(&self, word: &str, width: usize) -> Vec<Strip> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut current_width = 0;

        for ch in word.chars() {
            let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);

            if current_width + ch_width <= width {
                current.push(ch);
                current_width += ch_width;
            } else {
                if !current.is_empty() {
                    let segment = match &self.style {
                        Some(s) => Segment::styled(&current, s.clone()),
                        None => Segment::new(&current),
                    };
                    result.push(Strip::from_segment(segment));
                }
                current = ch.to_string();
                current_width = ch_width;
            }
        }

        if !current.is_empty() {
            let segment = match &self.style {
                Some(s) => Segment::styled(&current, s.clone()),
                None => Segment::new(&current),
            };
            result.push(Strip::from_segment(segment));
        }

        result
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tcss::types::RgbaColor;

    #[test]
    fn content_new() {
        let content = Content::new("hello");
        assert_eq!(content.text(), "hello");
    }

    #[test]
    fn content_cell_length() {
        let content = Content::new("hello\nworld!");
        assert_eq!(content.cell_length(), 6); // "world!" is longest
    }

    #[test]
    fn content_height() {
        let content = Content::new("line1\nline2\nline3");
        assert_eq!(content.height(), 3);
    }

    #[test]
    fn content_height_empty() {
        let content = Content::new("");
        assert_eq!(content.height(), 1);
    }

    #[test]
    fn content_lines() {
        let content = Content::new("hello\nworld");
        let lines = content.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text(), "hello");
        assert_eq!(lines[1].text(), "world");
    }

    #[test]
    fn content_lines_with_style() {
        let style = Style::with_fg(RgbaColor::rgb(255, 0, 0));
        let content = Content::new("hello").with_style(style);
        let lines = content.lines();
        assert!(lines[0].segments()[0].style().is_some());
    }

    #[test]
    fn content_wrap_fits() {
        let content = Content::new("hello world");
        let lines = content.wrap(20);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text(), "hello world");
    }

    #[test]
    fn content_wrap_splits() {
        let content = Content::new("hello world");
        let lines = content.wrap(6);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text(), "hello");
        assert_eq!(lines[1].text(), "world");
    }

    #[test]
    fn content_wrap_long_word() {
        let content = Content::new("supercalifragilistic");
        let lines = content.wrap(5);
        assert!(lines.len() >= 4); // Word broken into multiple parts
    }

    #[test]
    fn content_wrap_empty() {
        let content = Content::new("");
        let lines = content.wrap(10);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn content_wrap_zero_width() {
        let content = Content::new("hello");
        let lines = content.wrap(0);
        assert_eq!(lines.len(), 1);
    }
}
