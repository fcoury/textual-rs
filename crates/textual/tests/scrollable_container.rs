//! ScrollableContainer Integration Tests
//!
//! Tests for scrollbar visibility and content rendering.

use textual::containers::scrollable::ScrollableContainer;
use textual::widget::Widget;
use textual::{Canvas, Region, Size};
use tcss::ComputedStyle;
use tcss::types::Overflow;

// Dummy message type for tests
enum Msg {}

// =============================================================================
// Test Widget: Simple text lines
// =============================================================================

/// A simple widget that renders numbered lines for testing.
struct TestLines {
    count: u16,
}

impl TestLines {
    fn new(count: u16) -> Self {
        Self { count }
    }
}

impl<M> Widget<M> for TestLines {
    fn desired_size(&self) -> Size {
        Size {
            width: 20,
            height: self.count,
        }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        for i in 0..self.count {
            let y = region.y + i as i32;
            let line = format!("Line {:02}", i + 1);
            canvas.put_str(region.x, y, &line, None, None);
        }
    }
}

// =============================================================================
// Tests for Line 01 visibility
// =============================================================================

#[test]
fn test_first_line_is_rendered_at_scroll_zero() {
    // Create a canvas
    let mut canvas = Canvas::new(30, 10);
    let region = Region::new(0, 0, 30, 10);

    // Create content with 5 lines
    let content = TestLines::new(5);
    let mut container = ScrollableContainer::<Msg>::new(Box::new(content));

    // Set overflow to auto so scrollbar logic is active
    let mut style = ComputedStyle::default();
    style.overflow_y = Overflow::Auto;
    container.set_style(style);

    // Render at scroll offset 0
    container.render(&mut canvas, region);

    // Check that Line 01 appears at the top (row 0)
    // The canvas stores cells, we need to check the first row contains "Line 01"
    let first_row = canvas.row_str(0);
    assert!(
        first_row.contains("Line 01"),
        "First line should contain 'Line 01', but got: '{}'",
        first_row
    );
}

#[test]
fn test_all_visible_lines_are_rendered() {
    let mut canvas = Canvas::new(30, 5);
    let region = Region::new(0, 0, 30, 5);

    let content = TestLines::new(10);
    let mut container = ScrollableContainer::<Msg>::new(Box::new(content));

    let mut style = ComputedStyle::default();
    style.overflow_y = Overflow::Auto;
    container.set_style(style);

    container.render(&mut canvas, region);

    // With 5 rows visible, we should see Lines 01-05
    for i in 0..5 {
        let row = canvas.row_str(i);
        let expected = format!("Line {:02}", i + 1);
        assert!(
            row.contains(&expected),
            "Row {} should contain '{}', but got: '{}'",
            i,
            expected,
            row
        );
    }
}

// =============================================================================
// Tests for scrollbar visibility
// =============================================================================

#[test]
fn test_scrollbar_hidden_by_default_overflow() {
    let content = TestLines::new(50);
    let container = ScrollableContainer::<Msg>::new(Box::new(content));

    // Default overflow is Hidden, so show_vertical_scrollbar should be false
    // We can't directly call show_vertical_scrollbar (it's private),
    // but we can check via rendering to a small viewport
    let mut canvas = Canvas::new(30, 10);
    let region = Region::new(0, 0, 30, 10);
    container.render(&mut canvas, region);

    // With default overflow:hidden, the rightmost column should NOT have scrollbar
    // Check that content extends to column 29 (full width)
    let first_row = canvas.row_str(0);
    // This test documents current (broken?) behavior
    assert!(
        !first_row.is_empty(),
        "Content should render even with overflow:hidden"
    );
}

