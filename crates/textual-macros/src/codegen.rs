//! Code generation for the ui! macro.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::{ChildItem, NamedAttr, UiRoot, WidgetNode};

/// Generate the final token stream from the parsed UI tree.
pub fn generate(root: UiRoot) -> TokenStream {
    generate_children_vec(&root.items)
}

/// Generate code for a single widget (used by widget! macro).
pub fn generate_single(node: WidgetNode) -> TokenStream {
    generate_widget(&node)
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
        // Container widget: generate children vec
        let children_code = generate_children_vec(children);

        if positional.is_empty() && node.named_attrs.is_empty() {
            // Container with children only: Widget::new(children)
            quote! {
                Box::new(#name::new(#children_code)) as Box<dyn Widget<_>>
            }
        } else if positional.is_empty() {
            // Container with attrs but no positional args
            quote! {
                Box::new(#name::new(#children_code) #attr_calls) as Box<dyn Widget<_>>
            }
        } else {
            // Container with positional args and children
            // Convention: children is the last argument to ::new()
            quote! {
                Box::new(#name::new(#(#positional,)* #children_code) #attr_calls) as Box<dyn Widget<_>>
            }
        }
    }
}

/// Generate code that builds a Vec from ChildItems (widgets + splats).
fn generate_children_vec(children: &[ChildItem]) -> TokenStream {
    // Check if we have any splats
    let has_splats = children.iter().any(|c| matches!(c, ChildItem::Splat(_)));

    if !has_splats {
        // No splats - use simple vec! literal (more efficient)
        let widget_exprs: Vec<TokenStream> = children
            .iter()
            .map(|c| match c {
                ChildItem::Widget(node) => generate_widget(node),
                ChildItem::Splat(_) => unreachable!(),
            })
            .collect();
        quote! { vec![#(#widget_exprs),*] }
    } else {
        // Has splats - use extend pattern
        let mut statements = Vec::new();

        for child in children {
            match child {
                ChildItem::Widget(node) => {
                    let widget_expr = generate_widget(node);
                    statements.push(quote! { __children.push(#widget_expr); });
                }
                ChildItem::Splat(expr) => {
                    statements.push(quote! { __children.extend(#expr); });
                }
            }
        }

        quote! {
            {
                let mut __children: Vec<Box<dyn Widget<_>>> = Vec::new();
                #(#statements)*
                __children
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
