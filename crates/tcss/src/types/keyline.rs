//! Keyline CSS types for drawing box borders around widgets.

use super::RgbaColor;

/// Line style for keylines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeylineStyle {
    /// No keyline (default).
    #[default]
    None,
    /// Thin line style (─ │ ┌ ┐ └ ┘).
    Thin,
    /// Heavy/thick line style (━ ┃ ┏ ┓ ┗ ┛).
    Heavy,
    /// Double line style (═ ║ ╔ ╗ ╚ ╝).
    Double,
}

impl KeylineStyle {
    /// Parse a keyline style from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "thin" => Some(Self::Thin),
            "heavy" => Some(Self::Heavy),
            "double" => Some(Self::Double),
            _ => None,
        }
    }

    /// Get the line type index for box drawing (0=none, 1=thin, 2=heavy, 3=double).
    pub fn line_type(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Thin => 1,
            Self::Heavy => 2,
            Self::Double => 3,
        }
    }
}

impl std::fmt::Display for KeylineStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Thin => write!(f, "thin"),
            Self::Heavy => write!(f, "heavy"),
            Self::Double => write!(f, "double"),
        }
    }
}

/// Keyline configuration combining style and color.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Keyline {
    /// The line style.
    pub style: KeylineStyle,
    /// The line color.
    pub color: RgbaColor,
}

impl Keyline {
    /// Create a new keyline with the given style and color.
    pub fn new(style: KeylineStyle, color: RgbaColor) -> Self {
        Self { style, color }
    }

    /// Check if keyline is enabled (style is not None).
    pub fn is_enabled(&self) -> bool {
        self.style != KeylineStyle::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyline_style_parse() {
        assert_eq!(KeylineStyle::parse("none"), Some(KeylineStyle::None));
        assert_eq!(KeylineStyle::parse("thin"), Some(KeylineStyle::Thin));
        assert_eq!(KeylineStyle::parse("heavy"), Some(KeylineStyle::Heavy));
        assert_eq!(KeylineStyle::parse("double"), Some(KeylineStyle::Double));
        assert_eq!(KeylineStyle::parse("HEAVY"), Some(KeylineStyle::Heavy));
        assert_eq!(KeylineStyle::parse("invalid"), None);
    }

    #[test]
    fn test_keyline_style_line_type() {
        assert_eq!(KeylineStyle::None.line_type(), 0);
        assert_eq!(KeylineStyle::Thin.line_type(), 1);
        assert_eq!(KeylineStyle::Heavy.line_type(), 2);
        assert_eq!(KeylineStyle::Double.line_type(), 3);
    }

    #[test]
    fn test_keyline_is_enabled() {
        let none = Keyline::new(KeylineStyle::None, RgbaColor::default());
        assert!(!none.is_enabled());

        let thin = Keyline::new(KeylineStyle::Thin, RgbaColor::rgb(0, 255, 0));
        assert!(thin.is_enabled());
    }

    #[test]
    fn test_keyline_default() {
        let keyline = Keyline::default();
        assert_eq!(keyline.style, KeylineStyle::None);
        assert!(!keyline.is_enabled());
    }
}
