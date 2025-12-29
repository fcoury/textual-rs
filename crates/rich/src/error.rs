//! Error types for Rich markup parsing.

use thiserror::Error;

/// Errors that can occur when parsing Rich markup.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum RichParseError {
    /// Invalid color specification.
    #[error("invalid color: {0}")]
    InvalidColor(String),

    /// Invalid style modifier.
    #[error("invalid style modifier: {0}")]
    InvalidModifier(String),

    /// Unclosed tag (missing `]`).
    #[error("unclosed tag starting at position {0}")]
    UnclosedTag(usize),

    /// Unexpected close tag without matching open.
    #[error("unexpected close tag at position {0}")]
    UnexpectedCloseTag(usize),

    /// Empty tag content.
    #[error("empty tag at position {0}")]
    EmptyTag(usize),

    /// Invalid escape sequence.
    #[error("invalid escape sequence at position {0}")]
    InvalidEscape(usize),
}

/// Errors that can occur when parsing a color.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ColorParseError {
    /// Unknown color name.
    #[error("unknown color name: {0}")]
    UnknownName(String),

    /// Invalid hex color format.
    #[error("invalid hex color: {0}")]
    InvalidHex(String),

    /// Invalid RGB color format.
    #[error("invalid RGB color: {0}")]
    InvalidRgb(String),

    /// Invalid HSL color format.
    #[error("invalid HSL color: {0}")]
    InvalidHsl(String),
}

/// Errors that can occur when parsing a style.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum StyleParseError {
    /// Invalid color in style.
    #[error("invalid color in style: {0}")]
    InvalidColor(#[from] ColorParseError),

    /// Unknown style modifier.
    #[error("unknown style modifier: {0}")]
    UnknownModifier(String),

    /// Empty style specification.
    #[error("empty style specification")]
    Empty,
}
