//! Main markup parser for Rich text.
//!
//! Combines the lexer and tag parser to produce ParsedMarkup.

use std::collections::HashMap;

use crate::error::RichParseError;
use crate::markup::ParsedMarkup;
use crate::span::Span;
use crate::style::Style;

use super::lexer::{Lexer, Token};
use super::tag::TagContent;

/// Parses Rich markup into a ParsedMarkup result.
///
/// This is the main entry point for parsing markup text.
///
/// # Examples
///
/// ```
/// use rich::parser::parse;
///
/// let parsed = parse("[bold]Hello[/] World").unwrap();
/// assert_eq!(parsed.text(), "Hello World");
/// assert_eq!(parsed.spans().len(), 1);
/// ```
pub fn parse(input: &str) -> Result<ParsedMarkup, RichParseError> {
    // Collect tokens first
    let tokens: Result<Vec<_>, _> = Lexer::new(input).collect();
    let tokens = tokens?;

    // Then process them
    let mut parser = Parser::new();
    for token in tokens {
        parser.process_token(token)?;
    }

    // Close any remaining open tags
    parser.close_all_remaining();

    Ok(ParsedMarkup::new(parser.output, parser.spans))
}

/// The main parser state.
struct Parser {
    /// Stack of active styles (for nesting).
    style_stack: Vec<StackEntry>,
    /// Output plain text (markup stripped).
    output: String,
    /// Generated spans.
    spans: Vec<Span>,
}

/// An entry on the style stack.
#[derive(Clone, Debug)]
struct StackEntry {
    /// The style applied by this tag.
    style: Style,
    /// Meta entries from this tag.
    meta: HashMap<String, String>,
    /// Position in output where this style started.
    start_pos: usize,
    /// Original tag content (for matching close tags).
    tag_content: String,
}

impl Parser {
    /// Create a new parser.
    fn new() -> Self {
        Self {
            style_stack: Vec::new(),
            output: String::new(),
            spans: Vec::new(),
        }
    }

    /// Process a single token.
    fn process_token(&mut self, token: Token<'_>) -> Result<(), RichParseError> {
        match token {
            Token::Text(text) => {
                self.output.push_str(text);
            }
            Token::OpenTag(content) => {
                self.process_open_tag(content)?;
            }
            Token::CloseTag(spec) => {
                self.process_close_tag(spec);
            }
            Token::EscapedBracket(c) => {
                self.output.push(c);
            }
        }
        Ok(())
    }

    /// Process an opening tag.
    fn process_open_tag(&mut self, content: &str) -> Result<(), RichParseError> {
        let tag_content = TagContent::parse(content)
            .map_err(|e| RichParseError::InvalidModifier(e.to_string()))?;

        match tag_content {
            TagContent::Style(style) => {
                self.push_style(style, HashMap::new(), content.to_string());
            }
            TagContent::Meta(key, value) => {
                let mut meta = HashMap::new();
                meta.insert(key, value);
                self.push_style(Style::default(), meta, content.to_string());
            }
            TagContent::CloseAll => {
                self.pop_style();
            }
            TagContent::CloseStyle(name) => {
                self.pop_style_matching(&name);
            }
        }

        Ok(())
    }

    /// Process a closing tag.
    fn process_close_tag(&mut self, spec: Option<&str>) {
        match spec {
            None => {
                // [/] - close most recent
                self.pop_style();
            }
            Some(name) => {
                // [/bold] - close matching
                self.pop_style_matching(name);
            }
        }
    }

    /// Push a new style onto the stack.
    fn push_style(&mut self, style: Style, meta: HashMap<String, String>, tag_content: String) {
        let entry = StackEntry {
            style,
            meta,
            start_pos: self.output.len(),
            tag_content,
        };
        self.style_stack.push(entry);
    }

    /// Pop the most recent style and create a span.
    fn pop_style(&mut self) {
        if let Some(entry) = self.style_stack.pop() {
            self.create_span_from_entry(entry);
        }
    }

    /// Pop styles until we find one matching the given name.
    fn pop_style_matching(&mut self, name: &str) {
        // Find the matching entry
        let mut to_close = Vec::new();

        while let Some(entry) = self.style_stack.pop() {
            let matches = self.entry_matches(&entry, name);
            to_close.push(entry);
            if matches {
                break;
            }
        }

        // Create spans for all closed entries
        for entry in to_close {
            self.create_span_from_entry(entry);
        }
    }

    /// Check if a stack entry matches a close tag name.
    fn entry_matches(&self, entry: &StackEntry, name: &str) -> bool {
        // Simple matching: check if the tag content starts with the name
        // or contains it as a word
        let content_lower = entry.tag_content.to_lowercase();
        let name_lower = name.to_lowercase();

        content_lower == name_lower
            || content_lower.starts_with(&format!("{} ", name_lower))
            || content_lower.contains(&format!(" {}", name_lower))
    }

