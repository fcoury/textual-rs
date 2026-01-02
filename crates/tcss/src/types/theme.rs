//! Theme definitions for TCSS color palettes.
//!
//! Themes provide named color variables that can be referenced in stylesheets
//! using the `$variable` syntax. This enables consistent theming and easy
//! switching between light/dark modes.
//!
//! ## Standard Variables
//!
//! Textual themes define these base color variables:
//!
//! | Variable     | Description                              |
//! |--------------|------------------------------------------|
//! | `$primary`   | Primary accent color                     |
//! | `$secondary` | Secondary accent color                   |
//! | `$warning`   | Warning/caution color                    |
//! | `$error`     | Error/danger color                       |
//! | `$success`   | Success/positive color                   |
//! | `$accent`    | Accent highlight color                   |
//! | `$foreground`| Default foreground/text color            |
//! | `$background`| Default background color                 |
//! | `$surface`   | Surface/card background                  |
//! | `$panel`     | Panel/container background               |
//! | `$boost`     | Boost/highlight overlay                  |
//!
//! ## Color Shades
//!
//! For every base color, Textual generates 3 dark and 3 light shades:
//!
//! - `$primary-lighten-1`, `$primary-lighten-2`, `$primary-lighten-3`
//! - `$primary-darken-1`, `$primary-darken-2`, `$primary-darken-3`
//!
//! ## Additional Variables
//!
//! Themes also generate:
//! - Text colors: `$text`, `$text-muted`, `$text-disabled`, `$text-primary`, etc.
//! - Scrollbar: `$scrollbar`, `$scrollbar-hover`, `$scrollbar-background`, etc.
//! - Links: `$link-color`, `$link-background`, `$link-color-hover`, etc.
//! - Muted variants: `$primary-muted`, `$error-muted`, etc.

use crate::types::color::RgbaColor;
use crate::types::text::TextStyle;
use std::collections::HashMap;

const NUMBER_OF_SHADES: i32 = 3;

/// A color system that generates theme variables from base colors.
///
/// This mirrors Python Textual's ColorSystem, generating ~100+ CSS variables
/// from a small set of base colors.
#[derive(Debug, Clone)]
pub struct ColorSystem {
    /// Primary accent color (required)
    pub primary: RgbaColor,
    /// Secondary accent color
    pub secondary: Option<RgbaColor>,
    /// Warning/caution color
    pub warning: Option<RgbaColor>,
    /// Error/danger color
    pub error: Option<RgbaColor>,
    /// Success/positive color
    pub success: Option<RgbaColor>,
    /// Accent highlight color
    pub accent: Option<RgbaColor>,
    /// Default foreground/text color
    pub foreground: Option<RgbaColor>,
    /// Default background color
    pub background: Option<RgbaColor>,
    /// Surface/card background
    pub surface: Option<RgbaColor>,
    /// Panel/container background
    pub panel: Option<RgbaColor>,
    /// Boost/highlight overlay
    pub boost: Option<RgbaColor>,
    /// Whether this is a dark theme
    pub dark: bool,
    /// Luminosity spread for generated shades (Textual default: 0.15)
    pub luminosity_spread: f32,
    /// Default text alpha for contrast text (Textual default: 0.95)
    pub text_alpha: f32,
}

impl ColorSystem {
    /// Creates a new color system with the given primary color.
    pub fn new(primary: RgbaColor, dark: bool) -> Self {
        Self {
            primary,
            secondary: None,
            warning: None,
            error: None,
            success: None,
            accent: None,
            foreground: None,
            background: None,
            surface: None,
            panel: None,
            boost: None,
            dark,
            luminosity_spread: 0.15,
            text_alpha: 0.95,
        }
    }

    /// Builder method to set secondary color.
    pub fn with_secondary(mut self, color: RgbaColor) -> Self {
        self.secondary = Some(color);
        self
    }

    /// Builder method to set warning color.
    pub fn with_warning(mut self, color: RgbaColor) -> Self {
        self.warning = Some(color);
        self
    }

    /// Builder method to set error color.
    pub fn with_error(mut self, color: RgbaColor) -> Self {
        self.error = Some(color);
        self
    }

    /// Builder method to set success color.
    pub fn with_success(mut self, color: RgbaColor) -> Self {
        self.success = Some(color);
        self
    }

