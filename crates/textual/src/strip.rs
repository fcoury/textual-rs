//! Strip: An immutable horizontal line of Segments.
//!
//! Strips represent a single line of styled text. They are the intermediate
//! form between Content (text) and Canvas (rendering). Strips support
//! operations like cropping, alignment, and style application.
//!
//! ## Pipeline Position
//! ```text
//! Content → Strip[] → Segment[] → Canvas
//!              ↑
//!         You are here
//! ```

use crate::segment::{Segment, Style};
use smallvec::{SmallVec, smallvec};
use std::collections::HashMap;
use tcss::types::RgbaColor;
use tcss::types::text::AlignHorizontal;

/// Inline storage for 2 segments covers most common text patterns:
/// - Single styled text (1 segment)
/// - "Label: " + "value" (2 segments)
/// Keeping this small reduces struct size and benefits single-segment operations.
pub type SegmentVec = SmallVec<[Segment; 2]>;

/// An immutable horizontal line of segments.
///
/// Strips are the rendering primitive for a single line of output.
/// They track their cell length for efficient layout operations.
///
/// Uses SmallVec internally to avoid heap allocation for lines with
/// 1-2 segments (covers most typical text rendering).
#[derive(Clone, Debug, Default)]
pub struct Strip {
    /// The segments that make up this line.
    /// SmallVec<[Segment; 2]> stores up to 2 segments inline without heap allocation.
    segments: SegmentVec,
    /// Cached total cell width.
    cell_length: usize,
}

impl Strip {
    /// Creates an empty strip.
    pub fn new() -> Self {
        Self {
            segments: SegmentVec::new(),
            cell_length: 0,
        }
    }

    /// Creates a strip from a Vec of segments.
    ///
    /// This is the fast path for Vec input - directly converts to SmallVec.
    pub fn from_segments(segments: Vec<Segment>) -> Self {
        let cell_length = segments.iter().map(|s| s.cell_length()).sum();
        Self {
            segments: SegmentVec::from_vec(segments),
            cell_length,
        }
    }

    /// Creates a strip from any iterable of segments.
    ///
    /// Use this when you have an array or iterator of segments.
    pub fn from_iter(segments: impl IntoIterator<Item = Segment>) -> Self {
        let segments: SegmentVec = segments.into_iter().collect();
        Self::from_smallvec(segments)
    }

    /// Internal constructor from pre-built SegmentVec.
    fn from_smallvec(segments: SegmentVec) -> Self {
        let cell_length = segments.iter().map(|s| s.cell_length()).sum();
        Self {
            segments,
            cell_length,
        }
    }

    /// Creates a strip from a single segment.
    pub fn from_segment(segment: Segment) -> Self {
        let cell_length = segment.cell_length();
        Self {
            segments: smallvec![segment],
            cell_length,
        }
    }

    /// Creates a blank strip of spaces with the given width and style.
    pub fn blank(width: usize, style: Option<Style>) -> Self {
        if width == 0 {
            return Self::new();
        }
        Self::from_segment(Segment::blank(width, style))
    }

    /// Returns the segments in this strip.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Returns the total cell width of this strip.
    pub fn cell_length(&self) -> usize {
        self.cell_length
    }

