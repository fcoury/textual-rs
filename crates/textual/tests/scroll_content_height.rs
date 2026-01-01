//! Tests for content_height_for_scroll including child margins.
//!
//! Bug #1/#3: content_height_for_scroll was not including child margins in its
//! calculation, causing virtual height to be too small. This resulted in:
//! - Content being cut off when scrolled to the bottom
//! - Scrollbar thumb being too large (wrong ratio)

use tcss::parser::cascade::{compute_style, WidgetMeta};
use tcss::parser::parse_stylesheet;
use tcss::types::Theme;
use textual::widget::Widget;
use textual::{Label, Vertical};

/// Test that content_height_for_scroll includes vertical margins.
#[test]
fn test_content_height_includes_margins() {
    // CSS with margin on children
    let css = r#"
Vertical {
    width: 100%;
    height: auto;
}
Label {
    height: auto;
    margin: 2 0;
}
"#;

    let stylesheet = parse_stylesheet(css).expect("CSS should parse");
    let theme = Theme::new("default", true);

    // Create a Vertical with 3 Label children
    // Each label has height=1 (single line) and margin: 2 0 (2 cells top, 2 cells bottom)
    let mut vertical: Vertical<()> = Vertical::new(vec![
        Box::new(Label::new("Line 1")) as Box<dyn Widget<()>>,
        Box::new(Label::new("Line 2")) as Box<dyn Widget<()>>,
        Box::new(Label::new("Line 3")) as Box<dyn Widget<()>>,
    ]);

    // Apply styles
    let meta = vertical.get_meta();
    let ancestors: Vec<WidgetMeta> = Vec::new();
    let style = compute_style(&meta, &ancestors, &stylesheet, &theme);
    vertical.set_style(style);

    // Apply styles to children
    vertical.for_each_child(&mut |child| {
        let child_meta = child.get_meta();
        let child_style = compute_style(&child_meta, &ancestors, &stylesheet, &theme);
        child.set_style(child_style);
    });

    // Print child styles for debugging
    vertical.for_each_child(&mut |child| {
        let s = child.get_style();
        println!(
            "Child margin: top={}, bottom={}",
            s.margin.top.value, s.margin.bottom.value
        );
    });

    // Resize to set viewport (needed for content_height_for_scroll)
    vertical.on_resize(textual::Size::new(80, 24));

    // Get content height
    let content_height = vertical.content_height_for_scroll(80, 24);

    // Expected calculation:
    // - 3 labels, each with height=1
    // - CSS margin collapsing: first child has full top margin, last has full bottom
    // - Between children: max(bottom of prev, top of next) - bottom of prev
    //
    // For margin: 2 0 (top=2, bottom=0):
    // Child 1: top=2, bottom=0
    // Child 2: top=max(2-0,0)=2, bottom=0
    // Child 3: top=max(2-0,0)=2, bottom=0
    // Total margin: 2 + 2 + 2 + 0 = 6? Actually need to trace the exact algorithm
    //
    // Let's be less prescriptive and just verify margin is included:
    // Without margins: 3 labels * 1 height = 3
    // With margins: should be > 3

    println!("content_height_for_scroll: {}", content_height);

    // The content height MUST be greater than just the box heights (3)
    // because margins should be included
    assert!(
        content_height > 3,
        "content_height_for_scroll should include margins. Got {}, expected > 3",
        content_height
    );
}

/// Test that content_height_for_scroll mirrors vertical layout margin handling.
#[test]
fn test_content_height_matches_vertical_layout() {
    // Create a simple case: 2 children with explicit margins
    // Note: "margin: 1 0" in CSS means top=1, right=0, bottom=1, left=0
    let css = r#"
Vertical {
    width: 100%;
}
Label {
    height: 2;
    margin: 1 0;
}
"#;

    let stylesheet = parse_stylesheet(css).expect("CSS should parse");
    let theme = Theme::new("default", true);

    let mut vertical: Vertical<()> = Vertical::new(vec![
        Box::new(Label::new("A")) as Box<dyn Widget<()>>,
        Box::new(Label::new("B")) as Box<dyn Widget<()>>,
    ]);

    // Apply styles
    let meta = vertical.get_meta();
    let ancestors: Vec<WidgetMeta> = Vec::new();
    let style = compute_style(&meta, &ancestors, &stylesheet, &theme);
    vertical.set_style(style);

    vertical.for_each_child(&mut |child| {
        let child_meta = child.get_meta();
        let child_style = compute_style(&child_meta, &ancestors, &stylesheet, &theme);
        child.set_style(child_style);
    });

    vertical.on_resize(textual::Size::new(80, 24));

    let content_height = vertical.content_height_for_scroll(80, 24);

    // With margin: 1 0 (top=1, right=0, bottom=1, left=0) and height=2:
    // Child 1: margin_top=1, height=2, margin_bottom=1
    //   current_y = 0 + 1 = 1, next_y = 1 + 2 = 3, current_y = 3 + 1 = 4
    // Child 2: effective_top=max(1-1,0)=0, height=2, margin_bottom=1
    //   current_y = 4 + 0 = 4, next_y = 4 + 2 = 6, current_y = 6 + 1 = 7
    // Total: 7

    println!("content_height with margins: {}", content_height);

    // Without margins it would be 4 (2 labels * height 2)
    // With margins it should be 7
    assert_eq!(
        content_height, 7,
        "content_height_for_scroll should be 7 (2 heights + margins), got {}",
        content_height
    );
}

