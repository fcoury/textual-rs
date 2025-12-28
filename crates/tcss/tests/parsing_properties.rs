//! Integration tests for TCSS property parsing.
//!
//! Tests property declarations as defined in Textual CSS:
//! - Color properties: color, background
//! - Dimension properties: width, height
//! - Box model properties: margin, padding
//! - Border properties: border
//!
//! Properties not yet implemented are marked with #[ignore]

use tcss::parser::{Declaration, parse_rule};
use tcss::types::border::BorderKind;
use tcss::types::color::RgbaColor;
use tcss::types::geometry::{Scalar, Spacing, Unit};
use tcss::types::text::TextStyle;
use tcss::types::{AlignHorizontal, AlignVertical, Layout, Overflow};

/// Helper to parse a simple rule and extract declarations
fn parse_declarations(input: &str) -> Vec<Declaration> {
    let (_, rule) = parse_rule(input).expect("failed to parse rule");
    rule.declarations()
}

/// Helper to parse a rule and get the first declaration
fn parse_first_declaration(input: &str) -> Declaration {
    let decls = parse_declarations(input);
    assert!(!decls.is_empty(), "expected at least one declaration");
    decls.into_iter().next().unwrap()
}

// ============================================================================
// COLOR PROPERTY
// ============================================================================

#[test]
fn test_property_color_named() {
    let decl = parse_first_declaration("Button { color: red; }");
    assert_eq!(decl, Declaration::Color(RgbaColor::rgb(255, 0, 0)));
}

#[test]
fn test_property_color_hex() {
    let decl = parse_first_declaration("Button { color: #00ff00; }");
    assert_eq!(decl, Declaration::Color(RgbaColor::rgb(0, 255, 0)));
}

#[test]
fn test_property_color_hex_short() {
    let decl = parse_first_declaration("Button { color: #00f; }");
    assert_eq!(decl, Declaration::Color(RgbaColor::rgb(0, 0, 255)));
}

#[test]
fn test_property_color_rgb() {
    let decl = parse_first_declaration("Button { color: rgb(128, 64, 32); }");
    assert_eq!(decl, Declaration::Color(RgbaColor::rgb(128, 64, 32)));
}

#[test]
fn test_property_color_hsl() {
    let decl = parse_first_declaration("Button { color: hsl(0, 100%, 50%); }");
    if let Declaration::Color(c) = decl {
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    } else {
        panic!("expected Color declaration");
    }
}

#[test]
fn test_property_color_auto() {
    let decl = parse_first_declaration("Button { color: auto; }");
    if let Declaration::Color(c) = decl {
        assert!(c.auto);
    } else {
        panic!("expected Color declaration");
    }
}

#[test]
fn test_property_color_transparent() {
    let decl = parse_first_declaration("Button { color: transparent; }");
    if let Declaration::Color(c) = decl {
        assert!(c.is_transparent());
    } else {
        panic!("expected Color declaration");
    }
}

// ============================================================================
// BACKGROUND PROPERTY
// ============================================================================

#[test]
fn test_property_background_named() {
    let decl = parse_first_declaration("Button { background: blue; }");
    assert_eq!(decl, Declaration::Background(RgbaColor::rgb(0, 0, 255)));
}

#[test]
fn test_property_background_hex() {
    let decl = parse_first_declaration("Button { background: #ff00ff; }");
    assert_eq!(decl, Declaration::Background(RgbaColor::rgb(255, 0, 255)));
}

#[test]
fn test_property_background_rgb() {
    let decl = parse_first_declaration("Button { background: rgb(100, 150, 200); }");
    assert_eq!(decl, Declaration::Background(RgbaColor::rgb(100, 150, 200)));
}

#[test]
fn test_property_background_transparent() {
    let decl = parse_first_declaration("Button { background: transparent; }");
    if let Declaration::Background(c) = decl {
        assert!(c.is_transparent());
    } else {
        panic!("expected Background declaration");
    }
}

// ============================================================================
// WIDTH PROPERTY
// ============================================================================

#[test]
fn test_property_width_cells() {
    let decl = parse_first_declaration("Button { width: 50; }");
    if let Declaration::Width(s) = decl {
        assert_eq!(s.value, 50.0);
        assert_eq!(s.unit, Unit::Cells);
    } else {
        panic!("expected Width declaration");
    }
}

