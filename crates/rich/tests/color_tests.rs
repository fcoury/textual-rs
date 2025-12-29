//! Comprehensive tests for the Color type.

use rich::{Color, ColorParseError};

// ============================================================================
// Named Colors - Basic
// ============================================================================

#[test]
fn parse_basic_colors() {
    let colors = [
        "black", "white", "red", "green", "blue",
        "yellow", "cyan", "magenta", "aqua", "fuchsia",
    ];

    for color in colors {
        let parsed = Color::parse(color);
        assert!(parsed.is_ok(), "Failed to parse: {}", color);
        assert_eq!(parsed.unwrap(), Color::Named(color.to_lowercase()));
    }
}

#[test]
fn parse_case_insensitive() {
    assert_eq!(Color::parse("RED").unwrap(), Color::Named("red".to_string()));
    assert_eq!(Color::parse("Red").unwrap(), Color::Named("red".to_string()));
    assert_eq!(Color::parse("rEd").unwrap(), Color::Named("red".to_string()));
}

#[test]
fn parse_with_whitespace() {
    assert_eq!(Color::parse("  red  ").unwrap(), Color::Named("red".to_string()));
    assert_eq!(Color::parse("\tblue\t").unwrap(), Color::Named("blue".to_string()));
}

// ============================================================================
// Named Colors - Bright Variants
// ============================================================================

#[test]
fn parse_bright_colors() {
    let colors = [
        "bright_black", "bright_white", "bright_red", "bright_green",
        "bright_blue", "bright_yellow", "bright_cyan", "bright_magenta",
    ];

    for color in colors {
        let parsed = Color::parse(color);
        assert!(parsed.is_ok(), "Failed to parse: {}", color);
    }
}

// ============================================================================
// Named Colors - Extended
// ============================================================================

#[test]
fn parse_extended_colors() {
    let colors = [
        "grey", "gray", "silver", "maroon", "olive", "navy", "purple",
        "teal", "lime", "orange", "pink", "brown", "coral", "gold",
        "indigo", "violet",
    ];

    for color in colors {
        let parsed = Color::parse(color);
        assert!(parsed.is_ok(), "Failed to parse: {}", color);
    }
}

// ============================================================================
// Named Colors - CSS Colors
// ============================================================================

#[test]
fn parse_css_colors_a() {
    let colors = ["aliceblue", "antiquewhite", "aquamarine", "azure"];
    for color in colors {
        assert!(Color::parse(color).is_ok(), "Failed to parse: {}", color);
    }
}

#[test]
fn parse_css_colors_b() {
    let colors = ["beige", "bisque", "blanchedalmond", "blueviolet", "burlywood"];
    for color in colors {
        assert!(Color::parse(color).is_ok(), "Failed to parse: {}", color);
    }
}

#[test]
fn parse_css_colors_c() {
    let colors = [
        "cadetblue", "chartreuse", "chocolate", "cornflowerblue",
        "cornsilk", "crimson",
    ];
    for color in colors {
        assert!(Color::parse(color).is_ok(), "Failed to parse: {}", color);
    }
}

#[test]
fn parse_css_colors_d() {
    let colors = [
        "darkblue", "darkcyan", "darkgoldenrod", "darkgray", "darkgrey",
        "darkgreen", "darkkhaki", "darkmagenta", "darkolivegreen",
        "darkorange", "darkorchid", "darkred", "darksalmon", "darkseagreen",
        "darkslateblue", "darkslategray", "darkslategrey", "darkturquoise",
        "darkviolet", "deeppink", "deepskyblue", "dimgray", "dimgrey",
        "dodgerblue",
    ];
    for color in colors {
        assert!(Color::parse(color).is_ok(), "Failed to parse: {}", color);
    }
}

#[test]
fn parse_css_colors_remaining() {
    let colors = [
        "firebrick", "floralwhite", "forestgreen", "gainsboro", "ghostwhite",
        "goldenrod", "greenyellow", "honeydew", "hotpink", "indianred",
        "ivory", "khaki", "lavender", "lavenderblush", "lawngreen",
        "lemonchiffon", "lightblue", "lightcoral", "lightcyan",
        "lightgoldenrodyellow", "lightgray", "lightgrey", "lightgreen",
        "lightpink", "lightsalmon", "lightseagreen", "lightskyblue",
        "lightslategray", "lightslategrey", "lightsteelblue", "lightyellow",
        "limegreen", "linen",
    ];
    for color in colors {
        assert!(Color::parse(color).is_ok(), "Failed to parse: {}", color);
    }
}

// ============================================================================
// Hex Colors
// ============================================================================

#[test]
fn parse_hex_short_format() {
    assert_eq!(Color::parse("#f00").unwrap(), Color::Rgb(255, 0, 0));
    assert_eq!(Color::parse("#0f0").unwrap(), Color::Rgb(0, 255, 0));
    assert_eq!(Color::parse("#00f").unwrap(), Color::Rgb(0, 0, 255));
    assert_eq!(Color::parse("#fff").unwrap(), Color::Rgb(255, 255, 255));
    assert_eq!(Color::parse("#000").unwrap(), Color::Rgb(0, 0, 0));
}

#[test]
fn parse_hex_long_format() {
    assert_eq!(Color::parse("#ff0000").unwrap(), Color::Rgb(255, 0, 0));
    assert_eq!(Color::parse("#00ff00").unwrap(), Color::Rgb(0, 255, 0));
    assert_eq!(Color::parse("#0000ff").unwrap(), Color::Rgb(0, 0, 255));
    assert_eq!(Color::parse("#ffffff").unwrap(), Color::Rgb(255, 255, 255));
    assert_eq!(Color::parse("#000000").unwrap(), Color::Rgb(0, 0, 0));
}

