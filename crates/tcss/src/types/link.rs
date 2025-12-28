//! Link styling and configuration types.
//!
//! This module provides [`LinkStyle`] for configuring link appearance,
//! including colors and text styles for both default and hover states.

use super::color::RgbaColor;
use super::text::TextStyle;

/// Complete link style configuration.
///
/// Stores colors and text styles for links in both default and hover states.
/// Used by the CSS cascade to resolve `link-*` properties.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LinkStyle {
    /// Link text color.
    pub color: Option<RgbaColor>,
    /// Link background color.
    pub background: Option<RgbaColor>,
    /// Link text color on hover.
    pub color_hover: Option<RgbaColor>,
    /// Link background color on hover.
    pub background_hover: Option<RgbaColor>,
    /// Link text style (bold, underline, etc.).
    pub style: TextStyle,
    /// Link text style on hover.
    pub style_hover: TextStyle,
}

impl LinkStyle {
    /// Returns the effective link color, with fallback to default blue.
    pub fn effective_color(&self) -> RgbaColor {
        self.color
            .clone()
            .unwrap_or_else(|| RgbaColor::rgb(0, 0, 255))
    }

    /// Returns the effective link hover color, falling back to base color.
    pub fn effective_color_hover(&self) -> RgbaColor {
        self.color_hover
            .clone()
            .or_else(|| self.color.clone())
            .unwrap_or_else(|| RgbaColor::rgb(0, 0, 255))
    }

    /// Returns the effective background color.
    pub fn effective_background(&self) -> Option<RgbaColor> {
        self.background.clone()
    }

    /// Returns the effective hover background color, falling back to base.
    pub fn effective_background_hover(&self) -> Option<RgbaColor> {
        self.background_hover
            .clone()
            .or_else(|| self.background.clone())
    }
}
