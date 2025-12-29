//! Vertical layout algorithm - stacks children top-to-bottom.

use crate::canvas::Region;
use tcss::types::ComputedStyle;

use super::{Layout, WidgetPlacement};

/// Vertical layout - stacks children top-to-bottom.
///
/// Each child gets its CSS-specified width (or full width if not specified)
/// and its CSS-specified height (or auto height if not specified).
#[derive(Debug, Clone, Copy, Default)]
pub struct VerticalLayout;

impl Layout for VerticalLayout {
    fn arrange(
        &mut self,
        _parent_style: &ComputedStyle,
        children: &[(usize, ComputedStyle)],
        available: Region,
    ) -> Vec<WidgetPlacement> {
        let mut placements = Vec::with_capacity(children.len());
        let mut current_y = available.y;

        for (child_index, child_style) in children {
            // Resolve child dimensions from CSS
            let height = resolve_height(child_style, available.height);
            let width = resolve_width(child_style, available.width);

            // Get margin for positioning (Scalar.value is f64)
            let margin_top = child_style.margin.top.value as i32;
            let margin_left = child_style.margin.left.value as i32;

            // Apply margin to y position
            current_y += margin_top;

            placements.push(WidgetPlacement {
                child_index: *child_index,
                region: Region {
                    x: available.x + margin_left,
                    y: current_y,
                    width,
                    height,
                },
            });

            // Advance by height + bottom margin
            let margin_bottom = child_style.margin.bottom.value as i32;
            current_y += height + margin_bottom;
        }

        placements
    }
}

/// Resolve the width for a child widget.
///
/// Returns the child's CSS width if specified, otherwise the available width.
fn resolve_width(child_style: &ComputedStyle, available_width: i32) -> i32 {
    if let Some(width) = &child_style.width {
        use tcss::types::Unit;
        match width.unit {
            Unit::Cells => return width.value as i32,
            Unit::Percent => return ((width.value / 100.0) * available_width as f64) as i32,
            Unit::Auto => return available_width, // Auto = fill available
            _ => return width.value as i32,
        }
    }
    // Default: fill available width
    available_width
}

/// Resolve the height for a child widget.
///
/// Returns the child's CSS height if specified, otherwise a default.
fn resolve_height(child_style: &ComputedStyle, available_height: i32) -> i32 {
    if let Some(height) = &child_style.height {
        use tcss::types::Unit;
        match height.unit {
            Unit::Cells => return height.value as i32,
            Unit::Percent => return ((height.value / 100.0) * available_height as f64) as i32,
            Unit::Auto => {
                // Auto height - use a reasonable default
                return 3;
            }
            _ => return height.value as i32,
        }
    }
    // Default: use a small fixed height
    3
}
