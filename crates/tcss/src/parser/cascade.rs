//! CSS cascade and style computation.
//!
//! This module implements the CSS cascade algorithm for TCSS:
//!
//! - [`compute_style`]: Main entry point for computing a widget's final styles
//! - [`WidgetMeta`]: Widget metadata for selector matching
//! - [`WidgetStates`]: Bitflags for widget pseudo-class states
//!
//! ## Cascade Algorithm
//!
//! 1. Find all rules whose selectors match the widget
//! 2. Sort by specificity (IDs > classes > types), then source order
//! 3. Apply declarations in order (later declarations override earlier)
//! 4. Resolve theme variables to actual colors
//!
//! ## Selector Matching
//!
//! The cascade matches selectors against the widget and its ancestors:
//!
//! - Type selectors match `widget.type_name`
//! - Class selectors match any class in `widget.classes`
//! - ID selectors match `widget.id`
//! - Pseudo-class selectors match `widget.states` (`:focus`, `:hover`, etc.)
//! - Combinators traverse the ancestor chain

use bitflags::bitflags;

use crate::{
    parser::{
        Combinator, ComplexSelector, Declaration, Rule, RuleItem, Selector, Specificity, StyleSheet,
    },
    types::{Border, ComputedStyle, RgbaColor, Theme},
};

bitflags! {
    /// Bitflags representing widget pseudo-class states.
    ///
    /// These states are used for matching CSS pseudo-class selectors like
    /// `:focus`, `:hover`, `:active`, and `:disabled`.
    ///
    /// # Example
    ///
    /// ```
    /// use tcss::parser::cascade::WidgetStates;
    ///
    /// let mut states = WidgetStates::empty();
    /// states |= WidgetStates::FOCUS;
    /// states |= WidgetStates::HOVER;
    ///
    /// assert!(states.contains(WidgetStates::FOCUS));
    /// assert!(states.contains(WidgetStates::HOVER));
    /// assert!(!states.contains(WidgetStates::ACTIVE));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct WidgetStates: u16 {
        /// Widget has keyboard focus
        const FOCUS    = 0b0000_0001;
        /// Mouse is hovering over widget
        const HOVER    = 0b0000_0010;
        /// Widget is being actively pressed/clicked
        const ACTIVE   = 0b0000_0100;
        /// Widget is disabled and not interactive
        const DISABLED = 0b0000_1000;
    }
}

/// Metadata about a widget used for selector matching.
///
/// This struct provides the information needed to determine if a CSS
/// selector matches a widget in the UI tree.
#[derive(Clone, Debug, Default)]
pub struct WidgetMeta {
    /// The widget's type name (e.g., "Button", "Label", "Container").
    /// Static str to avoid allocation on every style resolution call.
    pub type_name: &'static str,
    /// The widget's unique ID, if set (e.g., "submit", "header").
    pub id: Option<String>,
    /// The widget's CSS classes (e.g., ["primary", "active"]).
    pub classes: Vec<String>,
    /// The widget's current pseudo-class states (focus, hover, active, disabled).
    pub states: WidgetStates,
}

/// A rule that matched a widget, bundled with its priority information.
#[derive(Debug)]
struct MatchedRule<'a> {
    specificity: Specificity,
    source_order: usize,
    rule: &'a Rule,
}

impl WidgetMeta {
    /// Checks if this widget matches a simple selector.
    pub fn matches_selector(&self, selector: &Selector) -> bool {
        match selector {
            Selector::Type(name) => self.type_name == *name,
            Selector::Id(id) => self.id.as_ref() == Some(id),
            Selector::Class(class) => self.classes.contains(class),
            Selector::Universal => true,
            Selector::PseudoClass(name) => match name.as_str() {
                "focus" => self.states.contains(WidgetStates::FOCUS),
                "hover" => self.states.contains(WidgetStates::HOVER),
                "active" => self.states.contains(WidgetStates::ACTIVE),
                "disabled" => self.states.contains(WidgetStates::DISABLED),
                _ => false,
            },
            Selector::Parent => false, // Will be handled by the nesting flattener
            Selector::Attribute(_name, _value) => {
                // Placeholder: In a real TUI engine, we'd check the widget's
                // internal attribute map (e.g., widget.get_attr("type") == "text")
                false
            }
        }
    }

