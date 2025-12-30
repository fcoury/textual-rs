//! Code generation for the ui! macro.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::{NamedAttr, UiRoot, WidgetNode};

/// Generate the final token stream from the parsed UI tree.
pub fn generate(root: UiRoot) -> TokenStream {
    let widget_exprs: Vec<TokenStream> = root.widgets.iter().map(generate_widget).collect();

    // Always return Vec<Box<dyn Widget<_>>>
    quote! {
        vec![#(#widget_exprs),*]
    }
}

/// Generate code for a single widget node.
fn generate_widget(node: &WidgetNode) -> TokenStream {
    let name = &node.name;
    let positional = &node.positional_args;
    let children = &node.children;

    // Generate builder method chains for named attributes
    let attr_calls = generate_attr_calls(&node.named_attrs);

    if children.is_empty() {
        // Leaf widget: Widget::new(args).with_attr1(v1).with_attr2(v2)
        quote! {
            Box::new(#name::new(#(#positional),*) #attr_calls) as Box<dyn Widget<_>>
        }
    } else {
        // Container widget: collect children first
        let child_exprs: Vec<TokenStream> = children.iter().map(generate_widget).collect();

        if positional.is_empty() && node.named_attrs.is_empty() {
            // Container with children only: Widget::new(children)
            quote! {
                Box::new(#name::new(vec![#(#child_exprs),*])) as Box<dyn Widget<_>>
            }
        } else if positional.is_empty() {
            // Container with attrs but no positional args
            quote! {
                Box::new(#name::new(vec![#(#child_exprs),*]) #attr_calls) as Box<dyn Widget<_>>
            }
        } else {
            // Container with positional args and children
            // Convention: children is the last argument to ::new()
            quote! {
                Box::new(#name::new(#(#positional,)* vec![#(#child_exprs),*]) #attr_calls) as Box<dyn Widget<_>>
            }
        }
    }
}

/// Generate `.with_attr(value)` chains from named attributes.
fn generate_attr_calls(attrs: &[NamedAttr]) -> TokenStream {
    let calls: Vec<TokenStream> = attrs
        .iter()
        .map(|attr| {
            let method_name = format_ident!("with_{}", attr.name);
            let value = &attr.value;
            quote! { .#method_name(#value) }
        })
        .collect();

    quote! { #(#calls)* }
}
