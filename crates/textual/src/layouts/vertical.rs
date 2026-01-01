//! Vertical layout algorithm - stacks children top-to-bottom.
//!
//! This layout matches Python Textual's behavior by:
//! 1. Calculating box_model heights as f64 (keeping fractional precision)
//! 2. Accumulating Y positions as f64
//! 3. Computing region.height as floor(next_y) - floor(y)
//!
//! This ensures proper remainder distribution across widgets.

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
        // Heights are kept as f64 for precise remainder distribution
        let mut fixed_height_total: f64 = 0.0;
        let mut total_fr: f64 = 0.0;
        let mut total_margin: f64 = 0.0;
        let mut prev_margin_bottom: f64 = 0.0;

        for (i, (_child_index, child_style, desired_size)) in children.iter().enumerate() {
            let margin_top = child_style.margin.top.value as f64;
            let margin_bottom = child_style.margin.bottom.value as f64;

            // CSS margin collapsing
            let effective_top_margin = if i == 0 {
                margin_top
            } else {
                (margin_top - prev_margin_bottom).max(0.0)
            };
            total_margin += effective_top_margin + margin_bottom;
            prev_margin_bottom = margin_bottom;

            if let Some(height) = &child_style.height {
                match height.unit {
                    Unit::Fraction => {
                        total_fr += height.value;
                    }
                    Unit::Cells => {
                        // Cells are exact integers, apply box-sizing
                        let css_height = apply_box_sizing_height(height.value as i32, child_style);
                        fixed_height_total += css_height as f64;
                    }
                    Unit::Percent | Unit::Height => {
                        // Python uses parent.app.size (terminal size) for these units
                        // So we use viewport, not available region
                        let raw = (height.value / 100.0) * viewport.height as f64;
                        fixed_height_total += raw;
                    }
                    Unit::Width => {
                        // Python uses parent.app.size (terminal size) for w units
                        let raw = (height.value / 100.0) * viewport.width as f64;
                        fixed_height_total += raw;
                    }
                    Unit::ViewWidth => {
                        let raw = (height.value / 100.0) * viewport.width as f64;
                        fixed_height_total += raw;
                    }
                    Unit::ViewHeight => {
                        let raw = (height.value / 100.0) * viewport.height as f64;
                        fixed_height_total += raw;
                    }
                    Unit::Auto => {
                        fixed_height_total += desired_size.height as f64;
                    }
                }
            } else if desired_size.height == u16::MAX {
                // No CSS height but widget wants to fill - treat as 1fr
                total_fr += 1.0;
            } else {
                // No height specified - use desired size
                fixed_height_total += desired_size.height as f64;
            }
        }

        // Calculate remaining space for fr units
        // Use viewport.height to match Python's behavior (uses parent.app.size for all calculations)
        let remaining_for_fr = (viewport.height as f64 - fixed_height_total - total_margin).max(0.0);

        // Calculate fraction_unit (size of 1fr)
        // When remaining_for_fr <= 0, use fraction_unit = 1.0 (matching Python Textual)
        let fraction_unit = if total_fr > 0.0 && remaining_for_fr > 0.0 {
            remaining_for_fr / total_fr
        } else {
            1.0 // Fallback: 1fr = 1 cell
        };

        // Second pass: calculate box_model heights as f64 and accumulate Y
        // Region heights are computed as floor(next_y) - floor(y)
        let mut current_y: f64 = available.y as f64;
        prev_margin_bottom = 0.0;

        for (i, (child_index, child_style, desired_size)) in children.iter().enumerate() {
            // Get horizontal margins
            let margin_left = child_style.margin.left.value as i32;
            let margin_right = child_style.margin.right.value as i32;

            // Resolve width
            let base_width =
                resolve_width_with_intrinsic(child_style, desired_size.width, available.width);
            let width = (base_width - margin_left - margin_right).max(0);

            // Apply max-width constraint
            let width = if let Some(max_w) = &child_style.max_width {
                let max_width_value = match max_w.unit {
                    Unit::Cells => max_w.value as i32,
                    Unit::Percent => ((max_w.value / 100.0) * available.width as f64) as i32,
                    Unit::Width => ((max_w.value / 100.0) * available.width as f64) as i32,
                    Unit::Height => ((max_w.value / 100.0) * available.height as f64) as i32,
                    Unit::ViewWidth => ((max_w.value / 100.0) * viewport.width as f64) as i32,
                    Unit::ViewHeight => ((max_w.value / 100.0) * viewport.height as f64) as i32,
                    _ => max_w.value as i32,
                };
                width.min(max_width_value)
            } else {
                width
            };

            // Apply min-width constraint (floor)
            let width = if let Some(min_w) = &child_style.min_width {
                let min_width_value = match min_w.unit {
                    Unit::Cells => min_w.value as i32,
                    Unit::Percent => ((min_w.value / 100.0) * available.width as f64) as i32,
                    Unit::Width => ((min_w.value / 100.0) * available.width as f64) as i32,
                    Unit::Height => ((min_w.value / 100.0) * available.height as f64) as i32,
                    Unit::ViewWidth => ((min_w.value / 100.0) * viewport.width as f64) as i32,
                    Unit::ViewHeight => ((min_w.value / 100.0) * viewport.height as f64) as i32,
                    _ => min_w.value as i32,
                };
                width.max(min_width_value)
            } else {
                width
            };

            // Resolve box_model height as f64 (keeping fractional precision)
            // This must match the calculation in the first pass
            let box_height: f64 = if let Some(h) = &child_style.height {
                match h.unit {
                    Unit::Fraction => {
                        // fr units use fraction_unit
                        h.value * fraction_unit
                    }
                    Unit::Auto => desired_size.height as f64,
                    Unit::Cells => {
                        // Cells are exact integers
                        apply_box_sizing_height(h.value as i32, child_style) as f64
                    }
                    Unit::Percent | Unit::Height => {
                        // For percentage heights, use available.height (parent container)
                        // This ensures height: 100% fills the parent, not the viewport
                        (h.value / 100.0) * available.height as f64
                    }
                    Unit::Width => {
                        // Python uses parent.app.size (terminal size) for w units
                        (h.value / 100.0) * viewport.width as f64
                    }
                    Unit::ViewWidth => {
                        (h.value / 100.0) * viewport.width as f64
                    }
                    Unit::ViewHeight => {
                        (h.value / 100.0) * viewport.height as f64
                    }
                }
            } else if desired_size.height == u16::MAX {
                // No CSS height but widget wants to fill - use fraction_unit (implicit 1fr)
                fraction_unit
            } else {
                // No CSS height - use intrinsic height
                desired_size.height as f64
            };

            // Apply max-height constraint
            let box_height = if let Some(max_h) = &child_style.max_height {
                let max_height_value = match max_h.unit {
                    Unit::Cells => max_h.value,
                    Unit::Percent => (max_h.value / 100.0) * available.height as f64,
                    Unit::Width => (max_h.value / 100.0) * available.width as f64,
                    Unit::Height => (max_h.value / 100.0) * available.height as f64,
                    Unit::ViewWidth => (max_h.value / 100.0) * viewport.width as f64,
                    Unit::ViewHeight => (max_h.value / 100.0) * viewport.height as f64,
                    _ => max_h.value,
                };
                box_height.min(max_height_value)
            } else {
                box_height
            };

            // Apply min-height constraint (floor)
            let box_height = if let Some(min_h) = &child_style.min_height {
                let min_height_value = match min_h.unit {
                    Unit::Cells => min_h.value,
                    Unit::Percent => (min_h.value / 100.0) * available.height as f64,
                    Unit::Width => (min_h.value / 100.0) * available.width as f64,
                    Unit::Height => (min_h.value / 100.0) * available.height as f64,
                    Unit::ViewWidth => (min_h.value / 100.0) * viewport.width as f64,
                    Unit::ViewHeight => (min_h.value / 100.0) * viewport.height as f64,
                    _ => min_h.value,
                };
                box_height.max(min_height_value)
            } else {
                box_height
            };

            // Get vertical margins
            let margin_top = child_style.margin.top.value as f64;
            let margin_bottom = child_style.margin.bottom.value as f64;

            // For percentage/fill heights, reduce by margins to fit within container
            // For explicit heights (cells), margins are outside the content box
            let should_reduce_by_margins = match &child_style.height {
                Some(h) => matches!(h.unit, Unit::Percent | Unit::Height | Unit::Fraction),
                None => desired_size.height == u16::MAX, // fill case
            };
            let box_height = if should_reduce_by_margins {
                (box_height - margin_top - margin_bottom).max(0.0)
            } else {
                box_height
            };

            // CSS margin collapsing
            let effective_top_margin = if i == 0 {
                margin_top
            } else {
                (margin_top - prev_margin_bottom).max(0.0)
            };

            current_y += effective_top_margin;

            // Calculate next_y and region height using floor arithmetic
            // This matches Python: region.height = floor(next_y) - floor(y)
            let next_y = current_y + box_height;
            let region_y = current_y.floor() as i32;
            let region_height = (next_y.floor() as i32 - region_y).max(0);

            let new_region = Region {
                x: available.x + margin_left,
                y: region_y,
                width,
                height: region_height,
            };

            placements.push(WidgetPlacement {
                child_index: *child_index,
                region: new_region,
            });

            current_y = next_y + margin_bottom;
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
    ///
    /// Note: When remaining space for fr is 0 or negative, Python uses fraction_unit=1.
    /// But when there IS remaining space, fr gets distributed proportionally.
    /// At 145x31 with fixed heights summing to ~30, there's ~1 cell left for 3fr total.
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

        // Print detailed height breakdown for debugging
        println!("\n=== Height breakdown at 145x31 ===");
        println!("vw raw: 6.25% * 145 = {}", 6.25 / 100.0 * 145.0);
        for (i, p) in placements.iter().enumerate() {
            println!("Widget {}: height = {}", i, p.region.height);
        }

        // Expected heights from Python Textual
        // Note: Fixed heights = 2+3+8+4+9+3+1 = 30, remaining = 1 for 3fr total
        // fr1 gets floor(1/3) = 0, fr2 gets floor(2/3 + 0.333) = 1
        let expected_heights = [2, 3, 8, 4, 9, 3, 1, 0, 1];

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

    /// Test height calculations at 80x31 viewport (typical tmux pane scenario)
    /// Python Textual produces:
    /// - cells (2): 2
    /// - percent (12.5%): 3
    /// - w (5w): 4
    /// - h (12.5h): 4
    /// - vw (6.25vw): 5
    /// - vh (12.5vh): 4
    /// - auto: 1 (intrinsic)
    /// - fr1 (1fr): 3
    /// - fr2 (2fr): 5
    /// Total: 31
    #[test]
    fn test_height_calculations_80x31() {
        let mut layout = VerticalLayout;
        let parent_style = ComputedStyle::default();

        let children: Vec<(usize, ComputedStyle, Size)> = vec![
            (0, style_with_height(2.0, Unit::Cells), Size::new(80, 1)),
            (1, style_with_height(12.5, Unit::Percent), Size::new(80, 1)),
            (2, style_with_height(5.0, Unit::Width), Size::new(80, 1)),
            (3, style_with_height(12.5, Unit::Height), Size::new(80, 1)),
            (4, style_with_height(6.25, Unit::ViewWidth), Size::new(80, 1)),
            (5, style_with_height(12.5, Unit::ViewHeight), Size::new(80, 1)),
            (6, style_with_height(0.0, Unit::Auto), Size::new(80, 1)),
            (7, style_with_height(1.0, Unit::Fraction), Size::new(80, 1)),
            (8, style_with_height(2.0, Unit::Fraction), Size::new(80, 1)),
        ];

        let available = Region {
            x: 0,
            y: 0,
            width: 80,
            height: 31,
        };

        let viewport = Viewport {
            width: 80,
            height: 31,
        };

        let placements = layout.arrange(&parent_style, &children, available, viewport);

        // Print detailed height breakdown for debugging
        println!("\n=== Height breakdown at 80x31 ===");
        for (i, p) in placements.iter().enumerate() {
            println!("Widget {}: height = {}", i, p.region.height);
        }

        // Expected heights from Python Textual at 80x31
        let expected_heights = [2, 3, 4, 4, 5, 4, 1, 3, 5];

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

    /// Test height calculations at 90x17 viewport (scrollable container scenario)
    /// Python Textual produces:
    /// - cells (2): 2
    /// - percent (12.5%): 2
    /// - w (5w): 4
    /// - h (12.5h): 2
    /// - vw (6.25vw): 6
    /// - vh (12.5vh): 2
    /// - auto: 1 (intrinsic)
    /// - fr1 (1fr): 1 (fraction_unit=1 when no remaining space)
    /// - fr2 (2fr): 2 (fraction_unit=1 when no remaining space)
    /// Total: 22 (exceeds viewport, hence scrollable)
    #[test]
    fn test_height_calculations_90x17() {
        let mut layout = VerticalLayout;
        let parent_style = ComputedStyle::default();

        let children: Vec<(usize, ComputedStyle, Size)> = vec![
            (0, style_with_height(2.0, Unit::Cells), Size::new(90, 1)),
            (1, style_with_height(12.5, Unit::Percent), Size::new(90, 1)),
            (2, style_with_height(5.0, Unit::Width), Size::new(90, 1)),
            (3, style_with_height(12.5, Unit::Height), Size::new(90, 1)),
            (4, style_with_height(6.25, Unit::ViewWidth), Size::new(90, 1)),
            (5, style_with_height(12.5, Unit::ViewHeight), Size::new(90, 1)),
            (6, style_with_height(0.0, Unit::Auto), Size::new(90, 1)),
            (7, style_with_height(1.0, Unit::Fraction), Size::new(90, 1)),
            (8, style_with_height(2.0, Unit::Fraction), Size::new(90, 1)),
        ];

        let available = Region {
            x: 0,
            y: 0,
            width: 90,
            height: 17,
        };

        let viewport = Viewport {
            width: 90,
            height: 17,
        };

        let placements = layout.arrange(&parent_style, &children, available, viewport);

        // Expected heights from Python Textual at 90x17
        let expected_heights = [2, 2, 4, 2, 6, 2, 1, 1, 2];

        for (i, (placement, &expected)) in placements.iter().zip(expected_heights.iter()).enumerate() {
            assert_eq!(
                placement.region.height, expected,
                "Widget {} height mismatch: got {}, expected {}",
                i, placement.region.height, expected
            );
        }

        // Total should be 22 (content that exceeds viewport)
        let total_height: i32 = placements.iter().map(|p| p.region.height).sum();
        assert_eq!(total_height, 22, "Total height should be 22");
    }
}
