//! Content: Text parser that produces Strips.
//!
//! Content is the entry point of the rendering pipeline. It takes plain text
//! (or Rich markup) and converts it into a series of Strips that can
//! be rendered to the canvas.
//!
//! ## Pipeline Position
//! ```text
//! Content → Strip[] → Segment[] → Canvas
//!    ↑
//! You are here
//! ```
//!
//! ## Markup Support
//!
//! Content supports Rich markup via `from_markup()`:
//!
//! ```
//! use textual::content::Content;
//!
//! let content = Content::from_markup("[bold red]Hello[/] World").unwrap();
//! let lines = content.lines();
//! // First segment is bold+red, rest is plain
//! ```

use std::collections::HashMap;

use crate::segment::{Segment, Style};
use crate::strip::Strip;
use tcss::types::RgbaColor;
use unicode_width::UnicodeWidthStr;

/// An internal span for styled content.
#[derive(Clone, Debug)]
struct InternalSpan {
    /// Start byte offset in text.
    start: usize,
    /// End byte offset in text.
    end: usize,
    /// Style to apply.
    style: Style,
    /// Metadata (e.g., @click actions).
    meta: HashMap<String, String>,
}

/// Text content that can be converted to Strips for rendering.
///
/// Content handles text parsing, line splitting, and word wrapping.
/// It produces Strips that can be rendered to the canvas.
///
/// Create with `Content::new()` for plain text or `Content::from_markup()`
/// for Rich-formatted text.
#[derive(Clone, Debug)]
pub struct Content {
    /// The raw text content (markup stripped if from_markup was used).
    text: String,
    /// Style to apply to all text (used when no spans).
    style: Option<Style>,
    /// Styled spans (from markup parsing).
    spans: Option<Vec<InternalSpan>>,
}

