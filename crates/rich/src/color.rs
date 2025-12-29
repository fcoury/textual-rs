//! Color types for Rich markup.
//!
//! Supports named colors, hex, and RGB formats.

use crate::error::ColorParseError;

/// A color specification in Rich markup.
#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    /// Named color (e.g., "red", "blue", "cyan").
    Named(String),
    /// RGB color components.
    Rgb(u8, u8, u8),
}

impl Color {
    /// Parse a color from a string.
    ///
    /// Supports:
    /// - Named colors: `red`, `blue`, `cyan`, etc.
    /// - Hex colors: `#RGB`, `#RRGGBB`
    /// - RGB function: `rgb(r, g, b)`
    ///
    /// # Examples
    ///
    /// ```
    /// use rich::Color;
    ///
    /// let red = Color::parse("red").unwrap();
    /// let hex = Color::parse("#ff5733").unwrap();
    /// let rgb = Color::parse("rgb(255, 87, 51)").unwrap();
    /// ```
    pub fn parse(input: &str) -> Result<Self, ColorParseError> {
        let input = input.trim();

        if input.is_empty() {
            return Err(ColorParseError::UnknownName(input.to_string()));
        }

        // Hex color
        if input.starts_with('#') {
            return Self::parse_hex(&input[1..]);
        }

        // RGB function
        if input.starts_with("rgb(") && input.ends_with(')') {
            return Self::parse_rgb_func(&input[4..input.len() - 1]);
        }

        // Named color
        Self::parse_named(input)
    }

    /// Parse a hex color (without the # prefix).
    fn parse_hex(hex: &str) -> Result<Self, ColorParseError> {
        match hex.len() {
            // #RGB
            3 => {
                let r = Self::parse_hex_digit(hex.chars().nth(0).unwrap())?;
                let g = Self::parse_hex_digit(hex.chars().nth(1).unwrap())?;
                let b = Self::parse_hex_digit(hex.chars().nth(2).unwrap())?;
                Ok(Color::Rgb(r * 17, g * 17, b * 17))
            }
            // #RRGGBB
            6 => {
                let r = Self::parse_hex_pair(
                    hex.chars().nth(0).unwrap(),
                    hex.chars().nth(1).unwrap(),
                )?;
                let g = Self::parse_hex_pair(
                    hex.chars().nth(2).unwrap(),
                    hex.chars().nth(3).unwrap(),
                )?;
                let b = Self::parse_hex_pair(
                    hex.chars().nth(4).unwrap(),
                    hex.chars().nth(5).unwrap(),
                )?;
                Ok(Color::Rgb(r, g, b))
            }
            _ => Err(ColorParseError::InvalidHex(format!("#{}", hex))),
        }
    }

    fn parse_hex_digit(c: char) -> Result<u8, ColorParseError> {
        match c {
            '0'..='9' => Ok(c as u8 - b'0'),
            'a'..='f' => Ok(c as u8 - b'a' + 10),
            'A'..='F' => Ok(c as u8 - b'A' + 10),
            _ => Err(ColorParseError::InvalidHex(format!("invalid hex digit: {}", c))),
        }
    }

    fn parse_hex_pair(c1: char, c2: char) -> Result<u8, ColorParseError> {
        let high = Self::parse_hex_digit(c1)?;
        let low = Self::parse_hex_digit(c2)?;
        Ok(high * 16 + low)
    }

    /// Parse an RGB function (the part inside the parentheses).
    fn parse_rgb_func(input: &str) -> Result<Self, ColorParseError> {
        let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();

        if parts.len() != 3 {
            return Err(ColorParseError::InvalidRgb(format!("rgb({})", input)));
        }

        let r = parts[0]
            .parse::<u8>()
            .map_err(|_| ColorParseError::InvalidRgb(format!("invalid red: {}", parts[0])))?;
        let g = parts[1]
            .parse::<u8>()
            .map_err(|_| ColorParseError::InvalidRgb(format!("invalid green: {}", parts[1])))?;
        let b = parts[2]
            .parse::<u8>()
            .map_err(|_| ColorParseError::InvalidRgb(format!("invalid blue: {}", parts[2])))?;

        Ok(Color::Rgb(r, g, b))
    }

    /// Parse a named color.
    fn parse_named(name: &str) -> Result<Self, ColorParseError> {
        let name_lower = name.to_lowercase();

        // Check if it's a known color name
        if Self::is_known_color(&name_lower) {
            Ok(Color::Named(name_lower))
        } else {
            Err(ColorParseError::UnknownName(name.to_string()))
        }
    }