#[test]
fn test_property_width_percent() {
    let decl = parse_first_declaration("Button { width: 100%; }");
    if let Declaration::Width(s) = decl {
        assert_eq!(s.value, 100.0);
        assert_eq!(s.unit, Unit::Percent);
    } else {
        panic!("expected Width declaration");
    }
}

#[test]
fn test_property_width_fraction() {
    let decl = parse_first_declaration("Button { width: 1fr; }");
    if let Declaration::Width(s) = decl {
        assert_eq!(s.value, 1.0);
        assert_eq!(s.unit, Unit::Fraction);
    } else {
        panic!("expected Width declaration");
    }
}

#[test]
fn test_property_width_viewport() {
    let decl = parse_first_declaration("Button { width: 50vw; }");
    if let Declaration::Width(s) = decl {
        assert_eq!(s.value, 50.0);
        assert_eq!(s.unit, Unit::ViewWidth);
    } else {
        panic!("expected Width declaration");
    }
}

#[test]
fn test_property_width_auto() {
    let decl = parse_first_declaration("Button { width: auto; }");
    if let Declaration::Width(s) = decl {
        assert!(s.is_auto());
    } else {
        panic!("expected Width declaration");
    }
}

// ============================================================================
// HEIGHT PROPERTY
// ============================================================================

#[test]
fn test_property_height_cells() {
    let decl = parse_first_declaration("Button { height: 10; }");
    if let Declaration::Height(s) = decl {
        assert_eq!(s.value, 10.0);
        assert_eq!(s.unit, Unit::Cells);
    } else {
        panic!("expected Height declaration");
    }
}

#[test]
fn test_property_height_percent() {
    let decl = parse_first_declaration("Button { height: 50%; }");
    if let Declaration::Height(s) = decl {
        assert_eq!(s.value, 50.0);
        assert_eq!(s.unit, Unit::Percent);
    } else {
        panic!("expected Height declaration");
    }
}

#[test]
fn test_property_height_viewport() {
    let decl = parse_first_declaration("Button { height: 100vh; }");
    if let Declaration::Height(s) = decl {
        assert_eq!(s.value, 100.0);
        assert_eq!(s.unit, Unit::ViewHeight);
    } else {
        panic!("expected Height declaration");
    }
}

#[test]
fn test_property_height_auto() {
    let decl = parse_first_declaration("Button { height: auto; }");
    if let Declaration::Height(s) = decl {
        assert!(s.is_auto());
    } else {
        panic!("expected Height declaration");
    }
}

// ============================================================================
// MARGIN PROPERTY
// ============================================================================

#[test]
fn test_property_margin_single() {
    let decl = parse_first_declaration("Button { margin: 5; }");
    if let Declaration::Margin(s) = decl {
        assert_eq!(s, Spacing::all(Scalar::cells(5.0)));
    } else {
        panic!("expected Margin declaration");
    }
}

#[test]
fn test_property_margin_two_values() {
    let decl = parse_first_declaration("Button { margin: 10 20; }");
    if let Declaration::Margin(s) = decl {
        // vertical = 10, horizontal = 20
        assert_eq!(s.top.value, 10.0);
        assert_eq!(s.bottom.value, 10.0);
        assert_eq!(s.left.value, 20.0);
        assert_eq!(s.right.value, 20.0);
    } else {
        panic!("expected Margin declaration");
    }
}

#[test]
fn test_property_margin_four_values() {
    let decl = parse_first_declaration("Button { margin: 1 2 3 4; }");
    if let Declaration::Margin(s) = decl {
        assert_eq!(s.top.value, 1.0);
        assert_eq!(s.right.value, 2.0);
        assert_eq!(s.bottom.value, 3.0);
        assert_eq!(s.left.value, 4.0);
    } else {
        panic!("expected Margin declaration");
    }
}

#[test]
fn test_property_margin_with_units() {
    let decl = parse_first_declaration("Button { margin: 10%; }");
    if let Declaration::Margin(s) = decl {
        assert_eq!(s.top.unit, Unit::Percent);
    } else {
        panic!("expected Margin declaration");
    }
}

// ============================================================================
// PADDING PROPERTY
// ============================================================================