#[test]
fn parse_hex_mixed_case() {
    assert_eq!(Color::parse("#FF5733").unwrap(), Color::Rgb(255, 87, 51));
    assert_eq!(Color::parse("#ff5733").unwrap(), Color::Rgb(255, 87, 51));
    assert_eq!(Color::parse("#Ff5733").unwrap(), Color::Rgb(255, 87, 51));
}

#[test]
fn parse_hex_all_digits() {
    assert_eq!(Color::parse("#123456").unwrap(), Color::Rgb(18, 52, 86));
    assert_eq!(Color::parse("#abcdef").unwrap(), Color::Rgb(171, 205, 239));
}

// ============================================================================
// RGB Function
// ============================================================================

#[test]
fn parse_rgb_basic() {
    assert_eq!(Color::parse("rgb(255, 0, 0)").unwrap(), Color::Rgb(255, 0, 0));
    assert_eq!(Color::parse("rgb(0, 255, 0)").unwrap(), Color::Rgb(0, 255, 0));
    assert_eq!(Color::parse("rgb(0, 0, 255)").unwrap(), Color::Rgb(0, 0, 255));
}

#[test]
fn parse_rgb_no_spaces() {
    assert_eq!(Color::parse("rgb(255,0,0)").unwrap(), Color::Rgb(255, 0, 0));
    assert_eq!(Color::parse("rgb(10,20,30)").unwrap(), Color::Rgb(10, 20, 30));
}

#[test]
fn parse_rgb_extra_spaces() {
    assert_eq!(Color::parse("rgb( 255 , 0 , 0 )").unwrap(), Color::Rgb(255, 0, 0));
    assert_eq!(Color::parse("rgb(  128,   128,   128  )").unwrap(), Color::Rgb(128, 128, 128));
}

#[test]
fn parse_rgb_boundary_values() {
    assert_eq!(Color::parse("rgb(0, 0, 0)").unwrap(), Color::Rgb(0, 0, 0));
    assert_eq!(Color::parse("rgb(255, 255, 255)").unwrap(), Color::Rgb(255, 255, 255));
    assert_eq!(Color::parse("rgb(128, 128, 128)").unwrap(), Color::Rgb(128, 128, 128));
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn parse_unknown_color_name() {
    let result = Color::parse("notacolor");
    assert!(result.is_err());
    assert!(matches!(result, Err(ColorParseError::UnknownName(_))));
}

#[test]
fn parse_empty_string() {
    let result = Color::parse("");
    assert!(result.is_err());
}

#[test]
fn parse_whitespace_only() {
    let result = Color::parse("   ");
    assert!(result.is_err());
}

#[test]
fn parse_invalid_hex_length() {
    assert!(Color::parse("#f").is_err());
    assert!(Color::parse("#ff").is_err());
    assert!(Color::parse("#ffff").is_err());
    assert!(Color::parse("#fffff").is_err());
    assert!(Color::parse("#fffffff").is_err());
}

#[test]
fn parse_invalid_hex_characters() {
    assert!(Color::parse("#gg0000").is_err());
    assert!(Color::parse("#xyz123").is_err());
    assert!(Color::parse("#g00").is_err());
}

#[test]
fn parse_rgb_overflow() {
    assert!(Color::parse("rgb(256, 0, 0)").is_err());
    assert!(Color::parse("rgb(0, 300, 0)").is_err());
    assert!(Color::parse("rgb(0, 0, 1000)").is_err());
}

#[test]
fn parse_rgb_negative() {
    assert!(Color::parse("rgb(-1, 0, 0)").is_err());
}

#[test]
fn parse_rgb_wrong_arg_count() {
    assert!(Color::parse("rgb(255, 0)").is_err());
    assert!(Color::parse("rgb(255, 0, 0, 0)").is_err());
    assert!(Color::parse("rgb(255)").is_err());
}

// ============================================================================
// Color to RGB Conversion
// ============================================================================

#[test]
fn named_color_to_rgb() {
    assert_eq!(Color::Named("red".into()).to_rgb(), (255, 0, 0));
    assert_eq!(Color::Named("green".into()).to_rgb(), (0, 128, 0));
    assert_eq!(Color::Named("blue".into()).to_rgb(), (0, 0, 255));
    assert_eq!(Color::Named("black".into()).to_rgb(), (0, 0, 0));
    assert_eq!(Color::Named("white".into()).to_rgb(), (255, 255, 255));
}

#[test]
fn rgb_to_rgb() {
    assert_eq!(Color::Rgb(10, 20, 30).to_rgb(), (10, 20, 30));
    assert_eq!(Color::Rgb(255, 128, 64).to_rgb(), (255, 128, 64));
}

#[test]
fn grey_gray_equivalence() {
    let grey = Color::parse("grey").unwrap();
    let gray = Color::parse("gray").unwrap();
    assert_eq!(grey.to_rgb(), gray.to_rgb());
}

#[test]
fn cyan_aqua_equivalence() {
    let cyan = Color::Named("cyan".into());
    let aqua = Color::Named("aqua".into());
    assert_eq!(cyan.to_rgb(), aqua.to_rgb());
}

#[test]
fn magenta_fuchsia_equivalence() {
    let magenta = Color::Named("magenta".into());
    let fuchsia = Color::Named("fuchsia".into());
    assert_eq!(magenta.to_rgb(), fuchsia.to_rgb());
}
