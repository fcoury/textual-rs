//! Render cache for line-by-line widget rendering.
//!
//! This module provides the `RenderCache` struct which orchestrates rendering
//! a widget line-by-line, handling borders and content placement.

use tcss::types::ComputedStyle;
use tcss::types::border::BorderKind;

use crate::border_box::{BoxSegments, get_box};
use crate::border_chars::{get_border_chars, get_border_locations};
use crate::border_render::render_row;
use crate::segment::Segment;
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
    /// Cached outline box segments (top, middle, bottom rows).
    /// Outline is rendered ON TOP of content (doesn't affect layout).
    outline_box: Option<[BoxSegments; 3]>,
    /// Whether the widget has any visible outline.
    has_outline: bool,
    /// Which edges have visible outlines.
    has_top_outline: bool,
    has_right_outline: bool,
    has_bottom_outline: bool,
    has_left_outline: bool,
}

impl RenderCache {
    /// Creates a new render cache from a computed style.
    pub fn new(style: &ComputedStyle) -> Self {
        // Check which edges have visible borders
        let has_top_border =
            !matches!(style.border.top.kind, BorderKind::None | BorderKind::Hidden);
        let has_right_border = !matches!(
            style.border.right.kind,
            BorderKind::None | BorderKind::Hidden
        );
        let has_bottom_border = !matches!(
            style.border.bottom.kind,
            BorderKind::None | BorderKind::Hidden
        );
        let has_left_border = !matches!(
            style.border.left.kind,
            BorderKind::None | BorderKind::Hidden
        );
        let has_border = has_top_border || has_right_border || has_bottom_border || has_left_border;

        // Use top border kind as the primary style (for box character selection)
        let border_kind = style.border.top.kind;

        // Use the centralized effective_background from ComputedStyle
        let effective_bg = style.effective_background();

        let border_box = if has_border {
            let border_type = border_kind_to_str(border_kind);
            let effective_opacity = style.opacity * style.opacity;
            let resolve_edge_color =
                |edge: &tcss::types::border::BorderEdge| -> Option<tcss::types::RgbaColor> {
                    let base = edge.color.clone().or_else(|| style.color.clone());
                    match (&base, &style.inherited_background) {
                        (Some(color), Some(bg)) => Some(color.blend_toward(bg, effective_opacity)),
                        (Some(color), None) => Some(color.with_opacity(effective_opacity)),
                        _ => None,
                    }
                };

            let top_color = resolve_edge_color(&style.border.top);
            let right_color = resolve_edge_color(&style.border.right);
            let bottom_color = resolve_edge_color(&style.border.bottom);
            let left_color = resolve_edge_color(&style.border.left);

            let make_style =
                |fg: Option<tcss::types::RgbaColor>, bg: Option<tcss::types::RgbaColor>| Style {
                    fg,
                    bg,
                    ..Default::default()
                };

            let top_inner = make_style(top_color.clone(), effective_bg.clone());
            let top_outer = make_style(top_color.clone(), style.inherited_background.clone());
            let bottom_inner = make_style(bottom_color.clone(), effective_bg.clone());
            let bottom_outer = make_style(bottom_color.clone(), style.inherited_background.clone());
            let left_inner = make_style(left_color.clone(), effective_bg.clone());
            let left_outer = make_style(left_color.clone(), style.inherited_background.clone());
            let right_inner = make_style(right_color.clone(), effective_bg.clone());
            let right_outer = make_style(right_color.clone(), style.inherited_background.clone());

            let uniform =
                top_color == bottom_color && top_color == left_color && top_color == right_color;

            if uniform {
                Some(get_box(border_type, &top_inner, &top_outer))
            } else {
                Some(build_border_box_per_edge(
                    border_type,
                    &top_inner,
                    &top_outer,
                    &bottom_inner,
                    &bottom_outer,
                    &left_inner,
                    &left_outer,
                    &right_inner,
                    &right_outer,
                ))
            }
        } else {
            None
        };

        // Extract padding (convert Scalar to cells)
        let padding_top = style.padding.top.value as usize;
        let padding_right = style.padding.right.value as usize;
        let padding_bottom = style.padding.bottom.value as usize;
        let padding_left = style.padding.left.value as usize;

        // Check which edges have visible outlines
        let has_top_outline = !matches!(
            style.outline.top.kind,
            BorderKind::None | BorderKind::Hidden
        );
        let has_right_outline = !matches!(
            style.outline.right.kind,
            BorderKind::None | BorderKind::Hidden
        );
        let has_bottom_outline = !matches!(
            style.outline.bottom.kind,
            BorderKind::None | BorderKind::Hidden
        );
        let has_left_outline = !matches!(
            style.outline.left.kind,
            BorderKind::None | BorderKind::Hidden
        );
        let has_outline =
            has_top_outline || has_right_outline || has_bottom_outline || has_left_outline;

        // Build outline box segments if any outline edge is visible
        let outline_box = if has_outline {
            let outline_kind = style.outline.top.kind;
            let outline_type = border_kind_to_str(outline_kind);
            // Outline color, falling back to text color
            let outline_color = style
                .outline
                .top
                .color
                .clone()
                .or_else(|| style.color.clone());
            let outline_inner_style = Style {
                fg: outline_color.clone(),
                bg: effective_bg.clone(),
                ..Default::default()
            };
            // Outer style uses inherited/parent background
            let outline_outer_style = Style {
                fg: outline_color.clone(),
                bg: style.inherited_background.clone(),
                ..Default::default()
            };
            Some(get_box(
                outline_type,
                &outline_inner_style,
                &outline_outer_style,
            ))
        } else {
            None
        };

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
            outline_box,
            has_outline,
            has_top_outline,
            has_right_outline,
            has_bottom_outline,
            has_left_outline,
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

        let base_strip = if !self.has_border || self.border_box.is_none() {
            // No border - but still apply horizontal padding
            self.render_content_row(width, content_line)
        } else {
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
            } else if y >= content_row_offset
                && y < height - if self.has_bottom_border { 1 } else { 0 }
            {
                // Content row - check if we have side borders
                if self.has_left_border || self.has_right_border {
                    // Has at least one side border
                    self.render_partial_middle_row(&box_segs[1], content_line, width)
                } else {
                    // No side borders - just content with padding
                    self.render_content_row(width, content_line)
                }
            } else {
                // This shouldn't happen, but return blank row
                Strip::blank(width, self.pad_style())
            }
        };

