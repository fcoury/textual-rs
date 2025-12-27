# Test Plan: Stage 9 (Layout Foundations) & Stage 10 (Scrollbar)

## Current State

**Existing tests:** 14 total in textual crate
- tree.rs: 2 (focus path)
- message.rs: 3 (envelope)
- tests/runtime_compat.rs: 1
- tests/message_pump.rs: 8

**Missing:** No tests for canvas, region, containers, or widgets.

---

## Stage 9: Layout Foundations Tests

### Unit Tests: Region (`canvas.rs`)

**File:** `crates/textual/src/canvas.rs` (add `#[cfg(test)]` module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // === Region::new ===

    #[test]
    fn region_new_basic() {
        let r = Region::new(10, 20, 100, 50);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 50);
    }

    #[test]
    fn region_new_clamps_negative_dimensions() {
        let r = Region::new(0, 0, -10, -20);
        assert_eq!(r.width, 0);
        assert_eq!(r.height, 0);
    }

    #[test]
    fn region_new_allows_negative_position() {
        let r = Region::new(-10, -20, 100, 50);
        assert_eq!(r.x, -10);
        assert_eq!(r.y, -20);
    }

    // === Region::from_u16 ===

    #[test]
    fn region_from_u16_converts_correctly() {
        let r = Region::from_u16(10, 20, 100, 50);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 50);
    }

    // === Region::intersection ===

    #[test]
    fn intersection_overlapping() {
        let a = Region::new(0, 0, 100, 100);
        let b = Region::new(50, 50, 100, 100);
        let i = a.intersection(&b);
        assert_eq!(i, Region::new(50, 50, 50, 50));
    }

    #[test]
    fn intersection_contained() {
        let outer = Region::new(0, 0, 100, 100);
        let inner = Region::new(25, 25, 50, 50);
        let i = outer.intersection(&inner);
        assert_eq!(i, inner);
    }

    #[test]
    fn intersection_no_overlap() {
        let a = Region::new(0, 0, 50, 50);
        let b = Region::new(100, 100, 50, 50);
        let i = a.intersection(&b);
        assert!(i.is_empty());
    }

    #[test]
    fn intersection_touching_edge() {
        let a = Region::new(0, 0, 50, 50);
        let b = Region::new(50, 0, 50, 50);
        let i = a.intersection(&b);
        assert!(i.is_empty()); // Touching but not overlapping
    }

    #[test]
    fn intersection_with_negative_coords() {
        let a = Region::new(-50, -50, 100, 100);
        let b = Region::new(0, 0, 100, 100);
        let i = a.intersection(&b);
        assert_eq!(i, Region::new(0, 0, 50, 50));
    }

    // === Region::contains_point ===

    #[test]
    fn contains_point_inside() {
        let r = Region::new(10, 10, 50, 50);
        assert!(r.contains_point(30, 30));
    }

    #[test]
    fn contains_point_on_edge() {
        let r = Region::new(10, 10, 50, 50);
        assert!(r.contains_point(10, 10)); // Top-left is inside
        assert!(!r.contains_point(60, 60)); // Bottom-right is outside (exclusive)
    }

    #[test]
    fn contains_point_outside() {
        let r = Region::new(10, 10, 50, 50);
        assert!(!r.contains_point(0, 0));
        assert!(!r.contains_point(100, 100));
    }

    #[test]
    fn contains_point_negative_region() {
        let r = Region::new(-50, -50, 100, 100);
        assert!(r.contains_point(-25, -25));
        assert!(r.contains_point(0, 0));
        assert!(!r.contains_point(50, 50)); // Just outside
    }

    // === Region::is_empty ===

    #[test]
    fn is_empty_zero_width() {
        let r = Region::new(0, 0, 0, 100);
        assert!(r.is_empty());
    }

    #[test]
    fn is_empty_zero_height() {
        let r = Region::new(0, 0, 100, 0);
        assert!(r.is_empty());
    }

    #[test]
    fn is_empty_both_zero() {
        let r = Region::new(0, 0, 0, 0);
        assert!(r.is_empty());
    }

    #[test]
    fn is_empty_has_area() {
        let r = Region::new(0, 0, 1, 1);
        assert!(!r.is_empty());
    }
}
```

### Unit Tests: Canvas Clipping

```rust
#[cfg(test)]
mod canvas_tests {
    use super::*;