    /// Checks if this widget matches a complex selector given its ancestors.
    /// Ancestors should be ordered from immediate parent to root.
    pub fn matches_complex(&self, complex: &ComplexSelector, ancestors: &[WidgetMeta]) -> bool {
        if complex.parts.is_empty() {
            return false;
        }

        // The rightmost part must match the widget itself.
        let parts = &complex.parts;
        let mut current_part_idx = parts.len() - 1;

        if !parts[current_part_idx]
            .compound
            .selectors
            .iter()
            .all(|s| self.matches_selector(s))
        {
            return false;
        }

        // If that was the only part, we matched.
        if parts.len() == 1 {
            return true;
        }

        // Match remaining parts against ancestors.
        let mut ancestor_idx = 0;
        current_part_idx -= 1;

        while current_part_idx < parts.len() && ancestor_idx < ancestors.len() {
            let part = &parts[current_part_idx];
            let ancestor = &ancestors[ancestor_idx];

            let matches = part
                .compound
                .selectors
                .iter()
                .all(|s| ancestor.matches_selector(s));

            match part.combinator {
                Combinator::Child => {
                    if matches {
                        if current_part_idx == 0 {
                            return true;
                        }
                        current_part_idx -= 1;
                    } else {
                        return false;
                    }
                    ancestor_idx += 1;
                }
                Combinator::Descendant | Combinator::None => {
                    if matches {
                        if current_part_idx == 0 {
                            return true;
                        }
                        current_part_idx -= 1;
                    }
                    ancestor_idx += 1;
                }
                // FIX: Add explicit return to satisfy type checker
                Combinator::AdjacentSibling | Combinator::GeneralSibling => {
                    return false;
                }
            }
        }

        current_part_idx == usize::MAX
            || current_part_idx == 0
                && ancestor_idx < ancestors.len()
                && parts[0]
                    .compound
                    .selectors
                    .iter()
                    .all(|s| ancestors[ancestor_idx].matches_selector(s))
    }
}

/// The core cascade function.
pub fn compute_style(
    widget: &WidgetMeta,
    ancestors: &[WidgetMeta],
    stylesheet: &StyleSheet,
    theme: &Theme,
) -> ComputedStyle {
    let mut matched_rules = Vec::new();

    // 1. Find all matching rules
    for (idx, rule) in stylesheet.rules.iter().enumerate() {
        for complex in &rule.selectors.selectors {
            if widget.matches_complex(complex, ancestors) {
                matched_rules.push(MatchedRule {
                    specificity: complex.specificity(),
                    source_order: idx,
                    rule,
                });
                // Once one selector in a rule matches, the whole rule applies.
                break;
            }
        }
    }

    // 2. Sort by Specificity, then Source Order
    matched_rules.sort_by(|a, b| {
        a.specificity
            .cmp(&b.specificity)
            .then(a.source_order.cmp(&b.source_order))
    });

    // 3. Apply declarations in order (Highest priority wins)
    let mut computed = ComputedStyle::default();
    for matched in matched_rules {
        for item in &matched.rule.items {
            if let RuleItem::Declaration(decl) = item {
                apply_declaration(&mut computed, decl, theme);
            }
        }
    }

    computed
}

