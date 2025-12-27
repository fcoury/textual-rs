//! Integration tests for TCSS selector parsing.
//!
//! Tests selector syntax as defined in Textual CSS:
//! - Type selectors: `Button`, `Header`
//! - Class selectors: `.primary`, `.success`
//! - ID selectors: `#sidebar`, `#main`
//! - Universal selector: `*`
//! - Pseudo-classes: `:hover`, `:focus`, `:disabled` (TODO)
//! - Combinators: descendant (space), child (`>`)
//! - Selector lists: `Button, .primary`

use tcss::parser::{Combinator, Selector, Specificity, parse_selector_list};

// ============================================================================
// TYPE SELECTORS
// ============================================================================

#[test]
fn test_type_selector_simple() {
    let (remaining, list) = parse_selector_list("Button").unwrap();
    assert!(remaining.is_empty() || remaining.starts_with(' ') || remaining.starts_with('{'));

    assert_eq!(list.selectors.len(), 1);
    let complex = &list.selectors[0];
    assert_eq!(complex.parts.len(), 1);

    let compound = &complex.parts[0].compound;
    assert_eq!(compound.selectors.len(), 1);
    assert_eq!(compound.selectors[0], Selector::Type("Button".to_string()));
}

#[test]
fn test_type_selector_lowercase() {
    let (_, list) = parse_selector_list("header").unwrap();
    assert_eq!(list.selectors.len(), 1);
    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("header".to_string())
    );
}

#[test]
fn test_type_selector_with_hyphen() {
    let (_, list) = parse_selector_list("my-widget").unwrap();
    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("my-widget".to_string())
    );
}

#[test]
fn test_type_selector_with_underscore() {
    let (_, list) = parse_selector_list("my_widget").unwrap();
    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("my_widget".to_string())
    );
}

// ============================================================================
// CLASS SELECTORS
// ============================================================================

#[test]
fn test_class_selector_simple() {
    let (_, list) = parse_selector_list(".primary").unwrap();
    assert_eq!(list.selectors.len(), 1);
    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Class("primary".to_string())
    );
}

#[test]
fn test_class_selector_with_hyphen() {
    let (_, list) = parse_selector_list(".btn-primary").unwrap();
    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Class("btn-primary".to_string())
    );
}

#[test]
fn test_multiple_classes_chained() {
    // .error.disabled should be a single compound selector with two class selectors
    let (_, list) = parse_selector_list(".error.disabled").unwrap();
    assert_eq!(list.selectors.len(), 1);

    let compound = &list.selectors[0].parts[0].compound;
    assert_eq!(compound.selectors.len(), 2);
    assert_eq!(compound.selectors[0], Selector::Class("error".to_string()));
    assert_eq!(
        compound.selectors[1],
        Selector::Class("disabled".to_string())
    );
}

#[test]
fn test_three_classes_chained() {
    let (_, list) = parse_selector_list(".a.b.c").unwrap();
    let compound = &list.selectors[0].parts[0].compound;
    assert_eq!(compound.selectors.len(), 3);
    assert_eq!(compound.selectors[0], Selector::Class("a".to_string()));
    assert_eq!(compound.selectors[1], Selector::Class("b".to_string()));
    assert_eq!(compound.selectors[2], Selector::Class("c".to_string()));
}

// ============================================================================
// ID SELECTORS
// ============================================================================

#[test]
fn test_id_selector_simple() {
    let (_, list) = parse_selector_list("#sidebar").unwrap();
    assert_eq!(list.selectors.len(), 1);
    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Id("sidebar".to_string())
    );
}

#[test]
fn test_id_selector_with_hyphen() {
    let (_, list) = parse_selector_list("#main-content").unwrap();
    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Id("main-content".to_string())
    );
}

// ============================================================================
// UNIVERSAL SELECTOR
// ============================================================================

#[test]
fn test_universal_selector() {
    let (_, list) = parse_selector_list("*").unwrap();
    assert_eq!(list.selectors.len(), 1);
    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Universal
    );
}

// ============================================================================
// COMPOUND SELECTORS (TYPE + CLASS/ID combinations)
// ============================================================================

