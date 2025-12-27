//! CSS cascade and style computation.
//!
//! This module implements the CSS cascade algorithm for TCSS:
//!
//! - [`compute_style`]: Main entry point for computing a widget's final styles
//! - [`WidgetMeta`]: Widget metadata for selector matching
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
//! - Combinators traverse the ancestor chain

use crate::{
    parser::{
        Combinator, ComplexSelector, Declaration, Rule, RuleItem, Selector, Specificity, StyleSheet,
    },
    types::{Border, ComputedStyle, RgbaColor, Theme},
};

/// Metadata about a widget used for selector matching.
///
/// This struct provides the information needed to determine if a CSS
/// selector matches a widget in the UI tree.
#[derive(Clone, Debug, Default)]
pub struct WidgetMeta {
    /// The widget's type name (e.g., "Button", "Label", "Container").
    pub type_name: String,
    /// The widget's unique ID, if set (e.g., "submit", "header").
    pub id: Option<String>,
    /// The widget's CSS classes (e.g., ["primary", "active"]).
    pub classes: Vec<String>,
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
            Selector::PseudoClass(_name) => false, // Requires runtime state (hover/focus)
            Selector::Parent => false,             // Will be handled by the nesting flattener
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
            style.color = Some(resolve_theme_color(c, theme));
        }
        Declaration::Background(c) => {
            style.background = Some(resolve_theme_color(c, theme));
        }
        Declaration::Width(s) => style.width = Some(*s),
        Declaration::Height(s) => style.height = Some(*s),
        Declaration::Margin(s) => style.margin = *s,
        Declaration::Padding(s) => style.padding = *s,
        Declaration::Border(b) => style.border = Border::all(b.clone()),
        Declaration::Unknown(_) => {}
    }
}

fn resolve_theme_color(color: &RgbaColor, theme: &Theme) -> RgbaColor {
    if let Some(var_name) = &color.theme_var {
        // Check for modifiers like "-lighten-1"
        let parts: Vec<&str> = var_name.split('-').collect();
        let base_name = parts[0];

        let mut resolved = theme.get_color(base_name).unwrap_or_else(RgbaColor::white);

        if parts.len() >= 3 {
            let mode = parts[1]; // "lighten" or "darken"
            let amount = parts[2].parse::<f32>().unwrap_or(0.0);

            resolved = match mode {
                "lighten" => resolved.lighten(amount),
                "darken" => resolved.darken(amount),
                _ => resolved,
            };
        }
        resolved
    } else {
        color.clone()
    }
}