#[test]
fn test_property_padding_single() {
    let decl = parse_first_declaration("Button { padding: 8; }");
    if let Declaration::Padding(s) = decl {
        assert_eq!(s, Spacing::all(Scalar::cells(8.0)));
    } else {
        panic!("expected Padding declaration");
    }
}

#[test]
fn test_property_padding_two_values() {
    let decl = parse_first_declaration("Button { padding: 5 10; }");
    if let Declaration::Padding(s) = decl {
        assert_eq!(s.top.value, 5.0);
        assert_eq!(s.left.value, 10.0);
    } else {
        panic!("expected Padding declaration");
    }
}

#[test]
fn test_property_padding_four_values() {
    let decl = parse_first_declaration("Button { padding: 2 4 6 8; }");
    if let Declaration::Padding(s) = decl {
        assert_eq!(s.top.value, 2.0);
        assert_eq!(s.right.value, 4.0);
        assert_eq!(s.bottom.value, 6.0);
        assert_eq!(s.left.value, 8.0);
    } else {
        panic!("expected Padding declaration");
    }
}

// ============================================================================
// BORDER PROPERTY
// ============================================================================

#[test]
fn test_property_border_solid() {
    let decl = parse_first_declaration("Button { border: solid red; }");
    if let Declaration::Border(b) = decl {
        assert_eq!(b.kind, BorderKind::Solid);
        assert_eq!(b.color, Some(RgbaColor::rgb(255, 0, 0)));
    } else {
        panic!("expected Border declaration");
    }
}

#[test]
fn test_property_border_heavy() {
    let decl = parse_first_declaration("Button { border: heavy green; }");
    if let Declaration::Border(b) = decl {
        assert_eq!(b.kind, BorderKind::Heavy);
        assert_eq!(b.color, Some(RgbaColor::rgb(0, 128, 0)));
    } else {
        panic!("expected Border declaration");
    }
}

#[test]
fn test_property_border_round() {
    let decl = parse_first_declaration("Button { border: round #ff0000; }");
    if let Declaration::Border(b) = decl {
        assert_eq!(b.kind, BorderKind::Round);
        assert_eq!(b.color, Some(RgbaColor::rgb(255, 0, 0)));
    } else {
        panic!("expected Border declaration");
    }
}

#[test]
fn test_property_border_double() {
    let decl = parse_first_declaration("Button { border: double coral; }");
    if let Declaration::Border(b) = decl {
        assert_eq!(b.kind, BorderKind::Double);
    } else {
        panic!("expected Border declaration");
    }
}

#[test]
fn test_property_border_none() {
    let decl = parse_first_declaration("Button { border: none; }");
    if let Declaration::Border(b) = decl {
        assert_eq!(b.kind, BorderKind::None);
    } else {
        panic!("expected Border declaration");
    }
}

// ============================================================================
// UNKNOWN PROPERTIES (should be parsed as Unknown)
// ============================================================================

#[test]
fn test_property_unknown() {
    let decl = parse_first_declaration("Button { unknown-property: value; }");
    if let Declaration::Unknown(name) = decl {
        assert_eq!(name, "unknown-property");
    } else {
        panic!("expected Unknown declaration");
    }
}

// ============================================================================
// MULTIPLE DECLARATIONS
// ============================================================================

#[test]
fn test_multiple_declarations() {
    let decls = parse_declarations("Button { color: red; background: blue; }");
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[0], Declaration::Color(RgbaColor::rgb(255, 0, 0)));
    assert_eq!(decls[1], Declaration::Background(RgbaColor::rgb(0, 0, 255)));
}

#[test]
fn test_multiple_declarations_no_semicolon_last() {
    // Last declaration without semicolon
    let decls = parse_declarations("Button { color: red; background: blue }");
    assert_eq!(decls.len(), 2);
}

#[test]
fn test_multiple_declarations_mixed() {
    let decls = parse_declarations("Button { width: 100%; height: 50; margin: 10; }");
    assert_eq!(decls.len(), 3);

    if let Declaration::Width(s) = &decls[0] {
        assert_eq!(s.unit, Unit::Percent);
    } else {
        panic!("expected Width");
    }

    if let Declaration::Height(s) = &decls[1] {
        assert_eq!(s.value, 50.0);
    } else {
        panic!("expected Height");
    }

    if let Declaration::Margin(s) = &decls[2] {
        assert_eq!(s.top.value, 10.0);
    } else {
        panic!("expected Margin");
    }
}

