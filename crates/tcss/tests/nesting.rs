use tcss::parser::parse_stylesheet;

#[test]
fn test_nesting_flattening() {
    let css = r#"
        Button {
            color: white;
            &:hover { color: red; }
        }
    "#;

    let sheet = parse_stylesheet(css).unwrap();
    // We expect TWO rules:
    // 1. Button { color: white; }
    // 2. Button:hover { color: red; }
    assert_eq!(sheet.rules.len(), 2);

    let second_rule_selector = &sheet.rules[1].selectors.selectors[0];
    assert!(format!("{:?}", second_rule_selector).contains("PseudoClass(\"hover\")"));
}

#[test]
fn test_deep_recursive_nesting() {
    let source = r#"
    Screen {
        Container {
            &:hover {
                background: blue;
            }
        }
    }
    "#;

    let sheet =
        tcss::parser::parse_stylesheet(source).expect("Deep nested stylesheet should parse");

    // We expect 1 rule because the outer blocks only contain nested rules, not direct declarations
    // Only the innermost block with 'background: blue' produces a Rule.
    assert_eq!(sheet.rules.len(), 1);

    let rule = &sheet.rules[0];
    let complex = &rule.selectors.selectors[0];

    // The selector should be "Screen Container:hover"
    // Part 0: Screen (Descendant)
    // Part 1: Container:hover (None)
    assert_eq!(complex.parts.len(), 2);

    let first_part = &complex.parts[0];
    assert_eq!(
        first_part.compound.selectors[0],
        tcss::parser::Selector::Type("Screen".into())
    );
    assert_eq!(first_part.combinator, tcss::parser::Combinator::Descendant);

    let second_part = &complex.parts[1];
    // Check for "Container" AND "hover" in the same compound selector
    let selectors = &second_part.compound.selectors;
    assert!(selectors.contains(&tcss::parser::Selector::Type("Container".into())));
    assert!(selectors.contains(&tcss::parser::Selector::PseudoClass("hover".into())));
}
