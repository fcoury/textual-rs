//! Span type for styled regions in parsed markup.

use std::collections::HashMap;

use crate::style::Style;

/// A styled region within parsed markup text.
///
/// Spans define ranges of text that share the same styling. They reference
/// byte positions in the plain text (with markup stripped).
///
/// The `meta` field provides an extension point for framework-specific
/// attributes like action links (`@click`).
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    /// Start byte offset (inclusive) in the plain text.
    pub start: usize,
    /// End byte offset (exclusive) in the plain text.
    pub end: usize,
    /// Style to apply to this region.
    pub style: Style,
    /// Extension metadata (e.g., "@click" -> "app.quit").
    pub meta: HashMap<String, String>,
}

impl Span {
    /// Create a new span with just style (no meta).
    pub fn new(start: usize, end: usize, style: Style) -> Self {
        Self {
            start,
            end,
            style,
            meta: HashMap::new(),
        }
    }

    /// Create a new span with style and metadata.
    pub fn with_meta(
        start: usize,
        end: usize,
        style: Style,
        meta: HashMap<String, String>,
    ) -> Self {
        Self {
            start,
            end,
            style,
            meta,
        }
    }

    /// Returns true if this span has no styling or metadata.
    pub fn is_empty(&self) -> bool {
        self.style.is_empty() && self.meta.is_empty()
    }

    /// Returns true if this span covers a zero-length range.
    pub fn is_zero_length(&self) -> bool {
        self.start >= self.end
    }

    /// Returns the length of this span in bytes.
    pub fn len(&self) -> usize {
        if self.end > self.start {
            self.end - self.start
        } else {
            0
        }
    }

    /// Check if this span contains a given byte offset.
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Check if this span overlaps with another span.
    pub fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Get a metadata value by key.
    pub fn get_meta(&self, key: &str) -> Option<&str> {
        self.meta.get(key).map(|s| s.as_str())
    }

    /// Set a metadata value.
    pub fn set_meta(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.meta.insert(key.into(), value.into());
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            style: Style::default(),
            meta: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_new() {
        let span = Span::new(0, 5, Style::default());
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert!(span.meta.is_empty());
    }

    #[test]
    fn span_with_meta() {
        let mut meta = HashMap::new();
        meta.insert("@click".to_string(), "app.quit".to_string());

        let span = Span::with_meta(0, 5, Style::default(), meta);
        assert_eq!(span.get_meta("@click"), Some("app.quit"));
    }

    #[test]
    fn span_len() {
        let span = Span::new(5, 10, Style::default());
        assert_eq!(span.len(), 5);

        let empty = Span::new(5, 5, Style::default());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn span_contains() {
        let span = Span::new(5, 10, Style::default());
        assert!(!span.contains(4));
        assert!(span.contains(5));
        assert!(span.contains(7));
        assert!(span.contains(9));
        assert!(!span.contains(10));
    }

    #[test]
    fn span_overlaps() {
        let span1 = Span::new(0, 10, Style::default());
        let span2 = Span::new(5, 15, Style::default());
        let span3 = Span::new(10, 20, Style::default());

        assert!(span1.overlaps(&span2));
        assert!(span2.overlaps(&span1));
        assert!(!span1.overlaps(&span3));
        assert!(span2.overlaps(&span3));
    }
}
