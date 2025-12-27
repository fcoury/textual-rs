//! Integration tests for TCSS property parsing.
//!
//! Tests property declarations as defined in Textual CSS:
//! - Color properties: color, background
//! - Dimension properties: width, height
//! - Box model properties: margin, padding
//! - Border properties: border
//!
//! Properties not yet implemented are marked with #[ignore]

use tcss::parser::{parse_rule, Declaration};
use tcss::types::border::BorderKind;
use tcss::types::color::RgbaColor;
use tcss::types::geometry::{Scalar, Spacing, Unit};

/// Helper to parse a simple rule and extract declarations
fn parse_declarations(input: &str) -> Vec<Declaration> {
    let (_, rule) = parse_rule(input).expect("failed to parse rule");
    rule.declarations
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
#[ignore = "layout property not yet implemented"]
fn test_property_layout_horizontal() {
    let decl = parse_first_declaration("Button { layout: horizontal; }");
    // Should parse as Declaration::Layout(Layout::Horizontal)
    assert!(matches!(decl, Declaration::Unknown(_)));
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
#[ignore = "content-align property not yet implemented"]
fn test_property_content_align() {
    let decl = parse_first_declaration("Button { content-align: center middle; }");
    // Should parse as Declaration::ContentAlign(Center, Middle)
    assert!(matches!(decl, Declaration::Unknown(_)));
}

#[test]
#[ignore = "overflow property not yet implemented"]
fn test_property_overflow() {
    let decl = parse_first_declaration("Button { overflow: auto; }");
    // Should parse as Declaration::Overflow(Overflow::Auto)
    assert!(matches!(decl, Declaration::Unknown(_)));
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