    // === Clipping Stack ===

    #[test]
    fn canvas_default_clip_is_full_screen() {
        let canvas = Canvas::new(80, 24);
        // current_clip is private, so test via put_char behavior
        // Characters at edges should be drawn
        let mut canvas = Canvas::new(80, 24);
        canvas.put_char(0, 0, 'X', None, None);
        canvas.put_char(79, 23, 'Y', None, None);
        // No panic = success
    }

    #[test]
    fn canvas_push_clip_restricts_drawing() {
        let mut canvas = Canvas::new(80, 24);
        canvas.push_clip(Region::new(10, 10, 20, 10));

        // Inside clip - should draw
        canvas.put_char(15, 15, 'A', None, None);

        // Outside clip - should be clipped (no effect)
        canvas.put_char(0, 0, 'B', None, None);
        canvas.put_char(50, 15, 'C', None, None);
    }

    #[test]
    fn canvas_nested_clips_intersect() {
        let mut canvas = Canvas::new(80, 24);
        canvas.push_clip(Region::new(0, 0, 50, 50));
        canvas.push_clip(Region::new(25, 25, 50, 50));

        // Effective clip should be (25, 25, 25, 25)
        canvas.put_char(30, 30, 'A', None, None); // Inside intersection
        canvas.put_char(10, 10, 'B', None, None); // Outside (in first but not second)
    }

    #[test]
    fn canvas_pop_clip_restores_previous() {
        let mut canvas = Canvas::new(80, 24);
        canvas.push_clip(Region::new(0, 0, 50, 50));
        canvas.push_clip(Region::new(10, 10, 10, 10));
        canvas.pop_clip();

        // Should be back to first clip
        canvas.put_char(5, 5, 'A', None, None); // Should work now
    }

    #[test]
    fn canvas_clear_resets_clip_stack() {
        let mut canvas = Canvas::new(80, 24);
        canvas.push_clip(Region::new(10, 10, 10, 10));
        canvas.push_clip(Region::new(15, 15, 5, 5));
        canvas.clear();

        // Clip stack should be empty, full screen available
        canvas.put_char(0, 0, 'A', None, None);
    }

    // === Clipping with Negative Coordinates ===

    #[test]
    fn canvas_clips_negative_coordinates() {
        let mut canvas = Canvas::new(80, 24);
        // Try to draw at negative position - should be clipped
        canvas.put_char(-5, -5, 'X', None, None);
        // No panic = success
    }

    #[test]
    fn canvas_put_str_clips_partial() {
        let mut canvas = Canvas::new(80, 24);
        canvas.push_clip(Region::new(5, 0, 10, 24));

        // String starting before clip should have first chars clipped
        canvas.put_str(0, 0, "Hello World", None, None);
        // Only chars at positions 5-14 should be drawn
    }
}
```

### Integration Tests: Container Hit Testing

**File:** `crates/textual/tests/hit_testing.rs`

```rust
//! Integration tests for mouse hit testing with i32 coordinates

use textual::{Region, MouseEvent, MouseEventKind, MouseButton};

mod mock_widget {
    // ... mock widget implementation for testing
}

#[test]
fn vertical_container_routes_mouse_to_correct_child() {
    // Create vertical with 3 children of height 10 each
    // Click at y=25 should route to child 2 (index 2)
}

#[test]
fn nested_containers_propagate_mouse_correctly() {
    // Vertical [ Horizontal [ Widget, Widget ] ]
    // Click should route through both containers
}