    /// Returns true if the strip is empty.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty() || self.cell_length == 0
    }

    /// Returns the text content of this strip without styling.
    pub fn text(&self) -> String {
        self.segments.iter().map(|s| s.text()).collect()
    }

    /// Apply text opacity by blending foreground colors toward the background.
    ///
    /// When opacity is 0, text is replaced with spaces while preserving background.
    pub fn apply_text_opacity(&self, opacity: f64, fallback_bg: Option<&RgbaColor>) -> Strip {
        if opacity >= 1.0 {
            return self.clone();
        }

        let mut segments: SegmentVec = SegmentVec::new();
        for segment in &self.segments {
            let meta = segment.meta().clone();
            let style = segment.style().cloned();
            let bg = style.as_ref().and_then(|s| s.bg.as_ref()).or(fallback_bg);

            if opacity <= 0.0 {
                let blank_style = style.map(|mut s| {
                    s.fg = None;
                    s
                });
                let blank = Segment::blank(segment.cell_length(), blank_style);
                segments.push(blank.with_meta(meta));
                continue;
            }

            let fg = style
                .as_ref()
                .and_then(|s| s.fg.as_ref())
                .cloned()
                .or_else(|| bg.map(|_| RgbaColor::rgba(255, 255, 255, 1.0)));

            let adjusted_style = if let Some(fg) = fg {
                let blended = if let Some(bg) = bg {
                    fg.blend_toward(bg, opacity)
                } else {
                    fg.with_opacity(opacity)
                };
                let mut new_style = style.unwrap_or_default();
                new_style.fg = Some(blended);
                Some(new_style)
            } else {
                style
            };

            let new_segment = match adjusted_style {
                Some(style) => Segment::styled(segment.text(), style).with_meta(meta),
                None => Segment::new(segment.text()).with_meta(meta),
            };
            segments.push(new_segment);
        }

        Strip::from_smallvec(segments)
    }

    /// Extracts a portion of the strip from `start` to `end` cell positions.
    ///
    /// The resulting strip contains cells in the range [start, end).
    pub fn crop(&self, start: usize, end: usize) -> Strip {
        if start >= end || start >= self.cell_length {
            return Strip::new();
        }

        let end = end.min(self.cell_length);
        let mut result_segments = SegmentVec::new();
        let mut current_pos = 0;

        for segment in &self.segments {
            let seg_len = segment.cell_length();
            let seg_end = current_pos + seg_len;

            // Skip segments entirely before start
            if seg_end <= start {
                current_pos = seg_end;
                continue;
            }

            // Stop after end
            if current_pos >= end {
                break;
            }

            // Calculate how much of this segment to include
            let local_start = start.saturating_sub(current_pos);
            let local_end = (end - current_pos).min(seg_len);

            if local_start == 0 && local_end == seg_len {
                // Include entire segment
                result_segments.push(segment.clone());
            } else if local_start == 0 {
                // Take from start
                let (left, _) = segment.split_at(local_end);
                if !left.is_empty() {
                    result_segments.push(left);
                }
            } else if local_end == seg_len {
                // Take to end
                let (_, right) = segment.split_at(local_start);
                if !right.is_empty() {
                    result_segments.push(right);
                }
            } else {
                // Take middle portion
                let (_, temp) = segment.split_at(local_start);
                let (middle, _) = temp.split_at(local_end - local_start);
                if !middle.is_empty() {
                    result_segments.push(middle);
                }
            }

            current_pos = seg_end;
        }

        Strip::from_smallvec(result_segments)
    }

    /// Splits the strip at the given cell positions.
    ///
    /// Returns a vector of strips, one for each segment between cuts.
    pub fn divide(&self, cuts: &[usize]) -> Vec<Strip> {
        if cuts.is_empty() {
            return vec![self.clone()];
        }

        let mut result = Vec::with_capacity(cuts.len() + 1);
        let mut last = 0;

        for &cut in cuts {
            if cut > last && cut <= self.cell_length {
                result.push(self.crop(last, cut));
                last = cut;
            }
        }

        // Add remaining portion
        if last < self.cell_length {
            result.push(self.crop(last, self.cell_length));
        }

        result
    }

    /// Applies a style to all segments in this strip.
    ///
    /// Returns a new strip with the style layered on top of existing styles.
    pub fn apply_style(&self, style: &Style) -> Strip {
        let segments: Vec<_> = self.segments.iter().map(|s| s.apply_style(style)).collect();
        Strip::from_segments(segments)
    }

    /// Merges adjacent segments with identical styles.
    ///
    /// This reduces memory usage and simplifies rendering.
    pub fn simplify(&self) -> Strip {
        if self.segments.len() <= 1 {
            return self.clone();
        }

        let mut result = SegmentVec::new();
        let mut current_text = String::new();
        let mut current_style: Option<Style> = None;
        let mut has_current = false;

        for segment in &self.segments {
            if !has_current {
                current_text = segment.text().to_string();
                current_style = segment.style().cloned();
                has_current = true;
            } else if segment.style() == current_style.as_ref() {
                // Same style, merge text
                current_text.push_str(segment.text());
            } else {
                // Different style, emit current and start new
                if !current_text.is_empty() {
                    match current_style {
                        Some(s) => result.push(Segment::styled(current_text, s)),
                        None => result.push(Segment::new(current_text)),
                    }
                }
                current_text = segment.text().to_string();
                current_style = segment.style().cloned();
            }
        }

        // Emit final segment
        if !current_text.is_empty() {
            match current_style {
                Some(s) => result.push(Segment::styled(current_text, s)),
                None => result.push(Segment::new(current_text)),
            }
        }

        Strip::from_smallvec(result)
    }

    /// Concatenates multiple strips into one.
    pub fn join(strips: impl IntoIterator<Item = Strip>) -> Strip {
        let mut segments = SegmentVec::new();
        for strip in strips {
            segments.extend(strip.segments);
        }
        Strip::from_smallvec(segments)
    }

    /// Adjusts the strip to exactly the given length.
    ///
    /// If shorter, pads with spaces using the provided style.
    /// If longer, crops to fit.
    pub fn adjust_cell_length(&self, length: usize, pad_style: Option<Style>) -> Strip {
        match self.cell_length.cmp(&length) {
            std::cmp::Ordering::Equal => self.clone(),
            std::cmp::Ordering::Greater => self.crop(0, length),
            std::cmp::Ordering::Less => {
                let padding = Segment::blank(length - self.cell_length, pad_style);
                let mut segments = self.segments.clone();
                segments.push(padding);
                Strip::from_smallvec(segments)
            }
        }
    }

    /// Aligns text within a given width.
    ///
    /// Returns a new strip of exactly `width` cells with the content aligned
    /// according to the specified alignment.
    pub fn text_align(
        &self,
        align: AlignHorizontal,
        width: usize,
        pad_style: Option<Style>,
    ) -> Strip {
        if self.cell_length >= width {
            return self.crop(0, width);
        }

        let gap = width - self.cell_length;

        match align {
            AlignHorizontal::Left => {
                // Content at start, padding at end
                let padding = Segment::blank(gap, pad_style);
                let mut segments = self.segments.clone();
                segments.push(padding);
                Strip::from_smallvec(segments)
            }
            AlignHorizontal::Right => {
                // Padding at start, content at end
                let padding = Segment::blank(gap, pad_style);
                let mut segments: SegmentVec = smallvec![padding];
                segments.extend(self.segments.iter().cloned());
                Strip::from_smallvec(segments)
            }
            AlignHorizontal::Center => {
                // Split padding between start and end
                let left_pad = gap / 2;
                let right_pad = gap - left_pad;

                let mut segments = SegmentVec::new();
                if left_pad > 0 {
                    segments.push(Segment::blank(left_pad, pad_style.clone()));
                }
                segments.extend(self.segments.iter().cloned());
                if right_pad > 0 {
                    segments.push(Segment::blank(right_pad, pad_style));
                }
                Strip::from_smallvec(segments)
            }
        }
    }

    /// Fully justifies text to fill the given width.
    pub fn justify(&self, width: usize, pad_style: Option<Style>) -> Strip {
        if self.cell_length >= width {
            return self.crop(0, width);
        }

        let gap_count: usize = self
            .segments
            .iter()
            .map(|seg| seg.text().chars().filter(|ch| *ch == ' ').count())
            .sum();

        if gap_count == 0 {
            return self.text_align(AlignHorizontal::Left, width, pad_style);
        }

        let extra = width.saturating_sub(self.cell_length);
        let mut extra_per_gap = vec![0usize; gap_count];
        for i in 0..extra {
            let idx = gap_count - 1 - (i % gap_count);
            extra_per_gap[idx] += 1;
        }

        let mut gap_index = 0;
        let mut segments = SegmentVec::new();

        for seg in &self.segments {
            let style = seg.style().cloned();
            let meta = seg.meta();
            let mut buffer = String::new();

            for ch in seg.text().chars() {
                if ch == ' ' {
                    if !buffer.is_empty() {
                        segments.push(build_segment(&buffer, style.clone(), meta));
                        buffer.clear();
                    }

                    let extra_spaces = extra_per_gap.get(gap_index).copied().unwrap_or(0);
                    gap_index += 1;
                    let total = 1 + extra_spaces;
                    let spaces = " ".repeat(total);
                    segments.push(build_segment(&spaces, style.clone(), meta));
                } else {
                    buffer.push(ch);
                }
            }

            if !buffer.is_empty() {
                segments.push(build_segment(&buffer, style.clone(), meta));
            }
        }

        Strip::from_smallvec(segments)
    }

    /// Applies a tint color to all segments in this strip.
    ///
    /// This is post-processing that tints both fg and bg colors using
    /// Textual's linear interpolation formula:
    /// `result = base + (overlay - base) * alpha`
    ///
    /// Returns a new strip with tinted colors. If the tint is fully
    /// transparent (alpha = 0), returns a clone of the original strip.
    pub fn apply_tint(&self, tint: &RgbaColor) -> Strip {
        // Skip if tint is fully transparent
        if tint.a == 0.0 {
            return self.clone();
        }

        let tinted_segments: SegmentVec = self
            .segments
            .iter()
            .map(|seg| seg.apply_tint(tint))
            .collect();

        Strip::from_smallvec(tinted_segments)
    }

    /// Apply a hatch pattern to this strip.
    ///
    /// Replaces space characters with the hatch character and applies
    /// the hatch color as the foreground. Non-space characters are unchanged.
    pub fn apply_hatch(&self, hatch_char: char, hatch_color: &RgbaColor, opacity: f32) -> Strip {
        let hatched_segments: SegmentVec = self
            .segments
            .iter()
            .map(|seg| seg.apply_hatch(hatch_char, hatch_color, opacity))
            .collect();

        Strip::from_smallvec(hatched_segments)
    }
}

