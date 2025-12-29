//! Integration tests for TCSS value parsing.
//!
//! Tests value syntax as defined in Textual CSS:
//! - Scalar values with units: cells, %, w, h, vw, vh, fr, auto
//! - Spacing values: 1-value, 2-value, 4-value syntax
//! - Color values: hex, rgb, hsl, named colors
//! - Border values: style + color

use tcss::parser::units::{parse_scalar, parse_spacing};
use tcss::parser::values::{parse_border_edge, parse_color};
use tcss::types::border::BorderKind;
use tcss::types::color::RgbaColor;
use tcss::types::geometry::{Scalar, Spacing, Unit};

// ============================================================================
// SCALAR VALUES - CELLS (unitless integers)
// ============================================================================

#[test]
fn test_scalar_integer() {
    let (remaining, scalar) = parse_scalar("10").unwrap();
    assert!(remaining.is_empty() || remaining.starts_with(';') || remaining.starts_with(' '));
    assert_eq!(scalar.value, 10.0);
    assert_eq!(scalar.unit, Unit::Cells);
}

#[test]
fn test_scalar_zero() {
    let (_, scalar) = parse_scalar("0").unwrap();
    assert_eq!(scalar.value, 0.0);
    assert_eq!(scalar.unit, Unit::Cells);
}

#[test]
fn test_scalar_negative() {
    let (_, scalar) = parse_scalar("-5").unwrap();
    assert_eq!(scalar.value, -5.0);
    assert_eq!(scalar.unit, Unit::Cells);
}

#[test]
fn test_scalar_float() {
    let (_, scalar) = parse_scalar("10.5").unwrap();
    assert_eq!(scalar.value, 10.5);
    assert_eq!(scalar.unit, Unit::Cells);
}

// ============================================================================
// SCALAR VALUES - PERCENTAGE (%)
// ============================================================================

#[test]
fn test_scalar_percent() {
    let (_, scalar) = parse_scalar("50%").unwrap();
    assert_eq!(scalar.value, 50.0);
    assert_eq!(scalar.unit, Unit::Percent);
}

#[test]
fn test_scalar_percent_100() {
    let (_, scalar) = parse_scalar("100%").unwrap();
    assert_eq!(scalar.value, 100.0);
    assert_eq!(scalar.unit, Unit::Percent);
}

#[test]
fn test_scalar_percent_float() {
    let (_, scalar) = parse_scalar("33.33%").unwrap();
    assert!((scalar.value - 33.33).abs() < 0.01);
    assert_eq!(scalar.unit, Unit::Percent);
}

// ============================================================================
// SCALAR VALUES - WIDTH/HEIGHT RELATIVE (w, h)
// ============================================================================

#[test]
fn test_scalar_width() {
    let (_, scalar) = parse_scalar("25w").unwrap();
    assert_eq!(scalar.value, 25.0);
    assert_eq!(scalar.unit, Unit::Width);
}

#[test]
fn test_scalar_height() {
    let (_, scalar) = parse_scalar("75h").unwrap();
    assert_eq!(scalar.value, 75.0);
    assert_eq!(scalar.unit, Unit::Height);
}

// ============================================================================
// SCALAR VALUES - VIEWPORT RELATIVE (vw, vh)
// ============================================================================

#[test]
fn test_scalar_viewport_width() {
    let (_, scalar) = parse_scalar("25vw").unwrap();
    assert_eq!(scalar.value, 25.0);
    assert_eq!(scalar.unit, Unit::ViewWidth);
}

#[test]
fn test_scalar_viewport_height() {
    let (_, scalar) = parse_scalar("75vh").unwrap();
    assert_eq!(scalar.value, 75.0);
    assert_eq!(scalar.unit, Unit::ViewHeight);
}

#[test]
fn test_scalar_viewport_100() {
    let (_, scalar) = parse_scalar("100vw").unwrap();
    assert_eq!(scalar.value, 100.0);
    assert_eq!(scalar.unit, Unit::ViewWidth);
}

// ============================================================================
// SCALAR VALUES - FRACTION (fr)
// ============================================================================

