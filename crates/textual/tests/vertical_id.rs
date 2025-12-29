use textual::{Vertical, Widget, Label};

#[test]
fn test_vertical_with_id() {
    let v: Vertical<()> = Vertical::new(vec![
        Box::new(Label::new("test")) as Box<dyn Widget<()>>
    ]).with_id("my-id");
    
    assert_eq!(v.id(), Some("my-id"));
    
    let meta = v.get_meta();
    println!("meta.id = {:?}", meta.id);
    assert_eq!(meta.id, Some("my-id".to_string()));
}

#[test]
fn test_vertical_boxed_with_id() {
    let v: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![
            Box::new(Label::new("test")) as Box<dyn Widget<()>>
        ]).with_id("boxed-id")
    );
    
    // Check through the dyn trait
    assert_eq!(v.id(), Some("boxed-id"));
    
    let meta = v.get_meta();
    println!("boxed meta.id = {:?}", meta.id);
    assert_eq!(meta.id, Some("boxed-id".to_string()));
}

#[test]
fn test_vertical_style_resolution_with_id() {
    use tcss::parser::{parse_stylesheet, cascade::{compute_style, WidgetMeta, WidgetStates}};
    use tcss::types::Theme;
    use textual::{Vertical, Widget, Label};
    
    let css = r#"
Vertical {
    background: $panel;
}
#tint1 { background-tint: $foreground 25%; }
"#;
    
    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = Theme::standard_themes().get("textual-dark").unwrap().clone();
    
    // Create a Vertical with id
    let v: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![
            Box::new(Label::new("test")) as Box<dyn Widget<()>>
        ]).with_id("tint1")
    );
    
    // Verify the widget meta has the correct id
    let meta = v.get_meta();
    println!("Widget meta: type={}, id={:?}", meta.type_name, meta.id);
    assert_eq!(meta.id, Some("tint1".to_string()));
    
    // Compute style using the widget's meta
    let tcss_meta = WidgetMeta {
        type_name: meta.type_name,
        id: meta.id,
        classes: meta.classes,
        states: WidgetStates::empty(),
    };
    
    let style = compute_style(&tcss_meta, &[], &stylesheet, &theme);
    println!("Computed style:");
    println!("  background: {:?}", style.background);
    println!("  background_tint: {:?}", style.background_tint);
    
    assert!(style.background.is_some(), "background should be set from Vertical rule");
    assert!(style.background_tint.is_some(), "background_tint should be set from #tint1 rule");
}

#[test]
fn test_style_resolver_applies_styles_correctly() {
    use tcss::parser::parse_stylesheet;
    use tcss::types::Theme;
    use textual::{Vertical, Widget, Label};
    use textual::style_resolver::resolve_styles;
    use tcss::parser::cascade::WidgetMeta;

    // Use actual CSS from the example
    let css = r#"
Vertical {
    background: $panel;
}
#tint1 { background-tint: $foreground 0%; }
#tint2 { background-tint: $foreground 25%; }
#tint3 { background-tint: $foreground 50%; }
"#;

    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = Theme::standard_themes().get("textual-dark").unwrap().clone();

    // Create a tree similar to what the app would create
    let mut v1: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![
            Box::new(Label::new("0%")) as Box<dyn Widget<()>>
        ]).with_id("tint1")
    );
    let mut v2: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![
            Box::new(Label::new("25%")) as Box<dyn Widget<()>>
        ]).with_id("tint2")
    );
    let mut v3: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![
            Box::new(Label::new("50%")) as Box<dyn Widget<()>>
        ]).with_id("tint3")
    );

    // Resolve styles for each widget
    let mut ancestors: Vec<WidgetMeta> = Vec::new();
    resolve_styles(v1.as_mut(), &stylesheet, &theme, &mut ancestors);
    let mut ancestors: Vec<WidgetMeta> = Vec::new();
    resolve_styles(v2.as_mut(), &stylesheet, &theme, &mut ancestors);
    let mut ancestors: Vec<WidgetMeta> = Vec::new();
    resolve_styles(v3.as_mut(), &stylesheet, &theme, &mut ancestors);

    // Check that styles were applied
    let style1 = v1.get_style();
    let style2 = v2.get_style();
    let style3 = v3.get_style();

    println!("v1 (tint1): background={:?}, tint={:?}", style1.background, style1.background_tint);
    println!("v2 (tint2): background={:?}, tint={:?}", style2.background, style2.background_tint);
    println!("v3 (tint3): background={:?}, tint={:?}", style3.background, style3.background_tint);

    assert!(style1.background.is_some(), "v1 should have background from Vertical rule");
    assert!(style1.background_tint.is_some(), "v1 should have background_tint from #tint1 rule");

    assert!(style2.background.is_some(), "v2 should have background from Vertical rule");
    assert!(style2.background_tint.is_some(), "v2 should have background_tint from #tint2 rule");

    assert!(style3.background.is_some(), "v3 should have background from Vertical rule");
    assert!(style3.background_tint.is_some(), "v3 should have background_tint from #tint3 rule");

    // Verify the tint values are different (different alpha values)
    let tint1 = style1.background_tint.as_ref().unwrap();
    let tint2 = style2.background_tint.as_ref().unwrap();
    let tint3 = style3.background_tint.as_ref().unwrap();

    println!("tint1.a = {}", tint1.a);
    println!("tint2.a = {}", tint2.a);
    println!("tint3.a = {}", tint3.a);

    assert!(tint1.a < tint2.a, "tint1 (0%) should have lower alpha than tint2 (25%)");
    assert!(tint2.a < tint3.a, "tint2 (25%) should have lower alpha than tint3 (50%)");
}

