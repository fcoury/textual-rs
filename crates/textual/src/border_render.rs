//! Border row rendering.
//!
//! This module provides the `render_row` function for assembling horizontal
//! border lines with optional labels (titles/subtitles).

use crate::border_box::BoxSegments;
use crate::segment::{Segment, Style};
use crate::strip::Strip;

/// Renders a horizontal border row with optional labels.
///
/// Assembles: left_corner + (label or fill) + right_corner
///
/// The `width` parameter is the total width including corners.
/// The fill character is repeated to fill the space between corners.
///
/// # Arguments
///
/// * `box_segments` - The (left, fill, right) segments for this row
/// * `width` - Total width of the row in cells
/// * `title` - Optional title strip to center in the top border
/// * `subtitle` - Optional subtitle strip (typically for bottom border)
///
/// # Example
///
/// ```ignore
/// let row = render_row(&box_segs[0], 20, Some(&title), None);
/// // Produces: "╭── Title ──────╮"
/// ```
pub fn render_row(
    box_segments: &BoxSegments,
    width: usize,
    title: Option<&Strip>,
    subtitle: Option<&Strip>,
) -> Strip {
    let (left, fill, right) = box_segments;

    if width <= 2 {
        // Too narrow for content, just corners
        if width == 2 {
            return Strip::from_segments(vec![left.clone(), right.clone()]);
        } else if width == 1 {
            return Strip::from_segment(left.clone());
        } else {
            return Strip::new();
        }
    }

    let inner_width = width - 2; // Space between left and right corners

    // Determine the content to place in the middle
    let label = title.or(subtitle);

    let middle = if let Some(label_strip) = label {
        // Center the label within the inner width
        render_label_in_row(label_strip, fill, inner_width)
    } else {
        // Just fill with the fill character
        repeat_segment(fill, inner_width)
    };

    // Assemble: left + middle + right
    let mut segments = vec![left.clone()];
    segments.extend(middle.segments().iter().cloned());
    segments.push(right.clone());

    Strip::from_segments(segments)
}

/// Renders a label centered within a row, with fill characters on either side.
fn render_label_in_row(label: &Strip, fill: &Segment, width: usize) -> Strip {
    let label_len = label.cell_length();

    if label_len >= width {
        // Label is too long, crop it
        return label.crop(0, width);
    }

    // Calculate padding on each side
    let total_padding = width - label_len;
    let left_padding = total_padding / 2;
    let right_padding = total_padding - left_padding;

    // Build the result
    let mut segments = Vec::new();

    // Left padding (fill characters)
    if left_padding > 0 {
        segments.push(repeat_char_segment(fill, left_padding));
    }

    // The label
    segments.extend(label.segments().iter().cloned());

    // Right padding (fill characters)
    if right_padding > 0 {
        segments.push(repeat_char_segment(fill, right_padding));
    }

    Strip::from_segments(segments)
}

/// Creates a segment by repeating the fill segment's character.
fn repeat_segment(fill: &Segment, count: usize) -> Strip {
    if count == 0 {
        return Strip::new();
    }

    let fill_char = fill.text().chars().next().unwrap_or(' ');
    let text: String = std::iter::repeat(fill_char).take(count).collect();
    let segment = match fill.style() {
        Some(s) => Segment::styled(text, s.clone()),
        None => Segment::new(text),
    };

    Strip::from_segment(segment)
}

/// Creates a single segment by repeating the fill character.
fn repeat_char_segment(fill: &Segment, count: usize) -> Segment {
    if count == 0 {
        return Segment::new("");
    }

    let fill_char = fill.text().chars().next().unwrap_or(' ');
    let text: String = std::iter::repeat(fill_char).take(count).collect();

    match fill.style() {
        Some(s) => Segment::styled(text, s.clone()),
        None => Segment::new(text),
    }
}

