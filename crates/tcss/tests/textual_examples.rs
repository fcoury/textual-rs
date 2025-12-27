//! Integration tests using real TCSS stylesheets from Textual examples.
//!
//! These tests validate `parse_stylesheet` against production TCSS from:
//! - Standalone .tcss files in textual/examples/
//! - Inlined CSS in Python files
//!
//! Tests are organized by feature complexity:
//! - Basic: Stylesheets that should parse with current implementation
//! - Advanced: Stylesheets requiring features not yet implemented (marked #[ignore])

use tcss::parser::parse_stylesheet;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Asserts that a stylesheet parses successfully and returns the expected number of rules.
fn assert_parses_with_rules(source: &str, expected_rules: usize) {
    match parse_stylesheet(source) {
        Ok(stylesheet) => {
            assert_eq!(
                stylesheet.rules.len(),
                expected_rules,
                "Expected {} rules, got {}",
                expected_rules,
                stylesheet.rules.len()
            );
        }
        Err(e) => {
            panic!("Failed to parse stylesheet: {:?}\n\nSource:\n{}", e, source);
        }
    }
}

/// Asserts that a stylesheet parses successfully (rule count doesn't matter).
fn assert_parses(source: &str) {
    match parse_stylesheet(source) {
        Ok(_) => {}
        Err(e) => {
            panic!("Failed to parse stylesheet: {:?}\n\nSource:\n{}", e, source);
        }
    }
}

// ============================================================================
// BASIC STYLESHEETS (should work today)
// ============================================================================

#[test]
fn test_clock_example() {
    // From examples/clock.py - simple inline CSS
    let source = r#"
    Screen { align: center middle; }
    Digits { width: auto; }
    "#;

    // align is unknown, but should parse as Unknown
    // width: auto should work
    assert_parses_with_rules(source, 2);
}

