//! TCSS parsing and stylesheet data structures.
//!
//! This module provides the core parsing functionality for TCSS stylesheets,
//! including:
//!
//! - [`parse_stylesheet`]: Main entry point for parsing TCSS source
//! - [`StyleSheet`]: Represents a complete parsed stylesheet
//! - [`Rule`]: A CSS rule with selectors and declarations
//! - [`Declaration`]: A property-value pair like `color: red`
//! - Selector types: [`Selector`], [`CompoundSelector`], [`ComplexSelector`]
//!
//! ## Submodules
//!
//! - [`cascade`]: CSS specificity and style computation
//! - [`selectors`]: Selector parsing (type, class, ID, combinators)
//! - [`stylesheet`]: Core data structures for rules and declarations
//! - [`units`]: Numeric value and unit parsing
//! - [`values`]: Color, border, and other value parsing
//! - [`variables`]: CSS variable extraction and resolution
//! - [`flatten`]: Nested rule flattening (for `&` parent selector support)
//!
//! ## Example
//!
//! ```rust
//! use tcss::parser::{parse_stylesheet, Declaration, Selector};
//!
//! let stylesheet = parse_stylesheet("Button { color: red; }").unwrap();
//! let rule = &stylesheet.rules[0];
//!
//! // Check the selector
//! assert_eq!(
//!     rule.selectors.selectors[0].parts[0].compound.selectors[0],
//!     Selector::Type("Button".to_string())
//! );
//! ```

pub mod cascade;
pub mod flatten;
pub mod selectors;
pub mod stylesheet;
pub mod units;
pub mod values;
pub mod variables;

pub use crate::parser::flatten::flatten_stylesheet;
pub use crate::parser::stylesheet::{
    Combinator, ComplexSelector, CompoundSelector, Declaration, Rule, RuleItem, Selector,
    SelectorList, SelectorPart, Specificity, StyleSheet,
};
pub use crate::parser::variables::{extract_variables, resolve_variables};

use crate::parser::selectors::parse_complex_selector;
use crate::parser::values::parse_ident;
use crate::TcssError;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::{
    IResult,
    character::complete::{char, multispace0},
    combinator::{map, opt},
    multi::many0,
    sequence::{delimited, preceded, tuple},
};

/// Parses a full TCSS stylesheet, including variable resolution.
pub fn parse_stylesheet(source: &str) -> Result<StyleSheet, TcssError> {
    let vars = extract_variables(source);
    let resolved_source = resolve_variables(source, &vars)?;

    let (remaining, raw_rules) =
        many0(parse_rule)(&resolved_source).map_err(|e| TcssError::InvalidSyntax(e.to_string()))?;

    if !remaining.trim().is_empty() {
        return Err(TcssError::InvalidSyntax(format!(
            "Unexpected tokens at end of stylesheet: {}",
            remaining.trim()
        )));
    }

    Ok(flatten_stylesheet(raw_rules))
}

/// Top-level parser for a CSS rule (e.g., "Button { color: red; }").
pub fn parse_rule(input: &str) -> IResult<&str, Rule> {
    let (input, _) = multispace0(input)?;
    let (input, selectors) = parse_selector_list(input)?;
    let (input, _) = multispace0(input)?;

    let (input, items) = delimited(
        char('{'),
        parse_rule_items,
        preceded(multispace0, char('}')),
    )(input)?;

    Ok((input, Rule::new(selectors, items)))
}

/// Parses either a declaration (color: red) or a nested rule (&:hover { ... })
fn parse_rule_items(input: &str) -> IResult<&str, Vec<RuleItem>> {
    many0(alt((
        map(parse_rule, RuleItem::NestedRule),
        map(parse_single_declaration, RuleItem::Declaration),
    )))(input)
}

/// Parses a comma-separated list of selectors (e.g., "Button, .primary").
pub fn parse_selector_list(input: &str) -> IResult<&str, SelectorList> {
    let (input, _) = multispace0(input)?;
    let (input, first) = parse_complex_selector(input)?;
    let (input, rest) = many0(preceded(
        tuple((multispace0, char(','), multispace0)),
        parse_complex_selector,
    ))(input)?;

    let mut selectors = vec![first];
    selectors.extend(rest);
    Ok((input, SelectorList::new(selectors)))
}

/// Parses multiple declarations inside a block.
pub fn parse_declarations(input: &str) -> IResult<&str, Vec<Declaration>> {
    many0(parse_single_declaration)(input)
}