/// Renders a middle row (left border + padding + content + padding + right border).
///
/// This is used for rows between the top and bottom borders.
pub fn render_middle_row(
    box_segments: &BoxSegments,
    content: Option<&Strip>,
    width: usize,
    pad_style: Option<Style>,
    padding_left: usize,
    padding_right: usize,
) -> Strip {
    let (left, _fill, right) = box_segments;

    if width <= 2 {
        if width == 2 {
            return Strip::from_segments(vec![left.clone(), right.clone()]);
        } else if width == 1 {
            return Strip::from_segment(left.clone());
        } else {
            return Strip::new();
        }
    }

    let inner_width = width - 2;

    // Calculate content width after accounting for horizontal padding
    let content_width = inner_width.saturating_sub(padding_left + padding_right);

    let mut segments = vec![left.clone()];

    // Add left padding
    if padding_left > 0 {
        segments.push(Segment::blank(padding_left, pad_style.clone()));
    }

    // Add content (or blank if no content)
    let content_strip = match content {
        Some(strip) => strip.adjust_cell_length(content_width, pad_style.clone()),
        None => Strip::blank(content_width, pad_style.clone()),
    };
    segments.extend(content_strip.segments().iter().cloned());

    // Add right padding
    if padding_right > 0 {
        segments.push(Segment::blank(padding_right, pad_style));
    }

    segments.push(right.clone());

    Strip::from_segments(segments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tcss::types::RgbaColor;

    fn make_round_top() -> BoxSegments {
        let style = Style::with_fg(RgbaColor::white());
        (
            Segment::styled("╭", style.clone()),
            Segment::styled("─", style.clone()),
            Segment::styled("╮", style),
        )
    }

    fn make_round_middle() -> BoxSegments {
        let style = Style::with_fg(RgbaColor::white());
        (
            Segment::styled("│", style.clone()),
            Segment::styled(" ", style.clone()),
            Segment::styled("│", style),
        )
    }

    #[test]
    fn render_row_basic() {
        let top = make_round_top();
        let row = render_row(&top, 10, None, None);
        assert_eq!(row.text(), "╭────────╮");
        assert_eq!(row.cell_length(), 10);
    }

    #[test]
    fn render_row_with_title() {
        let top = make_round_top();
        let title = Strip::from_segment(Segment::new("Hi"));
        let row = render_row(&top, 10, Some(&title), None);
        // "╭───Hi───╮" with centered title
        assert_eq!(row.cell_length(), 10);
        assert!(row.text().contains("Hi"));
    }

    #[test]
    fn render_row_width_2() {
        let top = make_round_top();
        let row = render_row(&top, 2, None, None);
        assert_eq!(row.text(), "╭╮");
    }

    #[test]
    fn render_row_width_1() {
        let top = make_round_top();
        let row = render_row(&top, 1, None, None);
        assert_eq!(row.text(), "╭");
    }

    #[test]
    fn render_middle_row_basic() {
        let middle = make_round_middle();
        let row = render_middle_row(&middle, None, 10, None, 0, 0);
        assert_eq!(row.text(), "│        │");
        assert_eq!(row.cell_length(), 10);
    }

    #[test]
    fn render_middle_row_with_content() {
        let middle = make_round_middle();
        let content = Strip::from_segment(Segment::new("Hello"));
        let row = render_middle_row(&middle, Some(&content), 10, None, 0, 0);
        assert_eq!(row.text(), "│Hello   │");
    }

    #[test]
    fn render_middle_row_with_padding() {
        let middle = make_round_middle();
        let content = Strip::from_segment(Segment::new("Hi"));
        let row = render_middle_row(&middle, Some(&content), 10, None, 2, 2);
        // Width 10: │(1) + padding_left(2) + content(4, "Hi" + 2 spaces) + padding_right(2) + │(1)
        // Content area is 4 cells (inner_width 8 - padding 4), "Hi" is 2 cells, padded to 4
        assert_eq!(row.text(), "│  Hi    │");
        assert_eq!(row.cell_length(), 10);
    }

    #[test]
    fn render_middle_row_padding_no_content() {
        let middle = make_round_middle();
        let row = render_middle_row(&middle, None, 10, None, 2, 2);
        // │ + 2 padding + 4 blank + 2 padding + │
        assert_eq!(row.text(), "│        │");
        assert_eq!(row.cell_length(), 10);
    }
}
