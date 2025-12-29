//! Style types for Rich markup.
//!
//! A Style combines colors and text modifiers into a single specification.

use crate::color::Color;
use crate::error::StyleParseError;

/// Text styling attributes (modifiers).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextStyle {
    /// Bold/increased intensity.
    pub bold: bool,
    /// Dim/decreased intensity.
    pub dim: bool,
    /// Italic text.
    pub italic: bool,
    /// Underlined text.
    pub underline: bool,
    /// Strikethrough text.
    pub strike: bool,
    /// Reverse video (swap fg/bg).
    pub reverse: bool,
    /// Blinking text.
    pub blink: bool,
}

impl TextStyle {
    /// Returns true if no modifiers are set.
    pub fn is_empty(&self) -> bool {
        !self.bold
            && !self.dim
            && !self.italic
            && !self.underline
            && !self.strike
            && !self.reverse
            && !self.blink
    }

    /// Merge another TextStyle on top of this one (OR'd together).
    pub fn apply(&self, other: &TextStyle) -> TextStyle {
        TextStyle {
            bold: self.bold || other.bold,
            dim: self.dim || other.dim,
            italic: self.italic || other.italic,
            underline: self.underline || other.underline,
            strike: self.strike || other.strike,
            reverse: self.reverse || other.reverse,
            blink: self.blink || other.blink,
        }
    }
}

/// Complete style specification including colors and modifiers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    /// Foreground (text) color.
    pub fg: Option<Color>,
    /// Background color.
    pub bg: Option<Color>,
    /// Text style modifiers.
    pub text: TextStyle,
}

impl Style {
    /// Create a new empty style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if no style properties are set.
    pub fn is_empty(&self) -> bool {
        self.fg.is_none() && self.bg.is_none() && self.text.is_empty()
    }

    /// Apply another style on top of this one.
    ///
    /// Non-None values in `other` override values in `self`.
    /// Boolean modifiers are OR'd together.
    pub fn apply(&self, other: &Style) -> Style {
        Style {
            fg: other.fg.clone().or_else(|| self.fg.clone()),
            bg: other.bg.clone().or_else(|| self.bg.clone()),
            text: self.text.apply(&other.text),
        }
    }

    /// Parse a style from a string like "bold red on blue".
    ///
    /// Supported syntax:
    /// - Modifiers: `bold`, `b`, `italic`, `i`, `underline`, `u`, `strike`, `s`,
    ///   `dim`, `d`, `reverse`, `r`, `blink`
    /// - Foreground color: `red`, `#ff5733`, `rgb(255,87,51)`
    /// - Background color: `on red`, `on #ff5733`
    ///
    /// # Examples
    ///
    /// ```
    /// use rich::Style;
    ///
    /// let style = Style::parse("bold red").unwrap();
    /// assert!(style.text.bold);
    /// assert!(style.fg.is_some());
    ///
    /// let style2 = Style::parse("white on blue").unwrap();
    /// assert!(style2.fg.is_some());
    /// assert!(style2.bg.is_some());
    /// ```
    pub fn parse(input: &str) -> Result<Self, StyleParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(StyleParseError::Empty);
        }

        let mut style = Style::new();
        let mut words = input.split_whitespace().peekable();

        while let Some(word) = words.next() {
            let word_lower = word.to_lowercase();

            // Check for "on" prefix for background color
            if word_lower == "on" {
                if let Some(color_word) = words.next() {
                    let color = Color::parse(color_word)?;
                    style.bg = Some(color);
                    continue;
                } else {
                    return Err(StyleParseError::UnknownModifier("on".to_string()));
                }
            }

            // Check for modifiers
            if let Some(text_style) = Self::parse_modifier(&word_lower) {
                style.text = style.text.apply(&text_style);
                continue;
            }

            // Try to parse as a color (foreground)
            if let Ok(color) = Color::parse(&word_lower) {
                style.fg = Some(color);
                continue;
            }

            // Also try the original case for hex colors
            if let Ok(color) = Color::parse(word) {
                style.fg = Some(color);
                continue;
            }

            return Err(StyleParseError::UnknownModifier(word.to_string()));
        }

        Ok(style)
    }

    /// Parse a style modifier keyword.
    fn parse_modifier(word: &str) -> Option<TextStyle> {
        let mut style = TextStyle::default();

        match word {
            "bold" | "b" => style.bold = true,
            "dim" | "d" => style.dim = true,
            "italic" | "i" => style.italic = true,
            "underline" | "u" => style.underline = true,
            "strike" | "s" | "strikethrough" => style.strike = true,
            "reverse" | "r" => style.reverse = true,
            "blink" => style.blink = true,
            // Negation modifiers (for closing tags)
            "not bold" | "not b" => return None,
            _ => return None,
        }

        Some(style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_modifier() {
        let style = Style::parse("bold").unwrap();
        assert!(style.text.bold);
        assert!(!style.text.italic);
    }

    #[test]
    fn parse_shorthand_modifier() {
        let style = Style::parse("b").unwrap();
        assert!(style.text.bold);

        let style = Style::parse("i").unwrap();
        assert!(style.text.italic);
    }

    #[test]
    fn parse_multiple_modifiers() {
        let style = Style::parse("bold italic underline").unwrap();
        assert!(style.text.bold);
        assert!(style.text.italic);
        assert!(style.text.underline);
    }

    #[test]
    fn parse_color() {
        let style = Style::parse("red").unwrap();
        assert_eq!(style.fg, Some(Color::Named("red".into())));
        assert!(style.bg.is_none());
    }

    #[test]
    fn parse_hex_color() {
        let style = Style::parse("#ff5733").unwrap();
        assert_eq!(style.fg, Some(Color::Rgb(255, 87, 51)));
    }

    #[test]
    fn parse_background() {
        let style = Style::parse("on red").unwrap();
        assert!(style.fg.is_none());
        assert_eq!(style.bg, Some(Color::Named("red".into())));
    }

    #[test]
    fn parse_fg_and_bg() {
        let style = Style::parse("white on blue").unwrap();
        assert_eq!(style.fg, Some(Color::Named("white".into())));
        assert_eq!(style.bg, Some(Color::Named("blue".into())));
    }

    #[test]
    fn parse_combined() {
        let style = Style::parse("bold white on blue").unwrap();
        assert!(style.text.bold);
        assert_eq!(style.fg, Some(Color::Named("white".into())));
        assert_eq!(style.bg, Some(Color::Named("blue".into())));
    }

    #[test]
    fn style_apply() {
        let base = Style {
            fg: Some(Color::Named("red".into())),
            text: TextStyle {
                bold: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let overlay = Style {
            bg: Some(Color::Named("blue".into())),
            text: TextStyle {
                italic: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let combined = base.apply(&overlay);
        assert_eq!(combined.fg, Some(Color::Named("red".into())));
        assert_eq!(combined.bg, Some(Color::Named("blue".into())));
        assert!(combined.text.bold);
        assert!(combined.text.italic);
    }

    #[test]
    fn style_is_empty() {
        assert!(Style::new().is_empty());
        assert!(!Style::parse("bold").unwrap().is_empty());
        assert!(!Style::parse("red").unwrap().is_empty());
    }
}