fn build_segment(text: &str, style: Option<Style>, meta: &HashMap<String, String>) -> Segment {
    let mut segment = match style {
        Some(style) => Segment::styled(text, style),
        None => Segment::new(text),
    };

    if !meta.is_empty() {
        segment = segment.with_meta(meta.clone());
    }

    segment
}

#[cfg(test)]
mod tests {
    use super::*;
    use tcss::types::RgbaColor;

    #[test]
    fn strip_new_is_empty() {
        let strip = Strip::new();
        assert!(strip.is_empty());
        assert_eq!(strip.cell_length(), 0);
    }

    #[test]
    fn strip_from_segment() {
        let seg = Segment::new("hello");
        let strip = Strip::from_segment(seg);
        assert_eq!(strip.cell_length(), 5);
        assert_eq!(strip.text(), "hello");
    }

    #[test]
    fn strip_from_segments() {
        let segments = vec![
            Segment::new("hello"),
            Segment::new(" "),
            Segment::new("world"),
        ];
        let strip = Strip::from_segments(segments);
        assert_eq!(strip.cell_length(), 11);
        assert_eq!(strip.text(), "hello world");
    }

    #[test]
    fn strip_blank() {
        let strip = Strip::blank(5, None);
        assert_eq!(strip.cell_length(), 5);
        assert_eq!(strip.text(), "     ");
    }

