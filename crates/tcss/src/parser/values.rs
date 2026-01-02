//! Value parsing for TCSS properties.
//!
//! This module handles parsing of CSS property values:
//!
//! - Colors: `red`, `#ff0000`, `rgb(255,0,0)`, `hsl(0,100%,50%)`
//! - Borders: `solid red`, `round #ff0000`
//! - Text alignment: `left`, `center`, `right`, `justify`
//! - Identifiers: Generic CSS identifier parsing
//!
//! ## Color Formats
//!
//! Supports all standard CSS color formats:
//! - Named colors: `red`, `blue`, `aliceblue`
//! - Hex: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`
//! - RGB: `rgb(r, g, b)`, `rgb(r, g, b, a)`
//! - HSL: `hsl(h, s%, l%)`, `hsla(h, s%, l%, a)`
//! - Special: `auto`, `transparent`
//! - Theme variables: `$primary`, `$panel`

use crate::types::border::{BorderEdge, BorderKind};
use crate::types::color::RgbaColor;
use crate::types::scrollbar::{ScrollbarGutter, ScrollbarSize, ScrollbarVisibility};
use crate::types::text::TextStyle;
use nom::{
    IResult,
    bytes::complete::take_while1,
    character::complete::{char, digit1, multispace1},
    combinator::opt,
    sequence::preceded,
};

/// Parses a CSS identifier (alphanumeric characters, dashes, and underscores).
///
/// Identifiers are used for property names, type selectors, class names, etc.
pub fn parse_ident(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)
}

/// Parse a color value.
/// Handles: hex (#rgb, #rrggbb), rgb(), rgba(), hsl(), hsla(), named colors, auto, transparent
/// Also handles optional alpha percentage suffix: `magenta 40%` -> magenta with 40% alpha
pub fn parse_color(input: &str) -> IResult<&str, RgbaColor> {
    let input = input.trim_start();
    let end = find_color_end(input);

    if end == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let color_str = &input[..end];
    let remaining = &input[end..];

    // Handle Theme Variables ($primary, etc.)
    if color_str.starts_with('$') {
        // Strip the '$' and store the name
        let color = RgbaColor::theme_variable(&color_str[1..]);
        // Try to parse optional alpha percentage suffix (e.g., " 40%")
        let (remaining, color) = parse_optional_alpha(remaining, color);
        return Ok((remaining, color));
    }

    match RgbaColor::parse(color_str) {
        Ok(color) => {
            // Try to parse optional alpha percentage suffix (e.g., " 40%")
            let (remaining, color) = parse_optional_alpha(remaining, color);
            Ok((remaining, color))
        }
        Err(_) => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse an optional alpha percentage suffix (e.g., " 40%").
/// Returns the remaining input and the color with updated alpha.
fn parse_optional_alpha(input: &str, mut color: RgbaColor) -> (&str, RgbaColor) {
    // Try to parse: whitespace + number + '%'
    let trimmed = input.trim_start();

    // Look for pattern: digits followed by '%'
    let mut end = 0;
    let mut found_digits = false;
    for (i, c) in trimmed.char_indices() {
        if c.is_ascii_digit() || c == '.' {
            found_digits = true;
            end = i + c.len_utf8();
        } else if c == '%' && found_digits {
            // Parse the percentage
            let percent_str = &trimmed[..end];
            if let Ok(percent) = percent_str.parse::<f32>() {
                color.a = percent / 100.0;
                // Return after the '%'
                return (&trimmed[end + 1..], color);
            }
            break;
        } else {
            break;
        }
    }

    // No alpha percentage found, return original
    (input, color)
}

/// Find the end of a color token, respecting parentheses.
/// For `rgb(255, 0, 0)` returns the index after the closing `)`.
/// For `red` or `#ff0000` returns the index at the first delimiter.
fn find_color_end(input: &str) -> usize {
    let mut paren_depth = 0;
    let mut end = 0;

    for (i, c) in input.char_indices() {
        match c {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    return i + 1;
                }
            }
            // Stop at delimiters if we aren't inside parens
            ';' | '}' if paren_depth == 0 => return i,
            c if c.is_whitespace() && paren_depth == 0 => return i,
            _ => {}
        }
        end = i + c.len_utf8();
    }
    end
}

