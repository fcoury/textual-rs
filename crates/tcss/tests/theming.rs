#[test]
fn test_theme_color_modifiers() {
    use tcss::parser::cascade::{WidgetMeta, compute_style};
    use tcss::parser::parse_stylesheet;
    use tcss::types::color::RgbaColor;
    use tcss::types::theme::Theme;

    // 1. Setup a theme with a base color
    let mut theme = Theme::new("test", true);
    theme
        .colors
        .insert("primary".into(), RgbaColor::rgb(100, 100, 100)); // Medium Gray

    // 2. CSS using a modifier
    let css = "Button { color: $primary-lighten-2; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    let widget = WidgetMeta {
        type_name: "Button".into(),
        ..Default::default()
    };

    // 3. Compute style
    let style = compute_style(&widget, &[], &stylesheet, &theme);

    // 4. Assert the color is lighter than the original (100, 100, 100)
    let color = style.color.expect("Color should be resolved");
    assert!(color.r > 100, "Red channel {} should be lightened", color.r);
}
