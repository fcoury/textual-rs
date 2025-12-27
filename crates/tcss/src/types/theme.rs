use crate::types::color::RgbaColor;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    /// Maps variable names (e.g., "primary", "surface") to actual colors
    pub colors: HashMap<String, RgbaColor>,
}

impl Theme {
    pub fn new(name: &str, is_dark: bool) -> Self {
        Self {
            name: name.to_string(),
            is_dark,
            colors: HashMap::new(),
        }
    }

    pub fn get_color(&self, name: &str) -> Option<RgbaColor> {
        self.colors.get(name).cloned()
    }

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
