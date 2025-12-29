//! Render cache for line-by-line widget rendering.
//!
//! This module provides the `RenderCache` struct which orchestrates rendering
//! a widget line-by-line, handling borders and content placement.

use tcss::types::ComputedStyle;
use tcss::types::border::BorderKind;

use crate::border_box::{BoxSegments, get_box};
use crate::border_render::render_row;
use crate::segment::Style;
use crate::strip::Strip;

/// Orchestrates line-by-line rendering with borders and padding.
///
/// The RenderCache handles the logic of determining what to render for each
/// row of a widget: top border, content rows, or bottom border.
pub struct RenderCache {
    /// Cached border box segments (top, middle, bottom rows).
    border_box: Option<[BoxSegments; 3]>,
    /// Whether the widget has any visible border.
    has_border: bool,
    /// Which edges have visible borders.
    has_top_border: bool,
    has_right_border: bool,
    has_bottom_border: bool,
    has_left_border: bool,
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
        // Check which edges have visible borders
        let has_top_border = !matches!(style.border.top.kind, BorderKind::None | BorderKind::Hidden);
        let has_right_border = !matches!(style.border.right.kind, BorderKind::None | BorderKind::Hidden);
        let has_bottom_border = !matches!(style.border.bottom.kind, BorderKind::None | BorderKind::Hidden);
        let has_left_border = !matches!(style.border.left.kind, BorderKind::None | BorderKind::Hidden);
        let has_border = has_top_border || has_right_border || has_bottom_border || has_left_border;

        // Use top border kind as the primary style (for box character selection)
        let border_kind = style.border.top.kind;

        // Compute effective background (with alpha compositing and tint applied)
        // If the background has alpha < 1.0, composite it over the inherited background
        let effective_bg = match (&style.background, &style.inherited_background) {
            (Some(bg), Some(inherited)) if bg.a < 1.0 => {
                // Composite semi-transparent background over inherited
                let composited = bg.blend_over(inherited);
                // Then apply tint if present
                match &style.background_tint {
                    Some(tint) => Some(composited.tint(tint)),
                    None => Some(composited),
                }
            }
            (Some(bg), _) => {
                // Opaque background or no inherited - just apply tint
                match &style.background_tint {
                    Some(tint) => Some(bg.tint(tint)),
                    None => Some(bg.clone()),
                }
            }
            (None, Some(inherited)) => {
                // No background specified, inherit from parent
                Some(inherited.clone())
            }
            (None, None) => None,
        };