#[test]
fn mouse_outside_container_returns_none() {
    // Click outside container bounds
}
```

---

## Stage 10: Scrollbar Tests

### Unit Tests: ScrollbarChrome (`chrome.rs`)

**File:** `crates/textual/src/chrome.rs` (add `#[cfg(test)]` module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // === Glyph Constants ===

    #[test]
    fn vertical_glyphs_correct() {
        assert_eq!(ScrollbarGlyphs::VERTICAL_ENDS.len(), 8);
        assert_eq!(ScrollbarGlyphs::VERTICAL_ENDS[0], '▁');
        assert_eq!(ScrollbarGlyphs::VERTICAL_ENDS[7], ' ');
    }

    #[test]
    fn horizontal_glyphs_correct() {
        assert_eq!(ScrollbarGlyphs::HORIZONTAL_ENDS.len(), 8);
        assert_eq!(ScrollbarGlyphs::HORIZONTAL_ENDS[0], '▉');
        assert_eq!(ScrollbarGlyphs::HORIZONTAL_ENDS[7], ' ');
    }

    // === Thumb Calculation ===

    #[test]
    fn thumb_size_proportional_to_viewport() {
        // viewport=50, content=100 -> thumb should be ~50% of track
        // This tests the math without rendering
    }

    #[test]
    fn thumb_position_at_start() {
        // scroll_offset=0 -> thumb at top
    }

    #[test]
    fn thumb_position_at_end() {
        // scroll_offset=max -> thumb at bottom
    }

    #[test]
    fn thumb_position_middle() {
        // scroll_offset=50% -> thumb in middle
    }

    #[test]
    fn thumb_minimum_size_is_one() {
        // Very large content should still have thumb of at least 1 cell
    }

    // === Rendering ===

    #[test]
    fn render_vertical_draws_track() {
        let mut canvas = Canvas::new(80, 24);
        let region = Region::new(79, 0, 1, 24);

        ScrollbarChrome::render_vertical(
            &mut canvas,
            region,
            0,      // scroll_offset
            100,    // content_size
            24,     // viewport_size
            RgbaColor::rgb(255, 0, 255),
            RgbaColor::rgb(85, 85, 85),
        );

        // Verify track was drawn (check canvas cells)
    }

    #[test]
    fn render_vertical_no_thumb_when_content_fits() {
        // content_size <= viewport_size -> no thumb, just track
    }

    #[test]
    fn render_horizontal_draws_track() {
        // Similar to vertical but horizontal
    }
}
```

### Unit Tests: CSS Scrollbar Properties (`tcss`)

**File:** `crates/tcss/tests/scrollbar_parsing.rs`

```rust
//! Tests for scrollbar CSS property parsing

use tcss::parser::parse_stylesheet;

#[test]
fn parse_scrollbar_color() {
    let css = r#"
        Widget {
            scrollbar-color: #ff00ff;
        }
    "#;
    let stylesheet = parse_stylesheet(css).unwrap();
    // Verify scrollbar_color is set
}

#[test]
fn parse_scrollbar_background() {
    let css = r#"
        Widget {
            scrollbar-background: #555555;
        }
    "#;
    let stylesheet = parse_stylesheet(css).unwrap();
}

#[test]
fn parse_scrollbar_size() {
    let css = r#"
        Widget {
            scrollbar-size: 2;
        }
    "#;
    let stylesheet = parse_stylesheet(css).unwrap();
}

#[test]
fn parse_all_scrollbar_properties() {
    let css = r#"
        Widget {
            scrollbar-color: red;
            scrollbar-color-hover: green;
            scrollbar-color-active: blue;
            scrollbar-background: #333;
            scrollbar-background-hover: #444;
            scrollbar-background-active: #555;
            scrollbar-corner-color: #666;
            scrollbar-size: 1;
        }
    "#;
    let stylesheet = parse_stylesheet(css).unwrap();
}
```

### Integration Tests: Scrolling Behavior

**File:** `crates/textual/tests/scrolling.rs`

```rust
//! Integration tests for scrolling containers