    /// Builder method to set accent color.
    pub fn with_accent(mut self, color: RgbaColor) -> Self {
        self.accent = Some(color);
        self
    }

    /// Builder method to set foreground color.
    pub fn with_foreground(mut self, color: RgbaColor) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Builder method to set background color.
    pub fn with_background(mut self, color: RgbaColor) -> Self {
        self.background = Some(color);
        self
    }

    /// Builder method to set surface color.
    pub fn with_surface(mut self, color: RgbaColor) -> Self {
        self.surface = Some(color);
        self
    }

    /// Builder method to set panel color.
    pub fn with_panel(mut self, color: RgbaColor) -> Self {
        self.panel = Some(color);
        self
    }

    /// Builder method to set boost color.
    pub fn with_boost(mut self, color: RgbaColor) -> Self {
        self.boost = Some(color);
        self
    }

    /// Builder method to set luminosity spread (for shade generation).
    pub fn with_luminosity_spread(mut self, spread: f32) -> Self {
        self.luminosity_spread = spread;
        self
    }

    /// Builder method to set default text alpha (for contrast text).
    pub fn with_text_alpha(mut self, alpha: f32) -> Self {
        self.text_alpha = alpha;
        self
    }

    /// Returns the default background for dark/light themes.
    fn default_background(&self) -> RgbaColor {
        if self.dark {
            RgbaColor::hex("#121212")
        } else {
            RgbaColor::hex("#efefef")
        }
    }

    /// Returns the default surface for dark/light themes.
    fn default_surface(&self) -> RgbaColor {
        if self.dark {
            RgbaColor::hex("#1e1e1e")
        } else {
            RgbaColor::hex("#f5f5f5")
        }
    }