#[test]
fn test_type_with_class() {
    // Button.primary
    let (_, list) = parse_selector_list("Button.primary").unwrap();
    let compound = &list.selectors[0].parts[0].compound;
    assert_eq!(compound.selectors.len(), 2);
    assert_eq!(compound.selectors[0], Selector::Type("Button".to_string()));
    assert_eq!(
        compound.selectors[1],
        Selector::Class("primary".to_string())
    );
}

#[test]
fn test_type_with_id() {
    // Button#submit
    let (_, list) = parse_selector_list("Button#submit").unwrap();
    let compound = &list.selectors[0].parts[0].compound;
    assert_eq!(compound.selectors.len(), 2);
    assert_eq!(compound.selectors[0], Selector::Type("Button".to_string()));
    assert_eq!(compound.selectors[1], Selector::Id("submit".to_string()));
}

#[test]
fn test_type_with_class_and_id() {
    // Button.primary#submit
    let (_, list) = parse_selector_list("Button.primary#submit").unwrap();
    let compound = &list.selectors[0].parts[0].compound;
    assert_eq!(compound.selectors.len(), 3);
    assert_eq!(compound.selectors[0], Selector::Type("Button".to_string()));
    assert_eq!(
        compound.selectors[1],
        Selector::Class("primary".to_string())
    );
    assert_eq!(compound.selectors[2], Selector::Id("submit".to_string()));
}

#[test]
fn test_id_with_class() {
    // #sidebar.collapsed
    let (_, list) = parse_selector_list("#sidebar.collapsed").unwrap();
    let compound = &list.selectors[0].parts[0].compound;
    assert_eq!(compound.selectors.len(), 2);
    assert_eq!(compound.selectors[0], Selector::Id("sidebar".to_string()));
    assert_eq!(
        compound.selectors[1],
        Selector::Class("collapsed".to_string())
    );
}

// ============================================================================
// DESCENDANT COMBINATOR (space)
// ============================================================================

#[test]
fn test_descendant_combinator_simple() {
    // Container Button
    let (_, list) = parse_selector_list("Container Button").unwrap();
    assert_eq!(list.selectors.len(), 1);

    let complex = &list.selectors[0];
    assert_eq!(complex.parts.len(), 2);

    assert_eq!(
        complex.parts[0].compound.selectors[0],
        Selector::Type("Container".to_string())
    );
    assert_eq!(complex.parts[0].combinator, Combinator::Descendant);

    assert_eq!(
        complex.parts[1].compound.selectors[0],
        Selector::Type("Button".to_string())
    );
    assert_eq!(complex.parts[1].combinator, Combinator::None);
}

#[test]
fn test_descendant_combinator_with_id() {
    // #dialog Button
    let (_, list) = parse_selector_list("#dialog Button").unwrap();
    let complex = &list.selectors[0];
    assert_eq!(complex.parts.len(), 2);

    assert_eq!(
        complex.parts[0].compound.selectors[0],
        Selector::Id("dialog".to_string())
    );
    assert_eq!(complex.parts[0].combinator, Combinator::Descendant);
}

#[test]
fn test_descendant_combinator_three_levels() {
    // App Container Button
    let (_, list) = parse_selector_list("App Container Button").unwrap();
    let complex = &list.selectors[0];
    assert_eq!(complex.parts.len(), 3);

    assert_eq!(
        complex.parts[0].compound.selectors[0],
        Selector::Type("App".to_string())
    );
    assert_eq!(complex.parts[0].combinator, Combinator::Descendant);

    assert_eq!(
        complex.parts[1].compound.selectors[0],
        Selector::Type("Container".to_string())
    );
    assert_eq!(complex.parts[1].combinator, Combinator::Descendant);

    assert_eq!(
        complex.parts[2].compound.selectors[0],
        Selector::Type("Button".to_string())
    );
    assert_eq!(complex.parts[2].combinator, Combinator::None);
}

#[test]
fn test_universal_descendant() {
    // VerticalScroll *
    let (_, list) = parse_selector_list("VerticalScroll *").unwrap();
    let complex = &list.selectors[0];
    assert_eq!(complex.parts.len(), 2);
    assert_eq!(complex.parts[1].compound.selectors[0], Selector::Universal);
}

// ============================================================================
// CHILD COMBINATOR (>)
// ============================================================================

