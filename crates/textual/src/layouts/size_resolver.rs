//! Shared size resolution utilities for layout algorithms.
//!
//! Provides consistent CSS dimension resolution across all layout types.
//!
//! ## Design Rationale
//!
//! Different layout directions have different default sizing behaviors:
//!
//! - **Vertical layouts**: Children fill the available width but have small fixed heights.
//!   This matches the natural reading flow where text expands horizontally and stacks
//!   vertically. Use `resolve_width_fill` and `resolve_height_fixed`.
//!
//! - **Horizontal layouts**: Children have small fixed widths but fill the available height.
//!   This matches typical horizontal arrangements like toolbars or button rows where
//!   items have intrinsic widths but stretch to container height. Use `resolve_width_fixed`
//!   and `resolve_height_fill`.
//!
//! The `Auto` unit behaves contextually:
//! - In vertical layouts: `width: auto` fills available space, `height: auto` uses a fixed default
//! - In horizontal layouts: `width: auto` uses a fixed default, `height: auto` fills available space

use tcss::types::{BoxSizing, ComputedStyle, Unit};

/// Default height when not specified (used by vertical layouts).
pub const DEFAULT_FIXED_HEIGHT: i32 = 3;

/// Default width when not specified (used by horizontal layouts).
pub const DEFAULT_FIXED_WIDTH: i32 = 10;

/// Resolve a width value from CSS, with configurable default behavior.
///
/// # Arguments
/// * `child_style` - The computed style containing the width property
/// * `available_width` - The available width from the parent container
/// * `fill_by_default` - If true, defaults to available_width; if false, defaults to DEFAULT_FIXED_WIDTH
///
/// # Behavior
/// - `Unit::Cells`: Returns the value as-is
/// - `Unit::Percent`: Returns percentage of available_width
/// - `Unit::Auto`: Returns available_width (fill)
/// - No width specified: Returns available_width if fill_by_default, else DEFAULT_FIXED_WIDTH
pub fn resolve_width(child_style: &ComputedStyle, available_width: i32, fill_by_default: bool) -> i32 {
    if let Some(width) = &child_style.width {
        match width.unit {
            Unit::Cells => return width.value as i32,
            Unit::Percent => return ((width.value / 100.0) * available_width as f64).round() as i32,
            Unit::Auto => return available_width, // Auto always fills available
            _ => return width.value as i32,
        }
    }
    // Default behavior depends on layout type
    if fill_by_default {
        available_width
    } else {
        DEFAULT_FIXED_WIDTH
    }
}

/// Resolve a height value from CSS, with configurable default behavior.
///
/// # Arguments
/// * `child_style` - The computed style containing the height property
/// * `available_height` - The available height from the parent container
/// * `fill_by_default` - If true, defaults to available_height; if false, defaults to DEFAULT_FIXED_HEIGHT
///
/// # Behavior
/// - `Unit::Cells`: Returns the value as-is
/// - `Unit::Percent`: Returns percentage of available_height
/// - `Unit::Auto`: Returns available_height if fill_by_default, else DEFAULT_FIXED_HEIGHT
/// - No height specified: Returns available_height if fill_by_default, else DEFAULT_FIXED_HEIGHT
pub fn resolve_height(child_style: &ComputedStyle, available_height: i32, fill_by_default: bool) -> i32 {
    if let Some(height) = &child_style.height {
        match height.unit {
            Unit::Cells => return height.value as i32,
            Unit::Percent => return ((height.value / 100.0) * available_height as f64).round() as i32,
            Unit::Auto => {
                // Auto behavior: fill if horizontal layout (fill_by_default), fixed if vertical
                if fill_by_default {
                    return available_height;
                } else {
                    return DEFAULT_FIXED_HEIGHT;
                }
            }
            _ => return height.value as i32,
        }
    }
    // Default behavior depends on layout type
    if fill_by_default {
        available_height
    } else {
        DEFAULT_FIXED_HEIGHT
    }
}

/// Convenience: resolve width for vertical layouts (fill by default).
#[inline]
pub fn resolve_width_fill(child_style: &ComputedStyle, available_width: i32) -> i32 {
    resolve_width(child_style, available_width, true)
}

/// Convenience: resolve width for horizontal layouts (fixed by default).
#[inline]
pub fn resolve_width_fixed(child_style: &ComputedStyle, available_width: i32) -> i32 {
    resolve_width(child_style, available_width, false)
}

/// Convenience: resolve height for vertical layouts (fixed by default).
#[inline]
pub fn resolve_height_fixed(child_style: &ComputedStyle, available_height: i32) -> i32 {
    resolve_height(child_style, available_height, false)
}

/// Convenience: resolve height for horizontal layouts (fill by default).
#[inline]
pub fn resolve_height_fill(child_style: &ComputedStyle, available_height: i32) -> i32 {
    resolve_height(child_style, available_height, true)
}

/// Resolve width, using intrinsic size for `auto`.
///
/// This function differs from `resolve_width` in how it handles `Unit::Auto`:
/// - `resolve_width`: Auto fills available space
/// - `resolve_width_with_intrinsic`: Auto uses the widget's intrinsic/desired width
///
/// This matches Python Textual's behavior where `width: auto` means "size to content".
pub fn resolve_width_with_intrinsic(
    child_style: &ComputedStyle,
    intrinsic_width: u16,
    available_width: i32,
) -> i32 {
    if let Some(width) = &child_style.width {
        match width.unit {
            Unit::Cells => width.value as i32,
            Unit::Percent => ((width.value / 100.0) * available_width as f64).round() as i32,
            Unit::Auto => intrinsic_width as i32, // Use intrinsic, not fill!
            Unit::Fraction => available_width,    // fr units fill available space
            _ => width.value as i32,
        }
    } else {
        // No width specified: fill available (default behavior for vertical/horizontal layouts)
        available_width
    }
}

