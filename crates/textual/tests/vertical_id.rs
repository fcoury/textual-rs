use textual::{Label, Vertical, Widget};

#[test]
fn test_vertical_with_id() {
    let v: Vertical<()> =
        Vertical::new(vec![Box::new(Label::new("test")) as Box<dyn Widget<()>>]).with_id("my-id");

    assert_eq!(v.id(), Some("my-id"));

    let meta = v.get_meta();
    println!("meta.id = {:?}", meta.id);
    assert_eq!(meta.id, Some("my-id".to_string()));
}

#[test]
fn test_vertical_boxed_with_id() {
    let v: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![Box::new(Label::new("test")) as Box<dyn Widget<()>>])
            .with_id("boxed-id"),
    );

    // Check through the dyn trait
    assert_eq!(v.id(), Some("boxed-id"));

    let meta = v.get_meta();
    println!("boxed meta.id = {:?}", meta.id);
    assert_eq!(meta.id, Some("boxed-id".to_string()));
}

#[test]
fn test_vertical_style_resolution_with_id() {
    use tcss::parser::{
        cascade::{WidgetMeta, WidgetStates, compute_style},
        parse_stylesheet,
    };
    use tcss::types::Theme;
    use textual::{Label, Vertical, Widget};

    let css = r#"
Vertical {
    background: $panel;
}
#tint1 { background-tint: $foreground 25%; }
"#;

    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = Theme::standard_themes()
        .get("textual-dark")
        .unwrap()
        .clone();

    // Create a Vertical with id
    let v: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![Box::new(Label::new("test")) as Box<dyn Widget<()>>]).with_id("tint1"),
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

    assert!(
        style.background.is_some(),
        "background should be set from Vertical rule"
    );
    assert!(
        style.background_tint.is_some(),
        "background_tint should be set from #tint1 rule"
    );
}

#[test]
fn test_style_resolver_applies_styles_correctly() {
    use std::collections::VecDeque;
    use tcss::parser::cascade::WidgetMeta;
    use tcss::parser::parse_stylesheet;
    use tcss::types::Theme;
    use textual::style_resolver::resolve_styles;
    use textual::{Label, Vertical, Widget};

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
    let theme = Theme::standard_themes()
        .get("textual-dark")
        .unwrap()
        .clone();

    // Create a tree similar to what the app would create
    let mut v1: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![Box::new(Label::new("0%")) as Box<dyn Widget<()>>]).with_id("tint1"),
    );
    let mut v2: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![Box::new(Label::new("25%")) as Box<dyn Widget<()>>]).with_id("tint2"),
    );
    let mut v3: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![Box::new(Label::new("50%")) as Box<dyn Widget<()>>]).with_id("tint3"),
    );

    // Resolve styles for each widget
    let mut ancestors: VecDeque<WidgetMeta> = VecDeque::new();
    resolve_styles(v1.as_mut(), &stylesheet, &theme, &mut ancestors);
    let mut ancestors: VecDeque<WidgetMeta> = VecDeque::new();
    resolve_styles(v2.as_mut(), &stylesheet, &theme, &mut ancestors);
    let mut ancestors: VecDeque<WidgetMeta> = VecDeque::new();
    resolve_styles(v3.as_mut(), &stylesheet, &theme, &mut ancestors);

    // Check that styles were applied
    let style1 = v1.get_style();
    let style2 = v2.get_style();
    let style3 = v3.get_style();

    println!(
        "v1 (tint1): background={:?}, tint={:?}",
        style1.background, style1.background_tint
    );
    println!(
        "v2 (tint2): background={:?}, tint={:?}",
        style2.background, style2.background_tint
    );
    println!(
        "v3 (tint3): background={:?}, tint={:?}",
        style3.background, style3.background_tint
    );

    assert!(
        style1.background.is_some(),
        "v1 should have background from Vertical rule"
    );
    assert!(
        style1.background_tint.is_some(),
        "v1 should have background_tint from #tint1 rule"
    );

    assert!(
        style2.background.is_some(),
        "v2 should have background from Vertical rule"
    );
    assert!(
        style2.background_tint.is_some(),
        "v2 should have background_tint from #tint2 rule"
    );

    assert!(
        style3.background.is_some(),
        "v3 should have background from Vertical rule"
    );
    assert!(
        style3.background_tint.is_some(),
        "v3 should have background_tint from #tint3 rule"
    );

    // Verify the tint values are different (different alpha values)
    let tint1 = style1.background_tint.as_ref().unwrap();
    let tint2 = style2.background_tint.as_ref().unwrap();
    let tint3 = style3.background_tint.as_ref().unwrap();

    println!("tint1.a = {}", tint1.a);
    println!("tint2.a = {}", tint2.a);
    println!("tint3.a = {}", tint3.a);

    assert!(
        tint1.a < tint2.a,
        "tint1 (0%) should have lower alpha than tint2 (25%)"
    );
    assert!(
        tint2.a < tint3.a,
        "tint2 (25%) should have lower alpha than tint3 (50%)"
    );
}

