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
    IResult, bytes::complete::take_while1, character::complete::{char, digit1, multispace1}, combinator::opt,
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

/// Parse a border edge (e.g., "solid red", "round #ff0000").
pub fn parse_border_edge(input: &str) -> IResult<&str, BorderEdge> {
    let (input, kind_str) = parse_ident(input)?;
    let kind = match kind_str.to_lowercase().as_str() {
        "none" | "hidden" => BorderKind::None,
        "ascii" => BorderKind::Ascii,
        "blank" => BorderKind::Blank,
        "block" => BorderKind::Block,
        "dashed" => BorderKind::Dashed,
        "double" => BorderKind::Double,
        "heavy" => BorderKind::Heavy,
        "hkey" => BorderKind::Hkey,
        "inner" => BorderKind::Inner,
        "outer" => BorderKind::Outer,
        "panel" => BorderKind::Panel,
        "round" => BorderKind::Round,
        "solid" => BorderKind::Solid,
        "tall" => BorderKind::Tall,
        "thick" => BorderKind::Thick,
        "vkey" => BorderKind::Vkey,
        "wide" => BorderKind::Wide,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };

    // Color is optional - some borders like "none" or "blank" may not have a color
    let (input, color) = opt(preceded(multispace1, parse_color))(input)?;

    Ok((input, BorderEdge { kind, color }))
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
