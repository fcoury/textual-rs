//! Numeric value and unit parsing for TCSS.
//!
//! This module handles parsing of dimensions and spacing values:
//!
//! - [`parse_scalar`]: Single dimension value (e.g., `10`, `50%`, `auto`)
//! - [`parse_spacing`]: Box model spacing (e.g., `10`, `1 2`, `1 2 3 4`)
//!
//! ## Supported Units
//!
//! - Cells (default): `10` - character cells in the terminal
//! - Percentage: `50%` - percentage of parent dimension
//! - Width: `50w` - percentage of parent width
//! - Height: `50h` - percentage of parent height
//! - Viewport: `50vw`, `50vh` - percentage of viewport
//! - Fraction: `1fr`, `2fr` - grid fraction units
//! - Auto: `auto` - automatic sizing
//!
//! ## Spacing Syntax
//!
//! Follows CSS shorthand conventions:
//! - 1 value: all sides (`margin: 10`)
//! - 2 values: vertical, horizontal (`margin: 1 2`)
//! - 4 values: top, right, bottom, left (`margin: 1 2 3 4`)

use crate::types::Layout;
use crate::types::geometry::{Scalar, Spacing, Unit};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map, map_res, opt, recognize},
    multi::many0,
    sequence::{pair, preceded, tuple},
};

/// Parses a floating point or integer number.
fn parse_number(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            opt(pair(char('.'), digit1)),
        ))),
        |s: &str| s.parse::<f64>(),
    )(input)
}

/// Parse the unit suffix (e.g., %, vw, fr).
fn parse_unit_suffix(input: &str) -> IResult<&str, Unit> {
    alt((
        map(tag("vw"), |_| Unit::ViewWidth),
        map(tag("vh"), |_| Unit::ViewHeight),
        map(tag("fr"), |_| Unit::Fraction),
        map(char('%'), |_| Unit::Percent),
        map(char('w'), |_| Unit::Width),
        map(char('h'), |_| Unit::Height),
    ))(input)
}

/// Parse a single Scalar value (e.g., "10", "50%", "auto").
pub fn parse_scalar(input: &str) -> IResult<&str, Scalar> {
    let input = input.trim_start();

    if let Ok((remaining, _)) = tag::<&str, &str, nom::error::Error<&str>>("auto")(input) {
        return Ok((remaining, Scalar::AUTO));
    }

    let (input, value) = parse_number(input)?;
    let (input, unit) = opt(parse_unit_suffix)(input)?;

    Ok((
        input,
        Scalar {
            value,
            unit: unit.unwrap_or(Unit::Cells),
        },
    ))
}

/// Parse CSS-style spacing (margin/padding).
/// Supports 1 value (all), 2 values (v, h), or 4 values (t, r, b, l).
pub fn parse_spacing(input: &str) -> IResult<&str, Spacing> {
    let (input, first) = parse_scalar(input)?;
    let (input, second) = opt(preceded(multispace0, parse_scalar))(input)?;

    match second {
        None => Ok((input, Spacing::all(first))),
        Some(h) => {
            let (input, third) = opt(preceded(multispace0, parse_scalar))(input)?;
            let (input, fourth) = opt(preceded(multispace0, parse_scalar))(input)?;

            match (third, fourth) {
                (Some(t), Some(l)) => {
                    // 4-value syntax: top, right, bottom, left
                    Ok((
                        input,
                        Spacing {
                            top: first,
                            right: h,
                            bottom: t,
                            left: l,
                        },
                    ))
                }
                _ => {
                    // 2-value syntax: vertical, horizontal
                    Ok((input, Spacing::vertical_horizontal(first, h)))
                }
            }
        }
    }
}

/// Parse a u16 integer value.
pub fn parse_u16(input: &str) -> IResult<&str, u16> {
    let input = input.trim_start();
    map_res(digit1, |s: &str| s.parse::<u16>())(input)
}

/// Parse layout value: vertical, horizontal, or grid.
pub fn parse_layout(input: &str) -> IResult<&str, Layout> {
    let input = input.trim_start();
    alt((
        map(tag("vertical"), |_| Layout::Vertical),
        map(tag("horizontal"), |_| Layout::Horizontal),
        map(tag("grid"), |_| Layout::Grid),
    ))(input)
}

/// Parse grid-size: `<columns>` or `<columns> <rows>`.
pub fn parse_grid_size(input: &str) -> IResult<&str, (u16, Option<u16>)> {
    let (input, cols) = parse_u16(input)?;
    let (input, rows) = opt(preceded(multispace1, parse_u16))(input)?;
    Ok((input, (cols, rows)))
}

/// Parse a list of scalar values (for grid-columns, grid-rows).
pub fn parse_scalar_list(input: &str) -> IResult<&str, Vec<Scalar>> {
    let (input, first) = parse_scalar(input)?;
    let (input, rest) = many0(preceded(multispace1, parse_scalar))(input)?;

    let mut result = vec![first];
    result.extend(rest);
    Ok((input, result))
}

/// Parse grid-gutter: `<vertical>` or `<vertical> <horizontal>`.
pub fn parse_grid_gutter(input: &str) -> IResult<&str, (Scalar, Option<Scalar>)> {
    let (input, v) = parse_scalar(input)?;
    let (input, h) = opt(preceded(multispace1, parse_scalar))(input)?;
    Ok((input, (v, h)))
}

/// Parse offset: `<x> <y>` - two required scalar values.
/// Positive X moves right, positive Y moves down.
pub fn parse_offset(input: &str) -> IResult<&str, (Scalar, Scalar)> {
    let (input, x) = parse_scalar(input)?;
    let (input, y) = preceded(multispace1, parse_scalar)(input)?;
    Ok((input, (x, y)))
}
