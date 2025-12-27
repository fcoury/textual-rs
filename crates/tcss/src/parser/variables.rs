//! CSS variable extraction and resolution.
//!
//! This module handles TCSS variables (SCSS-style `$variable` syntax):
//!
//! - [`extract_variables`]: Scans source for variable definitions
//! - [`resolve_variables`]: Replaces variable references with values
//!
//! ## Variable Syntax
//!
//! Variables are defined at the top level of a stylesheet:
//!
//! ```css
//! $primary-color: blue;
//! $spacing: 10;
//!
//! Button {
//!     color: $primary-color;
//!     margin: $spacing;
//! }
//! ```
//!
//! ## Processing Steps
//!
//! 1. Block comments (`/* */`) are stripped
//! 2. Variable definitions (`$name: value;`) are extracted
//! 3. Variable references (`$name`) are replaced with values
//! 4. Definition lines are removed from output

use crate::error::TcssError;
use std::collections::HashMap;

/// Storage for stylesheet-defined variables.
///
/// Variables are extracted from the source before parsing and then
/// resolved by replacing `$name` references with their values.
#[derive(Debug, Clone, Default)]
pub struct StylesheetVariables {
    variables: HashMap<String, String>,
}

impl StylesheetVariables {
    /// Creates an empty variable storage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Defines a variable with the given name and value.
    pub fn define(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    /// Resolves a variable name to its value, if defined.
    pub fn resolve(&self, name: &str) -> Option<String> {
        self.variables.get(name).cloned()
    }
}

pub fn resolve_variables(source: &str, vars: &StylesheetVariables) -> Result<String, TcssError> {
    let mut output = String::new();

    // Pass 1: Strip Block Comments entirely before processing
    let mut clean_source = String::new();
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(inner) = chars.next() {
                if inner == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
            continue;
        }
        clean_source.push(c);
    }

    // Pass 2: Resolve variables line by line
    for line in clean_source.lines() {
        let trimmed = line.trim();
        // Skip definition lines - they shouldn't be resolved or included in final CSS
        if trimmed.starts_with('$') {
            continue;
        }

        let mut resolved_line = String::new();
        let mut line_chars = line.chars().peekable();
        while let Some(c) = line_chars.next() {
            if c == '$' {
                let mut name = String::new();
                while let Some(&next) = line_chars.peek() {
                    if next.is_alphanumeric() || next == '-' || next == '_' {
                        name.push(line_chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if let Some(val) = vars.resolve(&name) {
                    resolved_line.push_str(&val);
                } else {
                    resolved_line.push('$');
                    resolved_line.push_str(&name);
                }
            } else {
                resolved_line.push(c);
            }
        }
        output.push_str(&resolved_line);
        output.push('\n');
    }

    Ok(output)
}

pub fn extract_variables(source: &str) -> StylesheetVariables {
    let mut vars = StylesheetVariables::new();
    let mut in_comment = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.contains("/*") {
            in_comment = true;
        }
        if trimmed.contains("*/") {
            in_comment = false;
            continue;
        }
        if in_comment {
            continue;
        }

        if trimmed.starts_with('$') {
            if let Some(colon_idx) = trimmed.find(':') {
                let name = trimmed[1..colon_idx].trim().to_string();
                let mut value = trimmed[colon_idx + 1..].trim();
                if value.ends_with(';') {
                    value = &value[..value.len() - 1];
                }
                vars.define(name, value.to_string());
            }
        }
    }
    vars
}