/// Dispatches parsing based on property name.
fn parse_single_declaration(input: &str) -> IResult<&str, Declaration> {
    let (input, _) = multispace0(input)?;

    // If we see a '{' or a selector pattern starting with '&', '.', '#', or ident followed by
    // '{' it's a nested rule. For now, we'll consume it to allow the main rule to finish
    // parsing.
    if input.starts_with('&') || input.starts_with('.') || input.starts_with('#') {
        if let Ok((after_nested, _)) = take_until_balanced_braces(input) {
            return Ok((
                after_nested,
                Declaration::Unknown("nested-rule".to_string()),
            ));
        }
    }

    let (input, property) = parse_ident(input)?;
    let (input, _) = tuple((multispace0, char(':'), multispace0))(input)?;

    let (input, declaration) = match property {
        "color" => map(values::parse_color, Declaration::Color)(input)?,
        "background" => map(values::parse_color, Declaration::Background)(input)?,
        "tint" => map(values::parse_color, Declaration::Tint)(input)?,
        "background-tint" => map(values::parse_color, Declaration::BackgroundTint)(input)?,
        "width" => map(units::parse_scalar, Declaration::Width)(input)?,
        "height" => map(units::parse_scalar, Declaration::Height)(input)?,
        "max-height" => map(units::parse_scalar, Declaration::MaxHeight)(input)?,
        "max-width" => map(units::parse_scalar, Declaration::MaxWidth)(input)?,
        "min-height" => map(units::parse_scalar, Declaration::MinHeight)(input)?,
        "min-width" => map(units::parse_scalar, Declaration::MinWidth)(input)?,
        "margin" => map(units::parse_spacing, Declaration::Margin)(input)?,
        "margin-top" => map(units::parse_scalar, Declaration::MarginTop)(input)?,
        "margin-right" => map(units::parse_scalar, Declaration::MarginRight)(input)?,
        "margin-bottom" => map(units::parse_scalar, Declaration::MarginBottom)(input)?,
        "margin-left" => map(units::parse_scalar, Declaration::MarginLeft)(input)?,
        "padding" => map(units::parse_spacing, Declaration::Padding)(input)?,
        "padding-top" => map(units::parse_scalar, Declaration::PaddingTop)(input)?,
        "padding-right" => map(units::parse_scalar, Declaration::PaddingRight)(input)?,
        "padding-bottom" => map(units::parse_scalar, Declaration::PaddingBottom)(input)?,
        "padding-left" => map(units::parse_scalar, Declaration::PaddingLeft)(input)?,
        "border" => map(values::parse_border_edge, Declaration::Border)(input)?,

        // Scrollbar properties
        "scrollbar-color" => map(values::parse_color, Declaration::ScrollbarColor)(input)?,
        "scrollbar-color-hover" => map(values::parse_color, Declaration::ScrollbarColorHover)(input)?,
        "scrollbar-color-active" => map(values::parse_color, Declaration::ScrollbarColorActive)(input)?,
        "scrollbar-background" => map(values::parse_color, Declaration::ScrollbarBackground)(input)?,
        "scrollbar-background-hover" => map(values::parse_color, Declaration::ScrollbarBackgroundHover)(input)?,
        "scrollbar-background-active" => map(values::parse_color, Declaration::ScrollbarBackgroundActive)(input)?,
        "scrollbar-corner-color" => map(values::parse_color, Declaration::ScrollbarCornerColor)(input)?,
        "scrollbar-size" => map(values::parse_scrollbar_size, Declaration::ScrollbarSize)(input)?,
        "scrollbar-size-horizontal" => map(values::parse_u16, Declaration::ScrollbarSizeHorizontal)(input)?,
        "scrollbar-size-vertical" => map(values::parse_u16, Declaration::ScrollbarSizeVertical)(input)?,
        "scrollbar-gutter" => map(values::parse_scrollbar_gutter, Declaration::ScrollbarGutter)(input)?,
        "scrollbar-visibility" => map(values::parse_scrollbar_visibility, Declaration::ScrollbarVisibility)(input)?,

        // Box model properties
        "box-sizing" => map(values::parse_box_sizing, Declaration::BoxSizing)(input)?,

        // Display properties
        "display" => map(values::parse_display, Declaration::Display)(input)?,
        "visibility" => map(values::parse_visibility, Declaration::Visibility)(input)?,
        "opacity" => map(values::parse_opacity, Declaration::Opacity)(input)?,
        "position" => map(values::parse_position, Declaration::Position)(input)?,

        // Overflow properties
        "overflow-x" => map(values::parse_overflow, Declaration::OverflowX)(input)?,
        "overflow-y" => map(values::parse_overflow, Declaration::OverflowY)(input)?,

        // Layout and Grid properties
        "layout" => map(units::parse_layout, Declaration::Layout)(input)?,
        "dock" => map(values::parse_dock, Declaration::Dock)(input)?,
        "layers" => map(values::parse_layers, Declaration::Layers)(input)?,
        "layer" => map(values::parse_layer, Declaration::Layer)(input)?,
        "grid-size" => {
            let (input, (cols, rows)) = units::parse_grid_size(input)?;
            (input, Declaration::GridSize(cols, rows))
        }
        "grid-columns" => map(units::parse_scalar_list, Declaration::GridColumns)(input)?,
        "grid-rows" => map(units::parse_scalar_list, Declaration::GridRows)(input)?,
        "grid-gutter" => {
            let (input, (v, h)) = units::parse_grid_gutter(input)?;
            (input, Declaration::GridGutter(v, h))
        }
        "column-span" => map(units::parse_u16, Declaration::ColumnSpan)(input)?,
        "row-span" => map(units::parse_u16, Declaration::RowSpan)(input)?,

        // Link properties
        "link-color" => map(values::parse_color, Declaration::LinkColor)(input)?,
        "link-color-hover" => map(values::parse_color, Declaration::LinkColorHover)(input)?,
        "link-background" => map(values::parse_color, Declaration::LinkBackground)(input)?,
        "link-background-hover" => map(values::parse_color, Declaration::LinkBackgroundHover)(input)?,
        "link-style" => map(values::parse_text_style, Declaration::LinkStyle)(input)?,
        "link-style-hover" => map(values::parse_text_style, Declaration::LinkStyleHover)(input)?,

        // Content alignment properties (text within widget)
        "content-align-horizontal" => map(values::parse_align_horizontal, Declaration::ContentAlignHorizontal)(input)?,
        "content-align-vertical" => map(values::parse_align_vertical, Declaration::ContentAlignVertical)(input)?,
        "content-align" => {
            let (input, (h, v)) = values::parse_content_align(input)?;
            (input, Declaration::ContentAlign(h, v))
        }

        // Container alignment properties (child positioning)
        "align-horizontal" => map(values::parse_align_horizontal, Declaration::AlignHorizontal)(input)?,
        "align-vertical" => map(values::parse_align_vertical, Declaration::AlignVertical)(input)?,
        "align" => {
            let (input, (h, v)) = values::parse_content_align(input)?;
            (input, Declaration::Align(h, v))
        }

        // Border title/subtitle properties
        "border-title-align" => map(values::parse_align_horizontal, Declaration::BorderTitleAlign)(input)?,
        "border-subtitle-align" => map(values::parse_align_horizontal, Declaration::BorderSubtitleAlign)(input)?,
        "border-title-color" => map(values::parse_color, Declaration::BorderTitleColor)(input)?,
        "border-subtitle-color" => map(values::parse_color, Declaration::BorderSubtitleColor)(input)?,
        "border-title-background" => map(values::parse_color, Declaration::BorderTitleBackground)(input)?,
        "border-subtitle-background" => map(values::parse_color, Declaration::BorderSubtitleBackground)(input)?,
        "border-title-style" => map(values::parse_text_style, Declaration::BorderTitleStyle)(input)?,
        "border-subtitle-style" => map(values::parse_text_style, Declaration::BorderSubtitleStyle)(input)?,

        // Edge-specific border properties
        "border-top" => map(values::parse_border_edge, Declaration::BorderTop)(input)?,
        "border-bottom" => map(values::parse_border_edge, Declaration::BorderBottom)(input)?,
        "border-left" => map(values::parse_border_edge, Declaration::BorderLeft)(input)?,
        "border-right" => map(values::parse_border_edge, Declaration::BorderRight)(input)?,

        // Outline properties (non-layout-affecting border overlay)
        "outline" => map(values::parse_border_edge, Declaration::Outline)(input)?,
        "outline-top" => map(values::parse_border_edge, Declaration::OutlineTop)(input)?,
        "outline-right" => map(values::parse_border_edge, Declaration::OutlineRight)(input)?,
        "outline-bottom" => map(values::parse_border_edge, Declaration::OutlineBottom)(input)?,
        "outline-left" => map(values::parse_border_edge, Declaration::OutlineLeft)(input)?,

        // Hatch pattern fill
        "hatch" => map(values::parse_hatch, Declaration::Hatch)(input)?,

        // Keyline (box-drawing borders around widgets)
        "keyline" => map(values::parse_keyline, Declaration::Keyline)(input)?,

        // Offset properties (visual position adjustment after layout)
        "offset" => {
            let (input, (x, y)) = units::parse_offset(input)?;
            (input, Declaration::Offset(x, y))
        }
        "offset-x" => map(units::parse_scalar, Declaration::OffsetX)(input)?,
        "offset-y" => map(units::parse_scalar, Declaration::OffsetY)(input)?,

        _ => {
            // Robustly consume until semicolon or brace for unknown properties
            let (input, _value) = take_until_semicolon_or_brace(input)?;
            (input, Declaration::Unknown(property.to_string()))
        }
    };

    // Consume !important if present (we'll just skip it for now to pass parsing)
    let (input, _) = opt(preceded(multispace0, tag("!important")))(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = opt(char(';'))(input)?;
    Ok((input, declaration))
}

fn take_until_balanced_braces(input: &str) -> IResult<&str, &str> {
    // Basic balanced brace matcher to skip nested blocks
    let mut depth = 0;
    let mut end = 0;
    for (i, c) in input.char_indices() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Ok((&input[i + 1..], &input[..i + 1]));
            }
        }
        end = i + c.len_utf8();
    }
    Ok((&input[end..], input))
}

fn take_until_semicolon_or_brace(input: &str) -> IResult<&str, &str> {
    nom::bytes::complete::take_until(";")(input).or_else(|_: nom::Err<nom::error::Error<&str>>| {
        nom::bytes::complete::take_until("}")(input)
    })
}
