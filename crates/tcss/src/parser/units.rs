use crate::types::geometry::{Scalar, Spacing, Unit};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    sequence::{pair, preceded, tuple},
};

/// Parse a floating point or integer number.
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
