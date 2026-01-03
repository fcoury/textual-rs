//! Grid layout types for CSS Grid support.
//!
//! This module provides types for CSS Grid layout:
//!
//! - [`GridStyle`]: Container-level grid configuration
//! - [`GridPlacement`]: Child placement within a grid
//!
//! ## CSS Syntax
//!
//! ```css
//! .container {
//!     layout: grid;
//!     grid-size: 4;           /* 4 columns */
//!     grid-columns: 1fr 2fr;  /* column widths (cyclic) */
//!     grid-rows: auto;        /* row heights */
//!     grid-gutter: 1 2;       /* vertical horizontal spacing */
//! }
//!
//! .child {
//!     column-span: 2;         /* span 2 columns */
//!     row-span: 1;            /* span 1 row */
//! }
//! ```

use super::Scalar;

/// Grid container configuration.
///
/// Defines the grid structure including column/row counts,
/// dimension definitions, and gutter spacing.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridStyle {
    /// Number of columns (from `grid-size`).
    /// If None, defaults to 1.
    pub columns: Option<u16>,

    /// Number of rows (from `grid-size`).
    /// If None, rows are created automatically based on children.
    pub rows: Option<u16>,

    /// Column width definitions (from `grid-columns`).
    /// Values cycle if fewer than column count.
    /// Empty means equal-width columns.
    pub column_widths: Vec<Scalar>,

    /// Row height definitions (from `grid-rows`).
    /// Values cycle if fewer than row count.
    /// Empty means equal-height rows.
    pub row_heights: Vec<Scalar>,

    /// Gutter spacing between cells (vertical, horizontal).
    /// From `grid-gutter` property, which is specified as
    /// `grid-gutter: <vertical> <horizontal>`.
    pub gutter: (Scalar, Scalar),
}

/// Child placement within a grid.
///
/// Controls how many columns/rows a child spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPlacement {
    /// Number of columns this child spans (default: 1).
    pub column_span: u16,

    /// Number of rows this child spans (default: 1).
    pub row_span: u16,
}

impl Default for GridPlacement {
    fn default() -> Self {
        Self {
            column_span: 1,
            row_span: 1,
        }
    }
}
