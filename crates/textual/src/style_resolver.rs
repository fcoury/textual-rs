use crate::widget::Widget;
use tcss::parser::StyleSheet;
use tcss::parser::cascade::{WidgetMeta, compute_style};
use tcss::types::Theme;

pub fn resolve_styles<M>(
    widget: &mut dyn Widget<M>,
    stylesheet: &StyleSheet,
    theme: &Theme,
    ancestors: &mut Vec<WidgetMeta>,
) {
    // 1. Compute style for the current widget
    let meta = widget.get_meta();
    let style = compute_style(&meta, ancestors, stylesheet, theme);

    // LOG: Check if the selector matches and what color it found
    log::debug!(
        "CASCADE: Widget='{}' ID={:?} Classes={:?} -> Color={:?}",
        meta.type_name,
        meta.id,
        meta.classes,
        style.color
    );

    widget.set_style(style);

    // 2. Prepare for children: push current widget onto ancestor stack
    ancestors.push(meta);

    // 3. Recurse into children
    widget.for_each_child(&mut |child| {
        resolve_styles(child, stylesheet, theme, ancestors);
    });

    // 4. Clean up stack after visiting subtree
    ancestors.pop();
}
