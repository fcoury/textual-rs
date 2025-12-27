use crate::parser::{
    Combinator, ComplexSelector, CompoundSelector, Selector, SelectorPart, values::parse_ident,
};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::take_until,
    character::complete::{char, multispace0},
    combinator::map,
    multi::many0,
    sequence::{delimited, preceded},
};

/// Parses a simple selector: Type, .Class, or #ID.
pub fn parse_simple_selector(input: &str) -> IResult<&str, Selector> {
    alt((
        map(preceded(char('#'), parse_ident), |s| {
            Selector::Id(s.to_string())
        }),
        map(preceded(char('.'), parse_ident), |s| {
            Selector::Class(s.to_string())
        }),
        map(preceded(char(':'), parse_ident), |s| {
            Selector::PseudoClass(s.to_string())
        }),
        map(char('&'), |_| Selector::Parent),
        map(char('*'), |_| Selector::Universal),
        parse_attribute_selector,
        map(parse_ident, |s| Selector::Type(s.to_string())),
    ))(input)
}

/// Parses a compound selector (e.g., "Button.primary#submit").
pub fn parse_compound_selector(input: &str) -> IResult<&str, CompoundSelector> {
    let (input, first) = parse_simple_selector(input)?;
    // Use many0 with no preceded(multispace0) to allow chaining like Button:hover
    let (input, rest) = many0(parse_simple_selector)(input)?;

    let mut selectors = vec![first];
    selectors.extend(rest);
    Ok((input, CompoundSelector::new(selectors)))
}

/// Parses a complex selector with combinators (e.g., "Container > Button").
pub fn parse_complex_selector(input: &str) -> IResult<&str, ComplexSelector> {
    // 1. Parse the first compound selector (e.g., "Container")
    let (mut input, mut current_compound) = parse_compound_selector(input)?;
    let mut parts = Vec::new();

    loop {
        // Peek at whitespace
        let (rem, ws) = multispace0(input)?;

        // 2. Try to match an explicit symbolic combinator: >, +, or ~
        let combinator_match: IResult<&str, Combinator> = alt((
            map(char('>'), |_| Combinator::Child),
            map(char('+'), |_| Combinator::AdjacentSibling),
            map(char('~'), |_| Combinator::GeneralSibling),
        ))(rem);

        if let Ok((after_op, found_combinator)) = combinator_match {
            // We found an operator like '>'. Now parse the next compound selector.
            let (after_ws, _) = multispace0(after_op)?;
            match parse_compound_selector(after_ws) {
                Ok((next_input, next_compound)) => {
                    parts.push(SelectorPart::new(current_compound, found_combinator));
                    current_compound = next_compound;
                    input = next_input;
                    continue;
                }
                Err(_) => break, // Trailing operator, shouldn't happen in valid CSS
            }
        }

        // 3. If no operator, check for a Descendant Combinator (whitespace)
        if !ws.is_empty() {
            // If there was whitespace, try to parse another compound selector
            match parse_compound_selector(rem) {
                Ok((next_input, next_compound)) => {
                    parts.push(SelectorPart::new(current_compound, Combinator::Descendant));
                    current_compound = next_compound;
                    input = next_input;
                    continue;
                }
                Err(_) => break, // Just trailing whitespace
            }
        }

        // No more combinators found
        break;
    }

    // 4. The last part always has Combinator::None
    parts.push(SelectorPart::new(current_compound, Combinator::None));
    Ok((input, ComplexSelector::new(parts)))
}

fn parse_attribute_selector(input: &str) -> IResult<&str, Selector> {
    let (input, content) = delimited(char('['), take_until("]"), char(']'))(input)?;

    if let Some(idx) = content.find('=') {
        let name = content[..idx].trim();
        let value = content[idx + 1..].trim();
        Ok((
            input,
            Selector::Attribute(name.to_string(), value.to_string()),
        ))
    } else {
        // Handle [attr] existence selector
        Ok((
            input,
            Selector::Attribute(content.trim().to_string(), "".to_string()),
        ))
    }
}