// ============================================================================
// WHITESPACE HANDLING
// ============================================================================

#[test]
fn test_declaration_extra_whitespace() {
    let decl = parse_first_declaration("Button {   color  :   red  ;   }");
    assert_eq!(decl, Declaration::Color(RgbaColor::rgb(255, 0, 0)));
}

#[test]
fn test_declaration_newlines() {
    let decl = parse_first_declaration("Button {\n  color: red;\n}");
    assert_eq!(decl, Declaration::Color(RgbaColor::rgb(255, 0, 0)));
}

#[test]
fn test_declaration_tabs() {
    let decl = parse_first_declaration("Button {\t\tcolor:\tred;\t}");
    assert_eq!(decl, Declaration::Color(RgbaColor::rgb(255, 0, 0)));
}

// ============================================================================
// PROPERTIES NOT YET IMPLEMENTED
// These tests document expected behavior for future implementation
// ============================================================================

#[test]
#[ignore = "dock property not yet implemented"]
fn test_property_dock_top() {
    let decl = parse_first_declaration("Button { dock: top; }");
    // Should parse as Declaration::Dock(Dock::Top)
    assert!(matches!(decl, Declaration::Unknown(_)));
}

#[test]
fn test_property_layout_horizontal() {
    let decl = parse_first_declaration("Button { layout: horizontal; }");
    assert!(matches!(decl, Declaration::Layout(Layout::Horizontal)));
}

#[test]
fn test_property_layout_vertical() {
    let decl = parse_first_declaration("Container { layout: vertical; }");
    assert!(matches!(decl, Declaration::Layout(Layout::Vertical)));
}

#[test]
fn test_property_layout_grid() {
    let decl = parse_first_declaration("Panel { layout: grid; }");
    assert!(matches!(decl, Declaration::Layout(Layout::Grid)));
}

#[test]
#[ignore = "display property not yet implemented"]
fn test_property_display_none() {
    let decl = parse_first_declaration("Button { display: none; }");
    // Should parse as Declaration::Display(Display::None)
    assert!(matches!(decl, Declaration::Unknown(_)));
}

#[test]
#[ignore = "visibility property not yet implemented"]
fn test_property_visibility_hidden() {
    let decl = parse_first_declaration("Button { visibility: hidden; }");
    // Should parse as Declaration::Visibility(Visibility::Hidden)
    assert!(matches!(decl, Declaration::Unknown(_)));
}

#[test]
#[ignore = "opacity property not yet implemented"]
fn test_property_opacity() {
    let decl = parse_first_declaration("Button { opacity: 0.5; }");
    // Should parse as Declaration::Opacity(0.5)
    assert!(matches!(decl, Declaration::Unknown(_)));
}

#[test]
#[ignore = "text-style property not yet implemented"]
fn test_property_text_style() {
    let decl = parse_first_declaration("Button { text-style: bold italic; }");
    // Should parse as Declaration::TextStyle with Bold | Italic
    assert!(matches!(decl, Declaration::Unknown(_)));
}

#[test]
#[ignore = "text-align property not yet implemented"]
fn test_property_text_align() {
    let decl = parse_first_declaration("Button { text-align: center; }");
    // Should parse as Declaration::TextAlign(TextAlign::Center)
    assert!(matches!(decl, Declaration::Unknown(_)));
}

#[test]
fn test_property_content_align() {
    let decl = parse_first_declaration("Button { content-align: center middle; }");
    assert!(matches!(
        decl,
        Declaration::ContentAlign(AlignHorizontal::Center, AlignVertical::Middle)
    ));
}

#[test]
fn test_property_overflow_x_hidden() {
    let decl = parse_first_declaration("Button { overflow-x: hidden; }");
    assert!(matches!(decl, Declaration::OverflowX(Overflow::Hidden)));
}

#[test]
fn test_property_overflow_x_auto() {
    let decl = parse_first_declaration("Button { overflow-x: auto; }");
    assert!(matches!(decl, Declaration::OverflowX(Overflow::Auto)));
}