    /// Generates all CSS variables from this color system.
    pub fn generate(&self) -> HashMap<String, RgbaColor> {
        let mut vars = HashMap::new();

        let primary = self.primary.clone();
        let secondary = self.secondary.clone().unwrap_or_else(|| primary.clone());
        let warning = self.warning.clone().unwrap_or_else(|| primary.clone());
        let error = self.error.clone().unwrap_or_else(|| secondary.clone());
        let success = self.success.clone().unwrap_or_else(|| secondary.clone());
        let accent = self.accent.clone().unwrap_or_else(|| primary.clone());

        let dark = self.dark;
        let luminosity_spread = self.luminosity_spread;

        let background = self.background.clone().unwrap_or_else(|| self.default_background());
        let surface = self.surface.clone().unwrap_or_else(|| self.default_surface());
        let foreground = self
            .foreground
            .clone()
            .unwrap_or_else(|| background.inverse());
        let contrast_text = background.contrast_text(self.text_alpha);
        let boost = self
            .boost
            .clone()
            .unwrap_or_else(|| contrast_text.with_alpha(0.04));
        let panel = self.panel.clone().unwrap_or_else(|| {
            let mut panel = surface.blend(&primary, 0.1, Some(1.0));
            if dark {
                panel = panel.tint(&boost);
            }
            panel
        });

        let colors = [
            ("primary", primary.clone()),
            ("secondary", secondary.clone()),
            ("primary-background", primary.clone()),
            ("secondary-background", secondary.clone()),
            ("background", background.clone()),
            ("foreground", foreground.clone()),
            ("panel", panel.clone()),
            ("boost", boost.clone()),
            ("surface", surface.clone()),
            ("warning", warning.clone()),
            ("error", error.clone()),
            ("success", success.clone()),
            ("accent", accent.clone()),
        ];

        let luminosity_step = luminosity_spread / 2.0;
        for (name, color) in colors {
            let is_dark_shade =
                dark && (name == "primary-background" || name == "secondary-background");
            for n in -NUMBER_OF_SHADES..=NUMBER_OF_SHADES {
                let (suffix, delta) = if n < 0 {
                    (format!("-darken-{}", -n), n as f32 * luminosity_step)
                } else if n > 0 {
                    (format!("-lighten-{}", n), n as f32 * luminosity_step)
                } else {
                    ("".to_string(), 0.0)
                };
                let key = format!("{name}{suffix}");
                let shade_color = if is_dark_shade {
                    let dark_background = background.blend(&color, 0.15, Some(1.0));
                    dark_background.blend(
                        &RgbaColor::white(),
                        luminosity_spread + delta,
                        Some(1.0),
                    )
                } else {
                    color.lighten(delta)
                };
                vars.insert(key, shade_color);
            }
        }

        vars.insert("foreground-muted".into(), foreground.with_alpha(0.6));
        vars.insert("foreground-disabled".into(), foreground.with_alpha(0.38));

        if foreground.ansi.is_none() {
            vars.insert("text".into(), RgbaColor::auto(0.87));
            vars.insert("text-muted".into(), RgbaColor::auto(0.60));
            vars.insert("text-disabled".into(), RgbaColor::auto(0.38));
        } else {
            vars.insert("text".into(), foreground.clone());
            vars.insert("text-muted".into(), foreground.clone());
            vars.insert("text-disabled".into(), foreground.clone());
        }

        vars.insert(
            "text-primary".into(),
            contrast_text.tint(&primary.with_alpha(0.66)),
        );
        vars.insert(
            "text-secondary".into(),
            contrast_text.tint(&secondary.with_alpha(0.66)),
        );
        vars.insert(
            "text-warning".into(),
            contrast_text.tint(&warning.with_alpha(0.66)),
        );
        vars.insert(
            "text-error".into(),
            contrast_text.tint(&error.with_alpha(0.66)),
        );
        vars.insert(
            "text-success".into(),
            contrast_text.tint(&success.with_alpha(0.66)),
        );
        vars.insert(
            "text-accent".into(),
            contrast_text.tint(&accent.with_alpha(0.66)),
        );

        vars.insert(
            "primary-muted".into(),
            primary.blend(&background, 0.7, None),
        );
        vars.insert(
            "secondary-muted".into(),
            secondary.blend(&background, 0.7, None),
        );
        vars.insert(
            "accent-muted".into(),
            accent.blend(&background, 0.7, None),
        );
        vars.insert(
            "warning-muted".into(),
            warning.blend(&background, 0.7, None),
        );
        vars.insert(
            "error-muted".into(),
            error.blend(&background, 0.7, None),
        );
        vars.insert(
            "success-muted".into(),
            success.blend(&background, 0.7, None),
        );

        let background_darken_1 = vars
            .get("background-darken-1")
            .cloned()
            .unwrap_or_else(|| background.darken(luminosity_step));
        let primary_40 = primary.with_alpha(0.4);
        let primary_50 = primary.with_alpha(0.5);
        let scrollbar = background_darken_1.blend(&primary_40, primary_40.a, Some(1.0));
        let scrollbar_hover = background_darken_1.blend(&primary_50, primary_50.a, Some(1.0));

        vars.insert("scrollbar".into(), scrollbar);
        vars.insert("scrollbar-hover".into(), scrollbar_hover);
        vars.insert("scrollbar-active".into(), primary.clone());
        vars.insert("scrollbar-background".into(), background_darken_1.clone());
        vars.insert(
            "scrollbar-corner-color".into(),
            background_darken_1.clone(),
        );
        vars.insert(
            "scrollbar-background-hover".into(),
            background_darken_1.clone(),
        );
        vars.insert(
            "scrollbar-background-active".into(),
            background_darken_1.clone(),
        );

        let text_color = vars
            .get("text")
            .cloned()
            .unwrap_or_else(|| foreground.clone());
        vars.insert("link-color".into(), text_color.clone());
        vars.insert("link-background".into(), RgbaColor::transparent());
        vars.insert("link-color-hover".into(), text_color);
        vars.insert("link-background-hover".into(), primary.clone());

        vars.insert("border".into(), primary.clone());
        vars.insert("border-blurred".into(), surface.darken(0.025));

        vars.insert(
            "surface-active".into(),
            surface.lighten(self.luminosity_spread / 2.5),
        );

        vars.insert("block-cursor-foreground".into(), vars["text"].clone());
        vars.insert("block-cursor-background".into(), primary.clone());
        vars.insert("block-cursor-blurred-foreground".into(), foreground.clone());
        vars.insert(
            "block-cursor-blurred-background".into(),
            primary.with_alpha(0.3),
        );
        vars.insert("block-hover-background".into(), boost.with_alpha(0.1));

        vars.insert("footer-foreground".into(), foreground.clone());
        vars.insert("footer-background".into(), panel.clone());
        vars.insert("footer-key-foreground".into(), accent.clone());
        vars.insert("footer-key-background".into(), RgbaColor::transparent());
        vars.insert(
            "footer-description-foreground".into(),
            foreground.clone(),
        );
        vars.insert("footer-description-background".into(), RgbaColor::transparent());
        vars.insert("footer-item-background".into(), RgbaColor::transparent());

        vars.insert("input-cursor-background".into(), foreground.clone());
        vars.insert("input-cursor-foreground".into(), background.clone());
        vars.insert(
            "input-selection-background".into(),
            primary.with_alpha(0.4),
        );

        vars.insert("markdown-h1-color".into(), primary.clone());
        vars.insert(
            "markdown-h1-background".into(),
            RgbaColor::transparent(),
        );
        vars.insert("markdown-h2-color".into(), primary.clone());
        vars.insert(
            "markdown-h2-background".into(),
            RgbaColor::transparent(),
        );
        vars.insert("markdown-h3-color".into(), primary.clone());
        vars.insert(
            "markdown-h3-background".into(),
            RgbaColor::transparent(),
        );
        vars.insert("markdown-h4-color".into(), foreground.clone());
        vars.insert(
            "markdown-h4-background".into(),
            RgbaColor::transparent(),
        );
        vars.insert("markdown-h5-color".into(), foreground.clone());
        vars.insert(
            "markdown-h5-background".into(),
            RgbaColor::transparent(),
        );
        vars.insert(
            "markdown-h6-color".into(),
            vars["foreground-muted"].clone(),
        );
        vars.insert(
            "markdown-h6-background".into(),
            RgbaColor::transparent(),
        );

        vars.insert("button-foreground".into(), foreground.clone());
        vars.insert("button-color-foreground".into(), vars["text"].clone());

        vars
    }