impl Content {
    /// Creates new content from plain text.
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: None,
            spans: None,
        }
    }

    /// Creates content from Rich markup text.
    ///
    /// Parses markup like `[bold red]Hello[/]` into styled spans.
    ///
    /// # Examples
    ///
    /// ```
    /// use textual::content::Content;
    ///
    /// let content = Content::from_markup("[bold]Hello[/] World").unwrap();
    /// assert_eq!(content.text(), "Hello World");
    /// ```
    pub fn from_markup(markup: &str) -> Result<Self, rich::RichParseError> {
        let parsed = rich::ParsedMarkup::parse(markup)?;

        // Convert rich spans to internal spans
        let spans: Vec<InternalSpan> = parsed
            .spans()
            .iter()
            .map(|s| InternalSpan {
                start: s.start,
                end: s.end,
                style: Self::convert_rich_style(&s.style),
                meta: s.meta.clone(),
            })
            .collect();

        Ok(Self {
            text: parsed.text().to_string(),
            style: None,
            spans: if spans.is_empty() { None } else { Some(spans) },
        })
    }

    /// Convert a rich::Style to a textual Style.
    fn convert_rich_style(rich_style: &rich::Style) -> Style {
        let mut style = Style::default();

        // Convert foreground color
        if let Some(color) = &rich_style.fg {
            let (r, g, b) = color.to_rgb();
            style.fg = Some(RgbaColor::rgb(r, g, b));
        }

        // Convert background color
        if let Some(color) = &rich_style.bg {
            let (r, g, b) = color.to_rgb();
            style.bg = Some(RgbaColor::rgb(r, g, b));
        }

        // Convert text modifiers
        style.bold = rich_style.text.bold;
        style.dim = rich_style.text.dim;
        style.italic = rich_style.text.italic;
        style.underline = rich_style.text.underline;
        style.strike = rich_style.text.strike;
        style.reverse = rich_style.text.reverse;

        style
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
    ///
    /// If the content was created from markup, spans are applied to create
    /// properly styled segments.
    pub fn lines(&self) -> Vec<Strip> {
        if self.text.is_empty() {
            return vec![Strip::new()];
        }

        // If we have spans, use the styled line rendering
        if let Some(spans) = &self.spans {
            return self.lines_with_spans(spans);
        }

        // Simple case: uniform style
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

    /// Render lines with styled spans.
    fn lines_with_spans(&self, spans: &[InternalSpan]) -> Vec<Strip> {
        let mut result = Vec::new();
        let mut line_start = 0;

        for line in self.text.lines() {
            let line_end = line_start + line.len();
            let strip = self.render_line_with_spans(line, line_start, line_end, spans);
            result.push(strip);
            line_start = line_end + 1; // +1 for newline
        }

        if result.is_empty() {
            result.push(Strip::new());
        }

        result
    }

    /// Render a single line with spans applied.
    fn render_line_with_spans(
        &self,
        line: &str,
        line_start: usize,
        line_end: usize,
        spans: &[InternalSpan],
    ) -> Strip {
        if line.is_empty() {
            return Strip::new();
        }

        // Find all relevant spans for this line
        let relevant_spans: Vec<&InternalSpan> = spans
            .iter()
            .filter(|s| s.start < line_end && s.end > line_start)
            .collect();

        if relevant_spans.is_empty() {
            // No spans affect this line
            let segment = match &self.style {
                Some(s) => Segment::styled(line, s.clone()),
                None => Segment::new(line),
            };
            return Strip::from_segment(segment);
        }

        // Build segments by walking through the line and applying spans
        let mut segments = Vec::new();
        let mut pos = 0;

        while pos < line.len() {
            // Find the style at this position
            let abs_pos = line_start + pos;
            let mut style = self.style.clone().unwrap_or_default();
            let mut meta = HashMap::new();

            for span in &relevant_spans {
                if abs_pos >= span.start && abs_pos < span.end {
                    style = style.apply(&span.style);
                    meta.extend(span.meta.clone());
                }
            }

            // Find where the style changes next
            let mut next_change = line.len();
            for span in &relevant_spans {
                if span.start > abs_pos && span.start - line_start < next_change {
                    next_change = span.start - line_start;
                }
                if span.end > abs_pos && span.end - line_start < next_change {
                    next_change = span.end - line_start;
                }
            }

            // Create segment for this range
            let end = next_change.min(line.len());
            let text = &line[pos..end];

            if !text.is_empty() {
                let mut segment = if style.is_empty() {
                    Segment::new(text)
                } else {
                    Segment::styled(text, style)
                };

                // Add meta if present
                if !meta.is_empty() {
                    segment = segment.with_meta(meta);
                }

                segments.push(segment);
            }

            pos = end;
        }

        Strip::from_segments(segments)
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

    #[test]
    fn content_from_markup_plain() {
        let content = Content::from_markup("Hello World").unwrap();
        assert_eq!(content.text(), "Hello World");
    }

    #[test]
    fn content_from_markup_styled() {
        let content = Content::from_markup("[bold]Hello[/] World").unwrap();
        assert_eq!(content.text(), "Hello World");

        let lines = content.lines();
        assert_eq!(lines.len(), 1);

        // First segment should be bold
        let segments = lines[0].segments();
        assert!(segments.len() >= 2);
        assert_eq!(segments[0].text(), "Hello");
        assert!(segments[0].style().is_some());
        assert!(segments[0].style().unwrap().bold);
    }

    #[test]
    fn content_from_markup_with_color() {
        let content = Content::from_markup("[red]Error[/]").unwrap();
        assert_eq!(content.text(), "Error");

        let lines = content.lines();
        let segments = lines[0].segments();
        assert!(segments[0].style().is_some());
        assert!(segments[0].fg().is_some());
    }

    #[test]
    fn content_from_markup_with_meta() {
        let content = Content::from_markup("Click [@click=app.quit]here[/] to exit").unwrap();
        assert_eq!(content.text(), "Click here to exit");

        let lines = content.lines();
        let segments = lines[0].segments();

        // Find the segment with "here" text
        let here_segment = segments.iter().find(|s| s.text() == "here").unwrap();
        assert_eq!(here_segment.get_meta("@click"), Some("app.quit"));
    }

    #[test]
    fn content_from_markup_multiline() {
        let content = Content::from_markup("[bold]Line 1[/]\nLine 2").unwrap();
        assert_eq!(content.text(), "Line 1\nLine 2");

        let lines = content.lines();
        assert_eq!(lines.len(), 2);

        // First line should have bold segment
        assert!(lines[0].segments()[0].style().unwrap().bold);

        // Second line should be plain
        assert_eq!(lines[1].text(), "Line 2");
    }

    #[test]
    fn content_from_markup_nested_styles() {
        let content = Content::from_markup("[bold][red]Important[/][/]").unwrap();
        assert_eq!(content.text(), "Important");

        let lines = content.lines();
        let segment = &lines[0].segments()[0];

        // Should have both bold and red
        let style = segment.style().unwrap();
        assert!(style.bold);
        assert!(segment.fg().is_some());
    }

    #[test]
    fn content_from_markup_escaped_brackets() {
        let content = Content::from_markup(r"\[not a tag\]").unwrap();
        assert_eq!(content.text(), "[not a tag]");
    }
}
