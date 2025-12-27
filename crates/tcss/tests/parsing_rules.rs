//! Integration tests for TCSS full rule and stylesheet parsing.
//!
//! Tests complete TCSS syntax including:
//! - Full rule parsing (selector + declaration block)
//! - Multiple rules
//! - Comments
//! - Variables (TODO)
//! - Nested rules (TODO)
//! - !important (TODO)

use tcss::parser::{Combinator, Declaration, Selector, parse_rule};
use tcss::types::border::BorderKind;
use tcss::types::color::RgbaColor;
use tcss::types::geometry::Unit;

// ============================================================================
// SIMPLE RULES
// ============================================================================

#[test]
fn test_rule_type_selector_single_declaration() {
    let (remaining, rule) = parse_rule("Button { color: red; }").unwrap();
    assert!(remaining.is_empty());

    assert_eq!(rule.selectors.selectors.len(), 1);
    assert_eq!(
        rule.selectors.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("Button".to_string())
    );

    assert_eq!(rule.declarations().len(), 1);
    assert_eq!(
        rule.declarations()[0],
        Declaration::Color(RgbaColor::rgb(255, 0, 0))
    );
}

#[test]
fn test_rule_class_selector() {
    let (_, rule) = parse_rule(".primary { background: blue; }").unwrap();

    assert_eq!(
        rule.selectors.selectors[0].parts[0].compound.selectors[0],
        Selector::Class("primary".to_string())
    );

    assert_eq!(
        rule.declarations()[0],
        Declaration::Background(RgbaColor::rgb(0, 0, 255))
    );
}

#[test]
fn test_rule_id_selector() {
    let (_, rule) = parse_rule("#main { width: 100%; }").unwrap();

    assert_eq!(
        rule.selectors.selectors[0].parts[0].compound.selectors[0],
        Selector::Id("main".to_string())
    );

    if let Declaration::Width(s) = &rule.declarations()[0] {
        assert_eq!(s.value, 100.0);
        assert_eq!(s.unit, Unit::Percent);
    } else {
        panic!("expected Width declaration");
    }
}

#[test]
fn test_rule_universal_selector() {
    let (_, rule) = parse_rule("* { margin: 0; }").unwrap();

    assert_eq!(
        rule.selectors.selectors[0].parts[0].compound.selectors[0],
        Selector::Universal
    );
}

// ============================================================================
// RULES WITH MULTIPLE DECLARATIONS
// ============================================================================

#[test]
fn test_rule_multiple_declarations() {
    let (_, rule) = parse_rule(
        "Button {
            color: red;
            background: blue;
            width: 50;
        }",
    )
    .unwrap();

    assert_eq!(rule.declarations().len(), 3);
    assert_eq!(
        rule.declarations()[0],
        Declaration::Color(RgbaColor::rgb(255, 0, 0))
    );
    assert_eq!(
        rule.declarations()[1],
        Declaration::Background(RgbaColor::rgb(0, 0, 255))
    );
    if let Declaration::Width(s) = &rule.declarations()[2] {
        assert_eq!(s.value, 50.0);
    } else {
        panic!("expected Width");
    }
}

#[test]
fn test_rule_all_box_model_properties() {
    let (_, rule) = parse_rule(
        "Container {
            margin: 10;
            padding: 5;
            border: solid red;
        }",
    )
    .unwrap();

    assert_eq!(rule.declarations().len(), 3);

    if let Declaration::Margin(s) = &rule.declarations()[0] {
        assert_eq!(s.top.value, 10.0);
    } else {
        panic!("expected Margin");
    }

    if let Declaration::Padding(s) = &rule.declarations()[1] {
        assert_eq!(s.top.value, 5.0);
    } else {
        panic!("expected Padding");
    }

    if let Declaration::Border(b) = &rule.declarations()[2] {
        assert_eq!(b.kind, BorderKind::Solid);
    } else {
        panic!("expected Border");
    }
}

// ============================================================================
// RULES WITH COMPLEX SELECTORS
// ============================================================================

#[test]
fn test_rule_compound_selector() {
    let (_, rule) = parse_rule("Button.primary { color: white; }").unwrap();

    let compound = &rule.selectors.selectors[0].parts[0].compound;
    assert_eq!(compound.selectors.len(), 2);
    assert_eq!(compound.selectors[0], Selector::Type("Button".to_string()));
    assert_eq!(
        compound.selectors[1],
        Selector::Class("primary".to_string())
    );
}

