//! Vertical layout algorithm - stacks children top-to-bottom.

use crate::canvas::{Region, Size};
use crate::fraction::Fraction;
use tcss::types::geometry::Unit;
use tcss::types::ComputedStyle;

use super::size_resolver::{resolve_height_fixed, resolve_width_with_intrinsic, DEFAULT_FIXED_HEIGHT};
use super::{Layout, WidgetPlacement};

/// Vertical layout - stacks children top-to-bottom.
///
/// Each child gets its CSS-specified width (or full width if not specified)
/// and its CSS-specified height (or auto height if not specified).
/// Supports `fr` units for proportional height distribution.
#[derive(Debug, Clone, Copy, Default)]
pub struct VerticalLayout;

impl Layout for VerticalLayout {
    fn arrange(
        &mut self,
        _parent_style: &ComputedStyle,
        children: &[(usize, ComputedStyle, Size)],
        available: Region,
    ) -> Vec<WidgetPlacement> {
        let mut placements = Vec::with_capacity(children.len());

        // First pass: calculate space used by non-fr items and collect fr totals
        let mut fixed_height_used: i32 = 0;
        let mut total_fr: i64 = 0;
        let mut total_margin: i32 = 0;
        let mut prev_margin_bottom: i32 = 0;

        for (i, (_child_index, child_style, desired_size)) in children.iter().enumerate() {
            let margin_top = child_style.margin.top.value as i32;
            let margin_bottom = child_style.margin.bottom.value as i32;

            // CSS margin collapsing
            let effective_top_margin = if i == 0 {
                margin_top
            } else {
                (margin_top - prev_margin_bottom).max(0)
            };
            total_margin += effective_top_margin + margin_bottom;
            prev_margin_bottom = margin_bottom;

            if let Some(height) = &child_style.height {
                match height.unit {
                    Unit::Fraction => {
                        // Scale fr values to integers (multiply by 1000 for precision)
                        total_fr += (height.value * 1000.0) as i64;
                    }
                    Unit::Cells => {
                        fixed_height_used += height.value as i32;
                    }
                    Unit::Percent => {
                        fixed_height_used += ((height.value / 100.0) * available.height as f64) as i32;
                    }
                    Unit::Auto => {
                        // Auto means size to content - use intrinsic height
                        fixed_height_used += desired_size.height as i32;
                    }
                    _ => {
                        // Other units - use default fixed height
                        fixed_height_used += DEFAULT_FIXED_HEIGHT;
                    }
                }
            } else if desired_size.height == u16::MAX {
                // No CSS height but widget wants to fill available space - treat as 1fr
                total_fr += 1000; // 1.0 * 1000
            } else {
                // No height specified - use desired size (intrinsic height)
                fixed_height_used += desired_size.height as i32;
            }
        }

        // Calculate remaining space for fr units
        let remaining_for_fr = (available.height - fixed_height_used - total_margin).max(0) as i64;

        // Second pass: place children with calculated heights using Fraction for precise remainder handling
        // NOTE: Alignment is handled centrally by apply_alignment() in mod.rs, not here.
        // This keeps VerticalLayout consistent with HorizontalLayout.
        let mut current_y = available.y;
        prev_margin_bottom = 0;
        let mut fr_remainder = Fraction::ZERO;

        for (i, (child_index, child_style, desired_size)) in children.iter().enumerate() {
            // Get horizontal margins
            let margin_left = child_style.margin.left.value as i32;
            let margin_right = child_style.margin.right.value as i32;

            // Resolve width, subtracting horizontal margins to prevent overflow
            let base_width = resolve_width_with_intrinsic(child_style, desired_size.width, available.width);
            let width = (base_width - margin_left - margin_right).max(0);

            // Resolve height - use Fraction for fr units to match Python Textual behavior
            let height = if let Some(h) = &child_style.height {
                match h.unit {
                    Unit::Fraction => {
                        if total_fr > 0 {
                            // Use Fraction arithmetic: extra pixels go to later widgets
                            let fr_value = (h.value * 1000.0) as i64;
                            let raw = Fraction::new(remaining_for_fr * fr_value, total_fr) + fr_remainder;
                            let result = raw.floor() as i32;
                            fr_remainder = raw.fract();
                            result
                        } else {
                            0
                        }
                    }
                    Unit::Auto => {
                        // Auto means size to content - use intrinsic height
                        desired_size.height as i32
                    }
                    _ => resolve_height_fixed(child_style, available.height),
                }
            } else if desired_size.height == u16::MAX && total_fr > 0 {
                // No CSS height but widget wants to fill - use fr distribution (implicit 1fr)
                let fr_value = 1000i64; // 1.0 * 1000
                let raw = Fraction::new(remaining_for_fr * fr_value, total_fr) + fr_remainder;
                let result = raw.floor() as i32;
                fr_remainder = raw.fract();
                result
            } else if desired_size.height != u16::MAX {
                // No CSS height - use intrinsic height
                desired_size.height as i32
            } else {
                resolve_height_fixed(child_style, available.height)
            };

            // Get vertical margins for positioning
            let margin_top = child_style.margin.top.value as i32;
            let margin_bottom = child_style.margin.bottom.value as i32;

            // CSS margin collapsing
            let effective_top_margin = if i == 0 {
                margin_top
            } else {
                (margin_top - prev_margin_bottom).max(0)
            };

            current_y += effective_top_margin;

            // NOTE: Horizontal alignment is handled centrally by apply_alignment() in mod.rs.
            // Here we just place children at the left edge with margin.
            let new_region = Region {
                x: available.x + margin_left,
                y: current_y,
                width,
                height,
            };

            placements.push(WidgetPlacement {
                child_index: *child_index,
                region: new_region,
            });

            current_y += height + margin_bottom;
            prev_margin_bottom = margin_bottom;
        }

        placements
    }
}
