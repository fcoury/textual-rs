use tcss::parser::cascade::{WidgetMeta, WidgetStates, compute_style};
use tcss::parser::parse_stylesheet;
use tcss::types::Theme;

#[test]
fn computes_text_opacity_for_label() {
    let css = r#"
    #quarter-opacity {
        text-opacity: 25%;
    }

    Label {
        text-style: bold;
    }
    "#;
    let stylesheet = parse_stylesheet(css).expect("failed to parse stylesheet");
    let meta = WidgetMeta {
        type_name: "Label",
        type_names: vec!["Label", "Widget", "DOMNode"],
        id: Some("quarter-opacity".to_string()),
        classes: Vec::new(),
        states: WidgetStates::empty(),
    };

    let style = compute_style(&meta, &[], &stylesheet, &Theme::new("test", true));
    assert_eq!(style.text_opacity, 0.25);
    assert!(style.text_style.bold);
}
