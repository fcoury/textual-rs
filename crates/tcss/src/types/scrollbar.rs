//! Scrollbar CSS types.

use super::RgbaColor;

/// Scrollbar size configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollbarSize {
    /// Horizontal scrollbar height (0 = hidden)
    pub horizontal: u16,
    /// Vertical scrollbar width (0 = hidden)
    pub vertical: u16,
}

impl ScrollbarSize {
    /// Default scrollbar size (matches Python Textual: vertical=2, horizontal=1).
    pub const DEFAULT: Self = Self {
        horizontal: 1,
        vertical: 2,
    };
}

impl Default for ScrollbarSize {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Scrollbar gutter behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScrollbarGutter {
    /// Only show scrollbar space when scrolling is possible
    #[default]
    Auto,
    /// Always reserve space for scrollbar
    Stable,
}

/// Scrollbar visibility (independent of overflow).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    #[default]
    Visible,
    /// Hide scrollbar but still allow scrolling
    Hidden,
}

/// Complete scrollbar style configuration.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScrollbarStyle {
    // Thumb colors
    pub color: Option<RgbaColor>,
    pub color_hover: Option<RgbaColor>,
    pub color_active: Option<RgbaColor>,

    // Track colors
    pub background: Option<RgbaColor>,
    pub background_hover: Option<RgbaColor>,
    pub background_active: Option<RgbaColor>,

    // Corner
    pub corner_color: Option<RgbaColor>,

    // Size & visibility
    pub size: ScrollbarSize,
    pub gutter: ScrollbarGutter,
    pub visibility: ScrollbarVisibility,
}

impl ScrollbarStyle {
    /// Fallback thumb color.
    /// Python Textual computes this as: background-darken-1 + primary.with_alpha(0.4)
    /// With textual-dark theme (primary=#0178D4, background=#121212), this produces ~#003054.
    pub fn fallback_thumb() -> RgbaColor {
        RgbaColor::rgb(0, 48, 84)
    }

    /// Fallback track color.
    /// Python Textual renders this as pure black in practice.
    pub fn fallback_track() -> RgbaColor {
        RgbaColor::rgb(0, 0, 0)
    }

    /// Fallback corner color (same as track).
    pub fn fallback_corner() -> RgbaColor {
        RgbaColor::rgb(0, 0, 0)
    }

    /// Get effective thumb color (with fallback).
    pub fn effective_color(&self) -> RgbaColor {
        self.color.clone().unwrap_or_else(Self::fallback_thumb)
    }

    /// Get effective thumb hover color (falls back to color, then fallback).
    pub fn effective_color_hover(&self) -> RgbaColor {
        self.color_hover
            .clone()
            .or_else(|| self.color.clone())
            .unwrap_or_else(Self::fallback_thumb)
    }

    /// Get effective thumb active color (falls back to hover, then color, then fallback).
    pub fn effective_color_active(&self) -> RgbaColor {
        self.color_active
            .clone()
            .or_else(|| self.color_hover.clone())
            .or_else(|| self.color.clone())
            .unwrap_or_else(Self::fallback_thumb)
    }

    /// Get effective track background color (with fallback).
    pub fn effective_background(&self) -> RgbaColor {
        self.background.clone().unwrap_or_else(Self::fallback_track)
    }

    /// Get effective track hover color (falls back to background, then fallback).
    pub fn effective_background_hover(&self) -> RgbaColor {
        self.background_hover
            .clone()
            .or_else(|| self.background.clone())
            .unwrap_or_else(Self::fallback_track)
    }

    /// Get effective track active color (falls back to hover, then background, then fallback).
    pub fn effective_background_active(&self) -> RgbaColor {
        self.background_active
            .clone()
            .or_else(|| self.background_hover.clone())
            .or_else(|| self.background.clone())
            .unwrap_or_else(Self::fallback_track)
    }

