//! Border box caching and segment generation.
//!
//! This module provides the `get_box` function which returns cached border
//! segments for rendering. The segments are organized as 3 rows (top, middle,
//! bottom), each containing 3 segments (left corner/edge, fill, right corner/edge).

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::border_chars::{get_border_chars, get_border_locations};
use crate::segment::{Segment, Style};

/// A row of border segments: (left, fill, right)
pub type BoxSegments = (Segment, Segment, Segment);

/// Cache key for border box lookups.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
struct BoxCacheKey {
    border_type: String,
    inner_style_key: StyleKey,
    outer_style_key: StyleKey,
}

/// Simplified style key for caching (colors only, not all attributes).
#[derive(Hash, PartialEq, Eq, Clone, Debug, Default)]
struct StyleKey {
    fg: Option<(u8, u8, u8)>,
    bg: Option<(u8, u8, u8)>,
}

impl From<&Style> for StyleKey {
    fn from(style: &Style) -> Self {
        Self {
            fg: style.fg.as_ref().map(|c| (c.r, c.g, c.b)),
            bg: style.bg.as_ref().map(|c| (c.r, c.g, c.b)),
        }
    }
}

/// Global cache for border box segments.
static BOX_CACHE: Lazy<RwLock<HashMap<BoxCacheKey, [BoxSegments; 3]>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Gets cached border segments for a border type with the given styles.
///
/// Returns an array of 3 BoxSegments (one per row: top, middle, bottom).
/// Each BoxSegments is a tuple of (left, fill, right) segments.
///
/// The `inner_style` is used for the widget's content area.
/// The `outer_style` is used for border elements that should use the parent's style
/// (only relevant for "inner" and "outer" border types).
///
/// # Example
///
/// ```ignore
/// let box_segs = get_box("round", &inner_style, &outer_style);
/// let (top_left, top_fill, top_right) = &box_segs[0];
/// let (mid_left, mid_fill, mid_right) = &box_segs[1];
/// let (bot_left, bot_fill, bot_right) = &box_segs[2];
/// ```
pub fn get_box(border_type: &str, inner_style: &Style, outer_style: &Style) -> [BoxSegments; 3] {
    let key = BoxCacheKey {
        border_type: border_type.to_string(),
        inner_style_key: StyleKey::from(inner_style),
        outer_style_key: StyleKey::from(outer_style),
    };

    // Try read lock first
    {
        let cache = BOX_CACHE.read().unwrap();
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    // Generate and cache
    let result = generate_box(border_type, inner_style, outer_style);

    {
        let mut cache = BOX_CACHE.write().unwrap();
        cache.insert(key, result.clone());
    }

    result
}

/// Generates border segments for a border type.
fn generate_box(border_type: &str, inner_style: &Style, outer_style: &Style) -> [BoxSegments; 3] {
    let chars = get_border_chars(border_type);
    let locations = get_border_locations(border_type);

    let mut rows: [BoxSegments; 3] = [
        default_box_segments(),
        default_box_segments(),
        default_box_segments(),
    ];

    for row_idx in 0..3 {
        let left_char = chars[row_idx][0];
        let fill_char = chars[row_idx][1];
        let right_char = chars[row_idx][2];

        let left_loc = locations[row_idx][0];
        let fill_loc = locations[row_idx][1];
        let right_loc = locations[row_idx][2];

        let left_style = if left_loc == 0 { inner_style } else { outer_style };
        let fill_style = if fill_loc == 0 { inner_style } else { outer_style };
        let right_style = if right_loc == 0 { inner_style } else { outer_style };

        rows[row_idx] = (
            Segment::styled(left_char.to_string(), left_style.clone()),
            Segment::styled(fill_char.to_string(), fill_style.clone()),
            Segment::styled(right_char.to_string(), right_style.clone()),
        );
    }

    rows
}

/// Clears the border box cache.
///
/// This should be called when the theme changes or styles are updated globally.
pub fn clear_cache() {
    let mut cache = BOX_CACHE.write().unwrap();
    cache.clear();
}

/// Creates a default BoxSegments tuple.
pub fn default_box_segments() -> BoxSegments {
    (Segment::default(), Segment::default(), Segment::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tcss::types::RgbaColor;

    #[test]
    fn get_box_round() {
        let style = Style::with_fg(RgbaColor::white());
        let box_segs = get_box("round", &style, &style);

        // Top row
        assert_eq!(box_segs[0].0.text(), "╭");
        assert_eq!(box_segs[0].1.text(), "─");
        assert_eq!(box_segs[0].2.text(), "╮");

        // Middle row
        assert_eq!(box_segs[1].0.text(), "│");
        assert_eq!(box_segs[1].1.text(), " ");
        assert_eq!(box_segs[1].2.text(), "│");

        // Bottom row
        assert_eq!(box_segs[2].0.text(), "╰");
        assert_eq!(box_segs[2].1.text(), "─");
        assert_eq!(box_segs[2].2.text(), "╯");
    }

    #[test]
    fn get_box_caches() {
        let style = Style::with_fg(RgbaColor::rgb(255, 0, 0));

        // Clear cache first
        clear_cache();

        // First call should generate
        let _box1 = get_box("solid", &style, &style);

        // Check cache has entry
        let cache = BOX_CACHE.read().unwrap();
        assert!(!cache.is_empty());
    }

    #[test]
    fn get_box_different_styles_different_cache() {
        let red = Style::with_fg(RgbaColor::rgb(255, 0, 0));
        let blue = Style::with_fg(RgbaColor::rgb(0, 0, 255));

        clear_cache();
        let initial_len = BOX_CACHE.read().unwrap().len();

        let _box1 = get_box("solid", &red, &red);
        let _box2 = get_box("solid", &blue, &blue);

        let final_len = BOX_CACHE.read().unwrap().len();
        // We should have added at least 2 entries (may be more if tests run in parallel)
        assert!(final_len >= initial_len + 2, "Expected at least 2 new cache entries");
    }
}
