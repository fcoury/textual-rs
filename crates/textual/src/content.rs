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
use tcss::types::link::LinkStyle;
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
    /// Link styling from CSS (applied to segments with @link or @click metadata).
    link_style: Option<LinkStyle>,
    /// Currently hovered link action (for applying hover styles).
    hovered_action: Option<String>,
}

impl Content {
    /// Creates new content from plain text.
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: None,
            spans: None,
            link_style: None,
            hovered_action: None,
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
            link_style: None,
            hovered_action: None,
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

    /// Sets the link styling for this content.
    ///
    /// Link styling is applied to segments that have `@link` or `@click` metadata
    /// from Rich markup (e.g., `[link='url']text[/]` or `[@click=action]text[/]`).
    pub fn with_link_style(mut self, link_style: LinkStyle) -> Self {
        self.link_style = Some(link_style);
        self
    }

    /// Sets the currently hovered link action for hover styling.
    ///
    /// When a segment's `@click` action matches this value, hover styles
    /// (link-color-hover, link-background-hover, link-style-hover) are applied.
    pub fn with_hovered_action(mut self, action: Option<String>) -> Self {
        self.hovered_action = action;
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

            // Apply link styling only to action links (@click), not URL links (@link)
            // URL links get OSC 8 hyperlink treatment instead of CSS link-* styling
            let click_action = meta.get("@click").cloned();
            if let Some(ref action) = click_action {
                if let Some(link_style) = &self.link_style {
                    // Check if this link is hovered
                    let is_hovered = self.hovered_action.as_ref() == Some(action);
                    style = self.apply_link_style(style, link_style, is_hovered);
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

    /// Apply link styling to a segment style.
    ///
    /// This converts LinkStyle (from CSS) to rendering Style and applies it
    /// on top of the existing segment style. Links default to underlined text
    /// and auto-contrast foreground color when a background is set.
    ///
    /// When `is_hovered` is true, hover variants (link-color-hover, link-background-hover,
    /// link-style-hover) are used instead of the normal variants.
    fn apply_link_style(&self, base_style: Style, link_style: &LinkStyle, is_hovered: bool) -> Style {
        // Select colors based on hover state (hover takes precedence if set)
        let link_bg = if is_hovered {
            link_style.background_hover.as_ref().or(link_style.background.as_ref())
        } else {
            link_style.background.as_ref()
        };
        let link_color = if is_hovered {
            link_style.color_hover.as_ref().or(link_style.color.as_ref())
        } else {
            link_style.color.as_ref()
        };
        // For text styles on hover: merge link-style with link-style-hover
        // (hover adds attributes to the base link-style, not replaces it)
        // This matches Python Textual's behavior where link-style is preserved on hover.
        let merged_hover_style;
        let link_text_style = if is_hovered {
            merged_hover_style = tcss::types::TextStyle {
                bold: link_style.style.bold || link_style.style_hover.bold,
                dim: link_style.style.dim || link_style.style_hover.dim,
                italic: link_style.style.italic || link_style.style_hover.italic,
                underline: link_style.style_hover.underline, // Hover controls underline
                underline2: link_style.style.underline2 || link_style.style_hover.underline2,
                blink: link_style.style.blink || link_style.style_hover.blink,
                blink2: link_style.style.blink2 || link_style.style_hover.blink2,
                reverse: link_style.style.reverse || link_style.style_hover.reverse,
                strike: link_style.style.strike || link_style.style_hover.strike,
                overline: link_style.style.overline || link_style.style_hover.overline,
                theme_var: None,
            };
            &merged_hover_style
        } else {
            &link_style.style
        };

        // Determine the effective background, compositing semi-transparent link backgrounds
        // over the base background (like CSS alpha compositing)
        let effective_bg = match (link_bg, &base_style.bg) {
            (Some(link_bg), Some(base_bg)) if link_bg.a < 1.0 => {
                // Composite semi-transparent link background over base
                Some(link_bg.blend_over(base_bg))
            }
            (Some(link_bg), _) => Some(link_bg.clone()),
            (None, base) => base.clone(),
        };

        // Determine foreground color:
        // 1. If link-color is "auto", compute contrast against link background (or base bg)
        // 2. If link-color is explicitly set to a real color, use it
        // 3. Otherwise use base foreground
        let fg = if let Some(color) = link_color {
            if color.auto {
                // Auto color - compute contrast against the effective background
                if let Some(bg) = &effective_bg {
                    Some(bg.get_contrasting_color(color.a))
                } else if let Some(base_bg) = &base_style.bg {
                    Some(base_bg.get_contrasting_color(color.a))
                } else {
                    // No background to contrast against, use default text color
                    base_style.fg
                }
            } else if color.a < 1.0 {
                // Semi-transparent explicit color - blend over background
                // This matches Python Textual's behavior: hover_background + link_color_hover
                if let Some(bg) = &effective_bg {
                    Some(color.blend_over(bg))
                } else if let Some(base_bg) = &base_style.bg {
                    Some(color.blend_over(base_bg))
                } else {
                    // No background to blend over, use color as-is (may look wrong)
                    Some(color.clone())
                }
            } else {
                // Opaque explicit color - use directly
                Some(color.clone())
            }
        } else if let Some(bg) = &effective_bg {
            // No link-color but has background - auto-contrast
            Some(bg.get_contrasting_color(0.87))
        } else {
            base_style.fg
        };

        // For hover: use the hover style (bold, no underline)
        // For normal: default to underline unless link_text_style specifies otherwise
        let underline = if is_hovered {
            // Hover style controls underline (default hover is bold, no underline)
            link_text_style.underline
        } else {
            // Normal links default to underline
            base_style.underline || link_text_style.underline || true
        };

        Style {
            fg,
            bg: effective_bg,
            // Merge text style attributes (link-style adds to base style)
            bold: base_style.bold || link_text_style.bold,
            dim: base_style.dim || link_text_style.dim,
            italic: base_style.italic || link_text_style.italic,
            underline,
            strike: base_style.strike || link_text_style.strike,
            reverse: base_style.reverse || link_text_style.reverse,
        }
    }

    /// Word-wraps the content to fit within the given width.
    ///
    /// Returns a vector of Strips, one per wrapped line.
    pub fn wrap(&self, width: usize) -> Vec<Strip> {
        if width == 0 {
            return vec![Strip::new()];
        }

        // If we have spans (from markup parsing), use span-aware wrapping
        if let Some(spans) = &self.spans {
            return self.wrap_with_spans(width, spans);
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

    /// Wraps content while preserving styled spans from markup.
    fn wrap_with_spans(&self, width: usize, spans: &[InternalSpan]) -> Vec<Strip> {
        let mut result = Vec::new();
        let mut line_start = 0;

        for line in self.text.lines() {
            let line_end = line_start + line.len();

            if line.is_empty() {
                result.push(Strip::new());
                line_start = line_end + 1;
                continue;
            }

            let line_width = line.width();
            if line_width <= width {
                // Line fits, render with spans
                let strip = self.render_line_with_spans(line, line_start, line_end, spans);
                result.push(strip);
            } else {
                // Need to wrap this line while preserving spans
                result.extend(self.wrap_line_with_spans(line, line_start, width, spans));
            }
            line_start = line_end + 1;
        }

        if result.is_empty() {
            result.push(Strip::new());
        }

        result
    }

    /// Wraps a single line while preserving styled spans.
    fn wrap_line_with_spans(
        &self,
        line: &str,
        line_start: usize,
        width: usize,
        spans: &[InternalSpan],
    ) -> Vec<Strip> {
        let mut result = Vec::new();
        let mut current_width = 0;

        // Process word by word
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut word_positions: Vec<(usize, usize)> = Vec::new(); // (start, end) byte positions

        // Find byte positions of each word in the line
        let mut search_start = 0;
        for word in &words {
            if let Some(pos) = line[search_start..].find(word) {
                let abs_pos = search_start + pos;
                word_positions.push((abs_pos, abs_pos + word.len()));
                search_start = abs_pos + word.len();
            }
        }

        let mut current_segment_start = 0;
        let mut i = 0;

        while i < words.len() {
            let word = words[i];
            let word_width = word.width();

            if current_width == 0 {
                // Start of line
                if word_width <= width {
                    current_width = word_width;
                    i += 1;
                } else {
                    // Word too long, need to break it - use the simple approach for now
                    let (word_byte_start, word_byte_end) = word_positions[i];
                    let segment_text = &line[word_byte_start..word_byte_end];
                    let abs_start = line_start + word_byte_start;
                    let abs_end = line_start + word_byte_end;
                    let strip = self.render_line_with_spans(segment_text, abs_start, abs_end, spans);
                    result.push(strip);
                    current_segment_start = if i + 1 < word_positions.len() {
                        word_positions[i + 1].0
                    } else {
                        line.len()
                    };
                    current_width = 0;
                    i += 1;
                }
            } else {
                let space_and_word = 1 + word_width;
                if current_width + space_and_word <= width {
                    // Word fits
                    current_width += space_and_word;
                    i += 1;
                } else {
                    // Word doesn't fit, emit current segment
                    let segment_end = if i > 0 { word_positions[i - 1].1 } else { 0 };
                    let segment_text = &line[current_segment_start..segment_end];
                    let abs_start = line_start + current_segment_start;
                    let abs_end = line_start + segment_end;
                    let strip = self.render_line_with_spans(segment_text, abs_start, abs_end, spans);
                    result.push(strip);

                    // Start new segment at this word
                    current_segment_start = word_positions[i].0;
                    current_width = 0;
                }
            }
        }

        // Emit final segment
        if current_width > 0 && !words.is_empty() {
            let segment_end = word_positions.last().map(|(_, e)| *e).unwrap_or(line.len());
            let segment_text = &line[current_segment_start..segment_end];
            if !segment_text.is_empty() {
                let abs_start = line_start + current_segment_start;
                let abs_end = line_start + segment_end;
                let strip = self.render_line_with_spans(segment_text, abs_start, abs_end, spans);
                result.push(strip);
            }
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

    #[test]
    fn content_wrap_preserves_markup_styles() {
        // This is the critical test - wrap() should preserve styles from markup
        let content = Content::from_markup("Hello [b]Bold[/] text").unwrap();
        let lines = content.wrap(80); // Wide enough to fit on one line

        assert_eq!(lines.len(), 1);
        let segments = lines[0].segments();

        // Should have at least 3 segments: "Hello ", "Bold", " text"
        assert!(segments.len() >= 2, "Expected at least 2 segments, got {}", segments.len());

        // Find the segment with "Bold" text
        let bold_segment = segments.iter().find(|s| s.text().contains("Bold"));
        assert!(bold_segment.is_some(), "Should have a segment containing 'Bold'");

        let bold_segment = bold_segment.unwrap();
        assert!(
            bold_segment.style().is_some(),
            "Bold segment should have a style"
        );
        assert!(
            bold_segment.style().unwrap().bold,
            "Bold segment should have bold=true"
        );
    }

    #[test]
    fn content_wrap_preserves_markup_styles_when_wrapping() {
        // Test that wrapping preserves styles across line breaks
        let content = Content::from_markup("[b]Long bold text that needs wrapping[/]").unwrap();
        let lines = content.wrap(15); // Force wrapping

        assert!(lines.len() >= 2, "Should wrap to multiple lines");

        // All lines should have bold segments
        for (i, line) in lines.iter().enumerate() {
            let has_bold = line.segments().iter().any(|s| {
                s.style().map(|st| st.bold).unwrap_or(false)
            });
            assert!(
                has_bold || line.text().is_empty(),
                "Line {} should have bold style: '{}'",
                i,
                line.text()
            );
        }
    }
}