    /// Check if a name is a known color.
    fn is_known_color(name: &str) -> bool {
        matches!(
            name,
            // Basic colors
            "black"
                | "white"
                | "red"
                | "green"
                | "blue"
                | "yellow"
                | "cyan"
                | "magenta"
                | "aqua"
                | "fuchsia"
                // Bright variants
                | "bright_black"
                | "bright_white"
                | "bright_red"
                | "bright_green"
                | "bright_blue"
                | "bright_yellow"
                | "bright_cyan"
                | "bright_magenta"
                // Extended colors
                | "grey"
                | "gray"
                | "silver"
                | "maroon"
                | "olive"
                | "navy"
                | "purple"
                | "teal"
                | "lime"
                | "orange"
                | "pink"
                | "brown"
                | "coral"
                | "gold"
                | "indigo"
                | "violet"
                // More CSS colors
                | "aliceblue"
                | "antiquewhite"
                | "aquamarine"
                | "azure"
                | "beige"
                | "bisque"
                | "blanchedalmond"
                | "blueviolet"
                | "burlywood"
                | "cadetblue"
                | "chartreuse"
                | "chocolate"
                | "cornflowerblue"
                | "cornsilk"
                | "crimson"
                | "darkblue"
                | "darkcyan"
                | "darkgoldenrod"
                | "darkgray"
                | "darkgrey"
                | "darkgreen"
                | "darkkhaki"
                | "darkmagenta"
                | "darkolivegreen"
                | "darkorange"
                | "darkorchid"
                | "darkred"
                | "darksalmon"
                | "darkseagreen"
                | "darkslateblue"
                | "darkslategray"
                | "darkslategrey"
                | "darkturquoise"
                | "darkviolet"
                | "deeppink"
                | "deepskyblue"
                | "dimgray"
                | "dimgrey"
                | "dodgerblue"
                | "firebrick"
                | "floralwhite"
                | "forestgreen"
                | "gainsboro"
                | "ghostwhite"
                | "goldenrod"
                | "greenyellow"
                | "honeydew"
                | "hotpink"
                | "indianred"
                | "ivory"
                | "khaki"
                | "lavender"
                | "lavenderblush"
                | "lawngreen"
                | "lemonchiffon"
                | "lightblue"
                | "lightcoral"
                | "lightcyan"
                | "lightgoldenrodyellow"
                | "lightgray"
                | "lightgrey"
                | "lightgreen"
                | "lightpink"
                | "lightsalmon"
                | "lightseagreen"
                | "lightskyblue"
                | "lightslategray"
                | "lightslategrey"
                | "lightsteelblue"
                | "lightyellow"
                | "limegreen"
                | "linen"
                | "mediumaquamarine"
                | "mediumblue"
                | "mediumorchid"
                | "mediumpurple"
                | "mediumseagreen"
                | "mediumslateblue"
                | "mediumspringgreen"
                | "mediumturquoise"
                | "mediumvioletred"
                | "midnightblue"
                | "mintcream"
                | "mistyrose"
                | "moccasin"
                | "navajowhite"
                | "oldlace"
                | "olivedrab"
                | "orangered"
                | "orchid"
                | "palegoldenrod"
                | "palegreen"
                | "paleturquoise"
                | "palevioletred"
                | "papayawhip"
                | "peachpuff"
                | "peru"
                | "plum"
                | "powderblue"
                | "rebeccapurple"
                | "rosybrown"
                | "royalblue"
                | "saddlebrown"
                | "salmon"
                | "sandybrown"
                | "seagreen"
                | "seashell"
                | "sienna"
                | "skyblue"
                | "slateblue"
                | "slategray"
                | "slategrey"
                | "snow"
                | "springgreen"
                | "steelblue"
                | "tan"
                | "thistle"
                | "tomato"
                | "turquoise"
                | "wheat"
                | "whitesmoke"
                | "yellowgreen"
        )
    }

