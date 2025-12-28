//! Render cache for line-by-line widget rendering.
//!
//! This module provides the `RenderCache` struct which orchestrates rendering
//! a widget line-by-line, handling borders and content placement.

use tcss::types::ComputedStyle;
use tcss::types::border::BorderKind;

use crate::border_box::{get_box, BoxSegments};
use crate::border_render::{render_middle_row, render_row};
use crate::segment::Style;
use crate::strip::Strip;

/// Orchestrates line-by-line rendering with borders and padding.
///
/// The RenderCache handles the logic of determining what to render for each
/// row of a widget: top border, content rows, or bottom border.
pub struct RenderCache {
    /// Cached border box segments (top, middle, bottom rows).
    border_box: Option<[BoxSegments; 3]>,
    /// Whether the widget has a visible border.
    has_border: bool,
    /// The computed style for this widget.
    style: ComputedStyle,
    /// Padding in cells (top, right, bottom, left).
    padding_top: usize,
    padding_right: usize,
    padding_bottom: usize,
    padding_left: usize,
}

impl RenderCache {
    /// Creates a new render cache from a computed style.
    pub fn new(style: &ComputedStyle) -> Self {
        let border_kind = style.border.top.kind;
        let has_border = !matches!(border_kind, BorderKind::None | BorderKind::Hidden);

        let border_box = if has_border {
            let border_type = border_kind_to_str(border_kind);
            let inner_style = style_from_computed(style);
            let outer_style = inner_style.clone(); // For now, same as inner

            Some(get_box(border_type, &inner_style, &outer_style))
        } else {
            None
        };

        // Extract padding (convert Scalar to cells)
        let padding_top = style.padding.top.value as usize;
        let padding_right = style.padding.right.value as usize;
        let padding_bottom = style.padding.bottom.value as usize;
        let padding_left = style.padding.left.value as usize;

        Self {
            border_box,
            has_border,
            style: style.clone(),
            padding_top,
            padding_right,
            padding_bottom,
            padding_left,
        }
    }

    /// Returns true if this widget has a visible border.
    pub fn has_border(&self) -> bool {
        self.has_border
    }

    /// Returns the inner content region dimensions.
    ///
    /// Accounts for both borders and padding. If the widget has borders,
    /// the content area is 2 cells smaller in each dimension (1 cell for
    /// each border edge). Padding is then subtracted from the remaining space.
    pub fn inner_size(&self, width: usize, height: usize) -> (usize, usize) {
        // First, account for borders
        let (w, h) = if self.has_border && width >= 2 && height >= 2 {
            (width - 2, height - 2)
        } else if self.has_border {
            (0, 0)
        } else {
            (width, height)
        };

        // Then account for padding
        let padded_w = w.saturating_sub(self.padding_left + self.padding_right);
        let padded_h = h.saturating_sub(self.padding_top + self.padding_bottom);
        (padded_w, padded_h)
    }

    /// Returns the top padding in cells.
    pub fn padding_top(&self) -> usize {
        self.padding_top
    }

    /// Returns the right padding in cells.
    pub fn padding_right(&self) -> usize {
        self.padding_right
    }

    /// Returns the bottom padding in cells.
    pub fn padding_bottom(&self) -> usize {
        self.padding_bottom
    }

    /// Returns the left padding in cells.
    pub fn padding_left(&self) -> usize {
        self.padding_left
    }

    /// Renders a single line of the widget.
    ///
    /// # Arguments
    ///
    /// * `y` - The line index (0 = top)
    /// * `height` - Total height of the widget
    /// * `width` - Total width of the widget
    /// * `content_line` - Optional content strip for this line (if applicable)
    /// * `title` - Optional title for the top border
    ///
    /// # Returns
    ///
    /// A Strip representing this line of the widget.
    pub fn render_line(
        &self,
        y: usize,
        height: usize,
        width: usize,
        content_line: Option<&Strip>,
        title: Option<&Strip>,
    ) -> Strip {
        if width == 0 || height == 0 {
            return Strip::new();
        }

        if !self.has_border || self.border_box.is_none() {
            // No border - but still apply horizontal padding
            let content_width = width.saturating_sub(self.padding_left + self.padding_right);
            let content_strip = match content_line {
                Some(strip) => strip.adjust_cell_length(content_width, self.pad_style()),
                None => Strip::blank(content_width, self.pad_style()),
            };

            if self.padding_left == 0 && self.padding_right == 0 {
                return content_strip;
            }

            // Build: padding_left + content + padding_right
            let mut segments = Vec::new();
            if self.padding_left > 0 {
                segments.push(crate::segment::Segment::blank(
                    self.padding_left,
                    self.pad_style(),
                ));
            }
            segments.extend(content_strip.segments().iter().cloned());
            if self.padding_right > 0 {
                segments.push(crate::segment::Segment::blank(
                    self.padding_right,
                    self.pad_style(),
                ));
            }
            return Strip::from_segments(segments);
        }

        let box_segs = self.border_box.as_ref().unwrap();

        if y == 0 {
            // Top border row
            render_row(&box_segs[0], width, title, None)
        } else if y == height - 1 {
            // Bottom border row
            render_row(&box_segs[2], width, None, None)
        } else {
            // Content row with side borders and padding
            render_middle_row(
                &box_segs[1],
                content_line,
                width,
                self.pad_style(),
                self.padding_left,
                self.padding_right,
            )
        }
    }

    /// Returns the padding style for blank areas.
    fn pad_style(&self) -> Option<Style> {
        self.style.background.as_ref().map(|bg| Style::with_bg(bg.clone()))
    }
}