#[test]
fn test_scalar_fraction() {
    let (_, scalar) = parse_scalar("1fr").unwrap();
    assert_eq!(scalar.value, 1.0);
    assert_eq!(scalar.unit, Unit::Fraction);
}

#[test]
fn test_scalar_fraction_2() {
    let (_, scalar) = parse_scalar("2fr").unwrap();
    assert_eq!(scalar.value, 2.0);
    assert_eq!(scalar.unit, Unit::Fraction);
}

#[test]
fn test_scalar_fraction_float() {
    let (_, scalar) = parse_scalar("1.5fr").unwrap();
    assert_eq!(scalar.value, 1.5);
    assert_eq!(scalar.unit, Unit::Fraction);
}

// ============================================================================
// SCALAR VALUES - AUTO
// ============================================================================

#[test]
fn test_scalar_auto() {
    let (_, scalar) = parse_scalar("auto").unwrap();
    assert_eq!(scalar.unit, Unit::Auto);
    assert!(scalar.is_auto());
}

#[test]
fn test_scalar_auto_const() {
    let (_, scalar) = parse_scalar("auto").unwrap();
    assert_eq!(scalar, Scalar::AUTO);
}

// ============================================================================
// SPACING VALUES - SINGLE VALUE (all sides)
// ============================================================================

#[test]
fn test_spacing_single_value() {
    let (_, spacing) = parse_spacing("10").unwrap();
    assert_eq!(spacing.top.value, 10.0);
    assert_eq!(spacing.right.value, 10.0);
    assert_eq!(spacing.bottom.value, 10.0);
    assert_eq!(spacing.left.value, 10.0);
}

#[test]
fn test_spacing_single_value_percent() {
    let (_, spacing) = parse_spacing("50%").unwrap();
    assert_eq!(spacing.top.value, 50.0);
    assert_eq!(spacing.top.unit, Unit::Percent);
    assert_eq!(spacing, Spacing::all(Scalar::percent(50.0)));
}

// ============================================================================
// SPACING VALUES - TWO VALUES (vertical, horizontal)
// ============================================================================

#[test]
fn test_spacing_two_values() {
    let (_, spacing) = parse_spacing("10 20").unwrap();
    // vertical = 10, horizontal = 20
    assert_eq!(spacing.top.value, 10.0);
    assert_eq!(spacing.bottom.value, 10.0);
    assert_eq!(spacing.left.value, 20.0);
    assert_eq!(spacing.right.value, 20.0);
}

#[test]
fn test_spacing_two_values_mixed_units() {
    let (_, spacing) = parse_spacing("10% 5").unwrap();
    assert_eq!(spacing.top.unit, Unit::Percent);
    assert_eq!(spacing.left.unit, Unit::Cells);
}

// ============================================================================
// SPACING VALUES - FOUR VALUES (top, right, bottom, left)
// ============================================================================

#[test]
fn test_spacing_four_values() {
    let (_, spacing) = parse_spacing("1 2 3 4").unwrap();
    assert_eq!(spacing.top.value, 1.0);
    assert_eq!(spacing.right.value, 2.0);
    assert_eq!(spacing.bottom.value, 3.0);
    assert_eq!(spacing.left.value, 4.0);
}

#[test]
fn test_spacing_four_values_with_units() {
    let (_, spacing) = parse_spacing("10% 5 20% 15").unwrap();
    assert_eq!(spacing.top.value, 10.0);
    assert_eq!(spacing.top.unit, Unit::Percent);
    assert_eq!(spacing.right.value, 5.0);
    assert_eq!(spacing.right.unit, Unit::Cells);
    assert_eq!(spacing.bottom.value, 20.0);
    assert_eq!(spacing.bottom.unit, Unit::Percent);
    assert_eq!(spacing.left.value, 15.0);
    assert_eq!(spacing.left.unit, Unit::Cells);
}

// ============================================================================
// COLOR VALUES - HEX
// ============================================================================

#[test]
fn test_color_hex_6_digit() {
    let (_, color) = parse_color("#ff0000").unwrap();
    assert_eq!(color, RgbaColor::rgb(255, 0, 0));
}

