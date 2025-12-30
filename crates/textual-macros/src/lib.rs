//! Procedural macros for the textual UI framework.
//!
//! This crate provides the `ui!` proc macro for declarative widget composition.

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod codegen;
mod parse;

/// Declarative UI macro for building widget trees.
///
/// # Syntax
///
/// ```ignore
/// // Container with children only
/// Vertical { child1 child2 }
///
/// // Widget with positional arg(s)
/// Static("Hello world")
///
/// // Widget with positional args and named attributes
/// Static("Hello", id: "greeting", classes: "bold")
///
/// // Container with attributes and children
/// Grid(id: "my-grid") { child1 child2 }
///
/// // Widget with callback
/// Switch(false, |v| Msg::Toggle(v), id: "toggle")
///
/// // Multiple root widgets
/// Label("First")
/// Label("Second")
/// ```
///
/// # Attribute Mapping
///
/// Named attributes are converted to builder method calls:
/// - `id: "foo"` becomes `.with_id("foo")`
/// - `classes: "a b"` becomes `.with_classes("a b")`
/// - `disabled: true` becomes `.with_disabled(true)`
/// - Any `name: value` becomes `.with_name(value)`
///
/// # Return Type
///
/// Always returns `Vec<Box<dyn Widget<_>>>`.
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as parse::UiRoot);
    codegen::generate(root).into()
}
