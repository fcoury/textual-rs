//! Scrollbar rendering with Textual-compatible glyphs.
//!
//! This module provides low-level scrollbar rendering that matches
//! Textual Python's visual appearance, including smooth gradient edges
//! using Unicode block characters.

use crate::{Canvas, Region};
use crate::canvas::TextAttributes;
use tcss::types::RgbaColor;

/// Glyphs for smooth scrollbar thumb edges.
///
/// These Unicode block characters create smooth visual transitions
/// at thumb edges for sub-cell precision.
pub struct ScrollbarGlyphs;

impl ScrollbarGlyphs {
    /// Vertical scrollbar gradient (bottom-to-top fill progression).
    /// Used for top/bottom edge rendering.
    pub const VERTICAL: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    /// Horizontal scrollbar gradient (right-to-left fill progression).
    /// Used for left/right edge rendering.
    pub const HORIZONTAL: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

    /// Body glyph (space with background color).
    pub const BODY: char = ' ';
}

/// Renders scrollbar visuals (used by ScrollBar widget).
///
/// This struct provides static methods for rendering vertical and horizontal
/// scrollbars with proper glyph gradients for smooth thumb edges.
pub struct ScrollBarRender;

impl ScrollBarRender {
    /// Render a vertical scrollbar with proper glyph gradients.
    ///
    /// # Arguments
    /// * `canvas` - Target canvas
    /// * `region` - Region for the scrollbar (width = thickness)
    /// * `virtual_size` - Total content height
    /// * `window_size` - Visible viewport height
    /// * `position` - Current scroll position (0.0 to virtual_size - window_size)
    /// * `thumb_color` - Thumb/grabber color
    /// * `track_color` - Track background color
    pub fn render_vertical(
        canvas: &mut Canvas,
        region: Region,
        virtual_size: f32,
        window_size: f32,
        position: f32,
        thumb_color: RgbaColor,
        track_color: RgbaColor,
    ) {
        // Guard against zero-dimension regions to prevent underflow
        if region.height <= 0 || region.width <= 0 {
            return;
        }

        let size = region.height as f32;
        let thickness = region.width;

        // Draw track background
        for y in 0..region.height {
            for x in 0..thickness {
                canvas.put_char(
                    region.x + x,
                    region.y + y,
                    ScrollbarGlyphs::BODY,
                    None,
                    Some(track_color.clone()),
                    TextAttributes::default(),
                );
            }
        }

        // No thumb if content fits in viewport
        if window_size >= virtual_size || size == 0.0 {
            return;
        }

        let len_bars = ScrollbarGlyphs::VERTICAL.len() as f32;

        // Calculate thumb size and position (Textual's algorithm)
        let bar_ratio = virtual_size / size;
        let thumb_size = (window_size / bar_ratio).max(1.0);

        let max_position = (virtual_size - window_size).max(1.0);
        let position_ratio = (position / max_position).clamp(0.0, 1.0);
        let thumb_position = (size - thumb_size) * position_ratio;

        // Convert to sub-cell precision for gradient glyphs
        let start = (thumb_position * len_bars) as i32;
        let end = start + (thumb_size * len_bars).ceil() as i32;

        let start_index = (start as f32 / len_bars).floor() as i32;
        let start_bar = (start % len_bars as i32).max(0) as usize;
        let end_index = (end as f32 / len_bars).floor() as i32;
        let end_bar = (end % len_bars as i32).max(0) as usize;

        // Draw thumb body with gradient edges
        for y_offset in start_index..=end_index.min(region.height - 1) {
            let screen_y = region.y + y_offset;
            if screen_y < region.y || screen_y >= region.y + region.height {
                continue;
            }

            for x in 0..thickness {
                let screen_x = region.x + x;

                // Determine glyph and colors based on position
                let (glyph, fg, bg) = if y_offset == start_index && start_bar > 0 {
                    // Top edge with gradient - character partially filled from bottom
                    let bar_char = ScrollbarGlyphs::VERTICAL[start_bar];
                    (bar_char, Some(thumb_color.clone()), Some(track_color.clone()))
                } else if y_offset == end_index && end_bar > 0 && y_offset > start_index {
                    // Bottom edge with gradient - character partially filled from top
                    // Use inverse: the unfilled part is at bottom
                    let inverse_bar = (len_bars as usize) - end_bar;
                    if inverse_bar < ScrollbarGlyphs::VERTICAL.len() {
                        let bar_char = ScrollbarGlyphs::VERTICAL[inverse_bar];
                        (bar_char, Some(track_color.clone()), Some(thumb_color.clone()))
                    } else {
                        (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                    }
                } else {
                    // Solid thumb body
                    (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                };

                canvas.put_char(screen_x, screen_y, glyph, fg, bg, TextAttributes::default());
            }
        }
    }

    /// Render a horizontal scrollbar with proper glyph gradients.
    ///
    /// # Arguments
    /// * `canvas` - Target canvas
    /// * `region` - Region for the scrollbar (height = thickness)
    /// * `virtual_size` - Total content width
    /// * `window_size` - Visible viewport width
    /// * `position` - Current scroll position (0.0 to virtual_size - window_size)
    /// * `thumb_color` - Thumb/grabber color
    /// * `track_color` - Track background color
    pub fn render_horizontal(
        canvas: &mut Canvas,
        region: Region,
        virtual_size: f32,
        window_size: f32,
        position: f32,
        thumb_color: RgbaColor,
        track_color: RgbaColor,
    ) {
        // Guard against zero-dimension regions to prevent underflow
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        let size = region.width as f32;
        let thickness = region.height;

        // Draw track background
        for y in 0..thickness {
            for x in 0..region.width {
                canvas.put_char(
                    region.x + x,
                    region.y + y,
                    ScrollbarGlyphs::BODY,
                    None,
                    Some(track_color.clone()),
                    TextAttributes::default(),
                );
            }
        }

        // No thumb if content fits in viewport
        if window_size >= virtual_size || size == 0.0 {
            return;
        }

        let len_bars = ScrollbarGlyphs::HORIZONTAL.len() as f32;

        // Calculate thumb size and position
        let bar_ratio = virtual_size / size;
        let thumb_size = (window_size / bar_ratio).max(1.0);

        let max_position = (virtual_size - window_size).max(1.0);
        let position_ratio = (position / max_position).clamp(0.0, 1.0);
        let thumb_position = (size - thumb_size) * position_ratio;

        // Convert to sub-cell precision
        let start = (thumb_position * len_bars) as i32;
        let end = start + (thumb_size * len_bars).ceil() as i32;

        let start_index = (start as f32 / len_bars).floor() as i32;
        let start_bar = (start % len_bars as i32).max(0) as usize;
        let end_index = (end as f32 / len_bars).floor() as i32;
        let end_bar = (end % len_bars as i32).max(0) as usize;

        // Draw thumb body with gradient edges
        for x_offset in start_index..=end_index.min(region.width - 1) {
            let screen_x = region.x + x_offset;
            if screen_x < region.x || screen_x >= region.x + region.width {
                continue;
            }

            for y in 0..thickness {
                let screen_y = region.y + y;

                // Determine glyph and colors based on position
                let (glyph, fg, bg) = if x_offset == start_index && start_bar > 0 {
                    // Left edge with gradient - fg=track (left part), bg=thumb (right part)
                    let bar_char = ScrollbarGlyphs::HORIZONTAL[start_bar];
                    (bar_char, Some(track_color.clone()), Some(thumb_color.clone()))
                } else if x_offset == end_index && end_bar > 0 && x_offset > start_index {
                    // Right edge with gradient - fg=thumb (left part), bg=track (right part)
                    let inverse_bar = (len_bars as usize) - end_bar;
                    if inverse_bar < ScrollbarGlyphs::HORIZONTAL.len() {
                        let bar_char = ScrollbarGlyphs::HORIZONTAL[inverse_bar];
                        (bar_char, Some(thumb_color.clone()), Some(track_color.clone()))
                    } else {
                        (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                    }
                } else {
                    // Solid thumb body
                    (ScrollbarGlyphs::BODY, Some(thumb_color.clone()), Some(thumb_color.clone()))
                };

                canvas.put_char(screen_x, screen_y, glyph, fg, bg, TextAttributes::default());
            }
        }
    }

    /// Calculate the thumb bounds (start, end) in the scrollbar region.
    ///
    /// Returns (thumb_start, thumb_end) as indices within the scrollbar.
    pub fn thumb_bounds(
        track_size: i32,
        virtual_size: f32,
        window_size: f32,
        position: f32,
    ) -> (i32, i32) {
        if window_size >= virtual_size || track_size == 0 {
            return (0, 0);
        }

        let size = track_size as f32;
        let bar_ratio = virtual_size / size;
        let thumb_size = (window_size / bar_ratio).max(1.0);

        let max_position = (virtual_size - window_size).max(1.0);
        let position_ratio = (position / max_position).clamp(0.0, 1.0);
        let thumb_position = ((size - thumb_size) * position_ratio) as i32;
        let thumb_end = thumb_position + thumb_size.ceil() as i32;

        (thumb_position, thumb_end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumb_bounds_no_scroll() {
        // Content fits in viewport - no thumb
        let (start, end) = ScrollBarRender::thumb_bounds(10, 50.0, 100.0, 0.0);
        assert_eq!((start, end), (0, 0));
    }

    #[test]
    fn test_thumb_bounds_at_top() {
        // At top position
        let (start, end) = ScrollBarRender::thumb_bounds(10, 100.0, 50.0, 0.0);
        assert_eq!(start, 0);
        assert!(end > start);
    }

    #[test]
    fn test_thumb_bounds_at_bottom() {
        // At bottom position (scroll = virtual - window)
        let (start, end) = ScrollBarRender::thumb_bounds(10, 100.0, 50.0, 50.0);
        // Thumb should be at bottom of track
        assert!(start > 0);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_thumb_bounds_middle() {
        // At middle position
        let (start, end) = ScrollBarRender::thumb_bounds(10, 100.0, 50.0, 25.0);
        // Thumb should be roughly in middle
        assert!(start > 0);
        assert!(end < 10);
    }

    #[test]
    fn test_thumb_bounds_zero_track() {
        let (start, end) = ScrollBarRender::thumb_bounds(0, 100.0, 50.0, 0.0);
        assert_eq!((start, end), (0, 0));
    }
}