    /// Generates all TextStyle variables from this color system.
    pub fn generate_styles(&self) -> HashMap<String, TextStyle> {
        let mut styles = HashMap::new();

        // Link styles
        let mut link_style = TextStyle::default();
        link_style.underline = true;
        styles.insert("link-style".into(), link_style);

        let mut link_style_hover = TextStyle::default();
        link_style_hover.bold = true;
        // No underline on hover (Python: "bold not underline")
        styles.insert("link-style-hover".into(), link_style_hover);

        // Block cursor text styles
        let mut block_cursor = TextStyle::default();
        block_cursor.bold = true;
        styles.insert("block-cursor-text-style".into(), block_cursor);
        styles.insert("block-cursor-blurred-text-style".into(), TextStyle::default());

        // Input cursor text style
        styles.insert("input-cursor-text-style".into(), TextStyle::default());

        // Markdown header text styles
        let mut h1_style = TextStyle::default();
        h1_style.bold = true;
        styles.insert("markdown-h1-text-style".into(), h1_style);

        let mut h2_style = TextStyle::default();
        h2_style.underline = true;
        styles.insert("markdown-h2-text-style".into(), h2_style);

        let mut h3_style = TextStyle::default();
        h3_style.bold = true;
        styles.insert("markdown-h3-text-style".into(), h3_style);

        let mut h4_style = TextStyle::default();
        h4_style.bold = true;
        h4_style.underline = true;
        styles.insert("markdown-h4-text-style".into(), h4_style);

        let mut h5_style = TextStyle::default();
        h5_style.bold = true;
        styles.insert("markdown-h5-text-style".into(), h5_style);

        let mut h6_style = TextStyle::default();
        h6_style.bold = true;
        styles.insert("markdown-h6-text-style".into(), h6_style);

        // Button focus text style
        let mut button_focus = TextStyle::default();
        button_focus.bold = true;
        button_focus.reverse = true;
        styles.insert("button-focus-text-style".into(), button_focus);

        styles
    }

}

/// A named color theme for styling widgets.
///
/// Themes map variable names to colors and text styles, allowing stylesheets
/// to reference semantic names that can be swapped at runtime.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Unique name for this theme (e.g., "textual-dark").
    pub name: String,
    /// Whether this is a dark theme (affects auto-color contrast).
    pub is_dark: bool,
    /// Maps variable names (e.g., "primary", "surface") to colors.
    pub colors: HashMap<String, RgbaColor>,
    /// Maps variable names (e.g., "link-style") to text styles.
    pub styles: HashMap<String, TextStyle>,
}

impl Theme {
    /// Creates a new empty theme with the given name.
    pub fn new(name: &str, is_dark: bool) -> Self {
        Self {
            name: name.to_string(),
            is_dark,
            colors: HashMap::new(),
            styles: HashMap::new(),
        }
    }