#[test]
fn test_rule_descendant_selector() {
    let (_, rule) = parse_rule("Container Button { color: red; }").unwrap();

    let complex = &rule.selectors.selectors[0];
    assert_eq!(complex.parts.len(), 2);
    assert_eq!(complex.parts[0].combinator, Combinator::Descendant);
}

#[test]
fn test_rule_child_selector() {
    let (_, rule) = parse_rule("Container > Button { color: red; }").unwrap();

    let complex = &rule.selectors.selectors[0];
    assert_eq!(complex.parts.len(), 2);
    assert_eq!(complex.parts[0].combinator, Combinator::Child);
}

#[test]
fn test_rule_selector_list() {
    let (_, rule) = parse_rule("Button, .link, #submit { color: blue; }").unwrap();

    assert_eq!(rule.selectors.selectors.len(), 3);
    assert_eq!(
        rule.selectors.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("Button".to_string())
    );
    assert_eq!(
        rule.selectors.selectors[1].parts[0].compound.selectors[0],
        Selector::Class("link".to_string())
    );
    assert_eq!(
        rule.selectors.selectors[2].parts[0].compound.selectors[0],
        Selector::Id("submit".to_string())
    );
}

// ============================================================================
// REALISTIC TCSS EXAMPLES
// ============================================================================

#[test]
fn test_rule_header_dock() {
    // From Textual docs: Header { dock: top; }
    // dock is not yet implemented, so it will be Unknown
    let (_, rule) = parse_rule("Header { dock: top; }").unwrap();

    assert_eq!(
        rule.selectors.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("Header".to_string())
    );

    // dock is unknown for now
    if let Declaration::Unknown(name) = &rule.declarations()[0] {
        assert_eq!(name, "dock");
    } else {
        panic!("expected Unknown declaration for dock");
    }
}

#[test]
fn test_rule_dialog_button() {
    // #dialog Button { text-style: bold; }
    let (_, rule) = parse_rule("#dialog Button { text-style: bold; }").unwrap();

    let complex = &rule.selectors.selectors[0];
    assert_eq!(complex.parts.len(), 2);
    assert_eq!(
        complex.parts[0].compound.selectors[0],
        Selector::Id("dialog".to_string())
    );
    assert_eq!(
        complex.parts[1].compound.selectors[0],
        Selector::Type("Button".to_string())
    );
}

#[test]
fn test_rule_sidebar_child_button() {
    // #sidebar > Button { text-style: underline; }
    let (_, rule) = parse_rule("#sidebar > Button { text-style: underline; }").unwrap();

    let complex = &rule.selectors.selectors[0];
    assert_eq!(complex.parts.len(), 2);
    assert_eq!(complex.parts[0].combinator, Combinator::Child);
}

#[test]
fn test_rule_chained_classes() {
    // .error.disabled { background: darkred; }
    let (_, rule) = parse_rule(".error.disabled { background: darkred; }").unwrap();

    let compound = &rule.selectors.selectors[0].parts[0].compound;
    assert_eq!(compound.selectors.len(), 2);
    assert_eq!(compound.selectors[0], Selector::Class("error".to_string()));
    assert_eq!(
        compound.selectors[1],
        Selector::Class("disabled".to_string())
    );

    assert_eq!(
        rule.declarations()[0],
        Declaration::Background(RgbaColor::rgb(139, 0, 0))
    );
}

#[test]
fn test_rule_universal_in_descendant() {
    // VerticalScroll * { background: red; }
    let (_, rule) = parse_rule("VerticalScroll * { background: red; }").unwrap();

    let complex = &rule.selectors.selectors[0];
    assert_eq!(complex.parts.len(), 2);
    assert_eq!(
        complex.parts[0].compound.selectors[0],
        Selector::Type("VerticalScroll".to_string())
    );
    assert_eq!(complex.parts[1].compound.selectors[0], Selector::Universal);
}

// ============================================================================
// WHITESPACE AND FORMATTING VARIATIONS
// ============================================================================

#[test]
fn test_rule_minimal_whitespace() {
    let (_, rule) = parse_rule("Button{color:red}").unwrap();
    assert_eq!(
        rule.declarations()[0],
        Declaration::Color(RgbaColor::rgb(255, 0, 0))
    );
}

#[test]
fn test_rule_excessive_whitespace() {
    let (_, rule) = parse_rule("   Button   {   color  :   red   ;   }   ").unwrap();
    assert_eq!(
        rule.declarations()[0],
        Declaration::Color(RgbaColor::rgb(255, 0, 0))
    );
}

