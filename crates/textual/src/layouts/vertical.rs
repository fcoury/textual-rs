//! Vertical layout algorithm - stacks children top-to-bottom.

use crate::canvas::Region;
use tcss::types::ComputedStyle;

use super::size_resolver::{resolve_height_fixed, resolve_width_fill};
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
        let mut prev_margin_bottom: i32 = 0;

        for (i, (child_index, child_style)) in children.iter().enumerate() {
            // Resolve child dimensions from CSS
            // Vertical layout: children fill width, have fixed/auto height
            let height = resolve_height_fixed(child_style, available.height);
            let width = resolve_width_fill(child_style, available.width);

            // Get margin for positioning (Scalar.value is f64)
            let margin_top = child_style.margin.top.value as i32;
            let margin_left = child_style.margin.left.value as i32;
            let margin_bottom = child_style.margin.bottom.value as i32;

            // CSS margin collapsing: use max of adjacent margins, not sum
            let effective_top_margin = if i == 0 {
                margin_top // First child: full top margin
            } else {
                // Collapse: the gap between siblings is max(prev_bottom, current_top)
                // We already advanced by prev_margin_bottom, so we only add the difference
                // if current_top is larger
                (margin_top - prev_margin_bottom).max(0)
            };

            current_y += effective_top_margin;

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
            current_y += height + margin_bottom;
            prev_margin_bottom = margin_bottom;
        }

        placements
    }
}
