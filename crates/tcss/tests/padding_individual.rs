//! Tests for individual padding properties and their cascade behavior.
//!
//! These tests verify that padding-top, padding-right, padding-bottom, padding-left
//! work correctly with the cascade, matching Python Textual's behavior.

use tcss::parser::cascade::{WidgetMeta, compute_style};
use tcss::parser::parse_stylesheet;
use tcss::types::Unit;
use tcss::types::theme::Theme;

fn get_theme() -> Theme {
    Theme::new("test", true)
}

fn button() -> WidgetMeta {
    WidgetMeta {
        type_name: "Button",
        ..Default::default()
    }
}

#[test]
fn test_padding_top_overrides_shorthand() {
    // padding: 1; padding-top: 5; → Spacing(5, 1, 1, 1)
    let css = "Button { padding: 1; padding-top: 5; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = get_theme();
    let style = compute_style(&button(), &[], &stylesheet, &theme);

    assert_eq!(style.padding.top.value, 5.0, "top should be 5");
    assert_eq!(style.padding.right.value, 1.0, "right should be 1");
    assert_eq!(style.padding.bottom.value, 1.0, "bottom should be 1");
    assert_eq!(style.padding.left.value, 1.0, "left should be 1");
}

#[test]
fn test_shorthand_overrides_individual() {
    // padding-top: 5; padding: 2; → Spacing(2, 2, 2, 2)
    let css = "Button { padding-top: 5; padding: 2; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = get_theme();
    let style = compute_style(&button(), &[], &stylesheet, &theme);

    assert_eq!(style.padding.top.value, 2.0, "top should be 2");
    assert_eq!(style.padding.right.value, 2.0, "right should be 2");
    assert_eq!(style.padding.bottom.value, 2.0, "bottom should be 2");
    assert_eq!(style.padding.left.value, 2.0, "left should be 2");
}

#[test]
fn test_multiple_individual_properties() {
    // padding: 1; padding-top: 2; padding-right: 3; → Spacing(2, 3, 1, 1)
    let css = "Button { padding: 1; padding-top: 2; padding-right: 3; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = get_theme();
    let style = compute_style(&button(), &[], &stylesheet, &theme);

    assert_eq!(style.padding.top.value, 2.0, "top should be 2");
    assert_eq!(style.padding.right.value, 3.0, "right should be 3");
    assert_eq!(style.padding.bottom.value, 1.0, "bottom should be 1");
    assert_eq!(style.padding.left.value, 1.0, "left should be 1");
}

#[test]
fn test_all_individual_properties() {
    // padding-top: 1; padding-right: 2; padding-bottom: 3; padding-left: 4;
    // → Spacing(1, 2, 3, 4)
    let css = "Button { padding-top: 1; padding-right: 2; padding-bottom: 3; padding-left: 4; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = get_theme();
    let style = compute_style(&button(), &[], &stylesheet, &theme);

    assert_eq!(style.padding.top.value, 1.0, "top should be 1");
    assert_eq!(style.padding.right.value, 2.0, "right should be 2");
    assert_eq!(style.padding.bottom.value, 3.0, "bottom should be 3");
    assert_eq!(style.padding.left.value, 4.0, "left should be 4");
}

#[test]
fn test_individual_padding_default_zero() {
    // padding-top: 5; (without shorthand first)
    // → Spacing(5, 0, 0, 0)
    let css = "Button { padding-top: 5; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = get_theme();
    let style = compute_style(&button(), &[], &stylesheet, &theme);

    assert_eq!(style.padding.top.value, 5.0, "top should be 5");
    assert_eq!(style.padding.right.value, 0.0, "right should default to 0");
    assert_eq!(
        style.padding.bottom.value, 0.0,
        "bottom should default to 0"
    );
    assert_eq!(style.padding.left.value, 0.0, "left should default to 0");
}

#[test]
fn test_individual_padding_with_percent_unit() {
    let css = "Button { padding-top: 50%; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = get_theme();
    let style = compute_style(&button(), &[], &stylesheet, &theme);

    assert_eq!(style.padding.top.value, 50.0, "top should be 50");
    assert_eq!(
        style.padding.top.unit,
        Unit::Percent,
        "unit should be percent"
    );
}

#[test]
fn test_padding_bottom_overrides_shorthand() {
    // padding: 1; padding-bottom: 5; → Spacing(1, 1, 5, 1)
    let css = "Button { padding: 1; padding-bottom: 5; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = get_theme();
    let style = compute_style(&button(), &[], &stylesheet, &theme);

    assert_eq!(style.padding.top.value, 1.0, "top should be 1");
    assert_eq!(style.padding.right.value, 1.0, "right should be 1");
    assert_eq!(style.padding.bottom.value, 5.0, "bottom should be 5");
    assert_eq!(style.padding.left.value, 1.0, "left should be 1");
}

#[test]
fn test_padding_left_overrides_shorthand() {
    // padding: 1; padding-left: 5; → Spacing(1, 1, 1, 5)
    let css = "Button { padding: 1; padding-left: 5; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = get_theme();
    let style = compute_style(&button(), &[], &stylesheet, &theme);

    assert_eq!(style.padding.top.value, 1.0, "top should be 1");
    assert_eq!(style.padding.right.value, 1.0, "right should be 1");
    assert_eq!(style.padding.bottom.value, 1.0, "bottom should be 1");
    assert_eq!(style.padding.left.value, 5.0, "left should be 5");
}