#[test]
fn test_color_hex_3_digit() {
    let (_, color) = parse_color("#f00").unwrap();
    assert_eq!(color, RgbaColor::rgb(255, 0, 0));
}

#[test]
fn test_color_hex_8_digit_with_alpha() {
    let (_, color) = parse_color("#ff000080").unwrap();
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 0);
    assert!((color.a - 0.5).abs() < 0.1);
}

// ============================================================================
// COLOR VALUES - RGB
// ============================================================================

#[test]
fn test_color_rgb() {
    let (_, color) = parse_color("rgb(255, 0, 0)").unwrap();
    assert_eq!(color, RgbaColor::rgb(255, 0, 0));
}

#[test]
fn test_color_rgb_no_spaces() {
    let (_, color) = parse_color("rgb(255,128,64)").unwrap();
    assert_eq!(color, RgbaColor::rgb(255, 128, 64));
}

#[test]
fn test_color_rgba() {
    let (_, color) = parse_color("rgba(255, 0, 0, 0.5)").unwrap();
    assert_eq!(color.r, 255);
    assert!((color.a - 0.5).abs() < 0.01);
}

// ============================================================================
// COLOR VALUES - HSL
// ============================================================================

#[test]
fn test_color_hsl_red() {
    let (_, color) = parse_color("hsl(0, 100%, 50%)").unwrap();
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 0);
}

#[test]
fn test_color_hsl_green() {
    let (_, color) = parse_color("hsl(120, 100%, 50%)").unwrap();
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 255);
    assert_eq!(color.b, 0);
}

#[test]
fn test_color_hsl_blue() {
    let (_, color) = parse_color("hsl(240, 100%, 50%)").unwrap();
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 255);
}

#[test]
fn test_color_hsla() {
    let (_, color) = parse_color("hsla(0, 100%, 50%, 0.5)").unwrap();
    assert_eq!(color.r, 255);
    assert!((color.a - 0.5).abs() < 0.01);
}

// ============================================================================
// COLOR VALUES - NAMED COLORS
// ============================================================================

#[test]
fn test_color_named_red() {
    let (_, color) = parse_color("red").unwrap();
    assert_eq!(color, RgbaColor::rgb(255, 0, 0));
}

#[test]
fn test_color_named_green() {
    let (_, color) = parse_color("green").unwrap();
    assert_eq!(color, RgbaColor::rgb(0, 128, 0)); // CSS green is 0,128,0
}

#[test]
fn test_color_named_blue() {
    let (_, color) = parse_color("blue").unwrap();
    assert_eq!(color, RgbaColor::rgb(0, 0, 255));
}

#[test]
fn test_color_named_white() {
    let (_, color) = parse_color("white").unwrap();
    assert_eq!(color, RgbaColor::rgb(255, 255, 255));
}

#[test]
fn test_color_named_black() {
    let (_, color) = parse_color("black").unwrap();
    assert_eq!(color, RgbaColor::rgb(0, 0, 0));
}

#[test]
fn test_color_named_extended() {
    let (_, color) = parse_color("coral").unwrap();
    assert_eq!(color, RgbaColor::rgb(255, 127, 80));
}

// ============================================================================
// COLOR VALUES - SPECIAL
// ============================================================================

#[test]
fn test_color_transparent() {
    let (_, color) = parse_color("transparent").unwrap();
    assert!(color.is_transparent());
    assert_eq!(color.a, 0.0);
}

#[test]
fn test_color_auto() {
    let (_, color) = parse_color("auto").unwrap();
    assert!(color.auto);
}

// ============================================================================
// BORDER VALUES
// ============================================================================

#[test]
fn test_border_solid_color() {
    let (_, border) = parse_border_edge("solid red").unwrap();
    assert_eq!(border.kind, BorderKind::Solid);
    assert_eq!(border.color, Some(RgbaColor::rgb(255, 0, 0)));
}

