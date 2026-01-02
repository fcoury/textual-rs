//! DSL parsing for the ui! macro.

use syn::{
    Expr, Ident, Result, Token, braced, parenthesized,
    parse::{Parse, ParseStream},
    token,
};

/// Root of the UI tree - may contain multiple widgets or splat expressions.
#[derive(Debug)]
pub struct UiRoot {
    pub items: Vec<ChildItem>,
}

/// A child item - either a widget node or a splat expression (..expr).
#[derive(Debug)]
pub enum ChildItem {
    Widget(WidgetNode),
    Splat(Expr),
}

/// A single widget node in the UI tree.
#[derive(Debug)]
pub struct WidgetNode {
    /// Widget type name (e.g., "Vertical", "Static", "Switch")
    pub name: Ident,
    /// Positional arguments passed to ::new()
    pub positional_args: Vec<Expr>,
    /// Named attributes (name: value pairs) -> .with_name(value)
    pub named_attrs: Vec<NamedAttr>,
    /// Child items (for containers) - widgets or splat expressions
    /// None = no braces (leaf widget), Some(vec![]) = empty braces (container with no children)
    pub children: Option<Vec<ChildItem>>,
}

/// A named attribute like `id: "my-id"`
#[derive(Debug)]
pub struct NamedAttr {
    pub name: Ident,
    pub value: Expr,
}

/// Parse a child item - either a widget or splat expression (..expr).
fn parse_child_item(input: ParseStream) -> Result<ChildItem> {
    if input.peek(Token![..]) {
        // Splat: ..expr
        let _dotdot: Token![..] = input.parse()?;
        let expr: Expr = input.parse()?;
        Ok(ChildItem::Splat(expr))
    } else {
        // Regular widget
        Ok(ChildItem::Widget(input.parse::<WidgetNode>()?))
    }
}

impl Parse for UiRoot {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();

        // Parse widgets and splats until input is exhausted
        while !input.is_empty() {
            items.push(parse_child_item(input)?);
        }

        if items.is_empty() {
            return Err(input.error("ui! macro requires at least one widget or splat"));
        }

        Ok(UiRoot { items })
    }
}

impl Parse for WidgetNode {
    fn parse(input: ParseStream) -> Result<Self> {
        // 1. Parse widget name (identifier like "Vertical", "Static")
        let name: Ident = input.parse()?;

        let mut positional_args = Vec::new();
        let mut named_attrs = Vec::new();
        let mut children = None;

        // 2. Check for arguments in parentheses: Widget(args)
        if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            parse_args(&content, &mut positional_args, &mut named_attrs)?;
        }

        // 3. Check for children in braces: Widget { children }
        // Some(vec![]) = empty braces, None = no braces at all
        if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            let mut child_items = Vec::new();
            while !content.is_empty() {
                child_items.push(parse_child_item(&content)?);
            }
            children = Some(child_items);
        }

        Ok(WidgetNode {
            name,
            positional_args,
            named_attrs,
            children,
        })
    }
}

/// Parse arguments inside parentheses.
/// Handles both positional args and named attrs (name: value).
fn parse_args(
    input: ParseStream,
    positional: &mut Vec<Expr>,
    named: &mut Vec<NamedAttr>,
) -> Result<()> {
    let mut seen_named = false;

    while !input.is_empty() {
        // Check if this is a named attribute: ident followed by single colon (not ::)
        if input.peek(Ident) && input.peek2(Token![:]) && !input.peek2(Token![::]) {
            // Named attribute: name: value
            seen_named = true;
            let attr_name: Ident = input.parse()?;
            let _colon: Token![:] = input.parse()?;
            let value: Expr = input.parse()?;

            named.push(NamedAttr {
                name: attr_name,
                value,
            });
        } else {
            // Positional argument
            if seen_named {
                return Err(input.error("positional arguments must come before named attributes"));
            }
            positional.push(input.parse::<Expr>()?);
        }

        // Consume optional trailing comma
        if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
        }
    }

    Ok(())
}
