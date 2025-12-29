//! Test utilities for snapshot testing textual applications.
//!
//! This module provides headless rendering without the event loop,
//! useful for snapshot testing UI layouts.

use crate::{
    canvas::Canvas,
    style_resolver::resolve_styles,
    tree::WidgetTree,
    widget::{screen::Screen, Compose},
    Size,
};

/// Render a Compose implementation to a Canvas without running the event loop.
///
/// This function:
/// 1. Parses the CSS from `<T as App>::CSS` (if implementing App)
/// 2. Builds a widget tree wrapped in Screen
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
    // Parse CSS
    let stylesheet = tcss::parser::parse_stylesheet(css).expect("CSS parsing failed");
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

    // Resolve styles
    let mut ancestors = Vec::new();
    resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);

    // Render to canvas
    let mut canvas = Canvas::new(width, height);
    let region = crate::canvas::Region::from_u16(0, 0, width, height);
    tree.root().render(&mut canvas, region);

    canvas
}
