//! Vertical layout algorithm - stacks children top-to-bottom.

use crate::canvas::{Region, Size};
use tcss::types::geometry::Unit;
use tcss::types::ComputedStyle;

use super::size_resolver::{apply_box_sizing_height, resolve_width_with_intrinsic};
use super::{Layout, Viewport, WidgetPlacement};

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
        viewport: Viewport,
    ) -> Vec<WidgetPlacement> {
        let mut placements = Vec::with_capacity(children.len());

        // First pass: calculate space used by non-fr items and collect fr totals
        // Uses same accumulated remainder algorithm as second pass to ensure consistency
        let mut fixed_height_used: i32 = 0;
        let mut total_fr: i64 = 0;
        let mut total_margin: i32 = 0;
        let mut prev_margin_bottom: i32 = 0;
        let mut first_pass_remainder: f64 = 0.0;

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
                        // Exact cell count - no fraction accumulation
                        let css_height = height.value as i32;
                        fixed_height_used += apply_box_sizing_height(css_height, child_style);
                    }
                    Unit::Percent | Unit::Height => {
                        // Percentage of parent height with accumulated remainder
                        let raw = (height.value / 100.0) * available.height as f64;
                        let with_remainder = raw + first_pass_remainder;
                        let css_height = with_remainder.floor() as i32;
                        first_pass_remainder = with_remainder - css_height as f64;
                        fixed_height_used += apply_box_sizing_height(css_height, child_style);
                    }
                    Unit::Width => {
                        // Percentage of parent width with accumulated remainder
                        let raw = (height.value / 100.0) * available.width as f64;
                        let with_remainder = raw + first_pass_remainder;
                        let css_height = with_remainder.floor() as i32;
                        first_pass_remainder = with_remainder - css_height as f64;
                        fixed_height_used += apply_box_sizing_height(css_height, child_style);
                    }
                    Unit::ViewWidth => {
                        // Percentage of VIEWPORT width (not container) with accumulated remainder
                        let raw = (height.value / 100.0) * viewport.width as f64;
                        let with_remainder = raw + first_pass_remainder;
                        let css_height = with_remainder.floor() as i32;
                        first_pass_remainder = with_remainder - css_height as f64;
                        fixed_height_used += apply_box_sizing_height(css_height, child_style);
                    }
                    Unit::ViewHeight => {
                        // Percentage of VIEWPORT height (not container) with accumulated remainder
                        let raw = (height.value / 100.0) * viewport.height as f64;
                        let with_remainder = raw + first_pass_remainder;
                        let css_height = with_remainder.floor() as i32;
                        first_pass_remainder = with_remainder - css_height as f64;
                        fixed_height_used += apply_box_sizing_height(css_height, child_style);
                    }
                    Unit::Auto => {
                        // Auto means size to content - use intrinsic height
                        fixed_height_used += desired_size.height as i32;
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
        //
        // IMPORTANT: Python Textual accumulates fractional remainders across ALL items (not just fr).
        // This ensures pixel-perfect distribution where extra pixels go to later items.
        let mut current_y = available.y;
        prev_margin_bottom = 0;
        let mut accumulated_remainder: f64 = 0.0;

        for (i, (child_index, child_style, desired_size)) in children.iter().enumerate() {
            // Get horizontal margins
            let margin_left = child_style.margin.left.value as i32;
            let margin_right = child_style.margin.right.value as i32;

            // Resolve width, subtracting horizontal margins to prevent overflow
            let base_width = resolve_width_with_intrinsic(child_style, desired_size.width, available.width);
            let width = (base_width - margin_left - margin_right).max(0);

            // Resolve height using accumulated fractional remainder across ALL items
            // This matches Python Textual's behavior: floor(raw + accumulated) with remainder carried forward
            let height = if let Some(h) = &child_style.height {
                match h.unit {
                    Unit::Fraction => {
                        if total_fr > 0 {
                            // fr units: distribute remaining space proportionally
                            let fr_value = (h.value * 1000.0) as i64;
                            let raw = (remaining_for_fr as f64 * fr_value as f64) / total_fr as f64;
                            let with_remainder = raw + accumulated_remainder;
                            let result = with_remainder.floor() as i32;
                            accumulated_remainder = with_remainder - result as f64;
                            result
                        } else {
                            0
                        }
                    }
                    Unit::Auto => {
                        // Auto means size to content - use intrinsic height (no fraction accumulation)
                        desired_size.height as i32
                    }
                    Unit::Cells => {
                        // Exact cell count - no fraction accumulation
                        apply_box_sizing_height(h.value as i32, child_style)
                    }
                    Unit::Percent | Unit::Height => {
                        // Percentage of parent height with accumulated remainder
                        let raw = (h.value / 100.0) * available.height as f64;
                        let with_remainder = raw + accumulated_remainder;
                        let css_height = with_remainder.floor() as i32;
                        accumulated_remainder = with_remainder - css_height as f64;
                        apply_box_sizing_height(css_height, child_style)
                    }
                    Unit::Width => {
                        // Percentage of parent width with accumulated remainder
                        let raw = (h.value / 100.0) * available.width as f64;
                        let with_remainder = raw + accumulated_remainder;
                        let css_height = with_remainder.floor() as i32;
                        accumulated_remainder = with_remainder - css_height as f64;
                        apply_box_sizing_height(css_height, child_style)
                    }
                    Unit::ViewWidth => {
                        // Percentage of VIEWPORT width (not container) with accumulated remainder
                        let raw = (h.value / 100.0) * viewport.width as f64;
                        let with_remainder = raw + accumulated_remainder;
                        let css_height = with_remainder.floor() as i32;
                        accumulated_remainder = with_remainder - css_height as f64;
                        apply_box_sizing_height(css_height, child_style)
                    }
                    Unit::ViewHeight => {
                        // Percentage of VIEWPORT height (not container) with accumulated remainder
                        let raw = (h.value / 100.0) * viewport.height as f64;
                        let with_remainder = raw + accumulated_remainder;
                        let css_height = with_remainder.floor() as i32;
                        accumulated_remainder = with_remainder - css_height as f64;
                        apply_box_sizing_height(css_height, child_style)
                    }
                }
            } else if desired_size.height == u16::MAX && total_fr > 0 {
                // No CSS height but widget wants to fill - use fr distribution (implicit 1fr)
                let fr_value = 1000i64; // 1.0 * 1000
                let raw = (remaining_for_fr as f64 * fr_value as f64) / total_fr as f64;
                let with_remainder = raw + accumulated_remainder;
                let result = with_remainder.floor() as i32;
                accumulated_remainder = with_remainder - result as f64;
                result
            } else if desired_size.height != u16::MAX {
                // No CSS height - use intrinsic height (no fraction accumulation)
                desired_size.height as i32
            } else {
                // Widget wants to fill but no fr distribution available, use available height
                available.height
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

#[cfg(test)]
mod tests {
    use super::*;
    use tcss::types::geometry::{Scalar, Unit};

    fn style_with_height(value: f64, unit: Unit) -> ComputedStyle {
        let mut style = ComputedStyle::default();
        style.height = Some(Scalar { value, unit });
        style
    }

    /// Test that height calculations match Python Textual's results.
    /// Python Textual (viewport 145x31) produces:
    /// - cells (2): 2
    /// - percent (12.5%): 3
    /// - w (5w): 8
    /// - h (12.5h): 4
    /// - vw (6.25vw): 9
    /// - vh (12.5vh): 3
    /// - auto: 1 (intrinsic)
    /// - fr1 (1fr): 1
    /// - fr2 (2fr): 0
    #[test]
    fn test_height_calculations_match_python() {
        let mut layout = VerticalLayout;
        let parent_style = ComputedStyle::default();

        // Create children matching the Python test
        let children: Vec<(usize, ComputedStyle, Size)> = vec![
            (0, style_with_height(2.0, Unit::Cells), Size::new(145, 1)),       // cells
            (1, style_with_height(12.5, Unit::Percent), Size::new(145, 1)),    // percent
            (2, style_with_height(5.0, Unit::Width), Size::new(145, 1)),       // w
            (3, style_with_height(12.5, Unit::Height), Size::new(145, 1)),     // h
            (4, style_with_height(6.25, Unit::ViewWidth), Size::new(145, 1)),  // vw
            (5, style_with_height(12.5, Unit::ViewHeight), Size::new(145, 1)), // vh
            (6, style_with_height(0.0, Unit::Auto), Size::new(145, 1)),        // auto (intrinsic=1)
            (7, style_with_height(1.0, Unit::Fraction), Size::new(145, 1)),    // fr1
            (8, style_with_height(2.0, Unit::Fraction), Size::new(145, 1)),    // fr2
        ];

        let available = Region {
            x: 0,
            y: 0,
            width: 145,
            height: 31,
        };

        let viewport = Viewport {
            width: 145,
            height: 31,
        };

        let placements = layout.arrange(&parent_style, &children, available, viewport);

        // Expected heights from Python Textual
        let expected_heights = [2, 3, 8, 4, 9, 3, 1, 1, 0];

        for (i, (placement, &expected)) in placements.iter().zip(expected_heights.iter()).enumerate() {
            assert_eq!(
                placement.region.height, expected,
                "Widget {} height mismatch: got {}, expected {}",
                i, placement.region.height, expected
            );
        }

        // Verify total height equals available height
        let total_height: i32 = placements.iter().map(|p| p.region.height).sum();
        assert_eq!(total_height, 31, "Total height should equal available height");
    }
}