#[test]
fn test_child_combinator_simple() {
    // Container > Button
    let (_, list) = parse_selector_list("Container > Button").unwrap();
    let complex = &list.selectors[0];
    assert_eq!(complex.parts.len(), 2);

    assert_eq!(
        complex.parts[0].compound.selectors[0],
        Selector::Type("Container".to_string())
    );
    assert_eq!(complex.parts[0].combinator, Combinator::Child);

    assert_eq!(
        complex.parts[1].compound.selectors[0],
        Selector::Type("Button".to_string())
    );
}

#[test]
fn test_child_combinator_no_spaces() {
    // Container>Button (no spaces around >)
    let (_, list) = parse_selector_list("Container>Button").unwrap();
    let complex = &list.selectors[0];
    assert_eq!(complex.parts.len(), 2);
    assert_eq!(complex.parts[0].combinator, Combinator::Child);
}

#[test]
fn test_child_combinator_with_id() {
    // #sidebar > Button
    let (_, list) = parse_selector_list("#sidebar > Button").unwrap();
    let complex = &list.selectors[0];
    assert_eq!(complex.parts.len(), 2);

    assert_eq!(
        complex.parts[0].compound.selectors[0],
        Selector::Id("sidebar".to_string())
    );
    assert_eq!(complex.parts[0].combinator, Combinator::Child);
}

#[test]
fn test_mixed_combinators() {
    // App Container > Button (descendant then child)
    let (_, list) = parse_selector_list("App Container > Button").unwrap();
    let complex = &list.selectors[0];
    assert_eq!(complex.parts.len(), 3);

    assert_eq!(complex.parts[0].combinator, Combinator::Descendant);
    assert_eq!(complex.parts[1].combinator, Combinator::Child);
    assert_eq!(complex.parts[2].combinator, Combinator::None);
}

// ============================================================================
// SELECTOR LISTS (comma-separated)
// ============================================================================

#[test]
fn test_selector_list_two_types() {
    // Button, Label
    let (_, list) = parse_selector_list("Button, Label").unwrap();
    assert_eq!(list.selectors.len(), 2);

    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("Button".to_string())
    );
    assert_eq!(
        list.selectors[1].parts[0].compound.selectors[0],
        Selector::Type("Label".to_string())
    );
}

#[test]
fn test_selector_list_mixed() {
    // Button, .primary, #main
    let (_, list) = parse_selector_list("Button, .primary, #main").unwrap();
    assert_eq!(list.selectors.len(), 3);

    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("Button".to_string())
    );
    assert_eq!(
        list.selectors[1].parts[0].compound.selectors[0],
        Selector::Class("primary".to_string())
    );
    assert_eq!(
        list.selectors[2].parts[0].compound.selectors[0],
        Selector::Id("main".to_string())
    );
}

#[test]
fn test_selector_list_no_spaces() {
    // Button,Label (no spaces)
    let (_, list) = parse_selector_list("Button,Label").unwrap();
    assert_eq!(list.selectors.len(), 2);
}

#[test]
fn test_selector_list_complex_selectors() {
    // Container Button, #dialog .btn
    let (_, list) = parse_selector_list("Container Button, #dialog .btn").unwrap();
    assert_eq!(list.selectors.len(), 2);

    // First: Container Button (descendant)
    assert_eq!(list.selectors[0].parts.len(), 2);
    assert_eq!(
        list.selectors[0].parts[0].combinator,
        Combinator::Descendant
    );

    // Second: #dialog .btn (descendant)
    assert_eq!(list.selectors[1].parts.len(), 2);
    assert_eq!(
        list.selectors[1].parts[0].combinator,
        Combinator::Descendant
    );
}

// ============================================================================
// SPECIFICITY
// ============================================================================

#[test]
fn test_specificity_type_only() {
    let (_, list) = parse_selector_list("Button").unwrap();
    let spec = list.selectors[0].specificity();
    assert_eq!(
        spec,
        Specificity {
            ids: 0,
            classes: 0,
            types: 1
        }
    );
}

#[test]
fn test_specificity_class_only() {
    let (_, list) = parse_selector_list(".primary").unwrap();
    let spec = list.selectors[0].specificity();
    assert_eq!(
        spec,
        Specificity {
            ids: 0,
            classes: 1,
            types: 0
        }
    );
}

#[test]
fn test_specificity_id_only() {
    let (_, list) = parse_selector_list("#main").unwrap();
    let spec = list.selectors[0].specificity();
    assert_eq!(
        spec,
        Specificity {
            ids: 1,
            classes: 0,
            types: 0
        }
    );
}