#[test]
fn test_border_heavy_color() {
    let (_, border) = parse_border_edge("heavy #00ff00").unwrap();
    assert_eq!(border.kind, BorderKind::Heavy);
    assert_eq!(border.color, Some(RgbaColor::rgb(0, 255, 0)));
}

#[test]
fn test_border_round() {
    let (_, border) = parse_border_edge("round blue").unwrap();
    assert_eq!(border.kind, BorderKind::Round);
    assert_eq!(border.color, Some(RgbaColor::rgb(0, 0, 255)));
}

#[test]
fn test_border_double() {
    let (_, border) = parse_border_edge("double white").unwrap();
    assert_eq!(border.kind, BorderKind::Double);
    assert_eq!(border.color, Some(RgbaColor::rgb(255, 255, 255)));
}

#[test]
fn test_border_dashed() {
    let (_, border) = parse_border_edge("dashed coral").unwrap();
    assert_eq!(border.kind, BorderKind::Dashed);
}

#[test]
fn test_border_none() {
    let (_, border) = parse_border_edge("none").unwrap();
    assert_eq!(border.kind, BorderKind::None);
}

#[test]
fn test_border_ascii() {
    let (_, border) = parse_border_edge("ascii green").unwrap();
    assert_eq!(border.kind, BorderKind::Ascii);
}

#[test]
fn test_border_blank() {
    let (_, border) = parse_border_edge("blank").unwrap();
    assert_eq!(border.kind, BorderKind::Blank);
}

#[test]
fn test_border_thick() {
    let (_, border) = parse_border_edge("thick crimson").unwrap();
    assert_eq!(border.kind, BorderKind::Thick);
}

#[test]
fn test_border_inner() {
    let (_, border) = parse_border_edge("inner yellow").unwrap();
    assert_eq!(border.kind, BorderKind::Inner);
}

#[test]
fn test_border_outer() {
    let (_, border) = parse_border_edge("outer cyan").unwrap();
    assert_eq!(border.kind, BorderKind::Outer);
}

#[test]
fn test_border_tall() {
    let (_, border) = parse_border_edge("tall blue").unwrap();
    assert_eq!(border.kind, BorderKind::Tall);
}

#[test]
fn test_border_wide() {
    let (_, border) = parse_border_edge("wide green").unwrap();
    assert_eq!(border.kind, BorderKind::Wide);
}

#[test]
fn test_border_panel() {
    let (_, border) = parse_border_edge("panel red").unwrap();
    assert_eq!(border.kind, BorderKind::Panel);
}

#[test]
fn test_border_hkey() {
    let (_, border) = parse_border_edge("hkey white").unwrap();
    assert_eq!(border.kind, BorderKind::Hkey);
}

#[test]
fn test_border_vkey() {
    let (_, border) = parse_border_edge("vkey gray").unwrap();
    assert_eq!(border.kind, BorderKind::Vkey);
}

// ============================================================================
// WHITESPACE HANDLING
// ============================================================================

#[test]
fn test_scalar_with_leading_whitespace() {
    let (_, scalar) = parse_scalar("  10").unwrap();
    assert_eq!(scalar.value, 10.0);
}

#[test]
fn test_color_with_leading_whitespace() {
    let (_, color) = parse_color("  red").unwrap();
    assert_eq!(color, RgbaColor::rgb(255, 0, 0));
}

#[test]
fn test_color_with_alpha_percentage() {
    // Tint syntax: `magenta 40%` means magenta with 40% alpha
    let (remaining, color) = parse_color("magenta 40%").unwrap();
    assert!(remaining.is_empty());
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 255);
    assert!((color.a - 0.4).abs() < 0.01);
}

#[test]
fn test_color_with_alpha_percentage_hex() {
    let (remaining, color) = parse_color("#ff0000 50%").unwrap();
    assert!(remaining.is_empty());
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 0);
    assert!((color.a - 0.5).abs() < 0.01);
}

#[test]
fn test_color_no_alpha_percentage() {
    // Color without alpha percentage should have alpha = 1.0
    let (_, color) = parse_color("blue").unwrap();
    assert_eq!(color.a, 1.0);
}
