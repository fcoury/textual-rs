//! DSL parsing for the ui! macro.

use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    token, Expr, Ident, Result, Token,
};

/// Root of the UI tree - may contain multiple widgets.
#[derive(Debug)]
pub struct UiRoot {
    pub widgets: Vec<WidgetNode>,
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
    /// Child widgets (for containers)
    pub children: Vec<WidgetNode>,
}

/// A named attribute like `id: "my-id"`
#[derive(Debug)]
pub struct NamedAttr {
    pub name: Ident,
    pub value: Expr,
}

impl Parse for UiRoot {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut widgets = Vec::new();

        // Parse widgets until input is exhausted
        while !input.is_empty() {
            widgets.push(input.parse::<WidgetNode>()?);
        }

        if widgets.is_empty() {
            return Err(input.error("ui! macro requires at least one widget"));
        }

        Ok(UiRoot { widgets })
    }
}

impl Parse for WidgetNode {
    fn parse(input: ParseStream) -> Result<Self> {
        // 1. Parse widget name (identifier like "Vertical", "Static")
        let name: Ident = input.parse()?;

        let mut positional_args = Vec::new();
        let mut named_attrs = Vec::new();
        let mut children = Vec::new();

        // 2. Check for arguments in parentheses: Widget(args)
        if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            parse_args(&content, &mut positional_args, &mut named_attrs)?;
        }

        // 3. Check for children in braces: Widget { children }
        if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            while !content.is_empty() {
                children.push(content.parse::<WidgetNode>()?);
            }
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
