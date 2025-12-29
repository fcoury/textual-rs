#[test]
fn debug_textual_dark_theme_colors() {
    use tcss::types::theme::Theme;

    let themes = Theme::standard_themes();
    let theme = themes.get("textual-dark").unwrap();

    let panel = theme.get_color("panel").unwrap();
    let foreground = theme.get_color("foreground").unwrap();
    let background = theme.get_color("background").unwrap();
    let surface = theme.get_color("surface").unwrap();

    println!("panel: #{:02x}{:02x}{:02x}", panel.r, panel.g, panel.b);
    println!("foreground: #{:02x}{:02x}{:02x}", foreground.r, foreground.g, foreground.b);
    println!("background: #{:02x}{:02x}{:02x}", background.r, background.g, background.b);
    println!("surface: #{:02x}{:02x}{:02x}", surface.r, surface.g, surface.b);

    // Compute tinted colors like background-tint does
    let mut tint = foreground.clone();

    for percent in [0, 25, 50, 75, 100] {
        tint.a = percent as f32 / 100.0;
        let tinted = panel.tint(&tint);
        let contrast = tinted.get_contrasting_color(0.9);
        let contrast_type = if contrast.r > 128 { "white" } else { "black" };
        println!("{}% tint: #{:02x}{:02x}{:02x} (brightness={:.2}) -> {} text",
            percent, tinted.r, tinted.g, tinted.b,
            tinted.perceived_brightness(), contrast_type);
    }
}

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
