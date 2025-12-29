//! Comprehensive tests for Style parsing and manipulation.

use rich::{Color, Style};

// ============================================================================
// Single Modifiers
// ============================================================================

#[test]
fn parse_bold() {
    let style = Style::parse("bold").unwrap();
    assert!(style.text.bold);
    assert!(!style.text.italic);
    assert!(!style.text.underline);
}

#[test]
fn parse_bold_shorthand() {
    let style = Style::parse("b").unwrap();
    assert!(style.text.bold);
}

#[test]
fn parse_italic() {
    let style = Style::parse("italic").unwrap();
    assert!(style.text.italic);
}

#[test]
fn parse_italic_shorthand() {
    let style = Style::parse("i").unwrap();
    assert!(style.text.italic);
}

#[test]
fn parse_underline() {
    let style = Style::parse("underline").unwrap();
    assert!(style.text.underline);
}

#[test]
fn parse_underline_shorthand() {
    let style = Style::parse("u").unwrap();
    assert!(style.text.underline);
}

#[test]
fn parse_strike() {
    let style = Style::parse("strike").unwrap();
    assert!(style.text.strike);
}

#[test]
fn parse_strikethrough() {
    let style = Style::parse("strikethrough").unwrap();
    assert!(style.text.strike);
}

#[test]
fn parse_strike_shorthand() {
    let style = Style::parse("s").unwrap();
    assert!(style.text.strike);
}

#[test]
fn parse_dim() {
    let style = Style::parse("dim").unwrap();
    assert!(style.text.dim);
}

#[test]
fn parse_dim_shorthand() {
    let style = Style::parse("d").unwrap();
    assert!(style.text.dim);
}

#[test]
fn parse_reverse() {
    let style = Style::parse("reverse").unwrap();
    assert!(style.text.reverse);
}

#[test]
fn parse_reverse_shorthand() {
    let style = Style::parse("r").unwrap();
    assert!(style.text.reverse);
}

#[test]
fn parse_blink() {
    let style = Style::parse("blink").unwrap();
    assert!(style.text.blink);
}

// ============================================================================
// Combined Modifiers
// ============================================================================

#[test]
fn parse_bold_italic() {
    let style = Style::parse("bold italic").unwrap();
    assert!(style.text.bold);
    assert!(style.text.italic);
}

#[test]
fn parse_all_modifiers() {
    let style = Style::parse("bold italic underline strike dim reverse blink").unwrap();
    assert!(style.text.bold);
    assert!(style.text.italic);
    assert!(style.text.underline);
    assert!(style.text.strike);
    assert!(style.text.dim);
    assert!(style.text.reverse);
    assert!(style.text.blink);
}

#[test]
fn parse_shorthand_combination() {
    let style = Style::parse("b i u s").unwrap();
    assert!(style.text.bold);
    assert!(style.text.italic);
    assert!(style.text.underline);
    assert!(style.text.strike);
}

// ============================================================================
// Foreground Colors
// ============================================================================

#[test]
fn parse_fg_named() {
    let style = Style::parse("red").unwrap();
    assert_eq!(style.fg, Some(Color::Named("red".into())));
    assert!(style.bg.is_none());
}

#[test]
fn parse_fg_hex() {
    let style = Style::parse("#ff5733").unwrap();
    assert_eq!(style.fg, Some(Color::Rgb(255, 87, 51)));
}

#[test]
fn parse_fg_rgb() {
    // Note: Style::parse splits on whitespace, so rgb with spaces doesn't work
    // Use no-space version instead
    let style = Style::parse("rgb(128,64,32)").unwrap();
    assert_eq!(style.fg, Some(Color::Rgb(128, 64, 32)));
}

// ============================================================================
// Background Colors
// ============================================================================

#[test]
fn parse_bg_named() {
    let style = Style::parse("on blue").unwrap();
    assert!(style.fg.is_none());
    assert_eq!(style.bg, Some(Color::Named("blue".into())));
}

#[test]
fn parse_bg_hex() {
    let style = Style::parse("on #336699").unwrap();
    assert_eq!(style.bg, Some(Color::Rgb(51, 102, 153)));
}

// ============================================================================
// Combined Colors
// ============================================================================

#[test]
fn parse_fg_and_bg() {
    let style = Style::parse("white on blue").unwrap();
    assert_eq!(style.fg, Some(Color::Named("white".into())));
    assert_eq!(style.bg, Some(Color::Named("blue".into())));
}