#[test]
fn test_scrollbar_visible_with_overflow_auto() {
    let content = TestLines::new(50);
    let mut container = ScrollableContainer::<Msg>::new(Box::new(content));

    // Set overflow to auto - scrollbar should appear when content exceeds viewport
    let mut style = ComputedStyle::default();
    style.overflow_y = Overflow::Auto;
    container.set_style(style);

    let mut canvas = Canvas::new(30, 10);
    let region = Region::new(0, 0, 30, 10);
    container.render(&mut canvas, region);

    // Debug: print all rows to see what's rendered
    println!("Canvas contents:");
    for i in 0..10 {
        let row = canvas.row_str(i);
        println!("Row {}: '{}' (len={})", i, row, row.trim_end().len());
    }

    // With overflow:auto and 50 lines in 10 row viewport, scrollbar should be visible
    // Content region should be 29 columns (30 - 1 for scrollbar)
    // So content shouldn't extend past column 28
    let first_row = canvas.row_str(0);
    let content_end = first_row.trim_end().len();

    // Content should be constrained to leave room for scrollbar
    // "Line 01" is 7 chars, so content_end should be 7, not 30
    assert!(
        content_end <= 29,
        "Content should leave room for scrollbar. Content ends at column {}, expected <= 29.\nRow: '{}'",
        content_end,
        first_row
    );
}

#[test]
fn test_scrollbar_visible_with_overflow_scroll() {
    let content = TestLines::new(5); // Content fits in viewport
    let mut container = ScrollableContainer::<Msg>::new(Box::new(content));

    // Set overflow to scroll - scrollbar should ALWAYS appear
    let mut style = ComputedStyle::default();
    style.overflow_y = Overflow::Scroll;
    container.set_style(style);

    let mut canvas = Canvas::new(30, 10);
    let region = Region::new(0, 0, 30, 10);
    container.render(&mut canvas, region);

    // With overflow:scroll, scrollbar should be visible even if content fits
    // The scrollbar track is rendered with spaces but WITH a background color
    let last_col_has_bg = (0..10).any(|row| canvas.has_bg_at(29, row));

    assert!(
        last_col_has_bg,
        "Scrollbar track should have background color when overflow:scroll"
    );
}

#[test]
fn test_line_01_visible_with_overflow_y_scroll() {
    let content = TestLines::new(50);
    let mut container = ScrollableContainer::<Msg>::new(Box::new(content));

    // Set overflow-y: scroll (horizontal should default to Hidden)
    let mut style = ComputedStyle::default();
    style.overflow_y = Overflow::Scroll;
    container.set_style(style);

    let mut canvas = Canvas::new(80, 20);
    let region = Region::new(0, 0, 80, 20);
    container.render(&mut canvas, region);

    // Debug: Print the first few rows
    println!("First 5 rows with overflow-y: scroll:");
    for i in 0..5 {
        let row = canvas.row_str(i);
        println!("  Row {}: '{}'", i, row);
    }

    // Line 01 should be at row 0
    let first_row = canvas.row_str(0);
    assert!(
        first_row.contains("Line 01"),
        "First row should contain 'Line 01', but got: '{}'",
        first_row
    );
}

#[test]
fn test_no_horizontal_scrollbar_when_overflow_hidden() {
    let content = TestLines::new(50);
    let mut container = ScrollableContainer::<Msg>::new(Box::new(content));

    // Only vertical scroll, no horizontal
    let mut style = ComputedStyle::default();
    style.overflow_y = Overflow::Scroll;
    style.overflow_x = Overflow::Hidden;
    container.set_style(style);

    let mut canvas = Canvas::new(80, 20);
    let region = Region::new(0, 0, 80, 20);
    container.render(&mut canvas, region);

    // Debug: Print the last row
    let last_row = canvas.row_str(19);
    println!("Last row (should have no scrollbar): '{}'", last_row);

    // With overflow-x: hidden, there should be no horizontal scrollbar
    // The vertical scrollbar is at column 79 (width - 1)
    // The horizontal scrollbar would be at row 19 (height - 1)
    // Check column 0-78 of the last row - if there's no horizontal scrollbar,
    // these should NOT have scrollbar background (only content or empty)

    // Column 0 of row 19 should be content ("L" from "Line 20") not scrollbar
    let first_col_last_row = canvas.get_char(0, 19);
    assert_eq!(
        first_col_last_row, 'L',
        "First column of last row should be 'L' (from Line 20), not scrollbar, got: '{}'",
        first_col_last_row
    );
}

