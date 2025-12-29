//! Horizontal layout algorithm - stacks children left-to-right.

use crate::canvas::Region;
use tcss::types::ComputedStyle;

use super::{Layout, WidgetPlacement};

/// Horizontal layout - stacks children left-to-right.
///
/// Each child gets the full height of the container and its desired width.
#[derive(Debug, Clone, Copy, Default)]
pub struct HorizontalLayout;

impl Layout for HorizontalLayout {
    fn arrange(
        &mut self,
        _parent_style: &ComputedStyle,
        children: &[(usize, ComputedStyle)],
        available: Region,
    ) -> Vec<WidgetPlacement> {
        let mut placements = Vec::with_capacity(children.len());
        let mut current_x = available.x;

        for (child_index, child_style) in children {
            // Use child's width from style, or a default
            let width = resolve_width(child_style, available.width);

            placements.push(WidgetPlacement {
                child_index: *child_index,
                region: Region {
                    x: current_x,
                    y: available.y,
                    width,
                    height: available.height,
                },
            });

            current_x += width;
        }

        placements
    }
}

/// Resolve the width for a child widget.
///
/// For now, this returns a default width. In the future, this should
/// read from the child's computed style (width property).
fn resolve_width(child_style: &ComputedStyle, available_width: i32) -> i32 {
    // Try to use the child's explicit width if set
    if let Some(width) = &child_style.width {
        use tcss::types::Unit;
        match width.unit {
            Unit::Cells => return width.value as i32,
            Unit::Percent => return ((width.value / 100.0) * available_width as f64) as i32,
            Unit::Auto => {
                // Auto width - use a reasonable default
                return 10;
            }
            _ => return width.value as i32,
        }
    }
    // Default: use a small fixed width
    10
}