    #[test]
    fn strip_crop_middle() {
        let strip = Strip::from_segment(Segment::new("Hello World"));
        let cropped = strip.crop(3, 8);
        assert_eq!(cropped.text(), "lo Wo");
    }

    #[test]
    fn strip_crop_start() {
        let strip = Strip::from_segment(Segment::new("Hello"));
        let cropped = strip.crop(0, 3);
        assert_eq!(cropped.text(), "Hel");
    }

    #[test]
    fn strip_crop_end() {
        let strip = Strip::from_segment(Segment::new("Hello"));
        let cropped = strip.crop(2, 5);
        assert_eq!(cropped.text(), "llo");
    }

    #[test]
    fn strip_crop_multiple_segments() {
        let segments = vec![
            Segment::new("AAA"),
            Segment::new("BBB"),
            Segment::new("CCC"),
        ];
        let strip = Strip::from_segments(segments);
        let cropped = strip.crop(2, 7);
        assert_eq!(cropped.text(), "ABBBC");
    }

    #[test]
    fn strip_crop_out_of_bounds() {
        let strip = Strip::from_segment(Segment::new("Hello"));
        let cropped = strip.crop(10, 20);
        assert!(cropped.is_empty());
    }

    #[test]
    fn strip_divide() {
        let strip = Strip::from_segment(Segment::new("AABBCCDD"));
        let parts = strip.divide(&[2, 4, 6]);
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].text(), "AA");
        assert_eq!(parts[1].text(), "BB");
        assert_eq!(parts[2].text(), "CC");
        assert_eq!(parts[3].text(), "DD");
    }

    #[test]
    fn strip_apply_style() {
        let strip = Strip::from_segment(Segment::new("hello"));
        let style = Style {
            bold: true,
            ..Default::default()
        };
        let styled = strip.apply_style(&style);
        assert!(styled.segments()[0].style().unwrap().bold);
    }

    #[test]
    fn strip_simplify_merges_same_style() {
        let style = Style::with_fg(RgbaColor::rgb(255, 0, 0));
        let segments = vec![
            Segment::styled("aa", style.clone()),
            Segment::styled("bb", style.clone()),
            Segment::styled("cc", style.clone()),
        ];
        let strip = Strip::from_segments(segments);
        let simplified = strip.simplify();
        assert_eq!(simplified.segments().len(), 1);
        assert_eq!(simplified.text(), "aabbcc");
    }

    #[test]
    fn strip_simplify_preserves_different_styles() {
        let red = Style::with_fg(RgbaColor::rgb(255, 0, 0));
        let blue = Style::with_fg(RgbaColor::rgb(0, 0, 255));
        let segments = vec![
            Segment::styled("aa", red.clone()),
            Segment::styled("bb", blue.clone()),
            Segment::styled("cc", red.clone()),
        ];
        let strip = Strip::from_segments(segments);
        let simplified = strip.simplify();
        assert_eq!(simplified.segments().len(), 3);
    }

    #[test]
    fn strip_join() {
        let s1 = Strip::from_segment(Segment::new("Hello"));
        let s2 = Strip::from_segment(Segment::new(" "));
        let s3 = Strip::from_segment(Segment::new("World"));
        let joined = Strip::join([s1, s2, s3]);
        assert_eq!(joined.text(), "Hello World");
        assert_eq!(joined.cell_length(), 11);
    }

    #[test]
    fn strip_adjust_cell_length_pad() {
        let strip = Strip::from_segment(Segment::new("Hi"));
        let adjusted = strip.adjust_cell_length(5, None);
        assert_eq!(adjusted.cell_length(), 5);
        assert_eq!(adjusted.text(), "Hi   ");
    }

    #[test]
    fn strip_adjust_cell_length_crop() {
        let strip = Strip::from_segment(Segment::new("Hello World"));
        let adjusted = strip.adjust_cell_length(5, None);
        assert_eq!(adjusted.cell_length(), 5);
        assert_eq!(adjusted.text(), "Hello");
    }

    #[test]
    fn strip_adjust_cell_length_exact() {
        let strip = Strip::from_segment(Segment::new("Hello"));
        let adjusted = strip.adjust_cell_length(5, None);
        assert_eq!(adjusted.cell_length(), 5);
        assert_eq!(adjusted.text(), "Hello");
    }

    #[test]
    fn strip_text_align_left() {
        let strip = Strip::from_segment(Segment::new("Hi"));
        let aligned = strip.text_align(AlignHorizontal::Left, 5, None);
        assert_eq!(aligned.text(), "Hi   ");
    }

    #[test]
    fn strip_text_align_right() {
        let strip = Strip::from_segment(Segment::new("Hi"));
        let aligned = strip.text_align(AlignHorizontal::Right, 5, None);
        assert_eq!(aligned.text(), "   Hi");
    }

    #[test]
    fn strip_text_align_center() {
        let strip = Strip::from_segment(Segment::new("Hi"));
        let aligned = strip.text_align(AlignHorizontal::Center, 6, None);
        assert_eq!(aligned.text(), "  Hi  ");
    }

    #[test]
    fn strip_text_align_center_odd() {
        let strip = Strip::from_segment(Segment::new("Hi"));
        let aligned = strip.text_align(AlignHorizontal::Center, 5, None);
        // 3 extra spaces, split as 1 left, 2 right
        assert_eq!(aligned.text(), " Hi  ");
    }

    #[test]
    fn strip_text_align_too_wide() {
        let strip = Strip::from_segment(Segment::new("Hello World"));
        let aligned = strip.text_align(AlignHorizontal::Center, 5, None);
        assert_eq!(aligned.text(), "Hello");
    }

    // ==================== TINT TESTS ====================

    #[test]
    fn strip_apply_tint_colors_fg_and_bg() {
        // Create a strip with both fg and bg colors
        let style = Style::with_colors(
            RgbaColor::rgb(200, 200, 200), // light gray fg
            RgbaColor::rgb(0, 0, 100),     // dark blue bg
        );
        let strip = Strip::from_segment(Segment::styled("Hello", style));

        // Apply 40% magenta tint
        let tint = RgbaColor::rgba(255, 0, 255, 0.4);
        let tinted = strip.apply_tint(&tint);

        // Verify fg was tinted (should have more red/magenta)
        let seg = &tinted.segments()[0];
        let tinted_fg = seg.style().unwrap().fg.as_ref().unwrap();
        // Original fg: (200, 200, 200)
        // Tint: (255, 0, 255, 0.4)
        // Expected r: 200 + (255 - 200) * 0.4 = 200 + 22 = 222
        // Expected g: 200 + (0 - 200) * 0.4 = 200 - 80 = 120
        // Expected b: 200 + (255 - 200) * 0.4 = 200 + 22 = 222
        assert_eq!(tinted_fg.r, 222);
        assert_eq!(tinted_fg.g, 120);
        assert_eq!(tinted_fg.b, 222);

        // Verify bg was also tinted
        let tinted_bg = seg.style().unwrap().bg.as_ref().unwrap();
        // Original bg: (0, 0, 100)
        // Expected r: 0 + (255 - 0) * 0.4 = 102
        // Expected g: 0 + (0 - 0) * 0.4 = 0
        // Expected b: 100 + (255 - 100) * 0.4 = 100 + 62 = 162
        assert_eq!(tinted_bg.r, 102);
        assert_eq!(tinted_bg.g, 0);
        assert_eq!(tinted_bg.b, 162);
    }

    #[test]
    fn strip_apply_tint_zero_alpha_no_change() {
        let style = Style::with_fg(RgbaColor::rgb(100, 150, 200));
        let strip = Strip::from_segment(Segment::styled("Test", style));

        // Apply transparent tint (alpha = 0)
        let tint = RgbaColor::rgba(255, 0, 0, 0.0);
        let tinted = strip.apply_tint(&tint);

        // Colors should be unchanged
        let seg = &tinted.segments()[0];
        let fg = seg.style().unwrap().fg.as_ref().unwrap();
        assert_eq!(fg.r, 100);
        assert_eq!(fg.g, 150);
        assert_eq!(fg.b, 200);
    }

    #[test]
    fn strip_apply_tint_preserves_text() {
        let strip = Strip::from_segment(Segment::styled(
            "Preserved",
            Style::with_fg(RgbaColor::rgb(255, 255, 255)),
        ));
        let tint = RgbaColor::rgba(255, 0, 0, 0.5);
        let tinted = strip.apply_tint(&tint);

        // Text should be unchanged
        assert_eq!(tinted.text(), "Preserved");
        assert_eq!(tinted.cell_length(), 9);
    }

    #[test]
    fn strip_apply_tint_multiple_segments() {
        let segments = vec![
            Segment::styled("Red", Style::with_fg(RgbaColor::rgb(255, 0, 0))),
            Segment::styled("Green", Style::with_fg(RgbaColor::rgb(0, 255, 0))),
        ];
        let strip = Strip::from_segments(segments);

        // Apply blue tint
        let tint = RgbaColor::rgba(0, 0, 255, 0.5);
        let tinted = strip.apply_tint(&tint);

        // Both segments should be tinted
        assert_eq!(tinted.segments().len(), 2);

        // First segment: red (255, 0, 0) + 50% blue = (127, 0, 127) with truncation
        let fg1 = tinted.segments()[0].style().unwrap().fg.as_ref().unwrap();
        assert_eq!(fg1.r, 127);
        assert_eq!(fg1.g, 0);
        assert_eq!(fg1.b, 127);

        // Second segment: green (0, 255, 0) + 50% blue = (0, 127, 127) with truncation
        let fg2 = tinted.segments()[1].style().unwrap().fg.as_ref().unwrap();
        assert_eq!(fg2.r, 0);
        assert_eq!(fg2.g, 127);
        assert_eq!(fg2.b, 127);
    }
}
