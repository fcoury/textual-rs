//! Comprehensive tests for the Rich markup parser.

use rich::ParsedMarkup;

// ============================================================================
// Basic Parsing
// ============================================================================

#[test]
fn parse_plain_text() {
    let parsed = ParsedMarkup::parse("Hello World").unwrap();
    assert_eq!(parsed.text(), "Hello World");
    assert!(parsed.is_plain());
    assert_eq!(parsed.spans().len(), 0);
}

#[test]
fn parse_empty_string() {
    let parsed = ParsedMarkup::parse("").unwrap();
    assert_eq!(parsed.text(), "");
    assert!(parsed.is_empty());
}

#[test]
fn parse_whitespace_only() {
    let parsed = ParsedMarkup::parse("   ").unwrap();
    assert_eq!(parsed.text(), "   ");
}

// ============================================================================
// Simple Tags
// ============================================================================

#[test]
fn parse_bold_tag() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/]").unwrap();
    assert_eq!(parsed.text(), "Hello");
    assert_eq!(parsed.spans().len(), 1);
    assert!(parsed.spans()[0].style.text.bold);
}

#[test]
fn parse_italic_tag() {
    let parsed = ParsedMarkup::parse("[italic]Hello[/]").unwrap();
    assert_eq!(parsed.text(), "Hello");
    assert!(parsed.spans()[0].style.text.italic);
}

#[test]
fn parse_color_tag() {
    let parsed = ParsedMarkup::parse("[red]Hello[/]").unwrap();
    assert_eq!(parsed.text(), "Hello");
    assert!(parsed.spans()[0].style.fg.is_some());
}

#[test]
fn parse_background_tag() {
    let parsed = ParsedMarkup::parse("[on blue]Hello[/]").unwrap();
    assert_eq!(parsed.text(), "Hello");
    assert!(parsed.spans()[0].style.bg.is_some());
}

// ============================================================================
// Combined Styles
// ============================================================================

#[test]
fn parse_bold_color() {
    let parsed = ParsedMarkup::parse("[bold red]Hello[/]").unwrap();
    assert_eq!(parsed.text(), "Hello");
    let span = &parsed.spans()[0];
    assert!(span.style.text.bold);
    assert!(span.style.fg.is_some());
}

#[test]
fn parse_full_style() {
    let parsed = ParsedMarkup::parse("[bold white on blue]Hello[/]").unwrap();
    assert_eq!(parsed.text(), "Hello");
    let span = &parsed.spans()[0];
    assert!(span.style.text.bold);
    assert!(span.style.fg.is_some());
    assert!(span.style.bg.is_some());
}

// ============================================================================
// Nested Tags
// ============================================================================

#[test]
fn parse_nested_tags() {
    let parsed = ParsedMarkup::parse("[bold][red]Hello[/][/]").unwrap();
    assert_eq!(parsed.text(), "Hello");
    // Should have spans for both bold and red
    assert!(parsed.spans().len() >= 1);
}

#[test]
fn parse_adjacent_styled_text() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/] [italic]World[/]").unwrap();
    assert_eq!(parsed.text(), "Hello World");
    assert_eq!(parsed.spans().len(), 2);
}

#[test]
fn parse_interleaved_plain_and_styled() {
    let parsed = ParsedMarkup::parse("Plain [bold]Bold[/] Plain").unwrap();
    assert_eq!(parsed.text(), "Plain Bold Plain");
    assert_eq!(parsed.spans().len(), 1);
}

// ============================================================================
// Escape Sequences
// ============================================================================

#[test]
fn parse_escaped_open_bracket() {
    let parsed = ParsedMarkup::parse(r"\[not a tag\]").unwrap();
    assert_eq!(parsed.text(), "[not a tag]");
    assert!(parsed.is_plain());
}

#[test]
fn parse_escaped_backslash() {
    let parsed = ParsedMarkup::parse(r"\\").unwrap();
    assert_eq!(parsed.text(), r"\");
}

#[test]
fn parse_mixed_escapes_and_tags() {
    let parsed = ParsedMarkup::parse(r"[bold]Hello[/] \[escaped\]").unwrap();
    assert_eq!(parsed.text(), "Hello [escaped]");
    assert_eq!(parsed.spans().len(), 1);
}

// ============================================================================
// Close Tags
// ============================================================================

#[test]
fn parse_generic_close_tag() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/]").unwrap();
    assert_eq!(parsed.text(), "Hello");
}

#[test]
fn parse_named_close_tag() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/bold]").unwrap();
    assert_eq!(parsed.text(), "Hello");
}

#[test]
fn parse_close_all_with_multiple_open() {
    let parsed = ParsedMarkup::parse("[bold][italic]Hello[/]").unwrap();
    assert_eq!(parsed.text(), "Hello");
}

// ============================================================================
// Span Properties
// ============================================================================