    /// Get effective corner color (with fallback).
    pub fn effective_corner_color(&self) -> RgbaColor {
        self.corner_color
            .clone()
            .unwrap_or_else(Self::fallback_corner)
    }

    /// Whether vertical scrollbar should be shown (size > 0 and visible).
    pub fn show_vertical(&self) -> bool {
        self.size.vertical > 0 && self.visibility == ScrollbarVisibility::Visible
    }

    /// Whether horizontal scrollbar should be shown (size > 0 and visible).
    pub fn show_horizontal(&self) -> bool {
        self.size.horizontal > 0 && self.visibility == ScrollbarVisibility::Visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollbar_size_default() {
        let size = ScrollbarSize::default();
        assert_eq!(size.horizontal, 1);
        assert_eq!(size.vertical, 2);
    }

    #[test]
    fn test_scrollbar_style_default() {
        let style = ScrollbarStyle::default();
        assert!(style.color.is_none());
        assert!(style.background.is_none());
        assert_eq!(style.size, ScrollbarSize::DEFAULT);
        assert_eq!(style.gutter, ScrollbarGutter::Auto);
        assert_eq!(style.visibility, ScrollbarVisibility::Visible);
    }

    #[test]
    fn test_effective_color_fallback() {
        let style = ScrollbarStyle::default();
        // Default thumb is dark blue (#003054)
        assert_eq!(style.effective_color(), RgbaColor::rgb(0, 48, 84));
    }

    #[test]
    fn test_effective_color_set() {
        let mut style = ScrollbarStyle::default();
        let custom_color = RgbaColor::rgb(100, 150, 200);
        style.color = Some(custom_color.clone());
        assert_eq!(style.effective_color(), custom_color);
    }

    #[test]
    fn test_effective_background_fallback() {
        let style = ScrollbarStyle::default();
        // Default track is pure black
        assert_eq!(style.effective_background(), RgbaColor::rgb(0, 0, 0));
    }

    #[test]
    fn test_hover_color_fallback_chain() {
        // No hover set -> falls back to color
        let mut style = ScrollbarStyle::default();
        let base_color = RgbaColor::rgb(100, 100, 100);
        style.color = Some(base_color.clone());
        assert_eq!(style.effective_color_hover(), base_color);

        // Hover set -> uses hover
        let hover_color = RgbaColor::rgb(150, 150, 150);
        style.color_hover = Some(hover_color.clone());
        assert_eq!(style.effective_color_hover(), hover_color);
    }

    #[test]
    fn test_active_color_fallback_chain() {
        let mut style = ScrollbarStyle::default();
        let base_color = RgbaColor::rgb(100, 100, 100);
        let hover_color = RgbaColor::rgb(150, 150, 150);
        let active_color = RgbaColor::rgb(200, 200, 200);

        // No colors set -> fallback (dark blue #003054)
        assert_eq!(style.effective_color_active(), RgbaColor::rgb(0, 48, 84));

        // Only base set -> uses base
        style.color = Some(base_color.clone());
        assert_eq!(style.effective_color_active(), base_color);

        // Base and hover set -> uses hover
        style.color_hover = Some(hover_color.clone());
        assert_eq!(style.effective_color_active(), hover_color);

        // All set -> uses active
        style.color_active = Some(active_color.clone());
        assert_eq!(style.effective_color_active(), active_color);
    }

    #[test]
    fn test_show_vertical() {
        let mut style = ScrollbarStyle::default();
        assert!(style.show_vertical());

        style.size.vertical = 0;
        assert!(!style.show_vertical());

        style.size.vertical = 1;
        style.visibility = ScrollbarVisibility::Hidden;
        assert!(!style.show_vertical());
    }

    #[test]
    fn test_show_horizontal() {
        let mut style = ScrollbarStyle::default();
        assert!(style.show_horizontal());

        style.size.horizontal = 0;
        assert!(!style.show_horizontal());

        style.size.horizontal = 1;
        style.visibility = ScrollbarVisibility::Hidden;
        assert!(!style.show_horizontal());
    }
}