    /// Creates a theme from a ColorSystem.
    pub fn from_color_system(name: &str, system: ColorSystem) -> Self {
        Self {
            name: name.to_string(),
            is_dark: system.dark,
            colors: system.generate(),
            styles: system.generate_styles(),
        }
    }

    /// Looks up a color by variable name.
    pub fn get_color(&self, name: &str) -> Option<RgbaColor> {
        self.colors.get(name).cloned()
    }

    /// Looks up a text style by variable name.
    pub fn get_style(&self, name: &str) -> Option<TextStyle> {
        self.styles.get(name).cloned()
    }

    /// Returns all 16 built-in Textual themes.
    pub fn standard_themes() -> HashMap<String, Theme> {
        let mut themes = HashMap::new();

        // textual-dark (default dark theme)
        // Uses blue-tinted grays matching Python Textual's color palette
        // background defaults to #121212 from ColorSystem::default_background()
        themes.insert("textual-dark".into(), Theme::from_color_system(
            "textual-dark",
            ColorSystem::new(RgbaColor::hex("#0178D4"), true)
                .with_secondary(RgbaColor::hex("#004578"))
                .with_warning(RgbaColor::hex("#ffa62b"))
                .with_error(RgbaColor::hex("#ba3c5b"))
                .with_success(RgbaColor::hex("#4EBF71"))
                .with_accent(RgbaColor::hex("#ffa62b"))
                .with_foreground(RgbaColor::hex("#e0e0e0"))
        ));

        // textual-light (default light theme)
        themes.insert("textual-light".into(), Theme::from_color_system(
            "textual-light",
            ColorSystem::new(RgbaColor::hex("#004578"), false)
                .with_secondary(RgbaColor::hex("#0178D4"))
                .with_warning(RgbaColor::hex("#ffa62b"))
                .with_error(RgbaColor::hex("#ba3c5b"))
                .with_success(RgbaColor::hex("#4EBF71"))
                .with_accent(RgbaColor::hex("#ffa62b"))
                .with_background(RgbaColor::hex("#E0E0E0"))
                .with_surface(RgbaColor::hex("#D8D8D8"))
                .with_panel(RgbaColor::hex("#D0D0D0"))
        ));

        // nord
        themes.insert("nord".into(), Theme::from_color_system(
            "nord",
            ColorSystem::new(RgbaColor::hex("#88C0D0"), true)
                .with_secondary(RgbaColor::hex("#81A1C1"))
                .with_warning(RgbaColor::hex("#EBCB8B"))
                .with_error(RgbaColor::hex("#BF616A"))
                .with_success(RgbaColor::hex("#A3BE8C"))
                .with_accent(RgbaColor::hex("#B48EAD"))
                .with_foreground(RgbaColor::hex("#D8DEE9"))
                .with_background(RgbaColor::hex("#2E3440"))
                .with_surface(RgbaColor::hex("#3B4252"))
                .with_panel(RgbaColor::hex("#434C5E"))
        ));

        // gruvbox
        themes.insert("gruvbox".into(), Theme::from_color_system(
            "gruvbox",
            ColorSystem::new(RgbaColor::hex("#85A598"), true)
                .with_secondary(RgbaColor::hex("#A89A85"))
                .with_warning(RgbaColor::hex("#fe8019"))
                .with_error(RgbaColor::hex("#fb4934"))
                .with_success(RgbaColor::hex("#b8bb26"))
                .with_accent(RgbaColor::hex("#fabd2f"))
                .with_foreground(RgbaColor::hex("#fbf1c7"))
                .with_background(RgbaColor::hex("#282828"))
                .with_surface(RgbaColor::hex("#3c3836"))
                .with_panel(RgbaColor::hex("#504945"))
        ));

        // catppuccin-mocha
        themes.insert("catppuccin-mocha".into(), Theme::from_color_system(
            "catppuccin-mocha",
            ColorSystem::new(RgbaColor::hex("#F5C2E7"), true)
                .with_secondary(RgbaColor::hex("#cba6f7"))
                .with_warning(RgbaColor::hex("#FAE3B0"))
                .with_error(RgbaColor::hex("#F28FAD"))
                .with_success(RgbaColor::hex("#ABE9B3"))
                .with_accent(RgbaColor::hex("#fab387"))
                .with_foreground(RgbaColor::hex("#cdd6f4"))
                .with_background(RgbaColor::hex("#181825"))
                .with_surface(RgbaColor::hex("#313244"))
                .with_panel(RgbaColor::hex("#45475a"))
        ));

        // catppuccin-latte
        themes.insert("catppuccin-latte".into(), Theme::from_color_system(
            "catppuccin-latte",
            ColorSystem::new(RgbaColor::hex("#8839EF"), false)
                .with_secondary(RgbaColor::hex("#DC8A78"))
                .with_warning(RgbaColor::hex("#DF8E1D"))
                .with_error(RgbaColor::hex("#D20F39"))
                .with_success(RgbaColor::hex("#40A02B"))
                .with_accent(RgbaColor::hex("#FE640B"))
                .with_foreground(RgbaColor::hex("#4C4F69"))
                .with_background(RgbaColor::hex("#EFF1F5"))
                .with_surface(RgbaColor::hex("#E6E9EF"))
                .with_panel(RgbaColor::hex("#CCD0DA"))
        ));

        // dracula
        themes.insert("dracula".into(), Theme::from_color_system(
            "dracula",
            ColorSystem::new(RgbaColor::hex("#BD93F9"), true)
                .with_secondary(RgbaColor::hex("#6272A4"))
                .with_warning(RgbaColor::hex("#FFB86C"))
                .with_error(RgbaColor::hex("#FF5555"))
                .with_success(RgbaColor::hex("#50FA7B"))
                .with_accent(RgbaColor::hex("#FF79C6"))
                .with_foreground(RgbaColor::hex("#F8F8F2"))
                .with_background(RgbaColor::hex("#282A36"))
                .with_surface(RgbaColor::hex("#2B2E3B"))
                .with_panel(RgbaColor::hex("#313442"))
        ));

        // tokyo-night
        themes.insert("tokyo-night".into(), Theme::from_color_system(
            "tokyo-night",
            ColorSystem::new(RgbaColor::hex("#BB9AF7"), true)
                .with_secondary(RgbaColor::hex("#7AA2F7"))
                .with_warning(RgbaColor::hex("#E0AF68"))
                .with_error(RgbaColor::hex("#F7768E"))
                .with_success(RgbaColor::hex("#9ECE6A"))
                .with_accent(RgbaColor::hex("#FF9E64"))
                .with_foreground(RgbaColor::hex("#a9b1d6"))
                .with_background(RgbaColor::hex("#1A1B26"))
                .with_surface(RgbaColor::hex("#24283B"))
                .with_panel(RgbaColor::hex("#414868"))
        ));

        // monokai
        themes.insert("monokai".into(), Theme::from_color_system(
            "monokai",
            ColorSystem::new(RgbaColor::hex("#AE81FF"), true)
                .with_secondary(RgbaColor::hex("#F92672"))
                .with_warning(RgbaColor::hex("#FD971F"))
                .with_error(RgbaColor::hex("#F92672"))
                .with_success(RgbaColor::hex("#A6E22E"))
                .with_accent(RgbaColor::hex("#66D9EF"))
                .with_foreground(RgbaColor::hex("#d6d6d6"))
                .with_background(RgbaColor::hex("#272822"))
                .with_surface(RgbaColor::hex("#2e2e2e"))
                .with_panel(RgbaColor::hex("#3E3D32"))
        ));

        // flexoki
        themes.insert("flexoki".into(), Theme::from_color_system(
            "flexoki",
            ColorSystem::new(RgbaColor::hex("#205EA6"), true)
                .with_secondary(RgbaColor::hex("#24837B"))
                .with_warning(RgbaColor::hex("#AD8301"))
                .with_error(RgbaColor::hex("#AF3029"))
                .with_success(RgbaColor::hex("#66800B"))
                .with_accent(RgbaColor::hex("#9B76C8"))
                .with_foreground(RgbaColor::hex("#FFFCF0"))
                .with_background(RgbaColor::hex("#100F0F"))
                .with_surface(RgbaColor::hex("#1C1B1A"))
                .with_panel(RgbaColor::hex("#282726"))
        ));

        // solarized-light
        themes.insert("solarized-light".into(), Theme::from_color_system(
            "solarized-light",
            ColorSystem::new(RgbaColor::hex("#268bd2"), false)
                .with_secondary(RgbaColor::hex("#2aa198"))
                .with_warning(RgbaColor::hex("#cb4b16"))
                .with_error(RgbaColor::hex("#dc322f"))
                .with_success(RgbaColor::hex("#859900"))
                .with_accent(RgbaColor::hex("#6c71c4"))
                .with_foreground(RgbaColor::hex("#586e75"))
                .with_background(RgbaColor::hex("#fdf6e3"))
                .with_surface(RgbaColor::hex("#eee8d5"))
                .with_panel(RgbaColor::hex("#eee8d5"))
        ));

        // solarized-dark
        themes.insert("solarized-dark".into(), Theme::from_color_system(
            "solarized-dark",
            ColorSystem::new(RgbaColor::hex("#268bd2"), true)
                .with_secondary(RgbaColor::hex("#2aa198"))
                .with_warning(RgbaColor::hex("#cb4b16"))
                .with_error(RgbaColor::hex("#dc322f"))
                .with_success(RgbaColor::hex("#859900"))
                .with_accent(RgbaColor::hex("#6c71c4"))
                .with_foreground(RgbaColor::hex("#839496"))
                .with_background(RgbaColor::hex("#002b36"))
                .with_surface(RgbaColor::hex("#073642"))
                .with_panel(RgbaColor::hex("#073642"))
        ));

        // rose-pine
        themes.insert("rose-pine".into(), Theme::from_color_system(
            "rose-pine",
            ColorSystem::new(RgbaColor::hex("#c4a7e7"), true)
                .with_secondary(RgbaColor::hex("#31748f"))
                .with_warning(RgbaColor::hex("#f6c177"))
                .with_error(RgbaColor::hex("#eb6f92"))
                .with_success(RgbaColor::hex("#9ccfd8"))
                .with_accent(RgbaColor::hex("#ebbcba"))
                .with_foreground(RgbaColor::hex("#e0def4"))
                .with_background(RgbaColor::hex("#191724"))
                .with_surface(RgbaColor::hex("#1f1d2e"))
                .with_panel(RgbaColor::hex("#26233a"))
        ));

        // rose-pine-moon
        themes.insert("rose-pine-moon".into(), Theme::from_color_system(
            "rose-pine-moon",
            ColorSystem::new(RgbaColor::hex("#c4a7e7"), true)
                .with_secondary(RgbaColor::hex("#3e8fb0"))
                .with_warning(RgbaColor::hex("#f6c177"))
                .with_error(RgbaColor::hex("#eb6f92"))
                .with_success(RgbaColor::hex("#9ccfd8"))
                .with_accent(RgbaColor::hex("#ea9a97"))
                .with_foreground(RgbaColor::hex("#e0def4"))
                .with_background(RgbaColor::hex("#232136"))
                .with_surface(RgbaColor::hex("#2a273f"))
                .with_panel(RgbaColor::hex("#393552"))
        ));

        // rose-pine-dawn
        themes.insert("rose-pine-dawn".into(), Theme::from_color_system(
            "rose-pine-dawn",
            ColorSystem::new(RgbaColor::hex("#907aa9"), false)
                .with_secondary(RgbaColor::hex("#286983"))
                .with_warning(RgbaColor::hex("#ea9d34"))
                .with_error(RgbaColor::hex("#b4637a"))
                .with_success(RgbaColor::hex("#56949f"))
                .with_accent(RgbaColor::hex("#d7827e"))
                .with_foreground(RgbaColor::hex("#575279"))
                .with_background(RgbaColor::hex("#faf4ed"))
                .with_surface(RgbaColor::hex("#fffaf3"))
                .with_panel(RgbaColor::hex("#f2e9e1"))
        ));

        // textual-ansi (uses ANSI terminal colors - placeholder with approximations)
        themes.insert("textual-ansi".into(), Theme::from_color_system(
            "textual-ansi",
            ColorSystem::new(RgbaColor::hex("#0000ff"), true) // ansi_blue
                .with_secondary(RgbaColor::hex("#00ffff")) // ansi_cyan
                .with_warning(RgbaColor::hex("#ffff00")) // ansi_yellow
                .with_error(RgbaColor::hex("#ff0000")) // ansi_red
                .with_success(RgbaColor::hex("#00ff00")) // ansi_green
                .with_accent(RgbaColor::hex("#5555ff")) // ansi_bright_blue
        ));

        themes
    }
}
