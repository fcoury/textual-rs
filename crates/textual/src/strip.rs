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
use tcss::types::text::AlignHorizontal;

/// An immutable horizontal line of segments.
///
/// Strips are the rendering primitive for a single line of output.
/// They track their cell length for efficient layout operations.
#[derive(Clone, Debug, Default)]
pub struct Strip {
    /// The segments that make up this line.
    segments: Vec<Segment>,
    /// Cached total cell width.
    cell_length: usize,
}

impl Strip {
    /// Creates an empty strip.
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            cell_length: 0,
        }
    }

    /// Creates a strip from a vector of segments.
    pub fn from_segments(segments: Vec<Segment>) -> Self {
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
            segments: vec![segment],
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

    /// Extracts a portion of the strip from `start` to `end` cell positions.
    ///
    /// The resulting strip contains cells in the range [start, end).
    pub fn crop(&self, start: usize, end: usize) -> Strip {
        if start >= end || start >= self.cell_length {
            return Strip::new();
        }

        let end = end.min(self.cell_length);
        let mut result_segments = Vec::new();
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

        Strip::from_segments(result_segments)
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

        let mut result = Vec::new();
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

        Strip::from_segments(result)
    }

    /// Concatenates multiple strips into one.
    pub fn join(strips: impl IntoIterator<Item = Strip>) -> Strip {
        let mut segments = Vec::new();
        for strip in strips {
            segments.extend(strip.segments);
        }
        Strip::from_segments(segments)
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
                Strip::from_segments(segments)
            }
        }
    }

    /// Aligns text within a given width.
    ///
    /// Returns a new strip of exactly `width` cells with the content aligned
    /// according to the specified alignment.
    pub fn text_align(&self, align: AlignHorizontal, width: usize, pad_style: Option<Style>) -> Strip {
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
                Strip::from_segments(segments)
            }
            AlignHorizontal::Right => {
                // Padding at start, content at end
                let padding = Segment::blank(gap, pad_style);
                let mut segments = vec![padding];
                segments.extend(self.segments.clone());
                Strip::from_segments(segments)
            }
            AlignHorizontal::Center => {
                // Split padding between start and end
                let left_pad = gap / 2;
                let right_pad = gap - left_pad;

                let mut segments = Vec::new();
                if left_pad > 0 {
                    segments.push(Segment::blank(left_pad, pad_style.clone()));
                }
                segments.extend(self.segments.clone());
                if right_pad > 0 {
                    segments.push(Segment::blank(right_pad, pad_style));
                }
                Strip::from_segments(segments)
            }
        }
    }
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
        let segments = vec![Segment::new("hello"), Segment::new(" "), Segment::new("world")];
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
        let segments = vec![Segment::new("AAA"), Segment::new("BBB"), Segment::new("CCC")];
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
}
