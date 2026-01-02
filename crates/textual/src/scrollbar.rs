//! Scrollbar rendering with Textual-compatible glyphs.
//!
//! This module provides low-level scrollbar rendering that matches
//! Textual Python's visual appearance, including smooth gradient edges
//! using Unicode block characters.

use crate::canvas::TextAttributes;
use crate::{Canvas, Region};
use tcss::types::RgbaColor;

/// Glyphs for smooth scrollbar thumb edges.
///
/// These Unicode block characters create smooth visual transitions
/// at thumb edges for sub-cell precision.
pub struct ScrollbarGlyphs;

impl ScrollbarGlyphs {
    /// Vertical scrollbar gradient (bottom-to-top fill progression).
    /// Used for top/bottom edge rendering.
    pub const VERTICAL: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', ' '];

    /// Horizontal scrollbar gradient (right-to-left fill progression).
    /// Used for left/right edge rendering.
    pub const HORIZONTAL: [char; 8] = ['▉', '▊', '▋', '▌', '▍', '▎', '▏', ' '];

    /// Body glyph (space with background color).
    pub const BODY: char = ' ';
}

/// Renders scrollbar visuals (used by ScrollBar widget).
///
/// This struct provides static methods for rendering vertical and horizontal
/// scrollbars with proper glyph gradients for smooth thumb edges.
pub struct ScrollBarRender;

impl ScrollBarRender {
    /// Compose track and thumb colors to match Textual's blending rules.
    ///
    /// Returns (thumb_color, track_color, draw_thumb).
    pub fn compose_colors(
        thumb: RgbaColor,
        track: RgbaColor,
        base_background: Option<RgbaColor>,
    ) -> (RgbaColor, RgbaColor, bool) {
        let mut track = track;
        if track.a < 1.0 {
            if let Some(base) = base_background {
                track = track.blend_over(&base);
            }
        }

        let draw_thumb = thumb.a > 0.0;
        let thumb = if draw_thumb {
            thumb.blend_over(&track)
        } else {
            thumb
        };

        (thumb, track, draw_thumb)
    }

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
        draw_thumb: bool,
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

        let virtual_size = virtual_size.ceil();
        let window_size = if window_size < virtual_size {
            window_size.ceil()
        } else {
            0.0
        };

        // No thumb if content fits in viewport or thumb hidden
        if !draw_thumb || window_size == 0.0 || size == 0.0 || virtual_size == size {
            return;
        }

        let len_bars = ScrollbarGlyphs::VERTICAL.len() as i32;

        // Calculate thumb size and position (Textual's algorithm)
        let bar_ratio = virtual_size / size;
        let thumb_size = (window_size / bar_ratio).max(1.0);

        let max_position = (virtual_size - window_size).max(1.0);
        let position_ratio = (position / max_position).clamp(0.0, 1.0);
        let thumb_position = (size - thumb_size) * position_ratio;

        // Convert to sub-cell precision for gradient glyphs
        let start = (thumb_position * len_bars as f32) as i32;
        let end = start + (thumb_size * len_bars as f32).ceil() as i32;

        let start = start.max(0);
        let end = end.max(0);
        let start_index = start / len_bars;
        let start_bar = (start % len_bars) as usize;
        let end_index = end / len_bars;
        let end_bar = (end % len_bars) as usize;

        let body_start = start_index.max(0).min(region.height);
        let body_end = end_index.max(0).min(region.height);

        // Draw thumb body (full cells)
        for y_offset in body_start..body_end {
            let screen_y = region.y + y_offset;
            for x in 0..thickness {
                let screen_x = region.x + x;
                canvas.put_char(
                    screen_x,
                    screen_y,
                    ScrollbarGlyphs::BODY,
                    Some(thumb_color.clone()),
                    Some(thumb_color.clone()),
                    TextAttributes::default(),
                );
            }
        }