#[test]
fn test_property_overflow_x_scroll() {
    let decl = parse_first_declaration("Button { overflow-x: scroll; }");
    assert!(matches!(decl, Declaration::OverflowX(Overflow::Scroll)));
}

#[test]
fn test_property_overflow_y_hidden() {
    let decl = parse_first_declaration("Button { overflow-y: hidden; }");
    assert!(matches!(decl, Declaration::OverflowY(Overflow::Hidden)));
}

#[test]
fn test_property_overflow_y_auto() {
    let decl = parse_first_declaration("Button { overflow-y: auto; }");
    assert!(matches!(decl, Declaration::OverflowY(Overflow::Auto)));
}

#[test]
fn test_property_overflow_y_scroll() {
    let decl = parse_first_declaration("Button { overflow-y: scroll; }");
    assert!(matches!(decl, Declaration::OverflowY(Overflow::Scroll)));
}

#[test]
#[ignore = "min-width property not yet implemented"]
fn test_property_min_width() {
    let decl = parse_first_declaration("Button { min-width: 10; }");
    // Should parse as Declaration::MinWidth(Scalar)
    assert!(matches!(decl, Declaration::Unknown(_)));
}

#[test]
#[ignore = "max-height property not yet implemented"]
fn test_property_max_height() {
    let decl = parse_first_declaration("Button { max-height: 100; }");
    // Should parse as Declaration::MaxHeight(Scalar)
    assert!(matches!(decl, Declaration::Unknown(_)));
}

// ============================================================================
// SCROLLBAR PROPERTIES
// ============================================================================

#[test]
fn test_property_scrollbar_color() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-color: magenta; }");
    if let Declaration::ScrollbarColor(c) = decl {
        assert_eq!(c, RgbaColor::rgb(255, 0, 255));
    } else {
        panic!("expected ScrollbarColor declaration, got {:?}", decl);
    }
}

#[test]
fn test_property_scrollbar_color_hex() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-color: #ff00ff; }");
    if let Declaration::ScrollbarColor(c) = decl {
        assert_eq!(c, RgbaColor::rgb(255, 0, 255));
    } else {
        panic!("expected ScrollbarColor declaration");
    }
}

#[test]
fn test_property_scrollbar_color_hover() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-color-hover: #aaaaaa; }");
    if let Declaration::ScrollbarColorHover(c) = decl {
        assert_eq!(c, RgbaColor::rgb(170, 170, 170));
    } else {
        panic!("expected ScrollbarColorHover declaration");
    }
}

#[test]
fn test_property_scrollbar_color_active() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-color-active: white; }");
    if let Declaration::ScrollbarColorActive(c) = decl {
        assert_eq!(c, RgbaColor::rgb(255, 255, 255));
    } else {
        panic!("expected ScrollbarColorActive declaration");
    }
}

#[test]
fn test_property_scrollbar_background() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-background: #555555; }");
    if let Declaration::ScrollbarBackground(c) = decl {
        assert_eq!(c, RgbaColor::rgb(85, 85, 85));
    } else {
        panic!("expected ScrollbarBackground declaration");
    }
}

#[test]
fn test_property_scrollbar_background_hover() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-background-hover: #666666; }");
    if let Declaration::ScrollbarBackgroundHover(c) = decl {
        assert_eq!(c, RgbaColor::rgb(102, 102, 102));
    } else {
        panic!("expected ScrollbarBackgroundHover declaration");
    }
}

#[test]
fn test_property_scrollbar_background_active() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-background-active: #777777; }");
    if let Declaration::ScrollbarBackgroundActive(c) = decl {
        assert_eq!(c, RgbaColor::rgb(119, 119, 119));
    } else {
        panic!("expected ScrollbarBackgroundActive declaration");
    }
}

#[test]
fn test_property_scrollbar_corner_color() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-corner-color: #333333; }");
    if let Declaration::ScrollbarCornerColor(c) = decl {
        assert_eq!(c, RgbaColor::rgb(51, 51, 51));
    } else {
        panic!("expected ScrollbarCornerColor declaration");
    }
}

#[test]
fn test_property_scrollbar_size_single() {
    use tcss::types::scrollbar::ScrollbarSize;
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-size: 2; }");
    if let Declaration::ScrollbarSize(size) = decl {
        assert_eq!(size, ScrollbarSize { horizontal: 2, vertical: 2 });
    } else {
        panic!("expected ScrollbarSize declaration, got {:?}", decl);
    }
}

