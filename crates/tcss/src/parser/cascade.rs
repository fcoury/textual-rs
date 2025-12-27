use crate::{
    parser::{Combinator, ComplexSelector, Declaration, Rule, Selector, Specificity, StyleSheet},
    types::{Border, ComputedStyle},
};

/// Metadata about a widget used for selector matching.
#[derive(Clone, Debug, Default)]
pub struct WidgetMeta {
    pub type_name: String,
    pub id: Option<String>,
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
        for declaration in &matched.rule.declarations {
            apply_declaration(&mut computed, declaration);
        }
    }

    computed
}

fn apply_declaration(style: &mut ComputedStyle, decl: &Declaration) {
    match decl {
        Declaration::Color(c) => {
            style.color = Some(c.clone());
            style.auto_color = c.auto;
        }
        Declaration::Background(c) => style.background = Some(c.clone()),
        Declaration::Width(s) => style.width = Some(*s),
        Declaration::Height(s) => style.height = Some(*s),
        Declaration::Margin(s) => style.margin = *s,
        Declaration::Padding(s) => style.padding = *s,
        Declaration::Border(b) => style.border = Border::all(b.clone()),
        Declaration::Unknown(_) => {}
    }
}