/// Try to parse a border kind from an identifier string.
fn try_parse_border_kind(s: &str) -> Option<BorderKind> {
    match s.to_lowercase().as_str() {
        "none" | "hidden" => Some(BorderKind::None),
        "ascii" => Some(BorderKind::Ascii),
        "blank" => Some(BorderKind::Blank),
        "block" => Some(BorderKind::Block),
        "dashed" => Some(BorderKind::Dashed),
        "double" => Some(BorderKind::Double),
        "heavy" => Some(BorderKind::Heavy),
        "hkey" => Some(BorderKind::Hkey),
        "inner" => Some(BorderKind::Inner),
        "outer" => Some(BorderKind::Outer),
        "panel" => Some(BorderKind::Panel),
        "round" => Some(BorderKind::Round),
        "solid" => Some(BorderKind::Solid),
        "tall" => Some(BorderKind::Tall),
        "thick" => Some(BorderKind::Thick),
        "vkey" => Some(BorderKind::Vkey),
        "wide" => Some(BorderKind::Wide),
        _ => None,
    }
}

/// Parse a border edge (e.g., "solid red", "round #ff0000", "blue wide").
///
/// Accepts both `<kind> <color>` and `<color> <kind>` orders for compatibility
/// with Python Textual's CSS parser.
pub fn parse_border_edge(input: &str) -> IResult<&str, BorderEdge> {
    let input = input.trim_start();

    // First, try to parse the first token as a border kind
    if let Ok((remaining, kind_str)) = parse_ident(input) {
        if let Some(kind) = try_parse_border_kind(kind_str) {
            // Successfully parsed kind first: "<kind> [<color>]"
            let (remaining, color) = opt(preceded(multispace1, parse_color))(remaining)?;
            return Ok((remaining, BorderEdge { kind, color }));
        }
    }

    // First token wasn't a valid border kind, try parsing as "<color> <kind>"
    let (input, color) = parse_color(input)?;
    let input = input.trim_start();

    // Parse the kind (required when color comes first)
    let (input, kind_str) = parse_ident(input)?;
    let kind = try_parse_border_kind(kind_str).ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;

    Ok((
        input,
        BorderEdge {
            kind,
            color: Some(color),
        },
    ))
}

