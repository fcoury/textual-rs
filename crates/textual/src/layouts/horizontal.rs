//! Horizontal layout algorithm - stacks children left-to-right.

use crate::canvas::{Region, Size};
use crate::fraction::Fraction;
use tcss::types::geometry::Unit;
use tcss::types::ComputedStyle;

use super::size_resolver::{resolve_height_with_intrinsic, DEFAULT_FIXED_WIDTH};
use super::{Layout, WidgetPlacement};

/// Horizontal layout - stacks children left-to-right.
///
/// Each child gets its CSS-specified height (or full height if not specified)
/// and its CSS-specified width (or auto width if not specified).
/// Supports `fr` units for proportional width distribution.
#[derive(Debug, Clone, Copy, Default)]
pub struct HorizontalLayout;

impl Layout for HorizontalLayout {
    fn arrange(
        &mut self,
        _parent_style: &ComputedStyle,
        children: &[(usize, ComputedStyle, Size)],
        available: Region,
    ) -> Vec<WidgetPlacement> {
        let mut placements = Vec::with_capacity(children.len());

        // First pass: calculate space used by non-fr items and collect fr totals
        let mut fixed_width_used: i32 = 0;
        let mut total_fr: i64 = 0;
        let mut total_margin: i32 = 0;
        let mut prev_margin_right: i32 = 0;

        for (i, (_child_index, child_style, _desired_size)) in children.iter().enumerate() {
            let margin_left = child_style.margin.left.value as i32;
            let margin_right = child_style.margin.right.value as i32;

            // CSS margin collapsing
            let effective_left_margin = if i == 0 {
                margin_left
            } else {
                (margin_left - prev_margin_right).max(0)
            };
            total_margin += effective_left_margin + margin_right;
            prev_margin_right = margin_right;

            if let Some(width) = &child_style.width {
                match width.unit {
                    Unit::Fraction => {
                        // Scale fr values to integers (multiply by 1000 for precision)
                        total_fr += (width.value * 1000.0) as i64;
                    }
                    Unit::Cells => {
                        fixed_width_used += width.value as i32;
                    }
                    Unit::Percent => {
                        fixed_width_used += ((width.value / 100.0) * available.width as f64) as i32;
                    }
                    _ => {
                        // Auto or other - use default fixed width
                        fixed_width_used += DEFAULT_FIXED_WIDTH;
                    }
                }
            } else {
                // No width specified - use default
                fixed_width_used += DEFAULT_FIXED_WIDTH;
            }
        }

        // Calculate remaining space for fr units
        let remaining_for_fr = (available.width - fixed_width_used - total_margin).max(0) as i64;

        // Second pass: place children with calculated widths using Fraction for precise remainder handling
        let mut current_x = available.x;
        prev_margin_right = 0;
        let mut fr_remainder = Fraction::ZERO;

        for (i, (child_index, child_style, desired_size)) in children.iter().enumerate() {
            // Resolve width - use Fraction for fr units to match Python Textual behavior
            let width = if let Some(w) = &child_style.width {
                match w.unit {
                    Unit::Fraction => {
                        if total_fr > 0 {
                            // Use Fraction arithmetic: extra pixels go to later widgets
                            let fr_value = (w.value * 1000.0) as i64;
                            let raw = Fraction::new(remaining_for_fr * fr_value, total_fr) + fr_remainder;
                            let result = raw.floor() as i32;
                            fr_remainder = raw.fract();
                            result
                        } else {
                            0
                        }
                    }
                    Unit::Cells => w.value as i32,
                    Unit::Percent => ((w.value / 100.0) * available.width as f64) as i32,
                    _ => DEFAULT_FIXED_WIDTH,
                }
            } else {
                DEFAULT_FIXED_WIDTH
            };

            // Resolve height - horizontal layout children fill available height by default
            let height = resolve_height_with_intrinsic(child_style, desired_size.height, available.height);

            // Get margins for positioning
            let margin_left = child_style.margin.left.value as i32;
            let margin_right = child_style.margin.right.value as i32;
            let margin_top = child_style.margin.top.value as i32;
            let margin_bottom = child_style.margin.bottom.value as i32;

            // CSS margin collapsing: use max of adjacent margins, not sum
            let effective_left_margin = if i == 0 {
                margin_left
            } else {
                (margin_left - prev_margin_right).max(0)
            };

            current_x += effective_left_margin;

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
            prev_margin_right = margin_right;
        }

        placements
    }
}