/// Resolve height, using intrinsic size for `auto`.
///
/// This function differs from `resolve_height` in how it handles `Unit::Auto`:
/// - `resolve_height`: Auto fills available space (in horizontal layouts) or uses default
/// - `resolve_height_with_intrinsic`: Auto uses the widget's intrinsic/desired height
///
/// This matches Python Textual's behavior where `height: auto` means "size to content".
pub fn resolve_height_with_intrinsic(
    child_style: &ComputedStyle,
    intrinsic_height: u16,
    available_height: i32,
) -> i32 {
    // Call full version with available_height for both h and vh units (approximation)
    resolve_height_full(
        child_style,
        intrinsic_height,
        available_height,
        available_height, // available_width - use height as fallback for w unit
        available_height, // viewport_width
        available_height, // viewport_height
    )
}

/// Resolve height with full context for all CSS units.
///
/// Handles all height units including:
/// - `Unit::Cells`: Fixed cell count
/// - `Unit::Percent`: Percentage of parent height
/// - `Unit::Width` (w): Percentage of parent width
/// - `Unit::Height` (h): Percentage of parent height (same as %)
/// - `Unit::ViewWidth` (vw): Percentage of viewport width
/// - `Unit::ViewHeight` (vh): Percentage of viewport height
/// - `Unit::Fraction` (fr): Fills available space
/// - `Unit::Auto`: Uses intrinsic height
pub fn resolve_height_full(
    child_style: &ComputedStyle,
    intrinsic_height: u16,
    available_height: i32,
    available_width: i32,
    viewport_width: i32,
    viewport_height: i32,
) -> i32 {
    if let Some(height) = &child_style.height {
        match height.unit {
            Unit::Cells => height.value as i32,
            Unit::Percent => ((height.value / 100.0) * available_height as f64).round() as i32,
            Unit::Width => ((height.value / 100.0) * available_width as f64).round() as i32,
            Unit::Height => ((height.value / 100.0) * available_height as f64).round() as i32,
            Unit::ViewWidth => ((height.value / 100.0) * viewport_width as f64).round() as i32,
            Unit::ViewHeight => ((height.value / 100.0) * viewport_height as f64).round() as i32,
            Unit::Auto => intrinsic_height as i32,
            Unit::Fraction => available_height,
        }
    } else {
        available_height
    }
}

// =============================================================================
// Box-Sizing Support
// =============================================================================

/// Calculate the vertical "chrome" (border + padding) for a style.
///
/// Chrome is the space used by borders and padding that surrounds the content.
pub fn vertical_chrome(style: &ComputedStyle) -> i32 {
    use tcss::types::border::BorderKind;
    let border_top = if !matches!(style.border.top.kind, BorderKind::None | BorderKind::Hidden) { 1 } else { 0 };
    let border_bottom = if !matches!(style.border.bottom.kind, BorderKind::None | BorderKind::Hidden) { 1 } else { 0 };
    let padding_top = style.padding.top.value as i32;
    let padding_bottom = style.padding.bottom.value as i32;
    border_top + border_bottom + padding_top + padding_bottom
}

/// Calculate the horizontal "chrome" (border + padding) for a style.
///
/// Chrome is the space used by borders and padding that surrounds the content.
pub fn horizontal_chrome(style: &ComputedStyle) -> i32 {
    use tcss::types::border::BorderKind;
    let border_left = if !matches!(style.border.left.kind, BorderKind::None | BorderKind::Hidden) { 1 } else { 0 };
    let border_right = if !matches!(style.border.right.kind, BorderKind::None | BorderKind::Hidden) { 1 } else { 0 };
    let padding_left = style.padding.left.value as i32;
    let padding_right = style.padding.right.value as i32;
    border_left + border_right + padding_left + padding_right
}

/// Adjust a CSS-specified height for box-sizing.
///
/// - `border-box`: The CSS height already includes chrome, return as-is
/// - `content-box`: The CSS height is content only, add chrome to get total height
///
/// # Arguments
/// * `css_height` - The height resolved from CSS (e.g., from `resolve_height_fixed`)
/// * `style` - The computed style containing box-sizing and border/padding values
///
/// # Returns
/// The total height the widget should occupy in the layout.
pub fn apply_box_sizing_height(css_height: i32, style: &ComputedStyle) -> i32 {
    match style.box_sizing {
        BoxSizing::BorderBox => css_height, // CSS height already includes chrome
        BoxSizing::ContentBox => css_height + vertical_chrome(style), // Add chrome
    }
}

/// Adjust a CSS-specified width for box-sizing.
///
/// - `border-box`: The CSS width already includes chrome, return as-is
/// - `content-box`: The CSS width is content only, add chrome to get total width
///
/// # Arguments
/// * `css_width` - The width resolved from CSS (e.g., from `resolve_width_fill`)
/// * `style` - The computed style containing box-sizing and border/padding values
///
/// # Returns
/// The total width the widget should occupy in the layout.
pub fn apply_box_sizing_width(css_width: i32, style: &ComputedStyle) -> i32 {
    match style.box_sizing {
        BoxSizing::BorderBox => css_width, // CSS width already includes chrome
        BoxSizing::ContentBox => css_width + horizontal_chrome(style), // Add chrome
    }
}