#[test]
fn test_simple_button_style() {
    // Minimal example similar to Textual patterns
    let source = r#"
    Button {
        width: 100%;
        height: 100%;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

#[test]
fn test_id_selector_with_padding() {
    // From examples/dictionary.tcss
    let source = r#"
    #results {
        width: 100%;
        height: auto;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

#[test]
fn test_compound_selector_with_id() {
    // Compound selector pattern from Textual
    let source = r#"
    Input#dictionary-search {
        margin: 1 0;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

#[test]
fn test_descendant_selector() {
    // Descendant selector from five_by_five.tcss
    let source = r#"
    GameHeader #app-title {
        width: 60%;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

#[test]
fn test_multiple_descendant_rules() {
    // Multiple rules with descendant selectors
    let source = r#"
    GameHeader #app-title {
        width: 60%;
    }

    GameHeader #moves {
        width: 20%;
    }

    GameHeader #progress {
        width: 20%;
    }
    "#;

    assert_parses_with_rules(source, 3);
}

#[test]
fn test_class_selector() {
    let source = r#"
    .visible {
        width: 100%;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

#[test]
fn test_type_with_class() {
    // Compound selector: Type + Class
    let source = r#"
    GameCell.filled {
        background: green;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

#[test]
fn test_multiple_rules_with_box_model() {
    let source = r#"
    Screen {
        padding: 1 2;
    }

    Button {
        margin: 1;
        width: 100%;
    }
    "#;

    assert_parses_with_rules(source, 2);
}

#[test]
#[ignore = "variable definition lines are incorrectly expanded - bug in resolve_variables"]
fn test_variables_defined_in_stylesheet() {
    // Variables that are defined within the stylesheet should work
    // BUG: The resolver replaces $my-color in "$my-color: red;" with "red",
    //      resulting in "red: red;" which is invalid syntax.
    let source = r#"
    $my-color: red;
    $my-width: 100;

    Button {
        color: $my-color;
        width: $my-width;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

// ============================================================================
// COMPLEX SELECTORS (descendant chains)
// ============================================================================

#[test]
fn test_descendant_with_class() {
    // From code_browser.tcss: "CodeBrowser.-show-tree #tree-view"
    let source = r#"
    Container.active Button {
        background: blue;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

#[test]
fn test_deep_descendant_chain() {
    let source = r#"
    Screen Container Button.primary {
        color: white;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

// ============================================================================
// STYLESHEETS REQUIRING PSEUDO-CLASSES (not yet implemented)
// ============================================================================

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_hover_pseudo_class() {
    // From five_by_five.tcss
    let source = r#"
    GameCell:hover {
        background: green;
    }
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_focus_pseudo_class() {
    // From dictionary.tcss
    let source = r#"
    #results-container:focus {
        border: tall blue;
    }
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "pseudo-classes not yet implemented"]
fn test_light_pseudo_class() {
    // Textual-specific pseudo-class from merlin.py
    let source = r#"
    Timer:light {
        color: blue;
    }
    "#;

    assert_parses(source);
}

// ============================================================================
// STYLESHEETS REQUIRING NESTING (not yet implemented)
// ============================================================================

#[test]
#[ignore = "nesting with & selector not yet implemented"]
fn test_nesting_inline_pseudo() {
    // From calculator.tcss
    let source = r#"
    #calculator {
        margin: 1 2;

        &:inline {
            margin: 0 2;
        }
    }
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "nesting with & selector not yet implemented"]
fn test_nesting_visible_class() {
    // From sidebar.py
    let source = r#"
    Sidebar {
        width: 30;

        &.-visible {
            width: 100%;
        }
    }
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "nesting with & selector not yet implemented"]
fn test_deep_nesting() {
    // From theme_sandbox.py - deeply nested styles
    let source = r#"
    ColorSample {
        width: 1fr;
        &.hover-surface {
            &:hover {
                background: green;
            }
        }
        &.primary {
            background: blue;
        }
    }
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "nesting with & selector not yet implemented"]
fn test_nested_child_combinator() {
    // From theme_sandbox.py
    let source = r#"
    #buttons {
        height: 3;
        & > Button {
            width: 10;
            margin: 1;
        }
    }
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "nesting with & selector not yet implemented"]
fn test_breakpoints_nesting() {
    // From breakpoints.py - complex nested breakpoint styles
    let source = r#"
    Screen {
        Grid { height: auto; }
        &.-narrow {
            Grid { width: 100%; }
        }
        &.-wide {
            Grid { width: 50%; }
        }
    }
    "#;

    assert_parses(source);
}

// ============================================================================
// STYLESHEETS REQUIRING THEME VARIABLES (not yet implemented)
// ============================================================================

#[test]
#[ignore = "theme variables ($panel, $primary, etc.) not yet implemented"]
fn test_theme_variables() {
    // From calculator.tcss
    let source = r#"
    #numbers {
        background: $panel;
        color: $text;
    }
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "theme variables not yet implemented"]
fn test_sidebar_theme_variables() {
    // From sidebar.py
    let source = r#"
    Sidebar {
        background: $primary;
        border: solid $background;
    }
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "theme variables not yet implemented"]
fn test_five_by_five_theme_variables() {
    // From five_by_five.tcss
    let source = r#"
    GameCell {
        background: $surface;
        border: round $surface;
    }

    GameCell.filled {
        background: $secondary;
        border: round $secondary;
    }
    "#;

    assert_parses(source);
}

// ============================================================================
// STYLESHEETS REQUIRING COMMENTS (not yet implemented)
// ============================================================================

#[test]
#[ignore = "CSS comments not yet implemented"]
fn test_stylesheet_with_comments() {
    // From five_by_five.tcss - has trailing comment
    let source = r#"
    /* Game styles */
    Help {
        border: round blue;
    }
    /* five_by_five.tcss ends here */
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "CSS comments not yet implemented"]
fn test_inline_comment() {
    // From sidebar.py
    let source = r#"
    Sidebar {
        /* Needs to go in its own layer */
        width: 30;
    }
    "#;

    assert_parses(source);
}

// ============================================================================
// STYLESHEETS REQUIRING ADVANCED PROPERTIES (not yet implemented)
// ============================================================================

#[test]
#[ignore = "transition property not yet implemented"]
fn test_transition_property() {
    // From sidebar.py and five_by_five.tcss
    let source = r#"
    $animation-type: linear;
    $animation-speed: 175ms;

    Sidebar {
        transition: offset 200ms;
    }
    "#;

    assert_parses(source);
}

#[test]
#[ignore = "grid properties not yet implemented"]
fn test_grid_layout() {
    // From calculator.tcss
    let source = r#"
    #calculator {
        layout: grid;
        grid-size: 4;
        grid-gutter: 1 2;
        grid-columns: 1fr;
        grid-rows: 2fr 1fr 1fr;
    }
    "#;

    assert_parses(source);
}

// ============================================================================
// FULL STYLESHEET TESTS (comprehensive examples)
// ============================================================================

#[test]
#[ignore = "requires theme variables, nesting, and pseudo-classes"]
fn test_calculator_full() {
    // Full calculator.tcss from Textual examples
    let source = r#"
Screen {
    overflow: auto;
}

#calculator {
    layout: grid;
    grid-size: 4;
    grid-gutter: 1 2;
    grid-columns: 1fr;
    grid-rows: 2fr 1fr 1fr 1fr 1fr 1fr;
    margin: 1 2;
    min-height: 25;
    min-width: 26;
    height: 100%;

    &:inline {
        margin: 0 2;
    }
}

Button {
    width: 100%;
    height: 100%;
}

#numbers {
    column-span: 4;
    padding: 0 1;
    height: 100%;
    background: $panel;
    color: $text;
    content-align: center middle;
    text-align: right;
}

#number-0 {
    column-span: 2;
}
"#;

    assert_parses(source);
}

#[test]
#[ignore = "requires theme variables, nesting, and pseudo-classes"]
fn test_five_by_five_full() {
    // Full five_by_five.tcss from Textual examples
    let source = r#"
$animation-type: linear;
$animation-speed: 175ms;

Game {
    align: center middle;
    layers: gameplay messages;
}

GameGrid {
    layout: grid;
    grid-size: 5 5;
    layer: gameplay;
}

GameHeader {
    background: $primary-background;
    color: $text;
    height: 1;
    dock: top;
    layer: gameplay;
}

GameCell {
    width: 100%;
    height: 100%;
    background: $surface;
    border: round $surface-darken-1;
    transition: background $animation-speed $animation-type, color $animation-speed $animation-type;
}

GameCell:hover {
    background: $panel-lighten-1;
    border: round $panel;
}

GameCell.filled {
    background: $secondary;
    border: round $secondary-darken-1;
}

WinnerMessage {
    width: 50%;
    height: 25%;
    layer: messages;
    visibility: hidden;
    content-align: center middle;
    text-align: center;
    background: $success;
    color: $text;
    border: round;
    padding: 2;
}

.visible {
    visibility: visible;
}

/* five_by_five.tcss ends here */
"#;

    assert_parses(source);
}

#[test]
#[ignore = "requires theme variables and nesting"]
fn test_merlin_full() {
    // CSS from merlin.py
    let source = r#"
    LabelSwitch Label {
        text-align: center;
        width: 1fr;
        text-style: bold;
    }

    LabelSwitch Label#label-5 {
        color: $text-disabled;
    }

    Timer {
        text-align: center;
        width: auto;
        margin: 2 8;
        color: $warning;
        &:light {
            color: $secondary;
        }
    }

    Screen {
        align: center middle;
    }

    Screen.-win {
        background: transparent;
    }

    Screen.-win Timer {
        color: $success;
    }

    Grid {
        width: auto;
        height: auto;
        border: thick $border;
        padding: 1 2;
        grid-size: 3 3;
        grid-rows: auto;
        grid-columns: auto;
        grid-gutter: 1 1;
        background: $surface;
    }
    "#;

    assert_parses(source);
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_empty_stylesheet() {
    let source = "";
    assert_parses_with_rules(source, 0);
}

#[test]
fn test_whitespace_only_stylesheet() {
    let source = "   \n\t\n   ";
    assert_parses_with_rules(source, 0);
}

#[test]
fn test_single_rule_no_trailing_newline() {
    let source = "Button { width: 100%; }";
    assert_parses_with_rules(source, 1);
}

#[test]
fn test_selector_list() {
    let source = r#"
    Button, Label, Input {
        margin: 1;
    }
    "#;

    assert_parses_with_rules(source, 1);
}

/// FEATURE: Pseudo-class selectors (:hover, :focus, :light, etc.)
/// Priority: HIGH - Used extensively in Textual stylesheets
#[test]
fn pseudo_class_hover() {
    let source = "Button:hover { background: blue; }";
    assert_parses(source);
}

/// FEATURE: Pseudo-class selectors
#[test]
fn pseudo_class_focus() {
    let source = "#input:focus { border: solid green; }";
    assert_parses(source);
}

/// FEATURE: Textual-specific pseudo-classes
#[test]
fn pseudo_class_inline() {
    // Textual uses :inline for inline mode styling
    let source = "Screen:inline { height: 50; }";
    assert_parses(source);
}

/// FEATURE: CSS Comments
/// Priority: HIGH - Every real stylesheet has comments
#[test]
fn css_block_comment() {
    let source = "/* Comment */ Button { width: 100%; }";
    assert_parses(source);
}

/// FEATURE: CSS Comments - inline
#[test]
fn css_inline_comment() {
    let source = "Button { width: 100%; /* full width */ }";
    assert_parses(source);
}

/// FEATURE: Theme variables ($panel, $primary, etc.)
/// Priority: HIGH - Textual's theming system depends on this
#[test]
fn theme_variable_panel() {
    // Theme variables are pre-defined by Textual, not in the stylesheet
    let source = "Screen { background: $panel; }";
    assert_parses(source);
}

/// FEATURE: Theme variables with modifiers
#[test]
fn theme_variable_with_modifier() {
    // Textual supports $color-lighten-1, $color-darken-2, etc.
    let source = "Button { background: $primary-lighten-1; }";
    assert_parses(source);
}

/// FEATURE: Nesting with & parent selector
/// Priority: HIGH - Heavily used in Textual styles
#[test]
fn nesting_with_class() {
    let source = r#"
        Button {
            color: white;
            &.active { background: blue; }
        }
        "#;
    assert_parses(source);
}

/// FEATURE: Nesting with pseudo-class
#[test]
fn nesting_with_pseudo() {
    let source = r#"
        Button {
            color: white;
            &:hover { background: green; }
        }
        "#;
    assert_parses(source);
}

/// FEATURE: Nesting with child combinator
#[test]
fn nesting_child_combinator() {
    let source = r#"
        Container {
            padding: 1;
            & > Button { margin: 1; }
        }
        "#;
    assert_parses(source);
}

/// FEATURE: Deep nesting
#[test]
fn deep_nesting() {
    let source = r#"
        Screen {
            &.-narrow {
                Grid { width: 100%; }
            }
        }
        "#;
    assert_parses(source);
}

/// FEATURE: Variable definition and resolution
/// Priority: MEDIUM - Useful for reusable values
#[test]
fn variable_definition_basic() {
    // Variable definitions should be stripped from output after resolution
    let source = r#"
        $primary: blue;
        Button { color: $primary; }
        "#;
    assert_parses_with_rules(source, 1);
}

/// FEATURE: Adjacent sibling combinator (+)
/// Priority: LOW - Less common but part of CSS spec
#[test]
fn adjacent_sibling_combinator() {
    let source = "Label + Input { margin: 1; }";
    assert_parses(source);
}

/// FEATURE: General sibling combinator (~)
#[test]
fn general_sibling_combinator() {
    let source = "Label ~ Input { margin: 1; }";
    assert_parses(source);
}

/// FEATURE: Attribute selectors
/// Priority: LOW - Less common in Textual
#[test]
fn attribute_selector() {
    let source = "Input[type=text] { width: 100%; }";
    assert_parses(source);
}

/// FEATURE: !important modifier
/// Priority: LOW - Should generally be avoided
#[test]
fn important_modifier() {
    let source = "Button { color: red !important; }";
    assert_parses(source);
}
