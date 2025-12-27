use crate::widget::Widget;
use tcss::parser::StyleSheet;
use tcss::parser::cascade::{WidgetMeta, compute_style};
use tcss::types::Theme;

/// Resolves styles for all widgets in the tree.
/// Used for initial style resolution at startup.
pub fn resolve_styles<M>(
    widget: &mut dyn Widget<M>,
    stylesheet: &StyleSheet,
    theme: &Theme,
    ancestors: &mut Vec<WidgetMeta>,
) {
    // Delegate to dirty resolver, forcing all widgets to be restyled
    resolve_dirty_styles(widget, stylesheet, theme, ancestors, true);
}

/// Resolves styles only for dirty widgets and their descendants.
///
/// A widget is restyled if:
/// - It is marked dirty (state changed like focus/hover/active)
/// - A parent was dirty (selectors like `Parent:focus > Child` may apply)
///
/// After restyling, widgets are marked clean.
pub fn resolve_dirty_styles<M>(
    widget: &mut dyn Widget<M>,
    stylesheet: &StyleSheet,
    theme: &Theme,
    ancestors: &mut Vec<WidgetMeta>,
    parent_dirty: bool,
) {
    let is_dirty = widget.is_dirty();
    let should_restyle = is_dirty || parent_dirty;

    if should_restyle {
        // Compute style for the current widget
        let meta = widget.get_meta();
        let style = compute_style(&meta, ancestors, stylesheet, theme);

        log::debug!(
            "CASCADE: Widget='{}' States={:?} -> Color={:?} (dirty={})",
            meta.type_name,
            meta.states,
            style.color,
            is_dirty
        );

        widget.set_style(style);
        widget.mark_clean();
    }

    // Prepare for children: push current widget onto ancestor stack
    let meta = widget.get_meta();
    ancestors.push(meta);

    // Recurse into children, propagating dirty state
    widget.for_each_child(&mut |child| {
        resolve_dirty_styles(child, stylesheet, theme, ancestors, should_restyle);
    });

    // Clean up stack after visiting subtree
    ancestors.pop();
}