#[test]
fn test_property_scrollbar_size_two_values() {
    use tcss::types::scrollbar::ScrollbarSize;
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-size: 1 2; }");
    if let Declaration::ScrollbarSize(size) = decl {
        assert_eq!(size, ScrollbarSize { horizontal: 1, vertical: 2 });
    } else {
        panic!("expected ScrollbarSize declaration");
    }
}

#[test]
fn test_property_scrollbar_size_horizontal() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-size-horizontal: 3; }");
    if let Declaration::ScrollbarSizeHorizontal(v) = decl {
        assert_eq!(v, 3);
    } else {
        panic!("expected ScrollbarSizeHorizontal declaration");
    }
}

#[test]
fn test_property_scrollbar_size_vertical() {
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-size-vertical: 2; }");
    if let Declaration::ScrollbarSizeVertical(v) = decl {
        assert_eq!(v, 2);
    } else {
        panic!("expected ScrollbarSizeVertical declaration");
    }
}

#[test]
fn test_property_scrollbar_gutter_auto() {
    use tcss::types::scrollbar::ScrollbarGutter;
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-gutter: auto; }");
    if let Declaration::ScrollbarGutter(g) = decl {
        assert_eq!(g, ScrollbarGutter::Auto);
    } else {
        panic!("expected ScrollbarGutter declaration");
    }
}

#[test]
fn test_property_scrollbar_gutter_stable() {
    use tcss::types::scrollbar::ScrollbarGutter;
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-gutter: stable; }");
    if let Declaration::ScrollbarGutter(g) = decl {
        assert_eq!(g, ScrollbarGutter::Stable);
    } else {
        panic!("expected ScrollbarGutter declaration");
    }
}

#[test]
fn test_property_scrollbar_visibility_visible() {
    use tcss::types::scrollbar::ScrollbarVisibility;
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-visibility: visible; }");
    if let Declaration::ScrollbarVisibility(v) = decl {
        assert_eq!(v, ScrollbarVisibility::Visible);
    } else {
        panic!("expected ScrollbarVisibility declaration, got {:?}", decl);
    }
}

#[test]
fn test_property_scrollbar_visibility_hidden() {
    use tcss::types::scrollbar::ScrollbarVisibility;
    let decl = parse_first_declaration("ScrollableContainer { scrollbar-visibility: hidden; }");
    if let Declaration::ScrollbarVisibility(v) = decl {
        assert_eq!(v, ScrollbarVisibility::Hidden);
    } else {
        panic!("expected ScrollbarVisibility declaration");
    }
}

#[test]
fn test_scrollbar_multiple_properties() {
    let decls = parse_declarations(
        "ScrollableContainer { scrollbar-color: cyan; scrollbar-background: #222; scrollbar-size: 1; }"
    );
    assert_eq!(decls.len(), 3);

    if let Declaration::ScrollbarColor(c) = &decls[0] {
        assert_eq!(*c, RgbaColor::rgb(0, 255, 255));
    } else {
        panic!("expected ScrollbarColor");
    }

    if let Declaration::ScrollbarBackground(c) = &decls[1] {
        assert_eq!(*c, RgbaColor::rgb(34, 34, 34));
    } else {
        panic!("expected ScrollbarBackground");
    }
}

// ============================================================================
// LINK PROPERTIES
// ============================================================================

#[test]
fn test_property_link_color() {
    let decl = parse_first_declaration("Link { link-color: blue; }");
    if let Declaration::LinkColor(c) = decl {
        assert_eq!(c, RgbaColor::rgb(0, 0, 255));
    } else {
        panic!("expected LinkColor declaration, got {:?}", decl);
    }
}

#[test]
fn test_property_link_color_hex() {
    let decl = parse_first_declaration("Link { link-color: #1e90ff; }");
    if let Declaration::LinkColor(c) = decl {
        assert_eq!(c, RgbaColor::rgb(30, 144, 255));
    } else {
        panic!("expected LinkColor declaration");
    }
}

#[test]
fn test_property_link_color_hover() {
    let decl = parse_first_declaration("Link { link-color-hover: cyan; }");
    if let Declaration::LinkColorHover(c) = decl {
        assert_eq!(c, RgbaColor::rgb(0, 255, 255));
    } else {
        panic!("expected LinkColorHover declaration");
    }
}

