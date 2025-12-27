#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Unit {
    /// Character cells (standard terminal units).
    #[default]
    Cells,
    /// Percentage of the parent's dimension.
    Percent,
    /// Percentage of parent width.
    Width,
    /// Percentage of parent height.
    Height,
    /// Percentage of viewport width.
    ViewWidth,
    /// Percentage of viewport height.
    ViewHeight,
    /// Grid fraction (fr).
    Fraction,
    /// Automatic sizing based on content.
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Scalar {
    pub value: f64,
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