#[test]
fn test_render_cache_produces_tinted_background() {
    use tcss::types::{ComputedStyle, RgbaColor};
    use textual::render_cache::RenderCache;

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
    println!(
        "Expected tinted color: R={} G={} B={}",
        expected_r, expected_g, expected_b
    );

    // Create render cache and render a line
    let cache = RenderCache::new(&style);
    let strip = cache.render_line(0, 5, 10, None, None, None);

    // Get the first segment and check its background color
    let segments = strip.segments();
    assert!(!segments.is_empty(), "Strip should have segments");

    let first_segment = &segments[0];
    let seg_bg = first_segment.bg();
    println!("Segment background: {:?}", seg_bg);

    assert!(seg_bg.is_some(), "Segment should have a background color");
    let bg_color = seg_bg.unwrap();

    // Verify the color is the tinted color (truncated to match blending behavior)
    assert_eq!(
        bg_color.r,
        expected_r as u8,
        "Red component should be tinted"
    );
    assert_eq!(
        bg_color.g,
        expected_g as u8,
        "Green component should be tinted"
    );
    assert_eq!(
        bg_color.b,
        expected_b as u8,
        "Blue component should be tinted"
    );
}

#[test]
fn test_auto_color_resolves_against_effective_background() {
    use std::collections::VecDeque;
    use tcss::parser::cascade::WidgetMeta;
    use tcss::parser::parse_stylesheet;
    use tcss::types::Theme;
    use textual::style_resolver::resolve_styles;
    use textual::{Label, Vertical, Widget};

    // CSS with auto color on containers with different tint levels
    let css = r#"
Vertical {
    background: $panel;
    color: auto 90%;
}
#tint1 { background-tint: $foreground 0%; }
#tint5 { background-tint: $foreground 100%; }
"#;

    let stylesheet = parse_stylesheet(css).unwrap();
    let theme = Theme::standard_themes()
        .get("textual-dark")
        .unwrap()
        .clone();

    // Create containers with different tint levels
    let mut v1: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![Box::new(Label::new("0%")) as Box<dyn Widget<()>>]).with_id("tint1"),
    );
    let mut v5: Box<dyn Widget<()>> = Box::new(
        Vertical::new(vec![Box::new(Label::new("100%")) as Box<dyn Widget<()>>]).with_id("tint5"),
    );

    // Resolve styles
    let mut ancestors: VecDeque<WidgetMeta> = VecDeque::new();
    resolve_styles(v1.as_mut(), &stylesheet, &theme, &mut ancestors);
    let mut ancestors: VecDeque<WidgetMeta> = VecDeque::new();
    resolve_styles(v5.as_mut(), &stylesheet, &theme, &mut ancestors);

    // Check that auto_color is set
    let style1 = v1.get_style();
    let style5 = v5.get_style();

    println!(
        "v1 (0% tint): auto_color={}, color={:?}",
        style1.auto_color, style1.color
    );
    println!(
        "v5 (100% tint): auto_color={}, color={:?}",
        style5.auto_color, style5.color
    );

    assert!(style1.auto_color, "v1 should have auto_color set");
    assert!(style5.auto_color, "v5 should have auto_color set");

    // The color should have been stored with the contrast ratio in alpha
    let color1 = style1.color.as_ref().expect("v1 should have color");
    let color5 = style5.color.as_ref().expect("v5 should have color");

    // auto 90% should have alpha = 0.9
    assert!(
        (color1.a - 0.9).abs() < 0.01,
        "v1 color alpha should be ~0.9, got {}",
        color1.a
    );
    assert!(
        (color5.a - 0.9).abs() < 0.01,
        "v5 color alpha should be ~0.9, got {}",
        color5.a
    );

    // Check that effective background is computed
    // tint1 has 0% tint - should be close to $panel (dark)
    // tint5 has 100% tint - should be close to $foreground (light)
    let bg1 = style1
        .background
        .as_ref()
        .expect("v1 should have background");
    let tint1 = style1
        .background_tint
        .as_ref()
        .expect("v1 should have background_tint");
    let tint5 = style5
        .background_tint
        .as_ref()
        .expect("v5 should have background_tint");

    let effective_bg1 = bg1.tint(tint1);
    let bg5 = style5.background.as_ref().unwrap();
    let effective_bg5 = bg5.tint(tint5);

    println!("effective_bg1 luminance: {}", effective_bg1.luminance());
    println!("effective_bg5 luminance: {}", effective_bg5.luminance());

    // 0% tint should have dark background (low luminance)
    assert!(
        effective_bg1.luminance() < 0.3,
        "0% tint should have dark background"
    );
    // 100% tint should have light background (high luminance)
    assert!(
        effective_bg5.luminance() > 0.5,
        "100% tint should have light background"
    );

    // Now test that contrasting colors are computed correctly
    let contrast1 = effective_bg1.get_contrasting_color(0.9);
    let contrast5 = effective_bg5.get_contrasting_color(0.9);

    println!(
        "contrast1 (dark bg): r={}, g={}, b={}",
        contrast1.r, contrast1.g, contrast1.b
    );
    println!(
        "contrast5 (light bg): r={}, g={}, b={}",
        contrast5.r, contrast5.g, contrast5.b
    );

    // Dark background should get light text
    assert!(
        contrast1.r > 200 && contrast1.g > 200 && contrast1.b > 200,
        "Dark background should get light text, got r={} g={} b={}",
        contrast1.r,
        contrast1.g,
        contrast1.b
    );
    // Light background should get dark text
    assert!(
        contrast5.r < 100 && contrast5.g < 100 && contrast5.b < 100,
        "Light background should get dark text, got r={} g={} b={}",
        contrast5.r,
        contrast5.g,
        contrast5.b
    );
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