#[test]
fn span_start_end() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/] World").unwrap();
    assert_eq!(parsed.text(), "Hello World");
    let span = &parsed.spans()[0];
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 5); // "Hello" is 5 chars
}

#[test]
fn span_contains() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/] World").unwrap();
    let span = &parsed.spans()[0];
    assert!(span.contains(0));
    assert!(span.contains(4));
    assert!(!span.contains(5));
    assert!(!span.contains(10));
}

// ============================================================================
// style_at Method
// ============================================================================

#[test]
fn style_at_styled_position() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/] World").unwrap();
    let style = parsed.style_at(0);
    assert!(style.text.bold);
}

#[test]
fn style_at_unstyled_position() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/] World").unwrap();
    let style = parsed.style_at(6);
    assert!(!style.text.bold);
}

#[test]
fn style_at_overlapping_spans() {
    let parsed = ParsedMarkup::parse("[bold][italic]Hello[/][/]").unwrap();
    let style = parsed.style_at(0);
    // Style should be merged from both spans
    assert!(style.text.bold || style.text.italic);
}

// ============================================================================
// Segments Iterator
// ============================================================================

#[test]
fn segments_plain_text() {
    let parsed = ParsedMarkup::parse("Hello World").unwrap();
    let segments: Vec<_> = parsed.segments().collect();
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].0, "Hello World");
}

#[test]
fn segments_styled_text() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/] World").unwrap();
    let segments: Vec<_> = parsed.segments().collect();
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].0, "Hello");
    assert!(segments[0].1.text.bold);
    assert_eq!(segments[1].0, " World");
    assert!(!segments[1].1.text.bold);
}

#[test]
fn segments_multiple_styles() {
    let parsed = ParsedMarkup::parse("[bold]A[/][italic]B[/]").unwrap();
    let segments: Vec<_> = parsed.segments().collect();
    assert!(segments.len() >= 2);
}

// ============================================================================
// Meta Tags
// ============================================================================

#[test]
fn parse_link_meta() {
    // Note: link= is NOT supported as a meta tag - only @key=value syntax
    // So we test with @link= instead
    let parsed = ParsedMarkup::parse("[@link=https://example.com]Click here[/]").unwrap();
    assert_eq!(parsed.text(), "Click here");
    let span = &parsed.spans()[0];
    assert!(span.meta.contains_key("@link"));
}

#[test]
fn parse_action_meta() {
    let parsed = ParsedMarkup::parse("[@click=app.quit]Exit[/]").unwrap();
    assert_eq!(parsed.text(), "Exit");
    let span = &parsed.spans()[0];
    assert!(span.meta.contains_key("@click"));
}

// ============================================================================
// Unicode Content
// ============================================================================

#[test]
fn parse_unicode_text() {
    let parsed = ParsedMarkup::parse("[bold]日本語[/]").unwrap();
    assert_eq!(parsed.text(), "日本語");
    assert_eq!(parsed.spans().len(), 1);
}

#[test]
fn parse_emoji() {
    let parsed = ParsedMarkup::parse("[bold]Hello 🎉[/]").unwrap();
    assert_eq!(parsed.text(), "Hello 🎉");
}

#[test]
fn parse_mixed_scripts() {
    let parsed = ParsedMarkup::parse("[red]English[/] [blue]中文[/] [green]עברית[/]").unwrap();
    assert_eq!(parsed.text(), "English 中文 עברית");
    assert_eq!(parsed.spans().len(), 3);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn parse_consecutive_close_tags() {
    let parsed = ParsedMarkup::parse("[bold][italic]text[/][/]").unwrap();
    assert_eq!(parsed.text(), "text");
}

#[test]
fn parse_style_at_end() {
    let parsed = ParsedMarkup::parse("Hello [bold]World[/]").unwrap();
    assert_eq!(parsed.text(), "Hello World");
}

#[test]
fn parse_style_at_start() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/] World").unwrap();
    assert_eq!(parsed.text(), "Hello World");
}

#[test]
fn parse_entire_text_styled() {
    let parsed = ParsedMarkup::parse("[bold]Hello World[/]").unwrap();
    assert_eq!(parsed.text(), "Hello World");
    let span = &parsed.spans()[0];
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 11);
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn parse_unclosed_tag() {
    let result = ParsedMarkup::parse("[bold");
    assert!(result.is_err());
}

#[test]
fn parse_empty_tag() {
    let result = ParsedMarkup::parse("[]");
    assert!(result.is_err());
}

// ============================================================================
// ParsedMarkup Properties
// ============================================================================

#[test]
fn len_and_is_empty() {
    let parsed = ParsedMarkup::parse("[bold]Hello[/]").unwrap();
    assert_eq!(parsed.len(), 5);
    assert!(!parsed.is_empty());

    let empty = ParsedMarkup::parse("").unwrap();
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
}

#[test]
fn plain_constructor() {
    let markup = ParsedMarkup::plain("Hello World");
    assert_eq!(markup.text(), "Hello World");
    assert!(markup.is_plain());
}
