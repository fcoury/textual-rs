use crate::types::border::{BorderEdge, BorderKind};
use crate::types::color::RgbaColor;
use nom::{
    IResult, bytes::complete::take_while1, character::complete::multispace1, combinator::opt,
    sequence::preceded,
};

/// Helper to parse CSS identifiers (alphanumeric + dashes/underscores).
pub fn parse_ident(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)
}

/// Parse a color value.
/// Handles: hex (#rgb, #rrggbb), rgb(), rgba(), hsl(), hsla(), named colors, auto, transparent
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

    // Handle Theme Variables ($primary, etc.)
    if color_str.starts_with('$') {
        // Strip the '$' and store the name
        return Ok((&input[end..], RgbaColor::theme_variable(&color_str[1..])));
    }

    match RgbaColor::parse(color_str) {
        Ok(color) => Ok((&input[end..], color)),
        Err(_) => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
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
        "inner" => BorderKind::Inner,
        "outer" => BorderKind::Outer,
        "round" => BorderKind::Round,
        "solid" => BorderKind::Solid,
        "thick" => BorderKind::Thick,
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