use textual::{Vertical, Widget, Region, MouseEvent, MouseEventKind};

// Helper to create a vertical with many children
fn create_tall_vertical() -> Vertical<()> {
    // 20 items, each height 3 = total height 60
    // In viewport of 24, needs scrolling
}

#[test]
fn scroll_down_increases_offset() {
    let mut container = create_tall_vertical();
    let region = Region::from_u16(0, 0, 80, 24);

    // Initial scroll is 0
    assert_eq!(container.scroll_y, 0);

    // Send scroll down event
    let event = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 40,
        row: 12,
        modifiers: Default::default(),
    };
    container.on_mouse(event, region);

    assert!(container.scroll_y > 0);
}

#[test]
fn scroll_up_decreases_offset() {
    let mut container = create_tall_vertical();
    container.scroll_y = 10;

    let event = MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 40,
        row: 12,
        modifiers: Default::default(),
    };
    let region = Region::from_u16(0, 0, 80, 24);
    container.on_mouse(event, region);

    assert!(container.scroll_y < 10);
}

#[test]
fn scroll_clamps_to_max() {
    let mut container = create_tall_vertical();

    // Scroll way past end
    for _ in 0..100 {
        let event = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 40,
            row: 12,
            modifiers: Default::default(),
        };
        container.on_mouse(event, Region::from_u16(0, 0, 80, 24));
    }

    // Should be clamped to max_scroll
    let content_height = 60; // 20 items * 3
    let viewport = 24;
    let max_scroll = content_height - viewport;
    assert_eq!(container.scroll_y, max_scroll);
}

#[test]
fn scroll_clamps_to_zero() {
    let mut container = create_tall_vertical();
    container.scroll_y = 5;

    // Scroll up more than current offset
    for _ in 0..10 {
        let event = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 40,
            row: 12,
            modifiers: Default::default(),
        };
        container.on_mouse(event, Region::from_u16(0, 0, 80, 24));
    }

    assert_eq!(container.scroll_y, 0);
}

#[test]
fn click_routes_to_scrolled_child() {
    // With scroll_y = 10, clicking at visual y=5 should hit
    // the child that's actually at y=15 in virtual space
}

#[test]
fn scrollbar_visible_when_overflow_auto_and_content_exceeds() {
    // Test that scrollbar appears when content > viewport
}

#[test]
fn scrollbar_hidden_when_content_fits() {
    // Test that scrollbar doesn't appear when content <= viewport
}
```

### Visual Tests (Manual)

**File:** `crates/textual/examples/scroll_test.rs`

An interactive example to manually verify:
1. Scrollbar thumb position matches content position
2. Scrollbar thumb size proportional to visible portion
3. Content clips properly at container edges
4. Mouse clicks work correctly on scrolled content
5. Scrollbar colors from CSS are applied

---

## Test Priority

| Priority | Category | Tests | Notes |
|----------|----------|-------|-------|
| **P0** | Region methods | 15+ | Core foundation, must work |
| **P0** | Canvas clipping | 8+ | Critical for scrolling |
| **P1** | Scrollbar math | 5+ | Thumb position/size |
| **P1** | Scroll events | 6+ | Scroll up/down clamping |
| **P2** | CSS parsing | 8+ | Scrollbar properties |
| **P2** | Integration | 5+ | End-to-end behavior |

---

## Running Tests

```bash
# All tests
cargo test

# Just canvas tests
cargo test --lib canvas

# Just scrollbar tests (after Stage 10)
cargo test --lib chrome
cargo test --test scrolling

# With output
cargo test -- --nocapture
```

---

## Coverage Goals

| Module | Current | Target |
|--------|---------|--------|
| canvas.rs (Region) | 0% | 90%+ |
| canvas.rs (Canvas) | 0% | 80%+ |
| chrome.rs | N/A | 90%+ |
| containers/vertical.rs | 0% | 70%+ |
| containers/horizontal.rs | 0% | 70%+ |