/// Converts a BorderKind enum to a string for border_chars lookup.
fn border_kind_to_str(kind: BorderKind) -> &'static str {
    match kind {
        BorderKind::None => "none",
        BorderKind::Hidden => "hidden",
        BorderKind::Ascii => "ascii",
        BorderKind::Blank => "blank",
        BorderKind::Block => "block",
        BorderKind::Dashed => "dashed",
        BorderKind::Double => "double",
        BorderKind::Heavy => "heavy",
        BorderKind::Inner => "inner",
        BorderKind::Outer => "outer",
        BorderKind::Round => "round",
        BorderKind::Solid => "solid",
        BorderKind::Thick => "thick",
    }
}

/// Creates a rendering Style from a ComputedStyle.
fn style_from_computed(computed: &ComputedStyle) -> Style {
    Style {
        fg: computed.color.clone(),
        bg: computed.background.clone(),
        bold: false,
        dim: false,
        italic: false,
        underline: false,
        strike: false,
        reverse: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tcss::types::border::{Border, BorderEdge};
    use tcss::types::RgbaColor;

    fn style_with_round_border() -> ComputedStyle {
        let mut style = ComputedStyle::default();
        style.border = Border::all(BorderEdge {
            kind: BorderKind::Round,
            color: Some(RgbaColor::white()),
        });
        style
    }

    #[test]
    fn render_cache_no_border() {
        let style = ComputedStyle::default();
        let cache = RenderCache::new(&style);
        assert!(!cache.has_border());
    }

    #[test]
    fn render_cache_with_border() {
        let style = style_with_round_border();
        let cache = RenderCache::new(&style);
        assert!(cache.has_border());
    }

    #[test]
    fn inner_size_no_border() {
        let style = ComputedStyle::default();
        let cache = RenderCache::new(&style);
        assert_eq!(cache.inner_size(10, 5), (10, 5));
    }

    #[test]
    fn inner_size_with_border() {
        let style = style_with_round_border();
        let cache = RenderCache::new(&style);
        assert_eq!(cache.inner_size(10, 5), (8, 3));
    }

    #[test]
    fn render_line_top_border() {
        let style = style_with_round_border();
        let cache = RenderCache::new(&style);
        let line = cache.render_line(0, 3, 10, None, None);
        assert_eq!(line.text(), "╭────────╮");
    }

    #[test]
    fn render_line_bottom_border() {
        let style = style_with_round_border();
        let cache = RenderCache::new(&style);
        let line = cache.render_line(2, 3, 10, None, None);
        assert_eq!(line.text(), "╰────────╯");
    }

    #[test]
    fn render_line_middle_with_content() {
        let style = style_with_round_border();
        let cache = RenderCache::new(&style);
        let content = Strip::from_segment(crate::segment::Segment::new("Hi"));
        let line = cache.render_line(1, 3, 10, Some(&content), None);
        assert_eq!(line.text(), "│Hi      │");
    }

    // Padding tests

    fn style_with_padding(top: f64, right: f64, bottom: f64, left: f64) -> ComputedStyle {
        use tcss::types::Scalar;
        let mut style = ComputedStyle::default();
        style.padding.top = Scalar::cells(top);
        style.padding.right = Scalar::cells(right);
        style.padding.bottom = Scalar::cells(bottom);
        style.padding.left = Scalar::cells(left);
        style
    }

    #[test]
    fn inner_size_with_padding() {
        let style = style_with_padding(2.0, 2.0, 2.0, 2.0);
        let cache = RenderCache::new(&style);
        // 20x10 widget with 2-cell padding each side = 16x6 content
        assert_eq!(cache.inner_size(20, 10), (16, 6));
    }

    #[test]
    fn inner_size_with_asymmetric_padding() {
        let style = style_with_padding(1.0, 2.0, 1.0, 2.0);
        let cache = RenderCache::new(&style);
        // 20x10 widget: width - 4 = 16, height - 2 = 8
        assert_eq!(cache.inner_size(20, 10), (16, 8));
    }

    #[test]
    fn inner_size_with_border_and_padding() {
        let mut style = style_with_round_border();
        style.padding.top = tcss::types::Scalar::cells(1.0);
        style.padding.right = tcss::types::Scalar::cells(1.0);
        style.padding.bottom = tcss::types::Scalar::cells(1.0);
        style.padding.left = tcss::types::Scalar::cells(1.0);
        let cache = RenderCache::new(&style);
        // 20x10 - 2 border - 2 padding each dimension = 16x6
        // width: 20 - 2 (border) - 2 (padding l+r) = 16
        // height: 10 - 2 (border) - 2 (padding t+b) = 6
        assert_eq!(cache.inner_size(20, 10), (16, 6));
    }

    #[test]
    fn padding_getters() {
        let style = style_with_padding(1.0, 2.0, 3.0, 4.0);
        let cache = RenderCache::new(&style);
        assert_eq!(cache.padding_top(), 1);
        assert_eq!(cache.padding_right(), 2);
        assert_eq!(cache.padding_bottom(), 3);
        assert_eq!(cache.padding_left(), 4);
    }

    #[test]
    fn render_line_no_border_with_padding() {
        let style = style_with_padding(0.0, 2.0, 0.0, 2.0);
        let cache = RenderCache::new(&style);
        let content = Strip::from_segment(crate::segment::Segment::new("Hi"));
        let line = cache.render_line(0, 1, 10, Some(&content), None);
        // 2 spaces (padding_left) + "Hi" + 4 spaces (content padding) + 2 spaces (padding_right)
        assert_eq!(line.text(), "  Hi      ");
        assert_eq!(line.cell_length(), 10);
    }
}
