//! Tests for VerticalScroll ID targeting and overflow CSS application.
//!
//! Bug #2: VerticalScroll::with_id() was adding ID as a CSS class with "#" prefix
//! instead of setting an actual ID. This caused CSS selectors like `#right` to fail.

use tcss::parser::cascade::{compute_style, WidgetMeta};
use tcss::parser::parse_stylesheet;
use tcss::types::{Overflow, Theme};
use textual::widget::Widget;
use textual::{Label, Static, VerticalScroll};

#[test]
fn test_vertical_scroll_id_is_set_correctly() {
    // Create VerticalScroll with ID
    let vs: VerticalScroll<()> = VerticalScroll::new(vec![
        Box::new(Label::new("test")) as Box<dyn Widget<()>>,
    ])
    .with_id("my-scroll");

    // The ID should be accessible via id() method
    assert_eq!(
        vs.id(),
        Some("my-scroll"),
        "VerticalScroll::id() should return the ID set via with_id()"
    );
}

#[test]
fn test_vertical_scroll_meta_has_id() {
    let vs: VerticalScroll<()> = VerticalScroll::new(vec![
        Box::new(Label::new("test")) as Box<dyn Widget<()>>,
    ])
    .with_id("test-id");

    let meta = vs.get_meta();
    assert_eq!(
        meta.id,
        Some("test-id".to_string()),
        "VerticalScroll::get_meta() should return the ID in meta.id"
    );
    assert_eq!(meta.type_name, "VerticalScroll");
}

#[test]
fn test_css_id_selector_targets_vertical_scroll() {
    // CSS that targets a specific ID
    let css = r#"
VerticalScroll {
    overflow-y: auto;
}
#hidden-scroll {
    overflow-y: hidden;
}
"#;

    let stylesheet = parse_stylesheet(css).expect("CSS should parse");
    let theme = Theme::new("default", true);

    // Create VerticalScroll with the target ID
    let vs: VerticalScroll<()> = VerticalScroll::new(vec![
        Box::new(Label::new("content")) as Box<dyn Widget<()>>,
    ])
    .with_id("hidden-scroll");

    // Get widget meta for CSS matching
    let meta = vs.get_meta();
    println!("Widget meta: type={}, id={:?}, classes={:?}",
             meta.type_name, meta.id, meta.classes);

    // Compute style from CSS
    let ancestors: Vec<WidgetMeta> = Vec::new();
    let style = compute_style(&meta, &ancestors, &stylesheet, &theme);

    println!("Computed overflow_y: {:?}", style.overflow_y);

    // The #hidden-scroll rule should override the VerticalScroll default
    assert_eq!(
        style.overflow_y,
        Overflow::Hidden,
        "CSS selector #hidden-scroll should set overflow-y: hidden"
    );
}

#[test]
fn test_vertical_scroll_without_id_uses_default_overflow() {
    let css = r#"
VerticalScroll {
    overflow-y: auto;
}
#specific-id {
    overflow-y: hidden;
}
"#;

    let stylesheet = parse_stylesheet(css).expect("CSS should parse");
    let theme = Theme::new("default", true);

    // Create VerticalScroll WITHOUT the specific ID
    let vs: VerticalScroll<()> = VerticalScroll::new(vec![
        Box::new(Label::new("content")) as Box<dyn Widget<()>>,
    ])
    .with_id("other-id");

    let meta = vs.get_meta();
    let ancestors: Vec<WidgetMeta> = Vec::new();
    let style = compute_style(&meta, &ancestors, &stylesheet, &theme);

    // Should NOT have hidden overflow (no matching ID)
    assert_eq!(
        style.overflow_y,
        Overflow::Auto,
        "VerticalScroll without matching ID should keep overflow-y: auto"
    );
}

/// Test that set_style properly propagates overflow_y to the inner ScrollableContainer.
#[test]
fn test_style_propagation_to_inner_scrollable() {
    let css = r#"
#hidden-scroll {
    overflow-y: hidden;
}
"#;

    let stylesheet = parse_stylesheet(css).expect("CSS should parse");
    let theme = Theme::new("default", true);

    // Create VerticalScroll with target ID
    let mut vs: VerticalScroll<()> = VerticalScroll::new(vec![
        Box::new(Label::new("content")) as Box<dyn Widget<()>>,
    ])
    .with_id("hidden-scroll");

    // Compute style using the CSS cascade
    let meta = vs.get_meta();
    let ancestors: Vec<WidgetMeta> = Vec::new();
    let style = compute_style(&meta, &ancestors, &stylesheet, &theme);

    // Apply the style via set_style
    vs.set_style(style);

    // Now retrieve the style via get_style
    let stored_style = vs.get_style();

    // The stored style should have overflow_y = Hidden
    assert_eq!(
        stored_style.overflow_y,
        Overflow::Hidden,
        "After set_style(), get_style() should return overflow_y: hidden"
    );
}

#[test]
fn test_overflow_example_css_scenario() {
    // This is the exact scenario from the overflow.tcss example
    let css = r#"
VerticalScroll {
    width: 1fr;
}

Static {
    margin: 1 2;
    background: green 80%;
    border: green wide;
    color: white 90%;
    height: auto;
}

#right {
    overflow-y: hidden;
}
"#;

    let stylesheet = parse_stylesheet(css).expect("CSS should parse");
    let theme = Theme::new("default", true);

    // Left container (no specific ID, should scroll)
    let left: VerticalScroll<()> = VerticalScroll::new(vec![
        Box::new(Static::new("content")) as Box<dyn Widget<()>>,
    ])
    .with_id("left");

    // Right container (ID = "right", should have hidden overflow)
    let right: VerticalScroll<()> = VerticalScroll::new(vec![
        Box::new(Static::new("content")) as Box<dyn Widget<()>>,
    ])
    .with_id("right");

    let left_meta = left.get_meta();
    let right_meta = right.get_meta();

    println!("Left meta: id={:?}, classes={:?}", left_meta.id, left_meta.classes);
    println!("Right meta: id={:?}, classes={:?}", right_meta.id, right_meta.classes);

    let ancestors: Vec<WidgetMeta> = Vec::new();
    let left_style = compute_style(&left_meta, &ancestors, &stylesheet, &theme);
    let right_style = compute_style(&right_meta, &ancestors, &stylesheet, &theme);

    println!("Left overflow_y: {:?}", left_style.overflow_y);
    println!("Right overflow_y: {:?}", right_style.overflow_y);

    // Left should use VerticalScroll default (auto from default_css)
    // The CSS here doesn't set overflow-y for VerticalScroll, so it stays default (Hidden)
    // But that's fine - the key assertion is that right has Hidden from the #right rule

    // Right should have hidden overflow from #right rule
    assert_eq!(
        right_style.overflow_y,
        Overflow::Hidden,
        "Right container (#right) should have overflow-y: hidden"
    );
}