#[test]
fn parse_fg_bg_and_modifier() {
    let style = Style::parse("bold red on white").unwrap();
    assert!(style.text.bold);
    assert_eq!(style.fg, Some(Color::Named("red".into())));
    assert_eq!(style.bg, Some(Color::Named("white".into())));
}

#[test]
fn parse_complex_style() {
    let style = Style::parse("bold italic #ff5733 on #336699").unwrap();
    assert!(style.text.bold);
    assert!(style.text.italic);
    assert_eq!(style.fg, Some(Color::Rgb(255, 87, 51)));
    assert_eq!(style.bg, Some(Color::Rgb(51, 102, 153)));
}

// ============================================================================
// Style Operations
// ============================================================================

#[test]
fn style_apply_fg() {
    let base = Style::parse("red").unwrap();
    let overlay = Style::parse("blue").unwrap();
    let combined = base.apply(&overlay);

    // Overlay's fg takes precedence
    assert_eq!(combined.fg, Some(Color::Named("blue".into())));
}

#[test]
fn style_apply_bg() {
    let base = Style::parse("on red").unwrap();
    let overlay = Style::parse("on blue").unwrap();
    let combined = base.apply(&overlay);

    assert_eq!(combined.bg, Some(Color::Named("blue".into())));
}

#[test]
fn style_apply_modifier() {
    let base = Style::parse("bold").unwrap();
    let overlay = Style::parse("italic").unwrap();
    let combined = base.apply(&overlay);

    // Modifiers are OR'd together
    assert!(combined.text.bold);
    assert!(combined.text.italic);
}

#[test]
fn style_apply_preserves_base() {
    let base = Style::parse("bold red on white").unwrap();
    let overlay = Style::parse("italic").unwrap();
    let combined = base.apply(&overlay);

    assert!(combined.text.bold);
    assert!(combined.text.italic);
    assert_eq!(combined.fg, Some(Color::Named("red".into())));
    assert_eq!(combined.bg, Some(Color::Named("white".into())));
}

#[test]
fn style_apply_overlay_wins() {
    let base = Style::parse("bold red on white").unwrap();
    let overlay = Style::parse("italic blue on black").unwrap();
    let combined = base.apply(&overlay);

    assert!(combined.text.bold);
    assert!(combined.text.italic);
    assert_eq!(combined.fg, Some(Color::Named("blue".into())));
    assert_eq!(combined.bg, Some(Color::Named("black".into())));
}

// ============================================================================
// Style Properties
// ============================================================================

#[test]
fn style_is_empty() {
    assert!(Style::new().is_empty());
    assert!(Style::default().is_empty());
}

#[test]
fn style_not_empty_with_fg() {
    let style = Style::parse("red").unwrap();
    assert!(!style.is_empty());
}

#[test]
fn style_not_empty_with_bg() {
    let style = Style::parse("on blue").unwrap();
    assert!(!style.is_empty());
}

#[test]
fn style_not_empty_with_modifier() {
    let style = Style::parse("bold").unwrap();
    assert!(!style.is_empty());
}

// ============================================================================
// TextStyle Properties
// ============================================================================

#[test]
fn text_style_is_empty() {
    let style = Style::new();
    assert!(style.text.is_empty());
}

#[test]
fn text_style_not_empty_with_modifier() {
    let style = Style::parse("bold").unwrap();
    assert!(!style.text.is_empty());
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn parse_empty_string() {
    let result = Style::parse("");
    assert!(result.is_err());
}

#[test]
fn parse_whitespace_only() {
    let result = Style::parse("   ");
    assert!(result.is_err());
}

#[test]
fn parse_unknown_modifier() {
    let result = Style::parse("notamodifier");
    assert!(result.is_err());
}

#[test]
fn parse_on_without_color() {
    let result = Style::parse("on");
    assert!(result.is_err());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn parse_extra_whitespace() {
    let style = Style::parse("  bold   red   on   blue  ").unwrap();
    assert!(style.text.bold);
    assert_eq!(style.fg, Some(Color::Named("red".into())));
    assert_eq!(style.bg, Some(Color::Named("blue".into())));
}

#[test]
fn parse_mixed_case() {
    let style = Style::parse("BOLD Red ON Blue").unwrap();
    assert!(style.text.bold);
    assert_eq!(style.fg, Some(Color::Named("red".into())));
    assert_eq!(style.bg, Some(Color::Named("blue".into())));
}
