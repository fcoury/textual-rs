//! ParsedMarkup result type.
//!
//! This is the result of parsing Rich markup text.

use crate::span::Span;
use crate::style::Style;

/// The result of parsing Rich markup.
///
/// Contains the plain text (with markup stripped) and a list of spans
/// that define styled regions.
///
/// # Examples
///
/// ```
/// use rich::ParsedMarkup;
///
/// let parsed = ParsedMarkup::parse("[bold]Hello[/] World").unwrap();
/// assert_eq!(parsed.text(), "Hello World");
/// assert_eq!(parsed.spans().len(), 1);
/// ```
#[derive(Clone, Debug, Default)]
pub struct ParsedMarkup {
    /// Plain text with all markup tags stripped.
    text: String,
    /// Style spans referencing positions in `text`.
    spans: Vec<Span>,
}

impl ParsedMarkup {
    /// Create a new ParsedMarkup with the given text and spans.
    pub fn new(text: String, spans: Vec<Span>) -> Self {
        Self { text, spans }
    }

    /// Create a ParsedMarkup from plain text (no spans).
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            spans: Vec::new(),
        }
    }

    /// Parse Rich markup text.
    ///
    /// This parses markup like `[bold red]Hello[/]` into plain text with styled spans.
    ///
    /// # Examples
    ///
    /// ```
    /// use rich::ParsedMarkup;
    ///
    /// let parsed = ParsedMarkup::parse("[bold]Hello[/] World").unwrap();
    /// assert_eq!(parsed.text(), "Hello World");
    /// assert_eq!(parsed.spans().len(), 1);
    /// ```
    pub fn parse(input: &str) -> Result<Self, crate::error::RichParseError> {
        crate::parser::parse(input)
    }

    /// Get the plain text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get all spans.
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    /// Get mutable access to spans.
    pub fn spans_mut(&mut self) -> &mut Vec<Span> {
        &mut self.spans
    }

    /// Add a span to this markup.
    pub fn add_span(&mut self, span: Span) {
        self.spans.push(span);
    }

    /// Returns true if there are no spans (plain text only).
    pub fn is_plain(&self) -> bool {
        self.spans.is_empty()
    }

    /// Get the length of the text in bytes.
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Returns true if the text is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Iterate over styled segments.
    ///
    /// Returns an iterator that yields `(text_slice, style, meta)` tuples
    /// for each distinct styled region. Unstyled regions are included with
    /// an empty style.
    pub fn segments(&self) -> SegmentIterator<'_> {
        SegmentIterator::new(self)
    }

    /// Get the style at a specific byte offset.
    ///
    /// Returns the merged style of all spans that contain the offset.
    pub fn style_at(&self, offset: usize) -> Style {
        let mut result = Style::default();
        for span in &self.spans {
            if span.contains(offset) {
                result = result.apply(&span.style);
            }
        }
        result
    }

    /// Get all spans that contain a specific byte offset.
    pub fn spans_at(&self, offset: usize) -> Vec<&Span> {
        self.spans.iter().filter(|s| s.contains(offset)).collect()
    }
}

/// Iterator over styled segments in ParsedMarkup.
pub struct SegmentIterator<'a> {
    markup: &'a ParsedMarkup,
    pos: usize,
}

impl<'a> SegmentIterator<'a> {
    fn new(markup: &'a ParsedMarkup) -> Self {
        Self { markup, pos: 0 }
    }
}

impl<'a> Iterator for SegmentIterator<'a> {
    type Item = (&'a str, Style, std::collections::HashMap<String, String>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.markup.text.len() {
            return None;
        }

        // Find the next boundary (where style changes)
        let mut end = self.markup.text.len();

        // Check all span boundaries
        for span in &self.markup.spans {
            if span.start > self.pos && span.start < end {
                end = span.start;
            }
            if span.end > self.pos && span.end < end {
                end = span.end;
            }
        }

        // Get the style and meta for this segment
        let mut style = Style::default();
        let mut meta = std::collections::HashMap::new();

        for span in &self.markup.spans {
            if span.contains(self.pos) {
                style = style.apply(&span.style);
                for (k, v) in &span.meta {
                    meta.insert(k.clone(), v.clone());
                }
            }
        }

        let text = &self.markup.text[self.pos..end];
        self.pos = end;

        Some((text, style, meta))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn parsed_markup_plain() {
        let markup = ParsedMarkup::plain("Hello World");
        assert_eq!(markup.text(), "Hello World");
        assert!(markup.is_plain());
    }

    #[test]
    fn parsed_markup_with_spans() {
        let mut style = Style::default();
        style.text.bold = true;

        let span = Span::new(0, 5, style);
        let markup = ParsedMarkup::new("Hello World".to_string(), vec![span]);

        assert_eq!(markup.spans().len(), 1);
        assert!(!markup.is_plain());
    }

    #[test]
    fn style_at() {
        let mut style = Style::default();
        style.fg = Some(Color::Named("red".into()));

        let span = Span::new(0, 5, style);
        let markup = ParsedMarkup::new("Hello World".to_string(), vec![span]);

        let style_0 = markup.style_at(0);
        assert!(style_0.fg.is_some());

        let style_6 = markup.style_at(6);
        assert!(style_6.fg.is_none());
    }

    #[test]
    fn segments_iterator() {
        let mut style = Style::default();
        style.text.bold = true;

        let span = Span::new(0, 5, style);
        let markup = ParsedMarkup::new("Hello World".to_string(), vec![span]);

        let segments: Vec<_> = markup.segments().collect();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].0, "Hello");
        assert!(segments[0].1.text.bold);
        assert_eq!(segments[1].0, " World");
        assert!(!segments[1].1.text.bold);
    }
}