fn apply_declaration(style: &mut ComputedStyle, decl: &Declaration, theme: &Theme) {
    match decl {
        Declaration::Color(c) => {
            let resolved = resolve_theme_color(c, theme);
            style.auto_color = c.auto;
            style.color = Some(resolved);
        }
        Declaration::Background(c) => {
            style.background = Some(resolve_theme_color(c, theme));
        }
        Declaration::Tint(c) => {
            style.tint = Some(resolve_theme_color(c, theme));
        }
        Declaration::BackgroundTint(c) => {
            style.background_tint = Some(resolve_theme_color(c, theme));
        }
        Declaration::Width(s) => style.width = Some(*s),
        Declaration::Height(s) => style.height = Some(*s),
        Declaration::MaxHeight(s) => style.max_height = Some(*s),
        Declaration::MaxWidth(s) => style.max_width = Some(*s),
        Declaration::MinHeight(s) => style.min_height = Some(*s),
        Declaration::Margin(s) => style.margin = *s,
        Declaration::MarginTop(s) => style.margin.top = *s,
        Declaration::MarginRight(s) => style.margin.right = *s,
        Declaration::MarginBottom(s) => style.margin.bottom = *s,
        Declaration::MarginLeft(s) => style.margin.left = *s,
        Declaration::Padding(s) => style.padding = *s,
        Declaration::Border(b) => {
            let mut resolved_edge = b.clone();
            if let Some(ref color) = b.color {
                resolved_edge.color = Some(resolve_theme_color(color, theme));
            }
            style.border = Border::all(resolved_edge);
        }

        // Border title/subtitle properties
        Declaration::BorderTitleAlign(a) => {
            style.border_title_align = *a;
        }
        Declaration::BorderSubtitleAlign(a) => {
            style.border_subtitle_align = *a;
        }
        Declaration::BorderTitleColor(c) => {
            style.border_title_color = Some(resolve_theme_color(c, theme));
        }
        Declaration::BorderSubtitleColor(c) => {
            style.border_subtitle_color = Some(resolve_theme_color(c, theme));
        }
        Declaration::BorderTitleBackground(c) => {
            style.border_title_background = Some(resolve_theme_color(c, theme));
        }
        Declaration::BorderSubtitleBackground(c) => {
            style.border_subtitle_background = Some(resolve_theme_color(c, theme));
        }
        Declaration::BorderTitleStyle(s) => {
            style.border_title_style.merge(s);
        }
        Declaration::BorderSubtitleStyle(s) => {
            style.border_subtitle_style.merge(s);
        }

        // Edge-specific border properties
        Declaration::BorderTop(b) => {
            let mut resolved_edge = b.clone();
            if let Some(ref color) = b.color {
                resolved_edge.color = Some(resolve_theme_color(color, theme));
            }
            style.border.top = resolved_edge;
        }
        Declaration::BorderBottom(b) => {
            let mut resolved_edge = b.clone();
            if let Some(ref color) = b.color {
                resolved_edge.color = Some(resolve_theme_color(color, theme));
            }
            style.border.bottom = resolved_edge;
        }
        Declaration::BorderLeft(b) => {
            let mut resolved_edge = b.clone();
            if let Some(ref color) = b.color {
                resolved_edge.color = Some(resolve_theme_color(color, theme));
            }
            style.border.left = resolved_edge;
        }
        Declaration::BorderRight(b) => {
            let mut resolved_edge = b.clone();
            if let Some(ref color) = b.color {
                resolved_edge.color = Some(resolve_theme_color(color, theme));
            }
            style.border.right = resolved_edge;
        }

        // Scrollbar properties
        Declaration::ScrollbarColor(c) => {
            style.scrollbar.color = Some(resolve_theme_color(c, theme));
        }
        Declaration::ScrollbarColorHover(c) => {
            style.scrollbar.color_hover = Some(resolve_theme_color(c, theme));
        }
        Declaration::ScrollbarColorActive(c) => {
            style.scrollbar.color_active = Some(resolve_theme_color(c, theme));
        }
        Declaration::ScrollbarBackground(c) => {
            style.scrollbar.background = Some(resolve_theme_color(c, theme));
        }
        Declaration::ScrollbarBackgroundHover(c) => {
            style.scrollbar.background_hover = Some(resolve_theme_color(c, theme));
        }
        Declaration::ScrollbarBackgroundActive(c) => {
            style.scrollbar.background_active = Some(resolve_theme_color(c, theme));
        }
        Declaration::ScrollbarCornerColor(c) => {
            style.scrollbar.corner_color = Some(resolve_theme_color(c, theme));
        }
        Declaration::ScrollbarSize(s) => {
            style.scrollbar.size = *s;
        }
        Declaration::ScrollbarSizeHorizontal(v) => {
            style.scrollbar.size.horizontal = *v;
        }
        Declaration::ScrollbarSizeVertical(v) => {
            style.scrollbar.size.vertical = *v;
        }
        Declaration::ScrollbarGutter(g) => {
            style.scrollbar.gutter = *g;
        }
        Declaration::ScrollbarVisibility(v) => {
            style.scrollbar.visibility = *v;
        }

        // Box model properties
        Declaration::BoxSizing(b) => {
            style.box_sizing = *b;
        }

        // Display properties
        Declaration::Display(d) => {
            style.display = *d;
        }
        Declaration::Visibility(v) => {
            style.visibility = *v;
        }

        // Overflow properties
        Declaration::OverflowX(o) => {
            style.overflow_x = *o;
        }
        Declaration::OverflowY(o) => {
            style.overflow_y = *o;
        }

        // Layout and Grid properties
        Declaration::Layout(l) => {
            style.layout = *l;
        }
        Declaration::Dock(d) => {
            style.dock = Some(*d);
        }
        Declaration::Layers(names) => {
            style.layers = Some(names.clone());
        }
        Declaration::Layer(name) => {
            style.layer = Some(name.clone());
        }
        Declaration::GridSize(cols, rows) => {
            style.grid.columns = Some(*cols);
            style.grid.rows = *rows;
        }
        Declaration::GridColumns(widths) => {
            style.grid.column_widths = widths.clone();
        }
        Declaration::GridRows(heights) => {
            style.grid.row_heights = heights.clone();
        }
        Declaration::GridGutter(v, h) => {
            style.grid.gutter = (*v, h.unwrap_or(*v));
        }
        Declaration::ColumnSpan(n) => {
            style.grid_placement.column_span = *n;
        }
        Declaration::RowSpan(n) => {
            style.grid_placement.row_span = *n;
        }

        // Link properties
        Declaration::LinkColor(c) => {
            style.link.color = Some(resolve_theme_color(c, theme));
        }
        Declaration::LinkColorHover(c) => {
            style.link.color_hover = Some(resolve_theme_color(c, theme));
        }
        Declaration::LinkBackground(c) => {
            style.link.background = Some(resolve_theme_color(c, theme));
        }
        Declaration::LinkBackgroundHover(c) => {
            style.link.background_hover = Some(resolve_theme_color(c, theme));
        }
        Declaration::LinkStyle(s) => {
            style.link.style = resolve_theme_style(s, theme);
        }
        Declaration::LinkStyleHover(s) => {
            style.link.style_hover = resolve_theme_style(s, theme);
        }

        // Content alignment properties (text within widget)
        Declaration::ContentAlignHorizontal(a) => {
            style.content_align_horizontal = *a;
        }
        Declaration::ContentAlignVertical(a) => {
            style.content_align_vertical = *a;
        }
        Declaration::ContentAlign(h, v) => {
            style.content_align_horizontal = *h;
            style.content_align_vertical = *v;
        }

        // Container alignment properties (child positioning)
        Declaration::AlignHorizontal(a) => {
            style.align_horizontal = *a;
        }
        Declaration::AlignVertical(a) => {
            style.align_vertical = *a;
        }
        Declaration::Align(h, v) => {
            style.align_horizontal = *h;
            style.align_vertical = *v;
        }

        // Hatch pattern fill
        Declaration::Hatch(h) => {
            let mut resolved_hatch = h.clone();
            resolved_hatch.color = resolve_theme_color(&h.color, theme);
            style.hatch = Some(resolved_hatch);
        }

        // Keyline (box-drawing borders around widgets)
        Declaration::Keyline(k) => {
            let mut resolved_keyline = k.clone();
            resolved_keyline.color = resolve_theme_color(&k.color, theme);
            style.keyline = resolved_keyline;
        }

        Declaration::Unknown(_) => {}
    }
}

