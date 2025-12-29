//! Vertical layout algorithm - stacks children top-to-bottom.

use crate::canvas::Region;
use tcss::types::ComputedStyle;

use super::{Layout, WidgetPlacement};

/// Vertical layout - stacks children top-to-bottom.
///
/// Each child gets the full width of the container and its desired height.
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
            // Use child's height from style, or a default
            let height = resolve_height(child_style, available.height);

            placements.push(WidgetPlacement {
                child_index: *child_index,
                region: Region {
                    x: available.x,
                    y: current_y,
                    width: available.width,
                    height,
                },
            });

            current_y += height;
        }

        placements
    }
}

/// Resolve the height for a child widget.
///
/// For now, this returns a default height. In the future, this should
/// read from the child's computed style (height property).
fn resolve_height(child_style: &ComputedStyle, available_height: i32) -> i32 {
    // Try to use the child's explicit height if set
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