#[test]
fn test_render_cache_produces_tinted_background() {
    use textual::render_cache::RenderCache;
    use tcss::types::{ComputedStyle, RgbaColor};

    // Create a style with background and background_tint
    let mut style = ComputedStyle::default();
    let mut bg = RgbaColor::rgb(31, 31, 31);
    bg.a = 1.0;
    style.background = Some(bg);

    let mut tint = RgbaColor::rgb(224, 224, 224);
    tint.a = 0.5; // 50% tint
    style.background_tint = Some(tint);

    // Calculate expected tinted color
    // tint function: lerp bg toward tint by tint.a
    // result = bg + (tint - bg) * tint.a
    let expected_r: f32 = 31.0 + (224.0 - 31.0) * 0.5;
    let expected_g: f32 = 31.0 + (224.0 - 31.0) * 0.5;
    let expected_b: f32 = 31.0 + (224.0 - 31.0) * 0.5;
    println!("Expected tinted color: R={} G={} B={}", expected_r, expected_g, expected_b);

    // Create render cache and render a line
    let cache = RenderCache::new(&style);
    let strip = cache.render_line(0, 5, 10, None, None);

    // Get the first segment and check its background color
    let segments = strip.segments();
    assert!(!segments.is_empty(), "Strip should have segments");

    let first_segment = &segments[0];
    let seg_bg = first_segment.bg();
    println!("Segment background: {:?}", seg_bg);

    assert!(seg_bg.is_some(), "Segment should have a background color");
    let bg_color = seg_bg.unwrap();

    // Verify the color is the tinted color (approximately)
    assert_eq!(bg_color.r, expected_r.round() as u8, "Red component should be tinted");
    assert_eq!(bg_color.g, expected_g.round() as u8, "Green component should be tinted");
    assert_eq!(bg_color.b, expected_b.round() as u8, "Blue component should be tinted");
}

#[test]
fn test_actual_css_file_parses() {
    use tcss::parser::parse_stylesheet;

    // Use the actual CSS from the example
    let css = r#"
Vertical {
    background: $panel;
    color: auto 90%;
}
#tint1 { background-tint: $foreground 0%; }
#tint2 { background-tint: $foreground 25%; }
#tint3 { background-tint: $foreground 50%; }
#tint4 { background-tint: $foreground 75%; }
#tint5 { background-tint: $foreground 100%; }
"#;

    let stylesheet = parse_stylesheet(css);
    match &stylesheet {
        Ok(s) => {
            println!("CSS parsed successfully! {} rules", s.rules.len());
            for rule in &s.rules {
                println!("Rule selectors: {:?}", rule.selectors);
                for decl in rule.declarations() {
                    println!("  Declaration: {:?}", decl);
                }
            }
        }
        Err(e) => {
            println!("CSS parse error: {}", e);
        }
    }

    assert!(stylesheet.is_ok(), "CSS should parse successfully");
    let stylesheet = stylesheet.unwrap();

    // Should have 6 rules: Vertical, #tint1-5
    assert_eq!(stylesheet.rules.len(), 6, "Should have 6 rules");
}
