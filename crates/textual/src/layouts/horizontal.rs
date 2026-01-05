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

        // First pass: calculate space used by non-fr items and collect fr totals
        let mut fixed_width_used: Fraction = Fraction::ZERO;
        let mut total_fr: i64 = 0;
        let mut total_margin: i32 = 0;
        let mut prev_margin_right: i32 = 0;

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

            if let Some(width) = &child_style.width {
                match width.unit {
                    Unit::Fraction => {
                        // Scale fr values to integers (multiply by 1000 for precision)
                        total_fr += (width.value * 1000.0) as i64;
                    }
                    Unit::Cells => {
                        // Apply box-sizing: content-box adds chrome, border-box uses as-is
                        let css_width = width.value as i32;
                        fixed_width_used = fixed_width_used
                            + Fraction::from(apply_box_sizing_width(css_width, child_style));
                    }
                    Unit::Percent => {
                        let css_width = percent_to_fraction(width.value, available.width);
                        fixed_width_used = fixed_width_used
                            + apply_box_sizing_width_fraction(css_width, child_style);
                    }
                    Unit::Width => {
                        let css_width = percent_to_fraction(width.value, available.width);
                        fixed_width_used = fixed_width_used
                            + apply_box_sizing_width_fraction(css_width, child_style);
                    }
                    Unit::Height => {
                        let css_width = percent_to_fraction(width.value, available.height);
                        fixed_width_used = fixed_width_used
                            + apply_box_sizing_width_fraction(css_width, child_style);
                    }
                    Unit::ViewWidth => {
                        let css_width = percent_to_fraction(width.value, viewport.width);
                        fixed_width_used = fixed_width_used
                            + apply_box_sizing_width_fraction(css_width, child_style);
                    }
                    Unit::ViewHeight => {
                        let css_width = percent_to_fraction(width.value, viewport.height);
                        fixed_width_used = fixed_width_used
                            + apply_box_sizing_width_fraction(css_width, child_style);
                    }
                    _ => {
                        // Auto or other - check if widget wants to fill (u16::MAX signals "fill available")
                        if desired_size.width == u16::MAX {
                            // Treat as 1fr (1000 in our scaled units)
                            total_fr += 1000;
                        } else {
                            // Use intrinsic width (already includes chrome)
                            fixed_width_used =
                                fixed_width_used + Fraction::from(desired_size.width as i32);
                        }
                    }
                }
            } else {
                // No width specified - check if widget wants to fill (u16::MAX signals "fill available")
                if desired_size.width == u16::MAX {
                    // Treat as 1fr (1000 in our scaled units)
                    total_fr += 1000;
                } else {
                    // Use intrinsic width (already includes chrome)
                    fixed_width_used = fixed_width_used + Fraction::from(desired_size.width as i32);
                }
            }
        }

        // Calculate remaining space for fr units
        let remaining_for_fr = {
            let remaining =
                Fraction::from(available.width) - fixed_width_used - Fraction::from(total_margin);
            if remaining.floor() < 0 {
                Fraction::ZERO
            } else {
                remaining
            }
        };

        // Second pass: place children with calculated widths using Fraction for precise remainder handling
        let mut current_x = available.x;
        prev_margin_right = 0;
        let mut width_remainder = Fraction::ZERO;

        for (i, child) in children.iter().enumerate() {
            let child_index = child.index;
            let child_style = &child.style;
            let desired_size = child.desired_size;
            // Resolve width - use Fraction for fr units to match Python Textual behavior
            // Apply box-sizing: content-box adds chrome, border-box uses CSS value as-is
            let width = if let Some(w) = &child_style.width {
                match w.unit {
                    Unit::Fraction => {
                        if total_fr > 0 {
                            // Use Fraction arithmetic: extra pixels go to later widgets
                            // fr units fill available space, so no box-sizing adjustment needed
                            let fr_value = (w.value * 1000.0) as i64;
                            let raw =
                                remaining_for_fr.mul_ratio(fr_value, total_fr) + width_remainder;
                            let width = raw.floor() as i32;
                            width_remainder = raw.fract();
                            width
                        } else {
                            0
                        }
                    }
                    Unit::Cells => {
                        let css_width = apply_box_sizing_width(w.value as i32, child_style);
                        let raw = Fraction::from(css_width) + width_remainder;
                        let width = raw.floor() as i32;
                        width_remainder = raw.fract();
                        width
                    }
                    Unit::Percent => {
                        let css_width = percent_to_fraction(w.value, available.width);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + width_remainder;
                        let width = raw.floor() as i32;
                        width_remainder = raw.fract();
                        width
                    }
                    Unit::Width => {
                        let css_width = percent_to_fraction(w.value, available.width);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + width_remainder;
                        let width = raw.floor() as i32;
                        width_remainder = raw.fract();
                        width
                    }
                    Unit::Height => {
                        let css_width = percent_to_fraction(w.value, available.height);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + width_remainder;
                        let width = raw.floor() as i32;
                        width_remainder = raw.fract();
                        width
                    }
                    Unit::ViewWidth => {
                        let css_width = percent_to_fraction(w.value, viewport.width);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + width_remainder;
                        let width = raw.floor() as i32;
                        width_remainder = raw.fract();
                        width
                    }
                    Unit::ViewHeight => {
                        let css_width = percent_to_fraction(w.value, viewport.height);
                        let css_width = apply_box_sizing_width_fraction(css_width, child_style);
                        let raw = css_width + width_remainder;
                        let width = raw.floor() as i32;
                        width_remainder = raw.fract();
                        width
                    }
                    _ => {
                        // Check if widget wants to fill (u16::MAX signals "fill available")
                        if desired_size.width == u16::MAX && total_fr > 0 {
                            // Treat as 1fr
                            let raw = remaining_for_fr.mul_ratio(1000, total_fr) + width_remainder;
                            let width = raw.floor() as i32;
                            width_remainder = raw.fract();
                            width
                        } else {
                            let raw = Fraction::from(desired_size.width as i32) + width_remainder;
                            let width = raw.floor() as i32;
                            width_remainder = raw.fract();
                            width
                        }
                    }
                }
            } else {
                // No width specified - check if widget wants to fill
                if desired_size.width == u16::MAX && total_fr > 0 {
                    // Treat as 1fr
                    let raw = remaining_for_fr.mul_ratio(1000, total_fr) + width_remainder;
                    let width = raw.floor() as i32;
                    width_remainder = raw.fract();
                    width
                } else {
                    // Use intrinsic width (already includes chrome)
                    let raw = Fraction::from(desired_size.width as i32) + width_remainder;
                    let width = raw.floor() as i32;
                    width_remainder = raw.fract();
                    width
                }
            };

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