/// Parse text-alignment keywords.
pub fn parse_text_align(input: &str) -> IResult<&str, crate::types::text::TextAlign> {
    let (input, ident) = parse_ident(input)?;
    use crate::types::text::TextAlign::*;
    match ident.to_lowercase().as_str() {
        "left" => Ok((input, Left)),
        "center" => Ok((input, Center)),
        "right" => Ok((input, Right)),
        "justify" => Ok((input, Justify)),
        "start" => Ok((input, Start)),
        "end" => Ok((input, End)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse a u16 integer.
pub fn parse_u16(input: &str) -> IResult<&str, u16> {
    let (input, digits) = digit1(input)?;
    let value = digits.parse::<u16>().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;
    Ok((input, value))
}

/// Parse scrollbar-size: `<int>` or `<int> <int>`.
/// Single value applies to both horizontal and vertical.
/// Two values: horizontal vertical.
pub fn parse_scrollbar_size(input: &str) -> IResult<&str, ScrollbarSize> {
    let (input, first) = parse_u16(input)?;
    let (input, second) = opt(preceded(multispace1, parse_u16))(input)?;

    let size = match second {
        Some(v) => ScrollbarSize {
            horizontal: first,
            vertical: v,
        },
        None => ScrollbarSize {
            horizontal: first,
            vertical: first,
        },
    };
    Ok((input, size))
}

/// Parse scrollbar-gutter: `auto` or `stable`.
pub fn parse_scrollbar_gutter(input: &str) -> IResult<&str, ScrollbarGutter> {
    let (input, ident) = parse_ident(input)?;
    match ident.to_lowercase().as_str() {
        "auto" => Ok((input, ScrollbarGutter::Auto)),
        "stable" => Ok((input, ScrollbarGutter::Stable)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse scrollbar-visibility: `visible` or `hidden`.
pub fn parse_scrollbar_visibility(input: &str) -> IResult<&str, ScrollbarVisibility> {
    let (input, ident) = parse_ident(input)?;
    match ident.to_lowercase().as_str() {
        "visible" => Ok((input, ScrollbarVisibility::Visible)),
        "hidden" => Ok((input, ScrollbarVisibility::Hidden)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse overflow: `hidden`, `auto`, or `scroll`.
pub fn parse_overflow(input: &str) -> IResult<&str, crate::types::Overflow> {
    use crate::types::Overflow;
    let (input, ident) = parse_ident(input)?;
    match ident.to_lowercase().as_str() {
        "hidden" => Ok((input, Overflow::Hidden)),
        "auto" => Ok((input, Overflow::Auto)),
        "scroll" => Ok((input, Overflow::Scroll)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse overflow shorthand: `overflow: <x> [<y>]`.
pub fn parse_overflow_shorthand(
    input: &str,
) -> IResult<&str, (crate::types::Overflow, crate::types::Overflow)> {
    let (input, first) = parse_overflow(input)?;
    let (input, second) = opt(preceded(multispace1, parse_overflow))(input)?;
    let second = second.unwrap_or(first);
    Ok((input, (first, second)))
}

/// Parse box-sizing: `content-box` or `border-box`.
pub fn parse_box_sizing(input: &str) -> IResult<&str, crate::types::BoxSizing> {
    use crate::types::BoxSizing;
    let (input, ident) = parse_ident(input)?;
    // Handle hyphenated identifiers by consuming the rest
    let (input, rest) = opt(preceded(char('-'), parse_ident))(input)?;
    let full_ident = match rest {
        Some(suffix) => format!("{}-{}", ident, suffix),
        None => ident.to_string(),
    };
    match full_ident.to_lowercase().as_str() {
        "content-box" | "contentbox" => Ok((input, BoxSizing::ContentBox)),
        "border-box" | "borderbox" => Ok((input, BoxSizing::BorderBox)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse display: `block` or `none`.
pub fn parse_display(input: &str) -> IResult<&str, crate::types::Display> {
    use crate::types::Display;
    let (input, ident) = parse_ident(input)?;
    match ident.to_lowercase().as_str() {
        "block" => Ok((input, Display::Block)),
        "none" => Ok((input, Display::None)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse visibility: `visible` or `hidden`.
pub fn parse_visibility(input: &str) -> IResult<&str, crate::types::Visibility> {
    use crate::types::Visibility;
    let (input, ident) = parse_ident(input)?;
    match ident.to_lowercase().as_str() {
        "visible" => Ok((input, Visibility::Visible)),
        "hidden" => Ok((input, Visibility::Hidden)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse position: `relative` or `absolute`.
pub fn parse_position(input: &str) -> IResult<&str, crate::types::Position> {
    use crate::types::Position;
    let (input, ident) = parse_ident(input)?;
    match ident.to_lowercase().as_str() {
        "relative" => Ok((input, Position::Relative)),
        "absolute" => Ok((input, Position::Absolute)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse horizontal alignment: `left`, `center`, or `right`.
pub fn parse_align_horizontal(input: &str) -> IResult<&str, crate::types::AlignHorizontal> {
    use crate::types::AlignHorizontal;
    let (input, ident) = parse_ident(input)?;
    match ident.to_lowercase().as_str() {
        "left" => Ok((input, AlignHorizontal::Left)),
        "center" => Ok((input, AlignHorizontal::Center)),
        "right" => Ok((input, AlignHorizontal::Right)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse vertical alignment: `top`, `middle`, or `bottom`.
pub fn parse_align_vertical(input: &str) -> IResult<&str, crate::types::AlignVertical> {
    use crate::types::AlignVertical;
    let (input, ident) = parse_ident(input)?;
    match ident.to_lowercase().as_str() {
        "top" => Ok((input, AlignVertical::Top)),
        "middle" => Ok((input, AlignVertical::Middle)),
        "bottom" => Ok((input, AlignVertical::Bottom)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse content-align shorthand: `<horizontal> <vertical>`.
///
/// Examples: "center middle", "left top", "right bottom"
pub fn parse_content_align(
    input: &str,
) -> IResult<&str, (crate::types::AlignHorizontal, crate::types::AlignVertical)> {
    let (input, h) = parse_align_horizontal(input)?;
    let (input, _) = multispace1(input)?;
    let (input, v) = parse_align_vertical(input)?;

    Ok((input, (h, v)))
}

/// Parse a text style: one or more space-separated keywords or a theme variable.
///
/// Supported keywords: `bold`, `dim`, `italic`, `underline`, `underline2`,
/// `blink`, `blink2`, `reverse`, `strike`, `overline`, `none`.
///
/// Theme variables: `$link-style`, `$link-style-hover`, etc.
///
/// # Examples
///
/// - `"bold"` → TextStyle { bold: true, .. }
/// - `"bold underline"` → TextStyle { bold: true, underline: true, .. }
/// - `"none"` → TextStyle::default()
/// - `"$link-style"` → TextStyle { theme_var: Some("link-style"), .. }
pub fn parse_text_style(input: &str) -> IResult<&str, TextStyle> {
    let input = input.trim_start();

    // Find where the text style value ends (at ; or })
    let end = input
        .find(|c: char| c == ';' || c == '}')
        .unwrap_or(input.len());
    let value_str = input[..end].trim();

    if value_str.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Handle theme variables ($link-style, etc.)
    if value_str.starts_with('$') {
        return Ok((&input[end..], TextStyle::theme_variable(&value_str[1..])));
    }

    let mut style = TextStyle::default();

    // Split by whitespace and parse each keyword
    for keyword in value_str.split_whitespace() {
        match keyword.to_lowercase().as_str() {
            "none" => {
                // none resets all styles
                style = TextStyle::default();
            }
            "bold" => style.bold = true,
            "dim" => style.dim = true,
            "italic" => style.italic = true,
            "underline" => style.underline = true,
            "underline2" => style.underline2 = true,
            "blink" => style.blink = true,
            "blink2" => style.blink2 = true,
            "reverse" => style.reverse = true,
            "strike" => style.strike = true,
            "overline" => style.overline = true,
            _ => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )));
            }
        }
    }

    Ok((&input[end..], style))
}

/// Parse dock: `top`, `bottom`, `left`, or `right`.
///
/// Docking removes a widget from normal layout flow and fixes it
/// to an edge of the container.
pub fn parse_dock(input: &str) -> IResult<&str, crate::types::Dock> {
    use crate::types::Dock;
    let (input, ident) = parse_ident(input)?;
    match ident.to_lowercase().as_str() {
        "top" => Ok((input, Dock::Top)),
        "bottom" => Ok((input, Dock::Bottom)),
        "left" => Ok((input, Dock::Left)),
        "right" => Ok((input, Dock::Right)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

/// Parse layer: a single layer name identifier.
///
/// Layer names are simple identifiers that assign a widget to a rendering layer.
/// Widgets on higher layer indices render on top of lower indices.
///
/// # Examples
///
/// - `layer: above;` → assigns to "above" layer
/// - `layer: ruler;` → assigns to "ruler" layer
pub fn parse_layer(input: &str) -> IResult<&str, String> {
    let input = input.trim_start();
    let (remaining, ident) = parse_ident(input)?;
    Ok((remaining, ident.to_lowercase()))
}

/// Parse layers: a space-separated list of layer names.
///
/// The `layers` property defines available layer names for a container's children.
/// Layers are rendered in order: lower indices first (bottom), higher on top.
///
/// # Examples
///
/// - `layers: below above;` → defines "below" (index 0), "above" (index 1)
/// - `layers: ruler;` → defines single "ruler" layer (index 0)
pub fn parse_layers(input: &str) -> IResult<&str, Vec<String>> {
    let input = input.trim_start();

    // Find where the value ends (at ; or })
    let end = input
        .find(|c: char| c == ';' || c == '}')
        .unwrap_or(input.len());
    let value_str = input[..end].trim();

    if value_str.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Split by whitespace and collect layer names
    let layers: Vec<String> = value_str
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    if layers.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    Ok((&input[end..], layers))
}

/// Parse hatch pattern: `<pattern> <color> [opacity%]`.
///
/// Pattern can be:
/// - Named: `left`, `right`, `cross`, `horizontal`, `vertical`
/// - Custom: A quoted single character like `"T"` or `'X'`
///
/// # Examples
///
/// - `hatch: cross $success` → cross pattern in success color
/// - `hatch: horizontal red 80%` → horizontal pattern in red at 80% opacity
/// - `hatch: "T" blue 50%` → custom 'T' pattern in blue at 50% opacity
pub fn parse_hatch(input: &str) -> IResult<&str, crate::types::Hatch> {
    use crate::types::hatch::{Hatch, HatchPattern};

    let input = input.trim_start();

    // Parse the pattern (identifier or quoted character)
    let (input, pattern) = if input.starts_with('"') || input.starts_with('\'') {
        // Quoted custom character
        let quote = input.chars().next().unwrap();
        let end = input[1..].find(quote).ok_or_else(|| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char))
        })?;
        let custom_char = &input[1..1 + end];
        let pattern = HatchPattern::parse(custom_char).ok_or_else(|| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
        })?;
        (&input[2 + end..], pattern)
    } else {
        // Named pattern
        let (remaining, ident) = parse_ident(input)?;
        let pattern = HatchPattern::parse(ident).ok_or_else(|| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
        })?;
        (remaining, pattern)
    };

    // Consume whitespace
    let input = input.trim_start();

    // Parse the color (required)
    let (input, color) = parse_color(input)?;

    // Parse optional opacity percentage
    let input = input.trim_start();
    let (input, opacity) =
        if !input.is_empty() && !input.starts_with(';') && !input.starts_with('}') {
            // Try to parse opacity percentage
            let mut end = 0;
            let mut found_digits = false;
            for (i, c) in input.char_indices() {
                if c.is_ascii_digit() || c == '.' {
                    found_digits = true;
                    end = i + c.len_utf8();
                } else if c == '%' && found_digits {
                    let percent_str = &input[..end];
                    if let Ok(percent) = percent_str.parse::<f32>() {
                        return Ok((
                            &input[end + 1..],
                            Hatch::new(pattern, color).with_opacity(percent / 100.0),
                        ));
                    }
                    break;
                } else {
                    break;
                }
            }
            (input, 1.0f32)
        } else {
            (input, 1.0f32)
        };

    Ok((input, Hatch::new(pattern, color).with_opacity(opacity)))
}

/// Parse a keyline value.
///
/// Keyline syntax: `<style> <color>`
/// - style: none, thin, heavy, double
/// - color: any valid CSS color
///
/// # Examples
///
/// - `keyline: heavy green` → heavy line style in green
/// - `keyline: thin #ff0000` → thin line style in red
/// - `keyline: double rgb(0, 128, 255)` → double line style in blue
pub fn parse_keyline(input: &str) -> IResult<&str, crate::types::Keyline> {
    use crate::types::keyline::{Keyline, KeylineStyle};

    let input = input.trim_start();

    // Parse the style (required)
    let (input, style_str) = parse_ident(input)?;
    let style = KeylineStyle::parse(style_str).ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;

    // If style is "none", color is optional (defaults to transparent)
    if style == KeylineStyle::None {
        let input = input.trim_start();
        if input.is_empty() || input.starts_with(';') || input.starts_with('}') {
            return Ok((input, Keyline::new(style, RgbaColor::transparent())));
        }
    }

    // Consume whitespace
    let input = input.trim_start();

    // Parse the color (required for non-none styles)
    let (input, color) = parse_color(input)?;

    Ok((input, Keyline::new(style, color)))
}

/// Parse opacity value: accepts 0.0-1.0 or 0%-100%.
///
/// The value is clamped to the range [0.0, 1.0].
///
/// # Examples
///
/// - `opacity: 0.5` → 0.5
/// - `opacity: 50%` → 0.5
/// - `opacity: 0%` → 0.0
/// - `opacity: 100%` → 1.0
pub fn parse_opacity(input: &str) -> IResult<&str, f64> {
    use super::units;
    use crate::types::geometry::Unit;

    let input = input.trim_start();

    // Try to parse as a scalar (handles both plain numbers and percentages)
    if let Ok((remaining, scalar)) = units::parse_scalar(input) {
        let value = if scalar.unit == Unit::Percent {
            scalar.value / 100.0
        } else {
            scalar.value
        };
        return Ok((remaining, value.clamp(0.0, 1.0)));
    }

    // Fallback: try parsing just a number without units
    let (remaining, value_str) =
        take_while1(|c: char| c.is_ascii_digit() || c == '.' || c == '-')(input)?;
    let value: f64 = value_str.parse().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Float))
    })?;

    Ok((remaining, value.clamp(0.0, 1.0)))
}
