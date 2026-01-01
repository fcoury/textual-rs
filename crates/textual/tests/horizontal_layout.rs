use tcss::parser::{cascade::{compute_style, WidgetMeta, WidgetStates}, parse_stylesheet};
use tcss::types::{Layout, Theme, Unit};
use textual::canvas::Region;
use textual::layouts::{HorizontalLayout, Layout as LayoutTrait, LayoutChild, LayoutNode};

struct DummyNode;

impl LayoutNode for DummyNode {
    fn desired_size(&self) -> textual::canvas::Size {
        textual::canvas::Size::new(0, 0)
    }

    fn intrinsic_height_for_width(&self, _width: u16) -> u16 {
        0
    }
}

#[test]
fn test_screen_gets_horizontal_layout() {
    let css = r#"
Screen {
    layout: horizontal;
}
"#;

    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = Theme::standard_themes().get("textual-dark").unwrap().clone();

    let screen_meta = WidgetMeta {
        type_name: "Screen",
        id: None,
        classes: vec![],
        states: WidgetStates::empty(),
    };
    let screen_style = compute_style(&screen_meta, &[], &stylesheet, &theme);

    assert!(matches!(screen_style.layout, Layout::Horizontal),
        "Screen should have horizontal layout, got {:?}", screen_style.layout);
}

#[test]
fn test_static_gets_height_100_percent_and_width_1fr() {
    let css = r#"
Static {
    height: 100%;
    width: 1fr;
}
"#;

    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = Theme::standard_themes().get("textual-dark").unwrap().clone();

    let static_meta = WidgetMeta {
        type_name: "Static",
        id: None,
        classes: vec![],
        states: WidgetStates::empty(),
    };
    let static_style = compute_style(&static_meta, &[], &stylesheet, &theme);

    assert!(static_style.height.is_some(), "Static should have height set");
    assert!(static_style.width.is_some(), "Static should have width set");

    let height = static_style.height.as_ref().unwrap();
    let width = static_style.width.as_ref().unwrap();

    assert!(matches!(height.unit, Unit::Percent), "Height should be percent, got {:?}", height.unit);
    assert!((height.value - 100.0).abs() < 0.01, "Height should be 100%, got {}", height.value);
    assert!(matches!(width.unit, Unit::Fraction), "Width should be fr, got {:?}", width.unit);
    assert!((width.value - 1.0).abs() < 0.01, "Width should be 1fr, got {}", width.value);
}

#[test]
fn test_horizontal_layout_distributes_fr_widths() {
    use textual::canvas::Size;
    use tcss::types::ComputedStyle;
    use tcss::types::geometry::Scalar;
    let dummy = DummyNode;

    // Create 3 children each with width: 1fr
    let mut child_style = ComputedStyle::default();
    child_style.width = Some(Scalar::fr(1.0));

    let children = vec![
        LayoutChild { index: 0, style: child_style.clone(), desired_size: Size::new(10, 3), node: &dummy },
        LayoutChild { index: 1, style: child_style.clone(), desired_size: Size::new(10, 3), node: &dummy },
        LayoutChild { index: 2, style: child_style.clone(), desired_size: Size::new(10, 3), node: &dummy },
    ];

    let available = Region::new(0, 0, 120, 30);
    let parent_style = ComputedStyle::default();

    let mut layout = HorizontalLayout;
    let placements = layout.arrange(&parent_style, &children, available, available.into());

    assert_eq!(placements.len(), 3);

    // Each child should get 40 columns (120 / 3)
    println!("Placements:");
    for (i, p) in placements.iter().enumerate() {
        println!("  Child {}: x={}, width={}", i, p.region.x, p.region.width);
    }

    // Check they divide the space equally
    let total_width: i32 = placements.iter().map(|p| p.region.width).sum();
    assert_eq!(total_width, 120, "Total width should be 120, got {}", total_width);

    // Each should be approximately equal (allowing for rounding)
    for (i, p) in placements.iter().enumerate() {
        assert!(p.region.width >= 39 && p.region.width <= 41,
            "Child {} width should be ~40, got {}", i, p.region.width);
    }
}

#[test]
fn test_horizontal_layout_height_100_percent() {
    use textual::canvas::Size;
    use tcss::types::ComputedStyle;
    use tcss::types::geometry::Scalar;
    let dummy = DummyNode;

    // Create a child with height: 100%
    let mut child_style = ComputedStyle::default();
    child_style.height = Some(Scalar::percent(100.0));
    child_style.width = Some(Scalar::fr(1.0));

    let children = vec![
        LayoutChild { index: 0, style: child_style.clone(), desired_size: Size::new(10, 3), node: &dummy },
    ];

    let available = Region::new(0, 0, 80, 24);
    let parent_style = ComputedStyle::default();

    let mut layout = HorizontalLayout;
    let placements = layout.arrange(&parent_style, &children, available, available.into());

    assert_eq!(placements.len(), 1);

    let p = &placements[0];
    println!("Child: x={}, y={}, width={}, height={}",
        p.region.x, p.region.y, p.region.width, p.region.height);

    // Height should be 100% of available (24)
    assert_eq!(p.region.height, 24, "Height should be 24 (100%), got {}", p.region.height);
    // Width should be full available (80)
    assert_eq!(p.region.width, 80, "Width should be 80 (1fr of 80), got {}", p.region.width);
}