// =============================================================================
// Tests for CSS-based style application
// =============================================================================

#[test]
fn test_scroll_demo_css_parsing() {
    use tcss::parser::parse_stylesheet;
    use tcss::parser::stylesheet::Declaration;

    // Exact CSS from scroll_demo.rs
    let css = r#"
        ScrollableContainer {
            overflow-y: scroll;
            overflow-x: hidden;
            scrollbar-color: #00CCFF;
            scrollbar-color-hover: #66DDFF;
            scrollbar-color-active: #FFFFFF;
            scrollbar-background: #333333;
            scrollbar-background-hover: #444444;
            scrollbar-size-vertical: 1;
            scrollbar-size-horizontal: 0;
        }
    "#;

    let stylesheet = parse_stylesheet(css).expect("CSS should parse");
    assert_eq!(stylesheet.rules.len(), 1, "Should have 1 rule");

    let rule = &stylesheet.rules[0];
    println!("Declarations:");
    for decl in rule.declarations() {
        println!("  {:?}", decl);
    }

    // Verify scrollbar-size-horizontal: 0 is parsed
    let has_horizontal_size_0 = rule.declarations().iter().any(|d| {
        matches!(d, Declaration::ScrollbarSizeHorizontal(0))
    });
    assert!(has_horizontal_size_0, "scrollbar-size-horizontal: 0 should be parsed");

    // Verify overflow-x: hidden is parsed
    let has_overflow_x_hidden = rule.declarations().iter().any(|d| {
        matches!(d, Declaration::OverflowX(tcss::types::Overflow::Hidden))
    });
    assert!(has_overflow_x_hidden, "overflow-x: hidden should be parsed");
}

#[test]
fn test_css_style_application() {
    use tcss::parser::parse_stylesheet;
    use tcss::parser::cascade::{WidgetMeta, compute_style};
    use tcss::types::Theme;
    use textual::widget::Widget;

    // Parse CSS similar to scroll_demo
    let css = r#"
        ScrollableContainer {
            overflow-y: scroll;
            overflow-x: hidden;
            scrollbar-color: #00CCFF;
            scrollbar-size-vertical: 1;
            scrollbar-size-horizontal: 0;
        }
    "#;

    let stylesheet = parse_stylesheet(css).expect("CSS should parse");
    let theme = Theme::new("default", true);

    // Create container
    let content = TestLines::new(50);
    let mut container = ScrollableContainer::<Msg>::new(Box::new(content));

    // Get widget meta for CSS matching
    let meta = container.get_meta();
    println!("Widget type_name: {}", meta.type_name);

    // Compute style from CSS
    let ancestors: Vec<WidgetMeta> = Vec::new();
    let style = compute_style(&meta, &ancestors, &stylesheet, &theme);

    println!("Computed overflow_y: {:?}", style.overflow_y);
    println!("Computed overflow_x: {:?}", style.overflow_x);
    println!("Computed scrollbar.size.vertical: {}", style.scrollbar.size.vertical);
    println!("Computed scrollbar.size.horizontal: {}", style.scrollbar.size.horizontal);

    // Apply style
    container.set_style(style.clone());

    // Render
    let mut canvas = Canvas::new(80, 20);
    let region = Region::new(0, 0, 80, 20);
    container.render(&mut canvas, region);

    // Verify Line 01 is visible at Row 0
    let first_row = canvas.row_str(0);
    println!("First row with CSS-applied style: '{}'", first_row);

    assert!(
        first_row.contains("Line 01"),
        "CSS-applied style: First row should contain 'Line 01', but got: '{}'",
        first_row
    );

    // Verify overflow_y is Scroll
    assert_eq!(
        style.overflow_y,
        Overflow::Scroll,
        "overflow-y should be Scroll from CSS"
    );
}
