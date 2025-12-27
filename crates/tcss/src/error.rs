//! Error types for TCSS parsing and processing.
//!
//! This module defines the error types that can occur when parsing TCSS stylesheets
//! or resolving variables within them.

use thiserror::Error;

/// Errors that can occur during TCSS parsing and processing.
///
/// # Examples
///
/// ```rust
/// use tcss::parser::parse_stylesheet;
/// use tcss::TcssError;
///
/// // Syntax error example - missing value after colon
/// let result = parse_stylesheet("Button { color: }");
/// assert!(result.is_err());
/// ```
#[derive(Error, Debug)]
pub enum TcssError {
    /// Invalid CSS syntax was encountered during parsing.
    ///
    /// The string contains details about what was unexpected and where.
    #[error("CSS syntax error: {0}")]
    InvalidSyntax(String),

    /// A variable was referenced but not defined.
    ///
    /// This occurs when a stylesheet uses `$variable-name` but no definition
    /// for that variable exists (either in the stylesheet or as a theme variable).
    #[error("Unknown variable: {0}")]
    UnknownVariable(String),

    /// An I/O error occurred while reading a stylesheet file.
    #[error("I/O error reading stylesheet")]
    Io(#[from] std::io::Error),
}
