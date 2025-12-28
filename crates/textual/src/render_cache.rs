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

/// Orchestrates line-by-line rendering with borders.
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

        Self {
            border_box,
            has_border,
            style: style.clone(),
        }
    }

    /// Returns true if this widget has a visible border.
    pub fn has_border(&self) -> bool {
        self.has_border
    }

    /// Returns the inner content region dimensions.
    ///
    /// If the widget has borders, the content area is 2 cells smaller in
    /// each dimension (1 cell for each border edge).
    pub fn inner_size(&self, width: usize, height: usize) -> (usize, usize) {
        if self.has_border && width >= 2 && height >= 2 {
            (width - 2, height - 2)
        } else if self.has_border {
            (0, 0)
        } else {
            (width, height)
        }
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
            // No border - just return content or blank
            return match content_line {
                Some(strip) => strip.adjust_cell_length(width, self.pad_style()),
                None => Strip::blank(width, self.pad_style()),
            };
        }

        let box_segs = self.border_box.as_ref().unwrap();

        if y == 0 {
            // Top border row
            render_row(&box_segs[0], width, title, None)
        } else if y == height - 1 {
            // Bottom border row
            render_row(&box_segs[2], width, None, None)
        } else {
            // Content row with side borders
            render_middle_row(&box_segs[1], content_line, width, self.pad_style())
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
        BorderKind::Block => "thick",
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
}