        let border_box = if has_border {
            let border_type = border_kind_to_str(border_kind);
            // Use border color for the foreground, falling back to text color
            let border_color = style.border.top.color.clone().or_else(|| style.color.clone());
            let inner_style = Style {
                fg: border_color.clone(),
                bg: effective_bg.clone(),
                ..Default::default()
            };
            // Outer style uses inherited/parent background for zone 2/3 color reversal
            let outer_style = Style {
                fg: border_color.clone(),
                bg: style.inherited_background.clone(),
                ..Default::default()
            };

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
            has_top_border,
            has_right_border,
            has_bottom_border,
            has_left_border,
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
    /// Accounts for both borders and padding. Each border edge takes 1 cell.
    /// Padding is then subtracted from the remaining space.
    pub fn inner_size(&self, width: usize, height: usize) -> (usize, usize) {
        // Account for borders on each edge
        let border_left = if self.has_left_border { 1 } else { 0 };
        let border_right = if self.has_right_border { 1 } else { 0 };
        let border_top = if self.has_top_border { 1 } else { 0 };
        let border_bottom = if self.has_bottom_border { 1 } else { 0 };

        let w = width.saturating_sub(border_left + border_right);
        let h = height.saturating_sub(border_top + border_bottom);

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
    /// * `subtitle` - Optional subtitle for the bottom border
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
        subtitle: Option<&Strip>,
    ) -> Strip {
        if width == 0 || height == 0 {
            return Strip::new();
        }

        if !self.has_border || self.border_box.is_none() {
            // No border - but still apply horizontal padding
            return self.render_content_row(width, content_line);
        }

        let box_segs = self.border_box.as_ref().unwrap();

        // Determine if this is a top border row
        let is_top_border_row = y == 0 && self.has_top_border;
        // Determine if this is a bottom border row
        let is_bottom_border_row = y == height - 1 && self.has_bottom_border;
        // Calculate the content row index (offset by top border if present)
        let content_row_offset = if self.has_top_border { 1 } else { 0 };

        if is_top_border_row {
            // Top border row - check if we have corners (left/right borders)
            if self.has_left_border && self.has_right_border {
                // Full border with corners
                render_row(
                    &box_segs[0],
                    width,
                    title,
                    None,
                    self.style.border_title_align,
                    self.style.border_subtitle_align,
                )
            } else {
                // Partial border - no corners, just horizontal line
                self.render_horizontal_border_row(
                    &box_segs[0],
                    width,
                    title,
                    self.style.border_title_align,
                )
            }
        } else if is_bottom_border_row {
            // Bottom border row - check if we have corners
            if self.has_left_border && self.has_right_border {
                // Full border with corners
                render_row(
                    &box_segs[2],
                    width,
                    None,
                    subtitle,
                    self.style.border_title_align,
                    self.style.border_subtitle_align,
                )
            } else {
                // Partial border - no corners, just horizontal line
                self.render_horizontal_border_row(
                    &box_segs[2],
                    width,
                    subtitle,
                    self.style.border_subtitle_align,
                )
            }
        } else if y >= content_row_offset && y < height - if self.has_bottom_border { 1 } else { 0 } {
            // Content row - check if we have side borders
            if self.has_left_border || self.has_right_border {
                // Has at least one side border
                self.render_partial_middle_row(
                    &box_segs[1],
                    content_line,
                    width,
                )
            } else {
                // No side borders - just content with padding
                self.render_content_row(width, content_line)
            }
        } else {
            // This shouldn't happen, but return blank row
            Strip::blank(width, self.pad_style())
        }
    }

    /// Renders a content row without any border (just padding and content).
    fn render_content_row(&self, width: usize, content_line: Option<&Strip>) -> Strip {
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
        Strip::from_segments(segments)
    }

    /// Renders a horizontal border row without corners (for partial borders).
    fn render_horizontal_border_row(
        &self,
        box_segments: &crate::border_box::BoxSegments,
        width: usize,
        label: Option<&Strip>,
        align: tcss::types::AlignHorizontal,
    ) -> Strip {
        use crate::segment::Segment;

        let (_left, fill, _right) = box_segments;
        let fill_char = fill.text().chars().next().unwrap_or('─');
        let fill_style = fill.style().cloned();

        if let Some(label_strip) = label {
            // Render label with fill characters on either side
            // Note: Partial borders (no corners) do NOT add spaces around labels
            let label_len = label_strip.cell_length();
            let min_padding = 1; // Minimum 1 fill char on each side
            let available = width.saturating_sub(min_padding * 2);

            if label_len > available {
                // Label too long - truncate with ellipsis
                // Reserve 1 char for ellipsis, so we can show (available - 1) chars of text
                let truncate_len = available.saturating_sub(1);
                let cropped = label_strip.crop(0, truncate_len);

                let mut segments = Vec::new();
                segments.push(Segment::styled(
                    std::iter::repeat(fill_char).take(min_padding).collect::<String>(),
                    fill_style.clone().unwrap_or_default(),
                ));

                segments.extend(cropped.segments().iter().cloned());

                // Add ellipsis with the same style as the label's last segment
                let ellipsis_style = cropped
                    .segments()
                    .last()
                    .and_then(|s| s.style().cloned())
                    .unwrap_or_default();
                segments.push(Segment::styled("…", ellipsis_style));

                // Fill remaining space (should be min_padding chars)
                let used = min_padding + cropped.cell_length() + 1; // fill + text + ellipsis
                let remaining = width.saturating_sub(used);
                if remaining > 0 {
                    segments.push(Segment::styled(
                        std::iter::repeat(fill_char).take(remaining).collect::<String>(),
                        fill_style.unwrap_or_default(),
                    ));
                }
                return Strip::from_segments(segments);
            }

            // Calculate padding based on alignment
            let total_padding = width - label_len;
            let (left_padding, right_padding) = match align {
                tcss::types::AlignHorizontal::Left => {
                    (min_padding, total_padding.saturating_sub(min_padding))
                }
                tcss::types::AlignHorizontal::Center => {
                    let left = total_padding / 2;
                    (left, total_padding - left)
                }
                tcss::types::AlignHorizontal::Right => {
                    (total_padding.saturating_sub(min_padding), min_padding)
                }
            };

            let mut segments = Vec::new();
            if left_padding > 0 {
                segments.push(Segment::styled(
                    std::iter::repeat(fill_char).take(left_padding).collect::<String>(),
                    fill_style.clone().unwrap_or_default(),
                ));
            }

            segments.extend(label_strip.segments().iter().cloned());

            if right_padding > 0 {
                segments.push(Segment::styled(
                    std::iter::repeat(fill_char).take(right_padding).collect::<String>(),
                    fill_style.unwrap_or_default(),
                ));
            }
            Strip::from_segments(segments)
        } else {
            // Just fill characters
            let text: String = std::iter::repeat(fill_char).take(width).collect();
            Strip::from_segment(Segment::styled(text, fill_style.unwrap_or_default()))
        }
    }

    /// Renders a middle row with partial side borders (left, right, both, or neither).
    fn render_partial_middle_row(
        &self,
        box_segments: &crate::border_box::BoxSegments,
        content: Option<&Strip>,
        width: usize,
    ) -> Strip {
        use crate::segment::Segment;

        let (left, _fill, right) = box_segments;

        // Calculate widths
        let left_border_width = if self.has_left_border { 1 } else { 0 };
        let right_border_width = if self.has_right_border { 1 } else { 0 };
        let inner_width = width.saturating_sub(left_border_width + right_border_width);
        let content_width = inner_width.saturating_sub(self.padding_left + self.padding_right);

        let mut segments = Vec::new();

        // Left border if present
        if self.has_left_border {
            segments.push(left.clone());
        }

        // Left padding
        if self.padding_left > 0 {
            segments.push(Segment::blank(self.padding_left, self.pad_style()));
        }

        // Content
        let content_strip = match content {
            Some(strip) => strip.adjust_cell_length(content_width, self.pad_style()),
            None => Strip::blank(content_width, self.pad_style()),
        };
        segments.extend(content_strip.segments().iter().cloned());

        // Right padding
        if self.padding_right > 0 {
            segments.push(Segment::blank(self.padding_right, self.pad_style()));
        }

        // Right border if present
        if self.has_right_border {
            segments.push(right.clone());
        }

        Strip::from_segments(segments)
    }

    /// Returns the padding style for blank areas.
    fn pad_style(&self) -> Option<Style> {
        self.effective_background()
            .as_ref()
            .map(|bg| Style::with_bg(bg.clone()))
    }

    /// Returns the background color with alpha compositing and background-tint applied.
    fn effective_background(&self) -> Option<tcss::types::RgbaColor> {
        match (&self.style.background, &self.style.inherited_background) {
            (Some(bg), Some(inherited)) if bg.a < 1.0 => {
                // Composite semi-transparent background over inherited
                let composited = bg.blend_over(inherited);
                // Then apply tint if present
                match &self.style.background_tint {
                    Some(tint) => Some(composited.tint(tint)),
                    None => Some(composited),
                }
            }
            (Some(bg), _) => {
                // Opaque background or no inherited - just apply tint
                match &self.style.background_tint {
                    Some(tint) => Some(bg.tint(tint)),
                    None => Some(bg.clone()),
                }
            }
            (None, Some(inherited)) => {
                // No background specified, inherit from parent
                Some(inherited.clone())
            }
            (None, None) => None,
        }
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
        BorderKind::Hkey => "hkey",
        BorderKind::Inner => "inner",
        BorderKind::Outer => "outer",
        BorderKind::Panel => "panel",
        BorderKind::Round => "round",
        BorderKind::Solid => "solid",
        BorderKind::Tall => "tall",
        BorderKind::Thick => "thick",
        BorderKind::Vkey => "vkey",
        BorderKind::Wide => "wide",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tcss::types::RgbaColor;
    use tcss::types::border::{Border, BorderEdge};

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
        let line = cache.render_line(0, 3, 10, None, None, None);
        assert_eq!(line.text(), "╭────────╮");
    }

    #[test]
    fn render_line_bottom_border() {
        let style = style_with_round_border();
        let cache = RenderCache::new(&style);
        let line = cache.render_line(2, 3, 10, None, None, None);
        assert_eq!(line.text(), "╰────────╯");
    }

    #[test]
    fn render_line_middle_with_content() {
        let style = style_with_round_border();
        let cache = RenderCache::new(&style);
        let content = Strip::from_segment(crate::segment::Segment::new("Hi"));
        let line = cache.render_line(1, 3, 10, Some(&content), None, None);
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
        let line = cache.render_line(0, 1, 10, Some(&content), None, None);
        // 2 spaces (padding_left) + "Hi" + 4 spaces (content padding) + 2 spaces (padding_right)
        assert_eq!(line.text(), "  Hi      ");
        assert_eq!(line.cell_length(), 10);
    }
}