#[test]
fn test_specificity_compound() {
    // Button.primary#submit = 1 id, 1 class, 1 type
    let (_, list) = parse_selector_list("Button.primary#submit").unwrap();
    let spec = list.selectors[0].specificity();
    assert_eq!(
        spec,
        Specificity {
            ids: 1,
            classes: 1,
            types: 1
        }
    );
}

#[test]
fn test_specificity_descendant() {
    // Container Button = 0 ids, 0 classes, 2 types
    let (_, list) = parse_selector_list("Container Button").unwrap();
    let spec = list.selectors[0].specificity();
    assert_eq!(
        spec,
        Specificity {
            ids: 0,
            classes: 0,
            types: 2
        }
    );
}

#[test]
fn test_specificity_complex() {
    // #dialog .btn.primary Button = 1 id, 2 classes, 1 type
    let (_, list) = parse_selector_list("#dialog .btn.primary Button").unwrap();
    let spec = list.selectors[0].specificity();
    assert_eq!(
        spec,
        Specificity {
            ids: 1,
            classes: 2,
            types: 1
        }
    );
}

#[test]
fn test_specificity_universal_adds_nothing() {
    let (_, list) = parse_selector_list("*").unwrap();
    let spec = list.selectors[0].specificity();
    assert_eq!(
        spec,
        Specificity {
            ids: 0,
            classes: 0,
            types: 0
        }
    );
}

#[test]
fn test_specificity_ordering() {
    // Higher specificity should be greater
    let (_, list1) = parse_selector_list("#main").unwrap(); // 1,0,0
    let (_, list2) = parse_selector_list(".primary.active").unwrap(); // 0,2,0
    let (_, list3) = parse_selector_list("Button").unwrap(); // 0,0,1

    let s1 = list1.selectors[0].specificity();
    let s2 = list2.selectors[0].specificity();
    let s3 = list3.selectors[0].specificity();

    assert!(s1 > s2); // ID beats classes
    assert!(s2 > s3); // Classes beat type
}

// ============================================================================
// PSEUDO-CLASSES (TODO - not yet implemented)
// ============================================================================

// These tests document expected behavior once pseudo-classes are implemented

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_pseudo_class_hover() {
    // Button:hover
    let (_, list) = parse_selector_list("Button:hover").unwrap();
    // Should parse as Button with :hover pseudo-class
    let compound = &list.selectors[0].parts[0].compound;
    assert_eq!(compound.selectors.len(), 2);
    assert_eq!(compound.selectors[0], Selector::Type("Button".to_string()));
    // compound.selectors[1] should be PseudoClass("hover")
}

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_pseudo_class_focus() {
    // Input:focus
    let _ = parse_selector_list("Input:focus").unwrap();
}

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_pseudo_class_disabled() {
    // Button:disabled
    let _ = parse_selector_list("Button:disabled").unwrap();
}

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_pseudo_class_dark_light() {
    // App:dark, App:light
    let _ = parse_selector_list("App:dark").unwrap();
    let _ = parse_selector_list("App:light").unwrap();
}

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_pseudo_class_first_last_child() {
    let _ = parse_selector_list("ListItem:first-child").unwrap();
    let _ = parse_selector_list("ListItem:last-child").unwrap();
}

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_pseudo_class_even_odd() {
    let _ = parse_selector_list("Row:even").unwrap();
    let _ = parse_selector_list("Row:odd").unwrap();
}

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_multiple_pseudo_classes() {
    // Button:hover:focus
    let _ = parse_selector_list("Button:hover:focus").unwrap();
}

// ============================================================================
// WHITESPACE HANDLING
// ============================================================================

#[test]
fn test_leading_whitespace() {
    let (_, list) = parse_selector_list("  Button").unwrap();
    assert_eq!(
        list.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("Button".to_string())
    );
}

#[test]
fn test_multiple_spaces_in_descendant() {
    // Container    Button (multiple spaces)
    let (_, list) = parse_selector_list("Container    Button").unwrap();
    assert_eq!(list.selectors[0].parts.len(), 2);
    assert_eq!(
        list.selectors[0].parts[0].combinator,
        Combinator::Descendant
    );
}
