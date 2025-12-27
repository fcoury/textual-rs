//! Theme definitions for TCSS color palettes.
//!
//! Themes provide named color variables that can be referenced in stylesheets
//! using the `$variable` syntax. This enables consistent theming and easy
//! switching between light/dark modes.
//!
//! ## Standard Variables
//!
//! Textual themes typically define these color variables:
//!
//! | Variable    | Description                              |
//! |-------------|------------------------------------------|
//! | `$primary`  | Primary accent color                     |
//! | `$secondary`| Secondary accent color                   |
//! | `$surface`  | Default widget background                |
//! | `$panel`    | Panel/container background               |
//! | `$text`     | Default text color                       |
//!
//! ## Color Modifiers
//!
//! Theme colors support HSL-based modifiers:
//!
//! - `$primary-lighten-1` - Lighten by 10%
//! - `$primary-darken-2` - Darken by 20%
//!
//! ## CSS Syntax
//!
//! ```css
//! Button {
//!     background: $primary;
//!     color: $text;
//! }
//! Button:hover {
//!     background: $primary-lighten-1;
//! }
//! ```

use crate::types::color::RgbaColor;
use std::collections::HashMap;

/// A named color theme for styling widgets.
///
/// Themes map variable names to colors, allowing stylesheets to reference
/// semantic color names that can be swapped at runtime.
///
/// # Examples
///
/// ```
/// use tcss::types::{Theme, RgbaColor};
///
/// let mut theme = Theme::new("my-theme", true);
/// theme.colors.insert("primary".into(), RgbaColor::rgb(0, 120, 215));
/// theme.colors.insert("surface".into(), RgbaColor::rgb(30, 30, 30));
///
/// // Resolve a theme variable
/// let primary = theme.get_color("primary").unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Theme {
    /// Unique name for this theme (e.g., "textual-dark").
    pub name: String,
    /// Whether this is a dark theme (affects auto-color contrast).
    pub is_dark: bool,
    /// Maps variable names (e.g., "primary", "surface") to colors.
    pub colors: HashMap<String, RgbaColor>,
}

impl Theme {
    /// Creates a new empty theme with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for this theme
    /// * `is_dark` - Whether this is a dark theme (affects contrast calculations)
    pub fn new(name: &str, is_dark: bool) -> Self {
        Self {
            name: name.to_string(),
            is_dark,
            colors: HashMap::new(),
        }
    }

    /// Looks up a color by variable name.
    ///
    /// Returns `None` if the variable is not defined in this theme.
    pub fn get_color(&self, name: &str) -> Option<RgbaColor> {
        self.colors.get(name).cloned()
    }

    /// Returns the built-in Textual themes.
    ///
    /// Includes `textual-dark` and `textual-light` with standard
    /// color definitions for primary, secondary, surface, panel, and text.
    pub fn standard_themes() -> HashMap<String, Theme> {
        let mut themes = HashMap::new();

        // --- Dark Theme ---
        let mut dark = Theme::new("textual-dark", true);
        dark.colors
            .insert("primary".into(), RgbaColor::rgb(0, 170, 255));
        dark.colors
            .insert("secondary".into(), RgbaColor::rgb(255, 0, 255));
        dark.colors
            .insert("surface".into(), RgbaColor::rgb(36, 36, 36));
        dark.colors
            .insert("panel".into(), RgbaColor::rgb(46, 46, 46));
        dark.colors.insert("text".into(), RgbaColor::white());
        themes.insert(dark.name.clone(), dark);

        // --- Light Theme ---
        let mut light = Theme::new("textual-light", false);
        light
            .colors
            .insert("primary".into(), RgbaColor::rgb(0, 100, 200));
        light
            .colors
            .insert("surface".into(), RgbaColor::rgb(240, 240, 240));
        light.colors.insert("text".into(), RgbaColor::black());
        themes.insert(light.name.clone(), light);

        themes
    }
}