#[test]
fn test_rule_multiline_formatted() {
    let (_, rule) = parse_rule(
        "
        Button {
            color: red;
            background: blue;
        }
        ",
    )
    .unwrap();

    assert_eq!(rule.declarations().len(), 2);
}

#[test]
fn test_rule_tabs_and_spaces() {
    let (_, rule) = parse_rule("\tButton\t{\n\t\tcolor:\tred;\n\t}").unwrap();
    assert_eq!(
        rule.declarations()[0],
        Declaration::Color(RgbaColor::rgb(255, 0, 0))
    );
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_rule_empty_declarations() {
    let (_, rule) = parse_rule("Button { }").unwrap();
    assert_eq!(rule.declarations().len(), 0);
}

#[test]
fn test_rule_declaration_without_final_semicolon() {
    let (_, rule) = parse_rule("Button { color: red }").unwrap();
    assert_eq!(rule.declarations().len(), 1);
}

#[test]
#[ignore = "empty declarations between semicolons not yet handled"]
fn test_rule_multiple_semicolons() {
    // Extra semicolons might appear in real CSS
    let (_, rule) = parse_rule("Button { color: red;; }").unwrap();
    // Should still parse, possibly with an empty/unknown declaration
    assert!(!rule.declarations().is_empty());
}

// ============================================================================
// FEATURES NOT YET IMPLEMENTED
// These tests document expected behavior for future implementation
// ============================================================================

#[test]
#[ignore = "CSS comments not yet implemented"]
fn test_rule_with_comment() {
    let (_, rule) = parse_rule(
        "/* Button styles */
        Button { color: red; }",
    )
    .unwrap();
    assert_eq!(rule.declarations().len(), 1);
}

#[test]
#[ignore = "inline comments not yet implemented"]
fn test_rule_inline_comment() {
    let (_, rule) = parse_rule("Button { color: red; /* primary color */ }").unwrap();
    assert_eq!(rule.declarations().len(), 1);
}

#[test]
#[ignore = "CSS variables not yet implemented"]
fn test_rule_with_variable() {
    // $primary: red;
    // Button { color: $primary; }
    let (_, rule) = parse_rule("Button { color: $primary; }").unwrap();
    // Should expand variable
    assert!(matches!(rule.declarations()[0], Declaration::Color(_)));
}

#[test]
#[ignore = "nested rules not yet implemented"]
fn test_nested_rule() {
    let input = "
        #questions {
            border: heavy red;

            .button {
                width: 1fr;
            }
        }
    ";
    let _ = parse_rule(input).unwrap();
    // Should produce multiple rules from nested structure
}

#[test]
#[ignore = "nesting selector (&) not yet implemented"]
fn test_nesting_selector() {
    let input = "
        .button {
            color: white;

            &.affirmative { color: green; }
            &.negative { color: red; }
        }
    ";
    let _ = parse_rule(input).unwrap();
}

#[test]
#[ignore = "!important not yet implemented"]
fn test_important_declaration() {
    let (_, rule) = parse_rule("Button { color: red !important; }").unwrap();
    // Declaration should be marked as important
    assert_eq!(rule.declarations().len(), 1);
}

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_rule_with_pseudo_class() {
    let (_, rule) = parse_rule("Button:hover { background: green; }").unwrap();
    // Should parse pseudo-class as part of selector
    assert_eq!(rule.declarations().len(), 1);
}

#[test]
#[ignore = "initial value not yet implemented"]
fn test_initial_value() {
    let (_, rule) = parse_rule("Button { color: initial; }").unwrap();
    // Should reset to default value
    assert_eq!(rule.declarations().len(), 1);
}

// ============================================================================
// MULTIPLE RULES (STYLESHEET)
// ============================================================================

#[test]
fn test_parse_two_rules_sequentially() {
    let input = "Button { color: red; } Label { color: blue; }";

    let (remaining, rule1) = parse_rule(input).unwrap();
    assert_eq!(
        rule1.selectors.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("Button".to_string())
    );

    let (remaining, rule2) = parse_rule(remaining).unwrap();
    assert_eq!(
        rule2.selectors.selectors[0].parts[0].compound.selectors[0],
        Selector::Type("Label".to_string())
    );

    assert!(remaining.trim().is_empty());
}

#[test]
fn test_parse_three_rules() {
    let input = "
        Header { height: 3; }
        Footer { height: 1; }
        .content { height: auto; }
    ";

    let (remaining, _) = parse_rule(input).unwrap();
    let (remaining, _) = parse_rule(remaining).unwrap();
    let (remaining, _) = parse_rule(remaining).unwrap();

    assert!(remaining.trim().is_empty());
}