        // Apply outline as final overlay (outline renders ON TOP of content)
        self.apply_outline(base_strip, y, height, width)
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
                    std::iter::repeat(fill_char)
                        .take(min_padding)
                        .collect::<String>(),
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
                        std::iter::repeat(fill_char)
                            .take(remaining)
                            .collect::<String>(),
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
                    std::iter::repeat(fill_char)
                        .take(left_padding)
                        .collect::<String>(),
                    fill_style.clone().unwrap_or_default(),
                ));
            }

            segments.extend(label_strip.segments().iter().cloned());

            if right_padding > 0 {
                segments.push(Segment::styled(
                    std::iter::repeat(fill_char)
                        .take(right_padding)
                        .collect::<String>(),
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
        self.style
            .effective_background()
            .as_ref()
            .map(|bg| Style::with_bg(bg.clone()))
    }

    /// Applies outline overlay to a rendered strip.
    ///
    /// Outline is rendered ON TOP of existing content, replacing edge characters.
    /// Unlike borders, outline doesn't affect layout - it's a visual overlay.
    fn apply_outline(&self, strip: Strip, y: usize, height: usize, width: usize) -> Strip {
        if !self.has_outline || self.outline_box.is_none() || width == 0 || height == 0 {
            return strip;
        }

        let box_segs = self.outline_box.as_ref().unwrap();
        use crate::segment::Segment;

        // Determine which outline row type we need
        let is_top_row = y == 0;
        let is_bottom_row = y == height - 1;

        if is_top_row && self.has_top_outline {
            // Top outline row - render full top edge
            let (left, fill, right) = &box_segs[0];
            let fill_char = fill.text().chars().next().unwrap_or('─');
            let fill_style = fill.style().cloned();

            let mut segments = Vec::new();
            if self.has_left_outline {
                segments.push(left.clone());
            }
            let inner_width = width
                .saturating_sub(if self.has_left_outline { 1 } else { 0 })
                .saturating_sub(if self.has_right_outline { 1 } else { 0 });
            if inner_width > 0 {
                segments.push(Segment::styled(
                    std::iter::repeat(fill_char)
                        .take(inner_width)
                        .collect::<String>(),
                    fill_style.unwrap_or_default(),
                ));
            }
            if self.has_right_outline {
                segments.push(right.clone());
            }
            Strip::from_segments(segments)
        } else if is_bottom_row && self.has_bottom_outline {
            // Bottom outline row - render full bottom edge
            let (left, fill, right) = &box_segs[2];
            let fill_char = fill.text().chars().next().unwrap_or('─');
            let fill_style = fill.style().cloned();

            let mut segments = Vec::new();
            if self.has_left_outline {
                segments.push(left.clone());
            }
            let inner_width = width
                .saturating_sub(if self.has_left_outline { 1 } else { 0 })
                .saturating_sub(if self.has_right_outline { 1 } else { 0 });
            if inner_width > 0 {
                segments.push(Segment::styled(
                    std::iter::repeat(fill_char)
                        .take(inner_width)
                        .collect::<String>(),
                    fill_style.unwrap_or_default(),
                ));
            }
            if self.has_right_outline {
                segments.push(right.clone());
            }
            Strip::from_segments(segments)
        } else if self.has_left_outline || self.has_right_outline {
            // Middle row - wrap content with side outlines
            let (left, _fill, right) = &box_segs[1];

            // Calculate content strip bounds
            let left_width = if self.has_left_outline { 1 } else { 0 };
            let right_width = if self.has_right_outline { 1 } else { 0 };
            let inner_width = width.saturating_sub(left_width + right_width);

            // Crop the existing strip to fit inside the outlines
            let content = strip.crop(left_width, left_width + inner_width);
            let content_adjusted = content.adjust_cell_length(inner_width, self.pad_style());

            let mut segments = Vec::new();
            if self.has_left_outline {
                segments.push(left.clone());
            }
            segments.extend(content_adjusted.segments().iter().cloned());
            if self.has_right_outline {
                segments.push(right.clone());
            }
            Strip::from_segments(segments)
        } else {
            // No outline edges for this row
            strip
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

fn build_border_box_per_edge(
    border_type: &str,
    top_inner: &Style,
    top_outer: &Style,
    bottom_inner: &Style,
    bottom_outer: &Style,
    left_inner: &Style,
    left_outer: &Style,
    right_inner: &Style,
    right_outer: &Style,
) -> [BoxSegments; 3] {
    let chars = get_border_chars(border_type);
    let locations = get_border_locations(border_type);

    let style_for_zone = |zone: u8, inner: &Style, outer: &Style| -> Style {
        match zone {
            0 => inner.clone(),
            1 => outer.clone(),
            2 => Style {
                fg: inner.fg.clone(),
                bg: outer.bg.clone(),
                reverse: true,
                ..inner.clone()
            },
            3 => Style {
                fg: outer.fg.clone(),
                bg: inner.bg.clone(),
                reverse: true,
                ..outer.clone()
            },
            _ => inner.clone(),
        }
    };

    let build_row = |row_idx: usize,
                     left_in: &Style,
                     left_out: &Style,
                     fill_in: &Style,
                     fill_out: &Style,
                     right_in: &Style,
                     right_out: &Style|
     -> BoxSegments {
        let left_char = chars[row_idx][0];
        let fill_char = chars[row_idx][1];
        let right_char = chars[row_idx][2];

        let left_loc = locations[row_idx][0];
        let fill_loc = locations[row_idx][1];
        let right_loc = locations[row_idx][2];

        let left_style = style_for_zone(left_loc, left_in, left_out);
        let fill_style = style_for_zone(fill_loc, fill_in, fill_out);
        let right_style = style_for_zone(right_loc, right_in, right_out);

        (
            Segment::styled(left_char.to_string(), left_style),
            Segment::styled(fill_char.to_string(), fill_style),
            Segment::styled(right_char.to_string(), right_style),
        )
    };

    let top_row = build_row(
        0, top_inner, top_outer, top_inner, top_outer, top_inner, top_outer,
    );
    let middle_row = build_row(
        1,
        left_inner,
        left_outer,
        top_inner,
        top_outer,
        right_inner,
        right_outer,
    );
    let bottom_row = build_row(
        2,
        bottom_inner,
        bottom_outer,
        bottom_inner,
        bottom_outer,
        bottom_inner,
        bottom_outer,
    );

    [top_row, middle_row, bottom_row]
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
    fn border_color_is_used_in_render() {
        let dodgerblue = RgbaColor::rgb(30, 144, 255);
        let mut style = ComputedStyle::default();
        style.border = Border::all(BorderEdge {
            kind: BorderKind::Outer, // Using outer border like the opacity example
            color: Some(dodgerblue.clone()),
        });
        style.background = Some(RgbaColor::rgb(32, 178, 170)); // lightseagreen

        let cache = RenderCache::new(&style);
        let line = cache.render_line(0, 3, 10, None, None, None);

        // Check that the first segment (border corner) has dodgerblue foreground
        let segments = line.segments();
        assert!(!segments.is_empty(), "Should have segments");

        let first_seg = &segments[0];
        let seg_style = first_seg.style();
        assert!(seg_style.is_some(), "Border segment should have style");

        let style_ref = seg_style.unwrap();
        let fg = style_ref.fg.as_ref();
        assert!(fg.is_some(), "Border should have foreground color");

        let fg_color = fg.unwrap();
        // dodgerblue is RGB(30, 144, 255)
        assert_eq!(
            fg_color.r, 30,
            "Red channel should be 30, got {}",
            fg_color.r
        );
        assert_eq!(
            fg_color.g, 144,
            "Green channel should be 144, got {}",
            fg_color.g
        );
        assert_eq!(
            fg_color.b, 255,
            "Blue channel should be 255, got {}",
            fg_color.b
        );
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
