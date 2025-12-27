//! Nested rule flattening for TCSS.
//!
//! This module handles flattening of nested CSS rules (SCSS-style nesting)
//! into flat rules suitable for the cascade.
//!
//! ## Nesting Syntax
//!
//! TCSS supports nested rules using the `&` parent selector:
//!
//! ```css
//! Button {
//!     color: white;
//!     &:hover { background: blue; }
//!     &.active { background: green; }
//! }
//! ```
//!
//! This flattens to:
//!
//! ```css
//! Button { color: white; }
//! Button:hover { background: blue; }
//! Button.active { background: green; }
//! ```
//!
//! ## Flattening Rules
//!
//! - `&` is replaced by the parent selector
//! - `&.class` appends class to parent's last compound selector
//! - `& > child` creates child combinator from parent
//! - Nested without `&` implies descendant combinator

use crate::parser::stylesheet::{
    Combinator, ComplexSelector, Rule, RuleItem, Selector, SelectorList, StyleSheet,
};

/// Flattens a list of potentially nested rules into a flat stylesheet.
///
/// Nested rules (using `&` parent selector) are expanded into multiple
/// flat rules with combined selectors.
pub fn flatten_stylesheet(raw_rules: Vec<Rule>) -> StyleSheet {
    let mut flat_rules = Vec::new();
    for rule in raw_rules {
        flatten_rule(&rule, None, &mut flat_rules);
    }
    StyleSheet { rules: flat_rules }
}

fn flatten_rule(rule: &Rule, parent_selectors: Option<&[ComplexSelector]>, output: &mut Vec<Rule>) {
    // 1. Resolve local declarations first
    let current_decls: Vec<RuleItem> = rule
        .items
        .iter()
        .filter(|i| matches!(i, RuleItem::Declaration(_)))
        .cloned()
        .collect();

    if !current_decls.is_empty() {
        output.push(Rule {
            selectors: rule.selectors.clone(),
            items: current_decls,
        });
    }

    // 2. Resolve and lift nested rules
    for item in &rule.items {
        if let RuleItem::NestedRule(nested) = item {
            let parents = parent_selectors.unwrap_or(&rule.selectors.selectors);
            let combined = combine_selectors(parents, &nested.selectors.selectors);

            let mut merged = nested.clone();
            merged.selectors = SelectorList::new(combined);
            // Recursively flatten with the new combined selectors as the parent context
            flatten_rule(&merged, Some(&merged.selectors.selectors), output);
        }
    }
}

fn combine_selectors(
    parents: &[ComplexSelector],
    children: &[ComplexSelector],
) -> Vec<ComplexSelector> {
    let mut combined = Vec::new();
    for p in parents {
        for c in children {
            let mut p_parts = p.parts.clone();
            let c_parts = &c.parts;

            if let Some(first_c_part) = c_parts.first() {
                // Check if the first compound selector of the child contains '&'
                let has_parent_ref = first_c_part
                    .compound
                    .selectors
                    .iter()
                    .any(|s| matches!(s, Selector::Parent));

                if has_parent_ref {
                    // CASE: &:hover
                    // Merge child selectors (minus &) into the last part of the parent
                    if let Some(last_p_part) = p_parts.last_mut() {
                        for s in &first_c_part.compound.selectors {
                            if !matches!(s, Selector::Parent) {
                                last_p_part.compound.selectors.push(s.clone());
                            }
                        }
                        // Inherit the combinator from the child's first part if it's not None
                        if first_c_part.combinator != Combinator::None {
                            last_p_part.combinator = first_c_part.combinator;
                        }
                    }
                    // Append subsequent parts of the child selector
                    p_parts.extend(c_parts.iter().skip(1).cloned());
                } else {
                    // CASE: Button Label { ... } (Descendant)
                    // Ensure the parent's last part has a Descendant combinator
                    if let Some(last) = p_parts.last_mut() {
                        if matches!(last.combinator, Combinator::None) {
                            last.combinator = Combinator::Descendant;
                        }
                    }
                    p_parts.extend(c_parts.iter().cloned());
                }
            }
            combined.push(ComplexSelector::new(p_parts));
        }
    }
    combined
}
