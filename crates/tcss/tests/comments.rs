use tcss::parser::parse_stylesheet;

#[test]
fn test_block_comments() {
    let css = r#"
        Button {
            /* this is a comment */
            color: red;
        }
    "#;
    let sheet = parse_stylesheet(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_inline_comments() {
    let css = r#"
        Button {
            color: red; /* inline comment */
            background: blue;
        }
    "#;
    let sheet = parse_stylesheet(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_line_comments() {
    let css = r#"
        Button {
            // line comment
            color: red;
        }
    "#;
    let sheet = parse_stylesheet(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_comment_before_rule() {
    let css = r#"
        /* Header comment */
        Button {
            color: red;
        }
    "#;
    let sheet = parse_stylesheet(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_comment_between_rules() {
    let css = r#"
        Button {
            color: red;
        }
        /* Comment between rules */
        Label {
            color: blue;
        }
    "#;
    let sheet = parse_stylesheet(css).unwrap();
    assert_eq!(sheet.rules.len(), 2);
}

#[test]
fn test_multiline_block_comment() {
    let css = r#"
        Button {
            /*
             * Multi-line
             * comment
             */
            color: red;
        }
    "#;
    let sheet = parse_stylesheet(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}