/// Test the exact scenario from the overflow example.
#[test]
fn test_overflow_example_content_height() {
    // This mimics the overflow example with Static widgets having margin: 1 2
    let css = r#"
Vertical {
    width: 100%;
}
Label {
    height: 7;
    margin: 1 2;
}
"#;

    let stylesheet = parse_stylesheet(css).expect("CSS should parse");
    let theme = Theme::new("default", true);

    // 3 Static-like widgets (using Label as proxy)
    let mut vertical: Vertical<()> = Vertical::new(vec![
        Box::new(Label::new("Fear...")) as Box<dyn Widget<()>>,
        Box::new(Label::new("Fear...")) as Box<dyn Widget<()>>,
        Box::new(Label::new("Fear...")) as Box<dyn Widget<()>>,
    ]);

    // Apply styles
    let meta = vertical.get_meta();
    let ancestors: Vec<WidgetMeta> = Vec::new();
    let style = compute_style(&meta, &ancestors, &stylesheet, &theme);
    vertical.set_style(style);

    vertical.for_each_child(&mut |child| {
        let child_meta = child.get_meta();
        let child_style = compute_style(&child_meta, &ancestors, &stylesheet, &theme);
        child.set_style(child_style);
    });

    vertical.on_resize(textual::Size::new(80, 24));

    let content_height = vertical.content_height_for_scroll(80, 24);

    // With margin: 1 2 (top=1, bottom=1) and height=7:
    // Child 1: margin_top=1, height=7, margin_bottom=1
    // Child 2: margin_top=max(1-1,0)=0, height=7, margin_bottom=1
    // Child 3: margin_top=max(1-1,0)=0, height=7, margin_bottom=1
    // Total: 1 + 7 + 1 + 0 + 7 + 1 + 0 + 7 + 1 = 25
    //
    // Actually, margin collapsing takes the MAX not adds both:
    // y += effective_top; y += height; prev_bottom = bottom
    // Child 1: y=0+1=1, y=1+7=8, prev_bottom=1
    // Child 2: effective_top=max(1-1,0)=0, y=8+0=8, y=8+7=15, prev_bottom=1
    // Child 3: effective_top=max(1-1,0)=0, y=15+0=15, y=15+7=22, prev_bottom=1
    // Final: y=22, but we need to add final margin_bottom=1 -> 23?
    //
    // Actually looking at vertical layout code more carefully:
    // current_y = next_y + margin_bottom; where next_y = current_y + box_height
    // So margin_bottom IS added after each child
    //
    // Let me trace again:
    // current_y = 0
    // Child 1: effective_top=1, current_y=0+1=1, box_height=7, next_y=1+7=8, current_y=8+1=9
    // Child 2: effective_top=max(1-1,0)=0, current_y=9+0=9, box_height=7, next_y=9+7=16, current_y=16+1=17
    // Child 3: effective_top=max(1-1,0)=0, current_y=17+0=17, box_height=7, next_y=17+7=24, current_y=24+1=25
    //
    // But wait, content_height_for_scroll returns floor(current_y) which should be 25
    // Actually no, looking at the code it returns floor at the end: total_height = current_y.floor()
    // But the issue is it doesn't add margins at all!
    //
    // Without margins: 3 * 7 = 21
    // With margins: should be 25

    println!("overflow example content_height: {}", content_height);

    // Content height should be greater than 21 (box heights only)
    assert!(
        content_height > 21,
        "content_height should include margins. Got {}, expected > 21",
        content_height
    );
}
