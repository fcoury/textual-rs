//! Geometry types for dimensions and spacing.
//!
//! This module provides types for CSS dimension values:
//!
//! - [`Unit`]: The unit of measurement (cells, percent, etc.)
//! - [`Scalar`]: A numeric value with a unit
//! - [`Spacing`]: Four-sided spacing for margins and padding

/// Units of measurement for dimension values.
///
/// TCSS extends CSS units with terminal-specific units like cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Unit {
    /// Character cells - the default terminal unit.
    /// One cell is one character width/height.
    #[default]
    Cells,
    /// Percentage of the parent's corresponding dimension.
    Percent,
    /// Percentage of the parent's width (for height values).
    Width,
    /// Percentage of the parent's height (for width values).
    Height,
    /// Percentage of the viewport (terminal) width.
    ViewWidth,
    /// Percentage of the viewport (terminal) height.
    ViewHeight,
    /// Grid fraction unit for flexible layouts.
    Fraction,
    /// Automatic sizing based on content or context.
    Auto,
}

/// A dimension value with its unit.
///
/// # Examples
///
/// - `10` → 10 cells
/// - `50%` → 50 percent
/// - `auto` → automatic sizing
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Scalar {
    /// The numeric value.
    pub value: f64,
    /// The unit of measurement.
    pub unit: Unit,
}

impl Scalar {
    pub const AUTO: Self = Self {
        value: 0.0,
        unit: Unit::Auto,
    };
    pub const ZERO: Self = Self {
        value: 0.0,
        unit: Unit::Cells,
    };

    pub fn cells(value: f64) -> Self {
        Self {
            value,
            unit: Unit::Cells,
        }
    }

    pub fn percent(value: f64) -> Self {
        Self {
            value,
            unit: Unit::Percent,
        }
    }

    pub fn is_auto(&self) -> bool {
        self.unit == Unit::Auto
    }
}

/// Defines spacing (margin or padding) using Scalars for flexible layouts.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Spacing {
    pub top: Scalar,
    pub right: Scalar,
    pub bottom: Scalar,
    pub left: Scalar,
}

impl Spacing {
    pub fn all(value: Scalar) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn vertical_horizontal(vertical: Scalar, horizontal: Scalar) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
}
