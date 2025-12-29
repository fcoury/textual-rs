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

    /// Returns the default foreground for dark/light themes.
    fn default_foreground(&self) -> RgbaColor {
        if self.dark {
            RgbaColor::hex("#e0e0e0")
        } else {
            RgbaColor::hex("#1e1e1e")
        }
    }

    /// Generates all CSS variables from this color system.
    pub fn generate(&self) -> HashMap<String, RgbaColor> {
        let mut vars = HashMap::new();

        // Resolve base colors with defaults
        let background = self.background.clone().unwrap_or_else(|| self.default_background());
        let surface = self.surface.clone().unwrap_or_else(|| self.default_surface());
        let foreground = self.foreground.clone().unwrap_or_else(|| self.default_foreground());
        let panel = self.panel.clone().unwrap_or_else(|| {
            if self.dark {
                surface.lighten(0.04)
            } else {
                surface.darken(0.04)
            }
        });
        // Boost is a semi-transparent overlay that gets pre-composited against background.
        // This produces the actual color to use (e.g., #1B1B1B for dark themes).
        let boost_overlay = self.boost.clone().unwrap_or_else(|| {
            if self.dark {
                RgbaColor::rgba(255, 255, 255, 0.04)
            } else {
                RgbaColor::rgba(0, 0, 0, 0.04)
            }
        });
        let boost = background.tint(&boost_overlay);

        // Secondary defaults to primary shifted
        let secondary = self.secondary.clone().unwrap_or_else(|| {
            if self.dark {
                self.primary.darken(0.15)
            } else {
                self.primary.lighten(0.15)
            }
        });

        // Semantic colors with defaults
        let warning = self.warning.clone().unwrap_or_else(|| RgbaColor::hex("#ffa62b"));
        let error = self.error.clone().unwrap_or_else(|| RgbaColor::hex("#ba3c5b"));
        let success = self.success.clone().unwrap_or_else(|| RgbaColor::hex("#4EBF71"));
        let accent = self.accent.clone().unwrap_or_else(|| warning.clone());

        // Insert base colors with darken/lighten variants
        self.insert_color_variants(&mut vars, "primary", &self.primary);
        self.insert_color_variants(&mut vars, "secondary", &secondary);
        self.insert_color_variants(&mut vars, "background", &background);
        self.insert_color_variants(&mut vars, "surface", &surface);
        self.insert_color_variants(&mut vars, "panel", &panel);
        self.insert_color_variants(&mut vars, "boost", &boost);
        self.insert_color_variants(&mut vars, "warning", &warning);
        self.insert_color_variants(&mut vars, "error", &error);
        self.insert_color_variants(&mut vars, "success", &success);
        self.insert_color_variants(&mut vars, "accent", &accent);

        // Foreground (no darken/lighten - just base)
        vars.insert("foreground".into(), foreground.clone());
        vars.insert("foreground-muted".into(), foreground.with_alpha(0.6));
        vars.insert("foreground-disabled".into(), foreground.with_alpha(0.3));

        // Primary/secondary background variants
        vars.insert("primary-background".into(), self.primary.with_alpha(0.2));
        self.insert_color_variants(&mut vars, "primary-background", &self.primary.with_alpha(0.2));
        vars.insert("secondary-background".into(), secondary.with_alpha(0.2));
        self.insert_color_variants(&mut vars, "secondary-background", &secondary.with_alpha(0.2));

        // Text colors (high contrast on backgrounds)
        let text_on_dark = RgbaColor::rgba(255, 255, 255, 0.9);
        let text_on_light = RgbaColor::rgba(0, 0, 0, 0.9);
        let text = if self.dark { text_on_dark.clone() } else { text_on_light.clone() };

        vars.insert("text".into(), text.clone());
        vars.insert("text-muted".into(), text.with_alpha(0.7));
        vars.insert("text-disabled".into(), text.with_alpha(0.4));

        // Text on semantic colors (contrast-aware)
        vars.insert("text-primary".into(), self.contrast_text(&self.primary));
        vars.insert("text-secondary".into(), self.contrast_text(&secondary));
        vars.insert("text-warning".into(), self.contrast_text(&warning));
        vars.insert("text-error".into(), self.contrast_text(&error));
        vars.insert("text-success".into(), self.contrast_text(&success));
        vars.insert("text-accent".into(), self.contrast_text(&accent));

        // Muted variants (for badges, pills, etc.)
        vars.insert("primary-muted".into(), self.primary.with_alpha(0.3));
        vars.insert("secondary-muted".into(), secondary.with_alpha(0.3));
        vars.insert("accent-muted".into(), accent.with_alpha(0.3));
        vars.insert("warning-muted".into(), warning.with_alpha(0.3));
        vars.insert("error-muted".into(), error.with_alpha(0.3));
        vars.insert("success-muted".into(), success.with_alpha(0.3));

        // Scrollbar colors
        let scrollbar_color = if self.dark {
            RgbaColor::hex("#666666")
        } else {
            RgbaColor::hex("#999999")
        };
        vars.insert("scrollbar".into(), scrollbar_color.clone());
        vars.insert("scrollbar-hover".into(), scrollbar_color.lighten(0.1));
        vars.insert("scrollbar-active".into(), scrollbar_color.lighten(0.2));
        vars.insert("scrollbar-background".into(), background.clone());
        vars.insert("scrollbar-background-hover".into(), if self.dark {
            background.lighten(0.05)
        } else {
            background.darken(0.05)
        });
        vars.insert("scrollbar-background-active".into(), if self.dark {
            background.lighten(0.08)
        } else {
            background.darken(0.08)
        });
        vars.insert("scrollbar-corner-color".into(), background.clone());

        // Link colors
        vars.insert("link-color".into(), self.primary.clone());
        vars.insert("link-background".into(), RgbaColor::transparent());
        vars.insert("link-color-hover".into(), self.primary.lighten(0.15));
        vars.insert("link-background-hover".into(), self.primary.with_alpha(0.15));

        // Border colors
        vars.insert("border".into(), if self.dark {
            foreground.with_alpha(0.2)
        } else {
            foreground.with_alpha(0.15)
        });
        vars.insert("border-blurred".into(), if self.dark {
            foreground.with_alpha(0.1)
        } else {
            foreground.with_alpha(0.08)
        });

        // Surface active (for selected items)
        vars.insert("surface-active".into(), if self.dark {
            surface.lighten(0.08)
        } else {
            surface.darken(0.08)
        });

        // Block cursor (for terminal-style cursors)
        vars.insert("block-cursor-foreground".into(), background.clone());
        vars.insert("block-cursor-background".into(), foreground.clone());
        vars.insert("block-cursor-blurred-foreground".into(), foreground.clone());
        vars.insert("block-cursor-blurred-background".into(), if self.dark {
            surface.lighten(0.2)
        } else {
            surface.darken(0.2)
        });
        vars.insert("block-hover-background".into(), if self.dark {
            boost.lighten(0.05)
        } else {
            boost.darken(0.05)
        });

        // Input cursor
        vars.insert("input-cursor-foreground".into(), background.clone());
        vars.insert("input-cursor-background".into(), foreground.clone());
        vars.insert("input-selection-background".into(), self.primary.with_alpha(0.4));

        // Footer
        vars.insert("footer-foreground".into(), foreground.clone());
        vars.insert("footer-background".into(), panel.clone());
        vars.insert("footer-key-foreground".into(), background.clone());
        vars.insert("footer-key-background".into(), self.primary.clone());
        vars.insert("footer-description-foreground".into(), foreground.with_alpha(0.7));
        vars.insert("footer-description-background".into(), panel.clone());
        vars.insert("footer-item-background".into(), panel.clone());

        // Button
        vars.insert("button-foreground".into(), foreground.clone());
        vars.insert("button-color-foreground".into(), RgbaColor::white());

        // Markdown header colors
        let header_colors = [
            ("#e0e0e0", "#1e1e1e"), // h1
            ("#c0c0c0", "#2e2e2e"), // h2
            ("#a0a0a0", "#3e3e3e"), // h3
            ("#909090", "#4e4e4e"), // h4
            ("#808080", "#5e5e5e"), // h5
            ("#707070", "#6e6e6e"), // h6
        ];
        for (i, (dark_fg, light_fg)) in header_colors.iter().enumerate() {
            let level = i + 1;
            let header_color = if self.dark {
                RgbaColor::hex(dark_fg)
            } else {
                RgbaColor::hex(light_fg)
            };
            vars.insert(format!("markdown-h{}-color", level), header_color);
            vars.insert(format!("markdown-h{}-background", level), RgbaColor::transparent());
        }

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
        link_style_hover.underline = true;
        styles.insert("link-style-hover".into(), link_style_hover);

        // Block cursor text styles
        styles.insert("block-cursor-text-style".into(), TextStyle::default());
        styles.insert("block-cursor-blurred-text-style".into(), TextStyle::default());

        // Input cursor text style
        styles.insert("input-cursor-text-style".into(), TextStyle::default());

        // Markdown header text styles
        let mut h1_style = TextStyle::default();
        h1_style.bold = true;
        styles.insert("markdown-h1-text-style".into(), h1_style);

        let mut h2_style = TextStyle::default();
        h2_style.bold = true;
        styles.insert("markdown-h2-text-style".into(), h2_style);

        let mut h3_style = TextStyle::default();
        h3_style.bold = true;
        styles.insert("markdown-h3-text-style".into(), h3_style);

        let mut h4_style = TextStyle::default();
        h4_style.bold = true;
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
        styles.insert("button-focus-text-style".into(), button_focus);

        styles
    }

    /// Inserts a color and its darken/lighten variants.
    fn insert_color_variants(&self, vars: &mut HashMap<String, RgbaColor>, name: &str, color: &RgbaColor) {
        vars.insert(name.into(), color.clone());

        // Darken variants (10%, 20%, 30%)
        vars.insert(format!("{}-darken-1", name), color.darken(0.10));
        vars.insert(format!("{}-darken-2", name), color.darken(0.20));
        vars.insert(format!("{}-darken-3", name), color.darken(0.30));

        // Lighten variants (10%, 20%, 30%)
        vars.insert(format!("{}-lighten-1", name), color.lighten(0.10));
        vars.insert(format!("{}-lighten-2", name), color.lighten(0.20));
        vars.insert(format!("{}-lighten-3", name), color.lighten(0.30));
    }

    /// Returns black or white text depending on which contrasts better.
    fn contrast_text(&self, bg: &RgbaColor) -> RgbaColor {
        if bg.luminance() > 0.5 {
            RgbaColor::rgba(0, 0, 0, 0.9)
        } else {
            RgbaColor::rgba(255, 255, 255, 0.9)
        }
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
        themes.insert("textual-dark".into(), Theme::from_color_system(
            "textual-dark",
            ColorSystem::new(RgbaColor::hex("#0178D4"), true)
                .with_secondary(RgbaColor::hex("#004578"))
                .with_warning(RgbaColor::hex("#ffa62b"))
                .with_error(RgbaColor::hex("#ba3c5b"))
                .with_success(RgbaColor::hex("#4EBF71"))
                .with_accent(RgbaColor::hex("#ffa62b"))
                .with_foreground(RgbaColor::hex("#e0e0e0"))
                .with_background(RgbaColor::hex("#1e1e1e"))
                .with_surface(RgbaColor::hex("#1e272e"))
                .with_panel(RgbaColor::hex("#212f39"))
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
