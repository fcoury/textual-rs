//! Test utilities for snapshot testing textual applications.
//!
//! This module provides headless rendering without the event loop,
//! useful for snapshot testing UI layouts.

use std::collections::HashSet;

use crate::{
    canvas::Canvas,
    style_resolver::resolve_styles,
    tree::WidgetTree,
    widget::{screen::Screen, Compose, Widget},
    Size,
};

/// Collect default CSS from a widget and all its descendants.
fn collect_default_css<M>(widget: &mut dyn Widget<M>, collected: &mut HashSet<&'static str>) {
    let default_css = widget.default_css();
    if !default_css.is_empty() {
        collected.insert(default_css);
    }
    widget.for_each_child(&mut |child| {
        collect_default_css(child, collected);
    });
}

/// Build a combined stylesheet from widget defaults and app CSS.
fn build_combined_css<M>(root: &mut dyn Widget<M>, app_css: &str) -> String {
    let mut defaults: HashSet<&'static str> = HashSet::new();
    collect_default_css(root, &mut defaults);

    let mut combined = String::new();
    for css in defaults {
        combined.push_str(css);
        combined.push('\n');
    }
    combined.push_str(app_css);
    combined
}

/// Render a Compose implementation to a Canvas without running the event loop.
///
/// This function:
/// 1. Builds a widget tree wrapped in Screen
/// 2. Collects widget default CSS and combines with provided CSS
/// 3. Resolves styles using the textual-dark theme
/// 4. Renders to a Canvas
///
/// # Example
/// ```ignore
/// let app = MyApp::new();
/// let canvas = render_to_canvas(&app, "MyApp::CSS", 80, 24);
/// assert_snapshot!(canvas.to_snapshot());
/// ```
pub fn render_to_canvas<T, M>(app: &T, css: &str, width: u16, height: u16) -> Canvas
where
    T: Compose<Message = M>,
    M: Send + 'static,
{
    let themes = tcss::types::Theme::standard_themes();
    let theme = themes
        .get("textual-dark")
        .cloned()
        .unwrap_or_else(|| tcss::types::Theme::new("default", true));

    // Build widget tree (wrapped in implicit Screen)
    let root = Box::new(Screen::new(app.compose()));
    let mut tree = WidgetTree::new(root);

    // Initialize Screen with size for breakpoints
    tree.root_mut().on_resize(Size::new(width, height));

    // Collect widget default CSS and combine with app CSS
    // Widget defaults are prepended (lower specificity), app CSS overrides
    let combined_css = build_combined_css(tree.root_mut(), css);
    let stylesheet = tcss::parser::parse_stylesheet(&combined_css).expect("CSS parsing failed");

    // Resolve styles
    let mut ancestors = Vec::new();
    resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);

    // Render to canvas
    let mut canvas = Canvas::new(width, height);
    let region = crate::canvas::Region::from_u16(0, 0, width, height);
    tree.root().render(&mut canvas, region);

    canvas
}