        // Apply gradient glyphs to head and tail (Textual's algorithm)
        if start_index >= 0 && start_index < region.height {
            let bar_char = ScrollbarGlyphs::VERTICAL[len_bars as usize - 1 - start_bar];
            if bar_char != ScrollbarGlyphs::BODY {
                let screen_y = region.y + start_index;
                for x in 0..thickness {
                    let screen_x = region.x + x;
                    canvas.put_char(
                        screen_x,
                        screen_y,
                        bar_char,
                        Some(thumb_color.clone()),
                        Some(track_color.clone()),
                        TextAttributes::default(),
                    );
                }
            }
        }
        if end_index >= 0 && end_index < region.height {
            let bar_char = ScrollbarGlyphs::VERTICAL[len_bars as usize - 1 - end_bar];
            if bar_char != ScrollbarGlyphs::BODY {
                let screen_y = region.y + end_index;
                for x in 0..thickness {
                    let screen_x = region.x + x;
                    canvas.put_char(
                        screen_x,
                        screen_y,
                        bar_char,
                        Some(track_color.clone()),
                        Some(thumb_color.clone()),
                        TextAttributes::default(),
                    );
                }
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
        draw_thumb: bool,
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

        let virtual_size = virtual_size.ceil();
        let window_size = if window_size < virtual_size {
            window_size.ceil()
        } else {
            0.0
        };

        // No thumb if content fits in viewport or thumb hidden
        if !draw_thumb || window_size == 0.0 || size == 0.0 || virtual_size == size {
            return;
        }

        let len_bars = ScrollbarGlyphs::HORIZONTAL.len() as i32;

        // Calculate thumb size and position
        let bar_ratio = virtual_size / size;
        let thumb_size = (window_size / bar_ratio).max(1.0);

        let max_position = (virtual_size - window_size).max(1.0);
        let position_ratio = (position / max_position).clamp(0.0, 1.0);
        let thumb_position = (size - thumb_size) * position_ratio;

        // Convert to sub-cell precision
        let start = (thumb_position * len_bars as f32) as i32;
        let end = start + (thumb_size * len_bars as f32).ceil() as i32;

        let start = start.max(0);
        let end = end.max(0);
        let start_index = start / len_bars;
        let start_bar = (start % len_bars) as usize;
        let end_index = end / len_bars;
        let end_bar = (end % len_bars) as usize;

        let body_start = start_index.max(0).min(region.width);
        let body_end = end_index.max(0).min(region.width);

        // Draw thumb body (full cells)
        for x_offset in body_start..body_end {
            let screen_x = region.x + x_offset;
            for y in 0..thickness {
                let screen_y = region.y + y;
                canvas.put_char(
                    screen_x,
                    screen_y,
                    ScrollbarGlyphs::BODY,
                    Some(thumb_color.clone()),
                    Some(thumb_color.clone()),
                    TextAttributes::default(),
                );
            }
        }

        // Apply gradient glyphs to head and tail (Textual's algorithm)
        if start_index >= 0 && start_index < region.width {
            let bar_char = ScrollbarGlyphs::HORIZONTAL[len_bars as usize - 1 - start_bar];
            if bar_char != ScrollbarGlyphs::BODY {
                let screen_x = region.x + start_index;
                for y in 0..thickness {
                    let screen_y = region.y + y;
                    canvas.put_char(
                        screen_x,
                        screen_y,
                        bar_char,
                        Some(track_color.clone()),
                        Some(thumb_color.clone()),
                        TextAttributes::default(),
                    );
                }
            }
        }
        if end_index >= 0 && end_index < region.width {
            let bar_char = ScrollbarGlyphs::HORIZONTAL[len_bars as usize - 1 - end_bar];
            if bar_char != ScrollbarGlyphs::BODY {
                let screen_x = region.x + end_index;
                for y in 0..thickness {
                    let screen_y = region.y + y;
                    canvas.put_char(
                        screen_x,
                        screen_y,
                        bar_char,
                        Some(thumb_color.clone()),
                        Some(track_color.clone()),
                        TextAttributes::default(),
                    );
                }
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
        let size = track_size as f32;
        let virtual_size = virtual_size.ceil();
        let window_size = if window_size < virtual_size {
            window_size.ceil()
        } else {
            0.0
        };

        if window_size == 0.0 || track_size == 0 || virtual_size == size {
            return (0, 0);
        }

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
