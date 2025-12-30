use crate::widget::Widget;
use tcss::parser::StyleSheet;
use tcss::parser::cascade::{WidgetMeta, compute_style};
use tcss::types::{ComputedStyle, RgbaColor, Theme};

/// Context inherited from parent for CSS property inheritance.
#[derive(Clone, Default)]
pub struct InheritedContext {
    /// Parent's color (inherited if child doesn't specify)
    pub color: Option<RgbaColor>,
    /// Whether parent's color is auto (contrast-based)
    pub auto_color: bool,
    /// The effective background at this level (for auto color resolution)
    /// This is the composed background including background-tint.
    pub effective_background: Option<RgbaColor>,
}

/// Resolves styles for all widgets in the tree.
/// Used for initial style resolution at startup.
pub fn resolve_styles<M>(
    widget: &mut dyn Widget<M>,
    stylesheet: &StyleSheet,
    theme: &Theme,
    ancestors: &mut Vec<WidgetMeta>,
) {
    // Delegate to dirty resolver, forcing all widgets to be restyled
    resolve_dirty_styles(
        widget,
        stylesheet,
        theme,
        ancestors,
        true,
        &InheritedContext::default(),
    );
}

/// Resolves styles only for dirty widgets and their descendants.
///
/// A widget is restyled if:
/// - It is marked dirty (state changed like focus/hover/active)
/// - A parent was dirty (selectors like `Parent:focus > Child` may apply)
///
/// Invisible widgets are skipped entirely (optimization).
///
/// After restyling, widgets are marked clean.
pub fn resolve_dirty_styles<M>(
    widget: &mut dyn Widget<M>,
    stylesheet: &StyleSheet,
    theme: &Theme,
    ancestors: &mut Vec<WidgetMeta>,
    parent_dirty: bool,
    inherited: &InheritedContext,
) {
    // Skip invisible widgets and their subtrees
    if !widget.is_visible() {
        return;
    }

    let is_dirty = widget.is_dirty();
    let should_restyle = is_dirty || parent_dirty;

    let mut style = if should_restyle {
        // Compute style for the current widget
        let meta = widget.get_meta();
        let mut style = compute_style(&meta, ancestors, stylesheet, theme);

        // Apply CSS inheritance for properties that weren't explicitly set
        apply_inheritance(&mut style, inherited);

        log::trace!(
            "CASCADE: Widget='{}' States={:?} -> Color={:?} (dirty={})",
            meta.type_name,
            meta.states,
            style.color,
            is_dirty
        );

        widget.set_style(style.clone());
        widget.mark_clean();
        style
    } else {
        widget.get_style()
    };

    // Build inherited context for children
    let child_inherited = build_inherited_context(&mut style, inherited);

    // Prepare for children: push current widget onto ancestor stack
    let meta = widget.get_meta();
    ancestors.push(meta);

    // Recurse into children, propagating dirty state
    widget.for_each_child(&mut |child| {
        resolve_dirty_styles(
            child,
            stylesheet,
            theme,
            ancestors,
            should_restyle,
            &child_inherited,
        );
    });

    // Clean up stack after visiting subtree
    ancestors.pop();
}

/// Apply CSS property inheritance from parent context.
fn apply_inheritance(style: &mut ComputedStyle, inherited: &InheritedContext) {
    // Inherit color if not explicitly set
    if style.color.is_none() && inherited.color.is_some() {
        style.color = inherited.color.clone();
        style.auto_color = inherited.auto_color;
    }

    // Store inherited effective background for auto color resolution
    // (used when this widget is transparent and needs parent's background for contrast)
    style.inherited_background = inherited.effective_background.clone();
}

/// Build the inherited context to pass to children.
fn build_inherited_context(
    style: &mut ComputedStyle,
    parent_inherited: &InheritedContext,
) -> InheritedContext {
    // Compute effective background for this widget
    let effective_bg = compute_effective_background(style, parent_inherited);

    InheritedContext {
        color: style.color.clone(),
        auto_color: style.auto_color,
        effective_background: effective_bg,
    }
}

/// Compute the effective background at this widget level.
/// This composites semi-transparent backgrounds over the parent's effective background,
/// then applies any tint.
fn compute_effective_background(
    style: &ComputedStyle,
    parent_inherited: &InheritedContext,
) -> Option<RgbaColor> {
    match (&style.background, &parent_inherited.effective_background) {
        (Some(bg), Some(parent_bg)) if bg.a < 1.0 => {
            // Composite semi-transparent background over parent
            let composited = bg.blend_over(parent_bg);
            // Apply tint if present
            match &style.background_tint {
                Some(tint) => Some(composited.tint(tint)),
                None => Some(composited),
            }
        }
        (Some(bg), _) => {
            // Opaque background - apply tint if present
            match &style.background_tint {
                Some(tint) => Some(bg.tint(tint)),
                None => Some(bg.clone()),
            }
        }
        (None, _) => parent_inherited.effective_background.clone(),
    }
}
