//! Parser for Rich markup.
//!
//! This module contains the lexer, tag parser, and main markup parser.

mod lexer;
mod markup;
mod tag;

pub use lexer::{Lexer, Token};
pub use markup::parse;
pub use tag::{ParsedTag, TagContent};
