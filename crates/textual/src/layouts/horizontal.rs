//! Horizontal layout algorithm - stacks children left-to-right.

use crate::canvas::{Region, Size};
use crate::fraction::Fraction;
use tcss::types::geometry::Unit;
use tcss::types::ComputedStyle;

use super::size_resolver::{apply_box_sizing_height, apply_box_sizing_width};
use super::{Layout, Viewport, WidgetPlacement};

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
        _viewport: Viewport,
    ) -> Vec<WidgetPlacement> {
        let mut placements = Vec::with_capacity(children.len());

        // First pass: calculate space used by non-fr items and collect fr totals
        let mut fixed_width_used: i32 = 0;
        let mut total_fr: i64 = 0;
        let mut total_margin: i32 = 0;
        let mut prev_margin_right: i32 = 0;

        for (i, (_child_index, child_style, desired_size)) in children.iter().enumerate() {
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
                        fixed_width_used += apply_box_sizing_width(css_width, child_style);
                    }
                    Unit::Percent => {
                        let css_width = ((width.value / 100.0) * available.width as f64) as i32;
                        fixed_width_used += apply_box_sizing_width(css_width, child_style);
                    }
                    _ => {
                        // Auto or other - check if widget wants to fill (u16::MAX signals "fill available")
                        if desired_size.width == u16::MAX {
                            // Treat as 1fr (1000 in our scaled units)
                            total_fr += 1000;
                        } else {
                            // Use intrinsic width (already includes chrome)
                            fixed_width_used += desired_size.width as i32;
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
                    fixed_width_used += desired_size.width as i32;
                }
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
            // Apply box-sizing: content-box adds chrome, border-box uses CSS value as-is
            let width = if let Some(w) = &child_style.width {
                match w.unit {
                    Unit::Fraction => {
                        if total_fr > 0 {
                            // Use Fraction arithmetic: extra pixels go to later widgets
                            // fr units fill available space, so no box-sizing adjustment needed
                            let fr_value = (w.value * 1000.0) as i64;
                            let raw = Fraction::new(remaining_for_fr * fr_value, total_fr) + fr_remainder;
                            let result = raw.floor() as i32;
                            fr_remainder = raw.fract();
                            result
                        } else {
                            0
                        }
                    }
                    Unit::Cells => {
                        let css_width = w.value as i32;
                        apply_box_sizing_width(css_width, child_style)
                    }
                    Unit::Percent => {
                        let css_width = ((w.value / 100.0) * available.width as f64) as i32;
                        apply_box_sizing_width(css_width, child_style)
                    }
                    _ => {
                        // Check if widget wants to fill (u16::MAX signals "fill available")
                        if desired_size.width == u16::MAX && total_fr > 0 {
                            // Treat as 1fr
                            let raw = Fraction::new(remaining_for_fr * 1000, total_fr) + fr_remainder;
                            let result = raw.floor() as i32;
                            fr_remainder = raw.fract();
                            result
                        } else {
                            desired_size.width as i32 // Use intrinsic width
                        }
                    }
                }
            } else {
                // No width specified - check if widget wants to fill
                if desired_size.width == u16::MAX && total_fr > 0 {
                    // Treat as 1fr
                    let raw = Fraction::new(remaining_for_fr * 1000, total_fr) + fr_remainder;
                    let result = raw.floor() as i32;
                    fr_remainder = raw.fract();
                    result
                } else {
                    // Use intrinsic width (already includes chrome)
                    desired_size.width as i32
                }
            };

            // Resolve height - horizontal layout children fill available height by default
            // Apply box-sizing only for explicit CSS heights, not auto/intrinsic
            let height = if let Some(h) = &child_style.height {
                match h.unit {
                    Unit::Auto => {
                        // Auto means intrinsic - already includes chrome
                        desired_size.height as i32
                    }
                    Unit::Fraction => {
                        // fr fills available - no box-sizing adjustment
                        available.height
                    }
                    Unit::Cells => {
                        apply_box_sizing_height(h.value as i32, child_style)
                    }
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
                    _ => max_h.value as i32,
                };
                height.min(max_height_value)
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
            let is_auto_height = child_style.height.as_ref().map(|h| h.unit == Unit::Auto).unwrap_or(false);
            let adjusted_height = if is_auto_height {
                height // Use intrinsic height directly
            } else {
                // Fill available: reduce by margins to prevent overflow
                (height - margin_top - margin_bottom).max(0)
            };

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