    /// Create a span from a stack entry.
    fn create_span_from_entry(&mut self, entry: StackEntry) {
        let end_pos = self.output.len();

        // Only create span if there's actual content and styling
        if end_pos > entry.start_pos && (!entry.style.is_empty() || !entry.meta.is_empty()) {
            let span = Span::with_meta(entry.start_pos, end_pos, entry.style, entry.meta);
            self.spans.push(span);
        }
    }

    /// Close all remaining open tags.
    fn close_all_remaining(&mut self) {
        while !self.style_stack.is_empty() {
            self.pop_style();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_text() {
        let parsed = parse("Hello World").unwrap();
        assert_eq!(parsed.text(), "Hello World");
        assert!(parsed.spans().is_empty());
    }

    #[test]
    fn parse_single_style() {
        let parsed = parse("[bold]Hello[/]").unwrap();
        assert_eq!(parsed.text(), "Hello");
        assert_eq!(parsed.spans().len(), 1);
        assert!(parsed.spans()[0].style.text.bold);
    }

    #[test]
    fn parse_style_with_text() {
        let parsed = parse("[bold]Hello[/] World").unwrap();
        assert_eq!(parsed.text(), "Hello World");
        assert_eq!(parsed.spans().len(), 1);
        assert_eq!(parsed.spans()[0].start, 0);
        assert_eq!(parsed.spans()[0].end, 5);
    }

    #[test]
    fn parse_nested_styles() {
        let parsed = parse("[bold][red]text[/][/]").unwrap();
        assert_eq!(parsed.text(), "text");
        assert_eq!(parsed.spans().len(), 2);
    }

    #[test]
    fn parse_color() {
        let parsed = parse("[red]text[/]").unwrap();
        assert_eq!(parsed.text(), "text");
        assert!(parsed.spans()[0].style.fg.is_some());
    }

    #[test]
    fn parse_hex_color() {
        let parsed = parse("[#ff5733]text[/]").unwrap();
        assert_eq!(parsed.text(), "text");
        assert!(parsed.spans()[0].style.fg.is_some());
    }

    #[test]
    fn parse_combined_style() {
        let parsed = parse("[bold red on blue]text[/]").unwrap();
        assert_eq!(parsed.text(), "text");
        let span = &parsed.spans()[0];
        assert!(span.style.text.bold);
        assert!(span.style.fg.is_some());
        assert!(span.style.bg.is_some());
    }

    #[test]
    fn parse_escaped_brackets() {
        let parsed = parse(r"\[not a tag\]").unwrap();
        assert_eq!(parsed.text(), "[not a tag]");
        assert!(parsed.spans().is_empty());
    }

    #[test]
    fn parse_meta_tag() {
        let parsed = parse("[@click=app.quit]Exit[/]").unwrap();
        assert_eq!(parsed.text(), "Exit");
        assert_eq!(parsed.spans().len(), 1);
        assert_eq!(parsed.spans()[0].get_meta("@click"), Some("app.quit"));
    }

    #[test]
    fn parse_style_and_meta() {
        let parsed = parse("[bold]Hello[/] [@click=test]Click[/]").unwrap();
        assert_eq!(parsed.text(), "Hello Click");
        assert_eq!(parsed.spans().len(), 2);
    }

    #[test]
    fn parse_close_specific() {
        let parsed = parse("[bold][red]text[/bold]more[/]").unwrap();
        assert_eq!(parsed.text(), "textmore");
        // Should have spans for bold and red
        assert!(parsed.spans().len() >= 1);
    }

    #[test]
    fn parse_unclosed_tags() {
        // Unclosed tags should still produce valid output
        let parsed = parse("[bold]Hello").unwrap();
        assert_eq!(parsed.text(), "Hello");
        assert_eq!(parsed.spans().len(), 1);
    }

    #[test]
    fn parse_unicode() {
        let parsed = parse("[bold]日本語[/]").unwrap();
        assert_eq!(parsed.text(), "日本語");
        assert_eq!(parsed.spans().len(), 1);
    }

    #[test]
    fn parse_empty_input() {
        let parsed = parse("").unwrap();
        assert_eq!(parsed.text(), "");
        assert!(parsed.spans().is_empty());
    }

    #[test]
    fn parse_complex_example() {
        let parsed =
            parse("[bold red]Important:[/] Click [@click=app.help]here[/] for help").unwrap();
        assert_eq!(parsed.text(), "Important: Click here for help");
        assert_eq!(parsed.spans().len(), 2);
    }
}