    /// Convert the color to RGB components.
    ///
    /// For named colors, returns the RGB values for that color.
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::Rgb(r, g, b) => (*r, *g, *b),
            Color::Named(name) => Self::named_to_rgb(name),
        }
    }

    /// Convert a named color to RGB.
    fn named_to_rgb(name: &str) -> (u8, u8, u8) {
        match name {
            // Basic colors
            "black" => (0, 0, 0),
            "white" => (255, 255, 255),
            "red" => (255, 0, 0),
            "green" => (0, 128, 0),
            "blue" => (0, 0, 255),
            "yellow" => (255, 255, 0),
            "cyan" | "aqua" => (0, 255, 255),
            "magenta" | "fuchsia" => (255, 0, 255),

            // Bright variants (terminal colors)
            "bright_black" => (128, 128, 128),
            "bright_white" => (255, 255, 255),
            "bright_red" => (255, 85, 85),
            "bright_green" => (85, 255, 85),
            "bright_blue" => (85, 85, 255),
            "bright_yellow" => (255, 255, 85),
            "bright_cyan" => (85, 255, 255),
            "bright_magenta" => (255, 85, 255),

            // Extended colors
            "grey" | "gray" => (128, 128, 128),
            "silver" => (192, 192, 192),
            "maroon" => (128, 0, 0),
            "olive" => (128, 128, 0),
            "navy" => (0, 0, 128),
            "purple" => (128, 0, 128),
            "teal" => (0, 128, 128),
            "lime" => (0, 255, 0),
            "orange" => (255, 165, 0),
            "pink" => (255, 192, 203),
            "brown" => (165, 42, 42),
            "coral" => (255, 127, 80),
            "gold" => (255, 215, 0),
            "indigo" => (75, 0, 130),
            "violet" => (238, 130, 238),

            // CSS named colors
            "aliceblue" => (240, 248, 255),
            "antiquewhite" => (250, 235, 215),
            "aquamarine" => (127, 255, 212),
            "azure" => (240, 255, 255),
            "beige" => (245, 245, 220),
            "bisque" => (255, 228, 196),
            "blanchedalmond" => (255, 235, 205),
            "blueviolet" => (138, 43, 226),
            "burlywood" => (222, 184, 135),
            "cadetblue" => (95, 158, 160),
            "chartreuse" => (127, 255, 0),
            "chocolate" => (210, 105, 30),
            "cornflowerblue" => (100, 149, 237),
            "cornsilk" => (255, 248, 220),
            "crimson" => (220, 20, 60),
            "darkblue" => (0, 0, 139),
            "darkcyan" => (0, 139, 139),
            "darkgoldenrod" => (184, 134, 11),
            "darkgray" | "darkgrey" => (169, 169, 169),
            "darkgreen" => (0, 100, 0),
            "darkkhaki" => (189, 183, 107),
            "darkmagenta" => (139, 0, 139),
            "darkolivegreen" => (85, 107, 47),
            "darkorange" => (255, 140, 0),
            "darkorchid" => (153, 50, 204),
            "darkred" => (139, 0, 0),
            "darksalmon" => (233, 150, 122),
            "darkseagreen" => (143, 188, 143),
            "darkslateblue" => (72, 61, 139),
            "darkslategray" | "darkslategrey" => (47, 79, 79),
            "darkturquoise" => (0, 206, 209),
            "darkviolet" => (148, 0, 211),
            "deeppink" => (255, 20, 147),
            "deepskyblue" => (0, 191, 255),
            "dimgray" | "dimgrey" => (105, 105, 105),
            "dodgerblue" => (30, 144, 255),
            "firebrick" => (178, 34, 34),
            "floralwhite" => (255, 250, 240),
            "forestgreen" => (34, 139, 34),
            "gainsboro" => (220, 220, 220),
            "ghostwhite" => (248, 248, 255),
            "goldenrod" => (218, 165, 32),
            "greenyellow" => (173, 255, 47),
            "honeydew" => (240, 255, 240),
            "hotpink" => (255, 105, 180),
            "indianred" => (205, 92, 92),
            "ivory" => (255, 255, 240),
            "khaki" => (240, 230, 140),
            "lavender" => (230, 230, 250),
            "lavenderblush" => (255, 240, 245),
            "lawngreen" => (124, 252, 0),
            "lemonchiffon" => (255, 250, 205),
            "lightblue" => (173, 216, 230),
            "lightcoral" => (240, 128, 128),
            "lightcyan" => (224, 255, 255),
            "lightgoldenrodyellow" => (250, 250, 210),
            "lightgray" | "lightgrey" => (211, 211, 211),
            "lightgreen" => (144, 238, 144),
            "lightpink" => (255, 182, 193),
            "lightsalmon" => (255, 160, 122),
            "lightseagreen" => (32, 178, 170),
            "lightskyblue" => (135, 206, 250),
            "lightslategray" | "lightslategrey" => (119, 136, 153),
            "lightsteelblue" => (176, 196, 222),
            "lightyellow" => (255, 255, 224),
            "limegreen" => (50, 205, 50),
            "linen" => (250, 240, 230),
            "mediumaquamarine" => (102, 205, 170),
            "mediumblue" => (0, 0, 205),
            "mediumorchid" => (186, 85, 211),
            "mediumpurple" => (147, 112, 219),
            "mediumseagreen" => (60, 179, 113),
            "mediumslateblue" => (123, 104, 238),
            "mediumspringgreen" => (0, 250, 154),
            "mediumturquoise" => (72, 209, 204),
            "mediumvioletred" => (199, 21, 133),
            "midnightblue" => (25, 25, 112),
            "mintcream" => (245, 255, 250),
            "mistyrose" => (255, 228, 225),
            "moccasin" => (255, 228, 181),
            "navajowhite" => (255, 222, 173),
            "oldlace" => (253, 245, 230),
            "olivedrab" => (107, 142, 35),
            "orangered" => (255, 69, 0),
            "orchid" => (218, 112, 214),
            "palegoldenrod" => (238, 232, 170),
            "palegreen" => (152, 251, 152),
            "paleturquoise" => (175, 238, 238),
            "palevioletred" => (219, 112, 147),
            "papayawhip" => (255, 239, 213),
            "peachpuff" => (255, 218, 185),
            "peru" => (205, 133, 63),
            "plum" => (221, 160, 221),
            "powderblue" => (176, 224, 230),
            "rebeccapurple" => (102, 51, 153),
            "rosybrown" => (188, 143, 143),
            "royalblue" => (65, 105, 225),
            "saddlebrown" => (139, 69, 19),
            "salmon" => (250, 128, 114),
            "sandybrown" => (244, 164, 96),
            "seagreen" => (46, 139, 87),
            "seashell" => (255, 245, 238),
            "sienna" => (160, 82, 45),
            "skyblue" => (135, 206, 235),
            "slateblue" => (106, 90, 205),
            "slategray" | "slategrey" => (112, 128, 144),
            "snow" => (255, 250, 250),
            "springgreen" => (0, 255, 127),
            "steelblue" => (70, 130, 180),
            "tan" => (210, 180, 140),
            "thistle" => (216, 191, 216),
            "tomato" => (255, 99, 71),
            "turquoise" => (64, 224, 208),
            "wheat" => (245, 222, 179),
            "whitesmoke" => (245, 245, 245),
            "yellowgreen" => (154, 205, 50),

            // Fallback for unknown colors
            _ => (0, 0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_named_color() {
        assert_eq!(Color::parse("red").unwrap(), Color::Named("red".into()));
        assert_eq!(Color::parse("Blue").unwrap(), Color::Named("blue".into()));
        assert_eq!(Color::parse("CYAN").unwrap(), Color::Named("cyan".into()));
    }

    #[test]
    fn parse_hex_short() {
        assert_eq!(Color::parse("#f00").unwrap(), Color::Rgb(255, 0, 0));
        assert_eq!(Color::parse("#0f0").unwrap(), Color::Rgb(0, 255, 0));
        assert_eq!(Color::parse("#00f").unwrap(), Color::Rgb(0, 0, 255));
    }

    #[test]
    fn parse_hex_long() {
        assert_eq!(Color::parse("#ff5733").unwrap(), Color::Rgb(255, 87, 51));
        assert_eq!(Color::parse("#000000").unwrap(), Color::Rgb(0, 0, 0));
        assert_eq!(Color::parse("#ffffff").unwrap(), Color::Rgb(255, 255, 255));
    }

    #[test]
    fn parse_rgb_func() {
        assert_eq!(
            Color::parse("rgb(255, 87, 51)").unwrap(),
            Color::Rgb(255, 87, 51)
        );
        assert_eq!(Color::parse("rgb(0,0,0)").unwrap(), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn parse_invalid() {
        assert!(Color::parse("notacolor").is_err());
        assert!(Color::parse("#gg0000").is_err());
        assert!(Color::parse("rgb(256, 0, 0)").is_err());
    }

    #[test]
    fn to_rgb() {
        assert_eq!(Color::Named("red".into()).to_rgb(), (255, 0, 0));
        assert_eq!(Color::Rgb(10, 20, 30).to_rgb(), (10, 20, 30));
    }
}
