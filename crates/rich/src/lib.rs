//! Rich markup parser for terminal text styling.
//!
//! This crate provides a parser for Rich-style markup, converting text like
//! `[bold red]Hello[/] World` into styled spans that can be rendered to a terminal.
//!
//! # Overview
//!
//! The Rich markup format uses square brackets for styling:
//!
//! - `[bold]text[/]` - Apply bold styling
//! - `[red]text[/]` - Apply red foreground color
//! - `[on blue]text[/]` - Apply blue background color
//! - `[bold white on blue]text[/]` - Combined styling
//! - `[/]` - Close most recent style
//! - `\[` - Escaped bracket (literal `[`)
//!
//! # Extension Mechanism
//!
//! The `meta` field on [`Span`] provides an extension point for framework-specific
//! attributes. For example, Textual uses `[@click=action]` for action links:
//!
//! ```text
//! [link=https://example.com]Click here[/]
//! [@click=app.quit]Exit[/]
//! ```
//!
//! These are parsed as meta entries: `span.meta["@click"] = "app.quit"`
//!
//! # Usage
//!
//! ```
//! use rich::{ParsedMarkup, Color, Style, Span};
//!
//! // Parse markup text
//! let parsed = ParsedMarkup::parse("[bold red]Hello[/] World").unwrap();
//! assert_eq!(parsed.text(), "Hello World");
//!
//! // Work with individual types
//! let color = Color::parse("red").unwrap();
//! let style = Style::parse("bold white on blue").unwrap();
//! ```

pub mod color;
pub mod error;
pub mod markup;
pub mod parser;
pub mod span;
pub mod style;

// Re-export main types at crate root
pub use color::Color;
pub use error::{ColorParseError, RichParseError, StyleParseError};
pub use markup::ParsedMarkup;
pub use span::Span;
pub use style::{Style, TextStyle};