fn resolve_theme_color(color: &RgbaColor, theme: &Theme) -> RgbaColor {
    if let Some(var_name) = &color.theme_var {
        // First, try the full variable name (handles multi-word names like "link-background-hover")
        let mut resolved = if let Some(c) = theme.get_color(var_name) {
            c
        } else {
            // Fallback: check for modifiers like "-lighten-1" at the end
            // Pattern: base-modifier-amount (e.g., "primary-lighten-1")
            let parts: Vec<&str> = var_name.rsplitn(3, '-').collect();
            // After rsplitn(3, "-") on "primary-lighten-1": ["1", "lighten", "primary"]
            // After rsplitn(3, "-") on "link-background-hover": ["hover", "background", "link-..."]

            if parts.len() >= 3 {
                // Check if this looks like a modifier pattern
                if let Ok(amount) = parts[0].parse::<f32>() {
                    let mode = parts[1];
                    if mode == "lighten" || mode == "darken" {
                        // Reconstruct base name from remaining parts
                        let base_name = parts[2];
                        let base_color =
                            theme.get_color(base_name).unwrap_or_else(RgbaColor::white);
                        match mode {
                            "lighten" => base_color.lighten(amount),
                            "darken" => base_color.darken(amount),
                            _ => base_color,
                        }
                    } else {
                        RgbaColor::white()
                    }
                } else {
                    RgbaColor::white()
                }
            } else {
                RgbaColor::white()
            }
        };

        // Preserve alpha from the original color if it was explicitly set
        // (e.g., "$foreground 50%" should use 0.5 alpha)
        if color.a < 1.0 {
            resolved.a = color.a;
        }

        resolved
    } else {
        color.clone()
    }
}

fn resolve_theme_style(style: &crate::types::TextStyle, theme: &Theme) -> crate::types::TextStyle {
    if let Some(var_name) = &style.theme_var {
        theme.get_style(var_name).unwrap_or_default()
    } else {
        style.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_stylesheet;

    #[test]
    fn test_id_selector_overrides_type_selector() {
        let css = r#"
Container {
    width: 1fr;
    height: 1fr;
}

#vertical-layout {
    height: auto;
}
"#;

        let stylesheet = parse_stylesheet(css).expect("Failed to parse CSS");
        let theme = Theme::new("test", false);

        let widget = WidgetMeta {
            type_name: "Container",
            id: Some("vertical-layout".to_string()),
            classes: vec![],
            states: WidgetStates::empty(),
        };

        let ancestors = vec![];
        let style = compute_style(&widget, &ancestors, &stylesheet, &theme);

        // ID selector should override type selector
        // height: auto is represented as Scalar { value: 0.0, unit: Unit::Auto }
        assert!(style.height.is_some(), "height should be set");
        let height = style.height.as_ref().unwrap();
        assert_eq!(
            height.unit,
            crate::types::geometry::Unit::Auto,
            "height should be 'auto', got {:?}",
            height
        );
    }
}
