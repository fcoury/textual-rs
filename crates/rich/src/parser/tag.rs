//! Tag content parser for Rich markup.
//!
//! Parses the content inside `[...]` tags.

use std::collections::HashMap;

use crate::error::StyleParseError;
use crate::style::Style;

/// The result of parsing tag content.
#[derive(Clone, Debug, PartialEq)]
pub enum TagContent {
    /// A style specification: `[bold red on blue]`
    Style(Style),
    /// A meta entry: `[@click=app.quit]`
    Meta(String, String),
    /// Close all styles: `[/]`
    CloseAll,
    /// Close specific style: `[/bold]`
    CloseStyle(String),
}

impl TagContent {
    /// Parse tag content (the text inside `[...]`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rich::parser::TagContent;
    ///
    /// let tag = TagContent::parse("bold red").unwrap();
    /// assert!(matches!(tag, TagContent::Style(_)));
    ///
    /// let meta = TagContent::parse("@click=app.quit").unwrap();
    /// assert!(matches!(meta, TagContent::Meta(_, _)));
    ///
    /// let close = TagContent::parse("/").unwrap();
    /// assert!(matches!(close, TagContent::CloseAll));
    /// ```
    pub fn parse(content: &str) -> Result<Self, StyleParseError> {
        let content = content.trim();

        if content.is_empty() {
            return Err(StyleParseError::Empty);
        }

        // Close tag: [/] or [/bold]
        if content.starts_with('/') {
            let rest = content[1..].trim();
            if rest.is_empty() {
                return Ok(TagContent::CloseAll);
            } else {
                return Ok(TagContent::CloseStyle(rest.to_string()));
            }
        }

        // Meta tag: [@key=value]
        if content.starts_with('@') {
            return Self::parse_meta(&content[1..]);
        }

        // Style tag: [bold red on blue]
        let style = Style::parse(content)?;
        Ok(TagContent::Style(style))
    }

    /// Parse a meta tag (the part after `@`).
    fn parse_meta(content: &str) -> Result<Self, StyleParseError> {
        // Find the = separator
        if let Some(eq_pos) = content.find('=') {
            let key = format!("@{}", content[..eq_pos].trim());
            let value = content[eq_pos + 1..].trim().to_string();
            Ok(TagContent::Meta(key, value))
        } else {
            // No value, just the key
            let key = format!("@{}", content.trim());
            Ok(TagContent::Meta(key, String::new()))
        }
    }

    /// Check if this is a close tag.
    pub fn is_close(&self) -> bool {
        matches!(self, TagContent::CloseAll | TagContent::CloseStyle(_))
    }

    /// Check if this is a meta tag.
    pub fn is_meta(&self) -> bool {
        matches!(self, TagContent::Meta(_, _))
    }

    /// Get the style if this is a style tag.
    pub fn as_style(&self) -> Option<&Style> {
        match self {
            TagContent::Style(s) => Some(s),
            _ => None,
        }
    }

    /// Get the meta key-value if this is a meta tag.
    pub fn as_meta(&self) -> Option<(&str, &str)> {
        match self {
            TagContent::Meta(k, v) => Some((k, v)),
            _ => None,
        }
    }

    /// Convert to a HashMap entry if this is a meta tag.
    pub fn into_meta_entry(self) -> Option<(String, String)> {
        match self {
            TagContent::Meta(k, v) => Some((k, v)),
            _ => None,
        }
    }
}

/// Accumulated state from parsing multiple tags.
///
/// Combines style and meta from multiple tag contents.
#[derive(Clone, Debug, Default)]
pub struct ParsedTag {
    /// The accumulated style.
    pub style: Style,
    /// Meta entries.
    pub meta: HashMap<String, String>,
}

impl ParsedTag {
    /// Create a new empty parsed tag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a TagContent to this parsed tag.
    pub fn apply(&mut self, content: TagContent) {
        match content {
            TagContent::Style(s) => {
                self.style = self.style.apply(&s);
            }
            TagContent::Meta(k, v) => {
                self.meta.insert(k, v);
            }
            TagContent::CloseAll | TagContent::CloseStyle(_) => {
                // Close tags don't add to the style
            }
        }
    }

    /// Check if this parsed tag is empty (no style or meta).
    pub fn is_empty(&self) -> bool {
        self.style.is_empty() && self.meta.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_style_tag() {
        let tag = TagContent::parse("bold").unwrap();
        match tag {
            TagContent::Style(s) => assert!(s.text.bold),
            _ => panic!("Expected Style"),
        }
    }

    #[test]
    fn parse_combined_style() {
        let tag = TagContent::parse("bold red on blue").unwrap();
        match tag {
            TagContent::Style(s) => {
                assert!(s.text.bold);
                assert!(s.fg.is_some());
                assert!(s.bg.is_some());
            }
            _ => panic!("Expected Style"),
        }
    }

    #[test]
    fn parse_close_all() {
        let tag = TagContent::parse("/").unwrap();
        assert_eq!(tag, TagContent::CloseAll);
    }

    #[test]
    fn parse_close_style() {
        let tag = TagContent::parse("/bold").unwrap();
        assert_eq!(tag, TagContent::CloseStyle("bold".to_string()));
    }

    #[test]
    fn parse_meta_with_value() {
        let tag = TagContent::parse("@click=app.quit").unwrap();
        assert_eq!(
            tag,
            TagContent::Meta("@click".to_string(), "app.quit".to_string())
        );
    }

    #[test]
    fn parse_meta_with_complex_value() {
        let tag = TagContent::parse("@click=set_background('cyan')").unwrap();
        assert_eq!(
            tag,
            TagContent::Meta("@click".to_string(), "set_background('cyan')".to_string())
        );
    }

    #[test]
    fn parse_meta_without_value() {
        let tag = TagContent::parse("@hover").unwrap();
        assert_eq!(
            tag,
            TagContent::Meta("@hover".to_string(), String::new())
        );
    }

    #[test]
    fn parse_link_not_supported_as_style() {
        // link=... is not a valid style modifier - links should use meta syntax
        // like [@link=https://example.com] if needed
        let result = TagContent::parse("link=https://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn parsed_tag_apply() {
        let mut parsed = ParsedTag::new();

        parsed.apply(TagContent::parse("bold").unwrap());
        assert!(parsed.style.text.bold);

        parsed.apply(TagContent::parse("red").unwrap());
        assert!(parsed.style.text.bold);
        assert!(parsed.style.fg.is_some());

        parsed.apply(TagContent::parse("@click=test").unwrap());
        assert_eq!(parsed.meta.get("@click"), Some(&"test".to_string()));
    }

    #[test]
    fn tag_content_helpers() {
        let style = TagContent::parse("bold").unwrap();
        assert!(!style.is_close());
        assert!(!style.is_meta());

        let close = TagContent::parse("/").unwrap();
        assert!(close.is_close());

        let meta = TagContent::parse("@click=test").unwrap();
        assert!(meta.is_meta());
        assert_eq!(meta.as_meta(), Some(("@click", "test")));
    }
}
