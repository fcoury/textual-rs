//! Horizontal layout algorithm - stacks children left-to-right.

use crate::canvas::Region;
use tcss::types::ComputedStyle;

use super::size_resolver::{resolve_height_fill, resolve_width_fixed};
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
            // Resolve child dimensions from CSS
            // Horizontal layout: children have fixed/auto width, fill height
            let width = resolve_width_fixed(child_style, available.width);
            let height = resolve_height_fill(child_style, available.height);

            // Get margins for positioning (Scalar.value is f64)
            let margin_left = child_style.margin.left.value as i32;
            let margin_right = child_style.margin.right.value as i32;
            let margin_top = child_style.margin.top.value as i32;
            let margin_bottom = child_style.margin.bottom.value as i32;

            // Apply margin to x position
            current_x += margin_left;

            // Reduce height by vertical margins to prevent overflow
            let adjusted_height = (height - margin_top - margin_bottom).max(0);

            placements.push(WidgetPlacement {
                child_index: *child_index,
                region: Region {
                    x: current_x,
                    y: available.y + margin_top,
                    width,
                    height: adjusted_height,
                },
            });

            // Advance by width + right margin
            current_x += width + margin_right;
        }

        placements
    }
}