#[test]
fn test_property_link_background() {
    let decl = parse_first_declaration("Link { link-background: yellow; }");
    if let Declaration::LinkBackground(c) = decl {
        assert_eq!(c, RgbaColor::rgb(255, 255, 0));
    } else {
        panic!("expected LinkBackground declaration");
    }
}

#[test]
fn test_property_link_background_hover() {
    let decl = parse_first_declaration("Link { link-background-hover: #333333; }");
    if let Declaration::LinkBackgroundHover(c) = decl {
        assert_eq!(c, RgbaColor::rgb(51, 51, 51));
    } else {
        panic!("expected LinkBackgroundHover declaration");
    }
}

#[test]
fn test_property_link_style_single() {
    let decl = parse_first_declaration("Link { link-style: underline; }");
    if let Declaration::LinkStyle(s) = decl {
        assert!(s.underline);
        assert!(!s.bold);
        assert!(!s.italic);
    } else {
        panic!("expected LinkStyle declaration, got {:?}", decl);
    }
}

#[test]
fn test_property_link_style_multiple() {
    let decl = parse_first_declaration("Link { link-style: bold underline; }");
    if let Declaration::LinkStyle(s) = decl {
        assert!(s.bold);
        assert!(s.underline);
        assert!(!s.italic);
    } else {
        panic!("expected LinkStyle declaration");
    }
}

#[test]
fn test_property_link_style_hover() {
    let decl = parse_first_declaration("Link { link-style-hover: bold italic reverse; }");
    if let Declaration::LinkStyleHover(s) = decl {
        assert!(s.bold);
        assert!(s.italic);
        assert!(s.reverse);
        assert!(!s.underline);
    } else {
        panic!("expected LinkStyleHover declaration");
    }
}

#[test]
fn test_property_link_style_none() {
    let decl = parse_first_declaration("Link { link-style: none; }");
    if let Declaration::LinkStyle(s) = decl {
        assert_eq!(s, TextStyle::default());
    } else {
        panic!("expected LinkStyle declaration");
    }
}

#[test]
fn test_property_link_style_all_modifiers() {
    let decl = parse_first_declaration("Link { link-style: bold dim italic underline blink reverse strike; }");
    if let Declaration::LinkStyle(s) = decl {
        assert!(s.bold);
        assert!(s.dim);
        assert!(s.italic);
        assert!(s.underline);
        assert!(s.blink);
        assert!(s.reverse);
        assert!(s.strike);
    } else {
        panic!("expected LinkStyle declaration");
    }
}

#[test]
fn test_link_multiple_properties() {
    let decls = parse_declarations(
        "Link { link-color: blue; link-background: transparent; link-style: underline; }"
    );
    assert_eq!(decls.len(), 3);

    if let Declaration::LinkColor(c) = &decls[0] {
        assert_eq!(*c, RgbaColor::rgb(0, 0, 255));
    } else {
        panic!("expected LinkColor");
    }

    if let Declaration::LinkBackground(c) = &decls[1] {
        assert!(c.is_transparent());
    } else {
        panic!("expected LinkBackground");
    }

    if let Declaration::LinkStyle(s) = &decls[2] {
        assert!(s.underline);
    } else {
        panic!("expected LinkStyle");
    }
}

#[test]
fn test_property_link_style_theme_variable() {
    let decl = parse_first_declaration("Link { link-style: $link-style; }");
    if let Declaration::LinkStyle(s) = decl {
        assert_eq!(s.theme_var, Some("link-style".to_string()));
        // When using a theme variable, the style flags should be default (false)
        assert!(!s.underline);
        assert!(!s.bold);
    } else {
        panic!("expected LinkStyle declaration, got {:?}", decl);
    }
}

#[test]
fn test_property_link_style_hover_theme_variable() {
    let decl = parse_first_declaration("Link { link-style-hover: $link-style-hover; }");
    if let Declaration::LinkStyleHover(s) = decl {
        assert_eq!(s.theme_var, Some("link-style-hover".to_string()));
    } else {
        panic!("expected LinkStyleHover declaration, got {:?}", decl);
    }
}
