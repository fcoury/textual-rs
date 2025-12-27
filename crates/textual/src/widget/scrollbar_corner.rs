//! ScrollBarCorner widget for filling the gap between scrollbars.
//!
//! When both horizontal and vertical scrollbars are visible, there's a
//! corner gap that needs to be filled. This widget handles that.

use crate::{Canvas, Region, Size, Widget};
use tcss::types::{RgbaColor, ScrollbarStyle};
use tcss::ComputedStyle;

/// Fills the corner gap when both scrollbars are visible.
///
/// This widget renders a simple colored rectangle in the corner
/// where horizontal and vertical scrollbars meet.
pub struct ScrollBarCorner {
    /// Corner color (from CSS or fallback)
    color: Option<RgbaColor>,
    /// Width of the corner (matches vertical scrollbar width)
    width: u16,
    /// Height of the corner (matches horizontal scrollbar height)
    height: u16,
    /// Computed style
    style: ComputedStyle,
    /// Dirty flag
    dirty: bool,
}

impl ScrollBarCorner {
    /// Create a new scrollbar corner.
    ///
    /// # Arguments
    /// * `width` - Width (typically matches vertical scrollbar thickness)
    /// * `height` - Height (typically matches horizontal scrollbar thickness)
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            color: None,
            width,
            height,
            style: ComputedStyle::default(),
            dirty: true,
        }
    }

    /// Set the corner color directly.
    pub fn with_color(mut self, color: RgbaColor) -> Self {
        self.color = Some(color);
        self
    }

    /// Update dimensions (call when scrollbar sizes change).
    pub fn update_size(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.dirty = true;
        }
    }

    /// Get the effective color (with fallback).
    fn effective_color(&self) -> RgbaColor {
        self.color
            .clone()
            .or_else(|| self.style.scrollbar.corner_color.clone())
            .unwrap_or_else(ScrollbarStyle::fallback_corner)
    }
}

impl<M> Widget<M> for ScrollBarCorner {
    fn desired_size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let bg = self.effective_color();

        // Fill the corner with the background color
        let render_width = region.width.min(self.width as i32);
        let render_height = region.height.min(self.height as i32);

        for y in 0..render_height {
            for x in 0..render_width {
                canvas.put_char(region.x + x, region.y + y, ' ', None, Some(bg.clone()));
            }
        }
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
        self.dirty = true;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }
}
