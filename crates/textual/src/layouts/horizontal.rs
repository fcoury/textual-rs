//! Horizontal layout algorithm - stacks children left-to-right.

use crate::canvas::Region;
use crate::fraction::Fraction;
use tcss::types::ComputedStyle;
use tcss::types::geometry::Unit;

use super::size_resolver::{apply_box_sizing_height, apply_box_sizing_width, horizontal_chrome};
use super::{Layout, LayoutChild, Viewport, WidgetPlacement};

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
        children: &[LayoutChild],
        available: Region,
        viewport: Viewport,
    ) -> Vec<WidgetPlacement> {
        let mut placements = Vec::with_capacity(children.len());
        let percent_to_fraction = |value: f64, basis: i32| {
            let scaled = (value * 1000.0).round() as i64;
            Fraction::new(basis as i64 * scaled, 100 * 1000)
        };
        let apply_box_sizing_width_fraction = |width: Fraction, style: &ComputedStyle| match style
            .box_sizing
        {
            tcss::types::BoxSizing::BorderBox => width,
            tcss::types::BoxSizing::ContentBox => width + Fraction::from(horizontal_chrome(style)),
        };

        // First pass: resolve fixed widths and collect fr values.
        let mut fixed_widths: Vec<Option<i32>> = vec![None; children.len()];
        let mut fr_values: Vec<Option<i64>> = vec![None; children.len()];
        let mut total_fr: i64 = 0;
        let mut total_margin: i32 = 0;
        let mut prev_margin_right: i32 = 0;
        let mut fixed_remainder = Fraction::ZERO;
        let mut fixed_width_sum: i32 = 0;

        for (i, child) in children.iter().enumerate() {
            let child_style = &child.style;
            let desired_size = child.desired_size;
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

            let resolved_fixed = if let Some(width) = &child_style.width {
                match width.unit {
                    Unit::Fraction => {
                        let fr_value = (width.value * 1000.0) as i64;
                        fr_values[i] = Some(fr_value);
                        total_fr += fr_value;
                        None
                    }
                    Unit::Cells => {
                        let css_width = apply_box_sizing_width(width.value as i32, child_style);
                        let raw = Fraction::from(css_width) + fixed_remainder;
                        let width = raw.floor() as i32;
                        fixed_remainder = raw.fract();
                        Some(width)
                    }
                    Unit::Percent => {
                        let css_width = percent_to_fraction(width.value, available.width);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + fixed_remainder;
                        let width = raw.floor() as i32;
                        fixed_remainder = raw.fract();
                        Some(width)
                    }
                    Unit::Width => {
                        let css_width = percent_to_fraction(width.value, available.width);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + fixed_remainder;
                        let width = raw.floor() as i32;
                        fixed_remainder = raw.fract();
                        Some(width)
                    }
                    Unit::Height => {
                        let css_width = percent_to_fraction(width.value, available.height);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + fixed_remainder;
                        let width = raw.floor() as i32;
                        fixed_remainder = raw.fract();
                        Some(width)
                    }
                    Unit::ViewWidth => {
                        let css_width = percent_to_fraction(width.value, viewport.width);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + fixed_remainder;
                        let width = raw.floor() as i32;
                        fixed_remainder = raw.fract();
                        Some(width)
                    }
                    Unit::ViewHeight => {
                        let css_width = percent_to_fraction(width.value, viewport.height);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + fixed_remainder;
                        let width = raw.floor() as i32;
                        fixed_remainder = raw.fract();
                        Some(width)
                    }
                    _ => {
                        // Auto or other - check if widget wants to fill (u16::MAX signals "fill available")
                        if desired_size.width == u16::MAX {
                            let fr_value = 1000;
                            fr_values[i] = Some(fr_value);
                            total_fr += fr_value;
                            None
                        } else {
                            let raw = Fraction::from(desired_size.width as i32) + fixed_remainder;
                            let width = raw.floor() as i32;
                            fixed_remainder = raw.fract();
                            Some(width)
                        }
                    }
                }
            } else {
                // No width specified - check if widget wants to fill (u16::MAX signals "fill available")
                if desired_size.width == u16::MAX {
                    let fr_value = 1000;
                    fr_values[i] = Some(fr_value);
                    total_fr += fr_value;
                    None
                } else {
                    let raw = Fraction::from(desired_size.width as i32) + fixed_remainder;
                    let width = raw.floor() as i32;
                    fixed_remainder = raw.fract();
                    Some(width)
                }
            };

            if let Some(width) = resolved_fixed {
                fixed_widths[i] = Some(width);
                fixed_width_sum += width;
            }
        }

        let remaining_for_fr = (available.width - fixed_width_sum - total_margin).max(0);

        let mut fr_widths: Vec<i32> = vec![0; children.len()];
        if total_fr > 0 && remaining_for_fr > 0 {
            let mut used = 0;
            let mut fr_indices = Vec::new();
            for (i, fr_value) in fr_values.iter().enumerate() {
                if let Some(value) = fr_value {
                    let raw = Fraction::from(remaining_for_fr).mul_ratio(*value, total_fr);
                    let width = raw.floor() as i32;
                    fr_widths[i] = width;
                    used += width;
                    fr_indices.push(i);
                }
            }

            let mut leftover = remaining_for_fr - used;
            for idx in fr_indices {
                if leftover <= 0 {
                    break;
                }
                fr_widths[idx] += 1;
                leftover -= 1;
            }
        }

        // Second pass: place children with resolved widths
        let mut current_x = available.x;
        prev_margin_right = 0;

        for (i, child) in children.iter().enumerate() {
            let child_index = child.index;
            let child_style = &child.style;
            // Resolve width from precomputed fixed/fr allocations.
            let width = fixed_widths[i].unwrap_or(fr_widths[i]);

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

            // Resolve height - horizontal layout children fill available height by default
            // Apply box-sizing only for explicit CSS heights, not auto/intrinsic
            let height = if let Some(h) = &child_style.height {
                match h.unit {
                    Unit::Auto => {
                        // Auto means intrinsic height for the resolved width
                        let width_u16 = width.clamp(0, u16::MAX as i32) as u16;
                        child.node.intrinsic_height_for_width(width_u16) as i32
                    }
                    Unit::Fraction => {
                        // fr fills available - no box-sizing adjustment
                        available.height
                    }
                    Unit::Cells => apply_box_sizing_height(h.value as i32, child_style),
                    Unit::Percent => {
                        let css_height = ((h.value / 100.0) * available.height as f64) as i32;
                        apply_box_sizing_height(css_height, child_style)
                    }
                    _ => {
                        // Other units - use value as-is with box-sizing
                        apply_box_sizing_height(h.value as i32, child_style)
                    }
                }
            } else {
                // No height specified - fill available (horizontal layout default)
                available.height
            };

            // Apply max-height constraint
            let height = if let Some(max_h) = &child_style.max_height {
                let max_height_value = match max_h.unit {
                    Unit::Cells => max_h.value as i32,
                    Unit::Percent => ((max_h.value / 100.0) * available.height as f64) as i32,
                    Unit::Width => ((max_h.value / 100.0) * available.width as f64) as i32,
                    Unit::Height => ((max_h.value / 100.0) * available.height as f64) as i32,
                    Unit::ViewWidth => ((max_h.value / 100.0) * viewport.width as f64) as i32,
                    Unit::ViewHeight => ((max_h.value / 100.0) * viewport.height as f64) as i32,
                    _ => max_h.value as i32,
                };
                height.min(max_height_value)
            } else {
                height
            };

            // Apply min-height constraint (floor)
            let height = if let Some(min_h) = &child_style.min_height {
                let min_height_value = match min_h.unit {
                    Unit::Cells => min_h.value as i32,
                    Unit::Percent => ((min_h.value / 100.0) * available.height as f64) as i32,
                    Unit::Width => ((min_h.value / 100.0) * available.width as f64) as i32,
                    Unit::Height => ((min_h.value / 100.0) * available.height as f64) as i32,
                    Unit::ViewWidth => ((min_h.value / 100.0) * viewport.width as f64) as i32,
                    Unit::ViewHeight => ((min_h.value / 100.0) * viewport.height as f64) as i32,
                    _ => min_h.value as i32,
                };
                height.max(min_height_value)
            } else {
                height
            };

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

            // For height: auto (intrinsic), use the height as-is since it's the content height.
            // For other heights (fill/fr), reduce by vertical margins to fit within container.
            let is_auto_height = child_style
                .height
                .as_ref()
                .map(|h| h.unit == Unit::Auto)
                .unwrap_or(false);
            let adjusted_height = if is_auto_height {
                height // Use intrinsic height directly
            } else {
                // Fill available: reduce by margins to prevent overflow
                (height - margin_top - margin_bottom).max(0)
            };

            placements.push(WidgetPlacement {
                child_index,
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
