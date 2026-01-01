//! Tests for Placeholder auto-sizing with padding
//!
//! These tests verify that Placeholder's desired_size() correctly includes
//! padding in its auto-height and auto-width calculations.

use tcss::types::{ComputedStyle, Scalar};
use textual::widget::placeholder::Placeholder;
use textual::widget::Widget;

fn create_placeholder_with_style(style: ComputedStyle) -> Placeholder {
    let mut placeholder = Placeholder::new().with_label("test");
    // Use Widget trait with () as the message type
    <Placeholder as Widget<()>>::set_style(&mut placeholder, style);
    placeholder
}

fn get_desired_size(placeholder: &Placeholder) -> textual::canvas::Size {
    <Placeholder as Widget<()>>::desired_size(placeholder)
}

// =============================================================================
// Height Tests
// =============================================================================

#[test]
fn test_placeholder_auto_height_with_padding_top() {
    // height: auto + padding-top: 4 → desired height = 1 + 4 = 5
    let mut style = ComputedStyle::default();
    style.height = Some(Scalar::AUTO);
    style.padding.top = Scalar::cells(4.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.height, 5,
        "height: auto with padding-top: 4 should give desired height = 1 (content) + 4 (padding) = 5, got {}",
        size.height
    );
}

#[test]
fn test_placeholder_auto_height_with_padding_bottom() {
    // height: auto + padding-bottom: 4 → desired height = 1 + 4 = 5
    let mut style = ComputedStyle::default();
    style.height = Some(Scalar::AUTO);
    style.padding.bottom = Scalar::cells(4.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.height, 5,
        "height: auto with padding-bottom: 4 should give desired height = 1 + 4 = 5, got {}",
        size.height
    );
}

#[test]
fn test_placeholder_auto_height_with_padding_both() {
    // height: auto + padding: 2 (all sides) → desired height = 1 + 2 + 2 = 5
    let mut style = ComputedStyle::default();
    style.height = Some(Scalar::AUTO);
    style.padding.top = Scalar::cells(2.0);
    style.padding.bottom = Scalar::cells(2.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.height, 5,
        "height: auto with padding 2 (top+bottom) should give desired height = 1 + 2 + 2 = 5, got {}",
        size.height
    );
}

#[test]
fn test_placeholder_default_height_with_padding() {
    // Default height (no explicit height) + padding-top: 3 → desired height = 1 + 3 = 4
    let mut style = ComputedStyle::default();
    // height is None (default)
    style.padding.top = Scalar::cells(3.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.height, 4,
        "default height with padding-top: 3 should give desired height = 1 + 3 = 4, got {}",
        size.height
    );
}

// =============================================================================
// Width Tests
// =============================================================================

#[test]
fn test_placeholder_auto_width_with_padding_left() {
    // width: auto + padding-left: 3 → desired width = 20 + 3 = 23
    let mut style = ComputedStyle::default();
    style.width = Some(Scalar::AUTO);
    style.padding.left = Scalar::cells(3.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.width, 23,
        "width: auto with padding-left: 3 should give desired width = 20 + 3 = 23, got {}",
        size.width
    );
}

#[test]
fn test_placeholder_auto_width_with_padding_right() {
    // width: auto + padding-right: 3 → desired width = 20 + 3 = 23
    let mut style = ComputedStyle::default();
    style.width = Some(Scalar::AUTO);
    style.padding.right = Scalar::cells(3.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.width, 23,
        "width: auto with padding-right: 3 should give desired width = 20 + 3 = 23, got {}",
        size.width
    );
}

#[test]
fn test_placeholder_auto_width_with_padding_both() {
    // width: auto + padding: 2 (left+right) → desired width = 20 + 2 + 2 = 24
    let mut style = ComputedStyle::default();
    style.width = Some(Scalar::AUTO);
    style.padding.left = Scalar::cells(2.0);
    style.padding.right = Scalar::cells(2.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.width, 24,
        "width: auto with padding 2 (left+right) should give desired width = 20 + 2 + 2 = 24, got {}",
        size.width
    );
}

#[test]
fn test_placeholder_default_width_with_padding() {
    // Default width (no explicit width) + padding-left: 5 → desired width = 20 + 5 = 25
    let mut style = ComputedStyle::default();
    // width is None (default)
    style.padding.left = Scalar::cells(5.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.width, 25,
        "default width with padding-left: 5 should give desired width = 20 + 5 = 25, got {}",
        size.width
    );
}

// =============================================================================
// Combined Tests
// =============================================================================

#[test]
fn test_placeholder_auto_both_with_padding() {
    // width: auto + height: auto + padding: 2 (all sides)
    // → width = 20 + 2 + 2 = 24, height = 1 + 2 + 2 = 5
    let mut style = ComputedStyle::default();
    style.width = Some(Scalar::AUTO);
    style.height = Some(Scalar::AUTO);
    style.padding.top = Scalar::cells(2.0);
    style.padding.right = Scalar::cells(2.0);
    style.padding.bottom = Scalar::cells(2.0);
    style.padding.left = Scalar::cells(2.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.width, 24,
        "width should be 20 + 2 + 2 = 24, got {}",
        size.width
    );
    assert_eq!(
        size.height, 5,
        "height should be 1 + 2 + 2 = 5, got {}",
        size.height
    );
}

#[test]
fn test_placeholder_explicit_cells_height_ignores_padding() {
    // height: 10 (explicit cells) + padding-top: 4 → desired height = 10 (unchanged)
    // When height is explicitly set in cells, padding doesn't affect desired_size
    let mut style = ComputedStyle::default();
    style.height = Some(Scalar::cells(10.0));
    style.padding.top = Scalar::cells(4.0);

    let placeholder = create_placeholder_with_style(style);
    let size = get_desired_size(&placeholder);

    assert_eq!(
        size.height, 10,
        "explicit height: 10 should remain 10 regardless of padding, got {}",
        size.height
    );
}
