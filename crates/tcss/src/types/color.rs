use std::fmt;

/// Error returned when color parsing fails.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorParseError {
    pub message: String,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ColorParseError {}

#[derive(Clone, Debug, PartialEq)]
pub struct RgbaColor {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
    /// Alpha component (0.0 to 1.0).
    pub a: f32,
    /// Optional ANSI 256-color palette index.
    pub ansi: Option<u8>,
    /// If true, this color resolves dynamically based on background contrast.
    pub auto: bool,
}

impl Default for RgbaColor {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0,
            ansi: None,
            auto: false,
        }
    }
}

impl RgbaColor {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            a: 1.0,
            ..Default::default()
        }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r,
            g,
            b,
            a,
            ..Default::default()
        }
    }

    pub fn ansi(index: u8) -> Self {
        Self {
            ansi: Some(index),
            ..Default::default()
        }
    }

    /// Returns a special 'auto' color used for dynamic theme contrast.
    pub fn auto(alpha: f32) -> Self {
        Self {
            auto: true,
            a: alpha,
            ..Default::default()
        }
    }

    /// Returns true if the color is fully transparent.
    pub fn is_transparent(&self) -> bool {
        self.a <= 0.0
    }

    /// Parse a color string in various formats.
    ///
    /// Supported formats:
    /// - Hex: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`
    /// - RGB: `rgb(r,g,b)`, `rgb(r,g,b,a)`
    /// - HSL: `hsl(h,s%,l%)`, `hsla(h,s%,l%,a)`
    /// - Named: CSS color names like `red`, `blue`, `aliceblue`
    /// - Special: `auto`, `transparent`
    pub fn parse(input: &str) -> Result<Self, ColorParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ColorParseError {
                message: "empty color string".to_string(),
            });
        }

        let lower = input.to_lowercase();

        // Special values
        match lower.as_str() {
            "auto" => return Ok(Self::auto(1.0)),
            "transparent" => return Ok(Self::rgba(0, 0, 0, 0.0)),
            _ => {}
        }

        // Hex colors
        if input.starts_with('#') {
            return Self::parse_hex(&input[1..]);
        }

        // RGB/RGBA functions
        if lower.starts_with("rgb") {
            return Self::parse_rgb_func(&lower);
        }

        // HSL/HSLA functions
        if lower.starts_with("hsl") {
            return Self::parse_hsl_func(&lower);
        }

        // Named colors
        Self::parse_named(&lower)
    }

    fn parse_hex(hex: &str) -> Result<Self, ColorParseError> {
        let hex = hex.to_lowercase();
        let chars: Vec<char> = hex.chars().collect();

        match chars.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = Self::parse_hex_digit(chars[0])? * 17;
                let g = Self::parse_hex_digit(chars[1])? * 17;
                let b = Self::parse_hex_digit(chars[2])? * 17;
                Ok(Self::rgb(r, g, b))
            }
            4 => {
                // #RGBA -> #RRGGBBAA
                let r = Self::parse_hex_digit(chars[0])? * 17;
                let g = Self::parse_hex_digit(chars[1])? * 17;
                let b = Self::parse_hex_digit(chars[2])? * 17;
                let a = Self::parse_hex_digit(chars[3])? * 17;
                Ok(Self::rgba(r, g, b, a as f32 / 255.0))
            }
            6 => {
                // #RRGGBB
                let r = Self::parse_hex_pair(chars[0], chars[1])?;
                let g = Self::parse_hex_pair(chars[2], chars[3])?;
                let b = Self::parse_hex_pair(chars[4], chars[5])?;
                Ok(Self::rgb(r, g, b))
            }
            8 => {
                // #RRGGBBAA
                let r = Self::parse_hex_pair(chars[0], chars[1])?;
                let g = Self::parse_hex_pair(chars[2], chars[3])?;
                let b = Self::parse_hex_pair(chars[4], chars[5])?;
                let a = Self::parse_hex_pair(chars[6], chars[7])?;
                Ok(Self::rgba(r, g, b, a as f32 / 255.0))
            }
            _ => Err(ColorParseError {
                message: format!("invalid hex color length: {}", chars.len()),
            }),
        }
    }

    fn parse_hex_digit(c: char) -> Result<u8, ColorParseError> {
        match c {
            '0'..='9' => Ok(c as u8 - b'0'),
            'a'..='f' => Ok(c as u8 - b'a' + 10),
            _ => Err(ColorParseError {
                message: format!("invalid hex digit: {}", c),
            }),
        }
    }

    fn parse_hex_pair(c1: char, c2: char) -> Result<u8, ColorParseError> {
        let high = Self::parse_hex_digit(c1)?;
        let low = Self::parse_hex_digit(c2)?;
        Ok(high * 16 + low)
    }

    fn parse_rgb_func(input: &str) -> Result<Self, ColorParseError> {
        // Extract content between parentheses
        let start = input.find('(').ok_or_else(|| ColorParseError {
            message: "missing '(' in rgb function".to_string(),
        })?;
        let end = input.find(')').ok_or_else(|| ColorParseError {
            message: "missing ')' in rgb function".to_string(),
        })?;

        let content = &input[start + 1..end];
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();

        if parts.len() < 3 {
            return Err(ColorParseError {
                message: "rgb requires at least 3 components".to_string(),
            });
        }

        let r = Self::parse_u8(parts[0])?;
        let g = Self::parse_u8(parts[1])?;
        let b = Self::parse_u8(parts[2])?;

        let a = if parts.len() >= 4 {
            Self::parse_f32(parts[3])?
        } else {
            1.0
        };

        Ok(Self::rgba(r, g, b, a))
    }

    fn parse_hsl_func(input: &str) -> Result<Self, ColorParseError> {
        // Extract content between parentheses
        let start = input.find('(').ok_or_else(|| ColorParseError {
            message: "missing '(' in hsl function".to_string(),
        })?;
        let end = input.find(')').ok_or_else(|| ColorParseError {
            message: "missing ')' in hsl function".to_string(),
        })?;

        let content = &input[start + 1..end];
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();

        if parts.len() < 3 {
            return Err(ColorParseError {
                message: "hsl requires at least 3 components".to_string(),
            });
        }

        let h: f32 = parts[0].parse().map_err(|_| ColorParseError {
            message: format!("invalid hue: {}", parts[0]),
        })?;

        let s = Self::parse_percentage(parts[1])?;
        let l = Self::parse_percentage(parts[2])?;

        let a = if parts.len() >= 4 {
            Self::parse_f32(parts[3])?
        } else {
            1.0
        };

        let (r, g, b) = Self::hsl_to_rgb(h, s, l);
        Ok(Self::rgba(r, g, b, a))
    }

    fn parse_u8(s: &str) -> Result<u8, ColorParseError> {
        let val: i32 = s.parse().map_err(|_| ColorParseError {
            message: format!("invalid number: {}", s),
        })?;
        if val < 0 || val > 255 {
            return Err(ColorParseError {
                message: format!("value out of range (0-255): {}", val),
            });
        }
        Ok(val as u8)
    }

    fn parse_f32(s: &str) -> Result<f32, ColorParseError> {
        s.parse().map_err(|_| ColorParseError {
            message: format!("invalid float: {}", s),
        })
    }

    fn parse_percentage(s: &str) -> Result<f32, ColorParseError> {
        let s = s.trim_end_matches('%');
        let val: f32 = s.parse().map_err(|_| ColorParseError {
            message: format!("invalid percentage: {}", s),
        })?;
        Ok(val / 100.0)
    }

    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
        if s == 0.0 {
            let gray = (l * 255.0).round() as u8;
            return (gray, gray, gray);
        }

        let h = h / 360.0;
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let r = Self::hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = Self::hue_to_rgb(p, q, h);
        let b = Self::hue_to_rgb(p, q, h - 1.0 / 3.0);

        (
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        )
    }

    fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }

        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    fn parse_named(name: &str) -> Result<Self, ColorParseError> {
        match name {
            // Basic colors
            "black" => Ok(Self::rgb(0, 0, 0)),
            "white" => Ok(Self::rgb(255, 255, 255)),
            "red" => Ok(Self::rgb(255, 0, 0)),
            "green" => Ok(Self::rgb(0, 128, 0)),
            "blue" => Ok(Self::rgb(0, 0, 255)),
            "yellow" => Ok(Self::rgb(255, 255, 0)),
            "cyan" | "aqua" => Ok(Self::rgb(0, 255, 255)),
            "magenta" | "fuchsia" => Ok(Self::rgb(255, 0, 255)),

            // Extended colors (CSS named colors)
            "aliceblue" => Ok(Self::rgb(240, 248, 255)),
            "antiquewhite" => Ok(Self::rgb(250, 235, 215)),
            "aquamarine" => Ok(Self::rgb(127, 255, 212)),
            "azure" => Ok(Self::rgb(240, 255, 255)),
            "beige" => Ok(Self::rgb(245, 245, 220)),
            "bisque" => Ok(Self::rgb(255, 228, 196)),
            "blanchedalmond" => Ok(Self::rgb(255, 235, 205)),
            "blueviolet" => Ok(Self::rgb(138, 43, 226)),
            "brown" => Ok(Self::rgb(165, 42, 42)),
            "burlywood" => Ok(Self::rgb(222, 184, 135)),
            "cadetblue" => Ok(Self::rgb(95, 158, 160)),
            "chartreuse" => Ok(Self::rgb(127, 255, 0)),
            "chocolate" => Ok(Self::rgb(210, 105, 30)),
            "coral" => Ok(Self::rgb(255, 127, 80)),
            "cornflowerblue" => Ok(Self::rgb(100, 149, 237)),
            "cornsilk" => Ok(Self::rgb(255, 248, 220)),
            "crimson" => Ok(Self::rgb(220, 20, 60)),
            "darkblue" => Ok(Self::rgb(0, 0, 139)),
            "darkcyan" => Ok(Self::rgb(0, 139, 139)),
            "darkgoldenrod" => Ok(Self::rgb(184, 134, 11)),
            "darkgray" | "darkgrey" => Ok(Self::rgb(169, 169, 169)),
            "darkgreen" => Ok(Self::rgb(0, 100, 0)),
            "darkkhaki" => Ok(Self::rgb(189, 183, 107)),
            "darkmagenta" => Ok(Self::rgb(139, 0, 139)),
            "darkolivegreen" => Ok(Self::rgb(85, 107, 47)),
            "darkorange" => Ok(Self::rgb(255, 140, 0)),
            "darkorchid" => Ok(Self::rgb(153, 50, 204)),
            "darkred" => Ok(Self::rgb(139, 0, 0)),
            "darksalmon" => Ok(Self::rgb(233, 150, 122)),
            "darkseagreen" => Ok(Self::rgb(143, 188, 143)),
            "darkslateblue" => Ok(Self::rgb(72, 61, 139)),
            "darkslategray" | "darkslategrey" => Ok(Self::rgb(47, 79, 79)),
            "darkturquoise" => Ok(Self::rgb(0, 206, 209)),
            "darkviolet" => Ok(Self::rgb(148, 0, 211)),
            "deeppink" => Ok(Self::rgb(255, 20, 147)),
            "deepskyblue" => Ok(Self::rgb(0, 191, 255)),
            "dimgray" | "dimgrey" => Ok(Self::rgb(105, 105, 105)),
            "dodgerblue" => Ok(Self::rgb(30, 144, 255)),
            "firebrick" => Ok(Self::rgb(178, 34, 34)),
            "floralwhite" => Ok(Self::rgb(255, 250, 240)),
            "forestgreen" => Ok(Self::rgb(34, 139, 34)),
            "gainsboro" => Ok(Self::rgb(220, 220, 220)),
            "ghostwhite" => Ok(Self::rgb(248, 248, 255)),
            "gold" => Ok(Self::rgb(255, 215, 0)),
            "goldenrod" => Ok(Self::rgb(218, 165, 32)),
            "gray" | "grey" => Ok(Self::rgb(128, 128, 128)),
            "greenyellow" => Ok(Self::rgb(173, 255, 47)),
            "honeydew" => Ok(Self::rgb(240, 255, 240)),
            "hotpink" => Ok(Self::rgb(255, 105, 180)),
            "indianred" => Ok(Self::rgb(205, 92, 92)),
            "indigo" => Ok(Self::rgb(75, 0, 130)),
            "ivory" => Ok(Self::rgb(255, 255, 240)),
            "khaki" => Ok(Self::rgb(240, 230, 140)),
            "lavender" => Ok(Self::rgb(230, 230, 250)),
            "lavenderblush" => Ok(Self::rgb(255, 240, 245)),
            "lawngreen" => Ok(Self::rgb(124, 252, 0)),
            "lemonchiffon" => Ok(Self::rgb(255, 250, 205)),
            "lightblue" => Ok(Self::rgb(173, 216, 230)),
            "lightcoral" => Ok(Self::rgb(240, 128, 128)),
            "lightcyan" => Ok(Self::rgb(224, 255, 255)),
            "lightgoldenrodyellow" => Ok(Self::rgb(250, 250, 210)),
            "lightgray" | "lightgrey" => Ok(Self::rgb(211, 211, 211)),
            "lightgreen" => Ok(Self::rgb(144, 238, 144)),
            "lightpink" => Ok(Self::rgb(255, 182, 193)),
            "lightsalmon" => Ok(Self::rgb(255, 160, 122)),
            "lightseagreen" => Ok(Self::rgb(32, 178, 170)),
            "lightskyblue" => Ok(Self::rgb(135, 206, 250)),
            "lightslategray" | "lightslategrey" => Ok(Self::rgb(119, 136, 153)),
            "lightsteelblue" => Ok(Self::rgb(176, 196, 222)),
            "lightyellow" => Ok(Self::rgb(255, 255, 224)),
            "lime" => Ok(Self::rgb(0, 255, 0)),
            "limegreen" => Ok(Self::rgb(50, 205, 50)),
            "linen" => Ok(Self::rgb(250, 240, 230)),
            "maroon" => Ok(Self::rgb(128, 0, 0)),
            "mediumaquamarine" => Ok(Self::rgb(102, 205, 170)),
            "mediumblue" => Ok(Self::rgb(0, 0, 205)),
            "mediumorchid" => Ok(Self::rgb(186, 85, 211)),
            "mediumpurple" => Ok(Self::rgb(147, 112, 219)),
            "mediumseagreen" => Ok(Self::rgb(60, 179, 113)),
            "mediumslateblue" => Ok(Self::rgb(123, 104, 238)),
            "mediumspringgreen" => Ok(Self::rgb(0, 250, 154)),
            "mediumturquoise" => Ok(Self::rgb(72, 209, 204)),
            "mediumvioletred" => Ok(Self::rgb(199, 21, 133)),
            "midnightblue" => Ok(Self::rgb(25, 25, 112)),
            "mintcream" => Ok(Self::rgb(245, 255, 250)),
            "mistyrose" => Ok(Self::rgb(255, 228, 225)),
            "moccasin" => Ok(Self::rgb(255, 228, 181)),
            "navajowhite" => Ok(Self::rgb(255, 222, 173)),
            "navy" => Ok(Self::rgb(0, 0, 128)),
            "oldlace" => Ok(Self::rgb(253, 245, 230)),
            "olive" => Ok(Self::rgb(128, 128, 0)),
            "olivedrab" => Ok(Self::rgb(107, 142, 35)),
            "orange" => Ok(Self::rgb(255, 165, 0)),
            "orangered" => Ok(Self::rgb(255, 69, 0)),
            "orchid" => Ok(Self::rgb(218, 112, 214)),
            "palegoldenrod" => Ok(Self::rgb(238, 232, 170)),
            "palegreen" => Ok(Self::rgb(152, 251, 152)),
            "paleturquoise" => Ok(Self::rgb(175, 238, 238)),
            "palevioletred" => Ok(Self::rgb(219, 112, 147)),
            "papayawhip" => Ok(Self::rgb(255, 239, 213)),
            "peachpuff" => Ok(Self::rgb(255, 218, 185)),
            "peru" => Ok(Self::rgb(205, 133, 63)),
            "pink" => Ok(Self::rgb(255, 192, 203)),
            "plum" => Ok(Self::rgb(221, 160, 221)),
            "powderblue" => Ok(Self::rgb(176, 224, 230)),
            "purple" => Ok(Self::rgb(128, 0, 128)),
            "rebeccapurple" => Ok(Self::rgb(102, 51, 153)),
            "rosybrown" => Ok(Self::rgb(188, 143, 143)),
            "royalblue" => Ok(Self::rgb(65, 105, 225)),
            "saddlebrown" => Ok(Self::rgb(139, 69, 19)),
            "salmon" => Ok(Self::rgb(250, 128, 114)),
            "sandybrown" => Ok(Self::rgb(244, 164, 96)),
            "seagreen" => Ok(Self::rgb(46, 139, 87)),
            "seashell" => Ok(Self::rgb(255, 245, 238)),
            "sienna" => Ok(Self::rgb(160, 82, 45)),
            "silver" => Ok(Self::rgb(192, 192, 192)),
            "skyblue" => Ok(Self::rgb(135, 206, 235)),
            "slateblue" => Ok(Self::rgb(106, 90, 205)),
            "slategray" | "slategrey" => Ok(Self::rgb(112, 128, 144)),
            "snow" => Ok(Self::rgb(255, 250, 250)),
            "springgreen" => Ok(Self::rgb(0, 255, 127)),
            "steelblue" => Ok(Self::rgb(70, 130, 180)),
            "tan" => Ok(Self::rgb(210, 180, 140)),
            "teal" => Ok(Self::rgb(0, 128, 128)),
            "thistle" => Ok(Self::rgb(216, 191, 216)),
            "tomato" => Ok(Self::rgb(255, 99, 71)),
            "turquoise" => Ok(Self::rgb(64, 224, 208)),
            "violet" => Ok(Self::rgb(238, 130, 238)),
            "wheat" => Ok(Self::rgb(245, 222, 179)),
            "whitesmoke" => Ok(Self::rgb(245, 245, 245)),
            "yellowgreen" => Ok(Self::rgb(154, 205, 50)),

            _ => Err(ColorParseError {
                message: format!("unknown color name: {}", name),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== HEX FORMAT TESTS ====================

    #[test]
    fn test_hex_3_digit() {
        // #RGB expands to #RRGGBB
        let color = RgbaColor::parse("#f00").unwrap();
        assert_eq!(color, RgbaColor::rgb(255, 0, 0));

        let color = RgbaColor::parse("#0f0").unwrap();
        assert_eq!(color, RgbaColor::rgb(0, 255, 0));

        let color = RgbaColor::parse("#00f").unwrap();
        assert_eq!(color, RgbaColor::rgb(0, 0, 255));

        let color = RgbaColor::parse("#abc").unwrap();
        assert_eq!(color, RgbaColor::rgb(0xaa, 0xbb, 0xcc));
    }

    #[test]
    fn test_hex_4_digit() {
        // #RGBA expands to #RRGGBBAA
        let color = RgbaColor::parse("#f00f").unwrap();
        assert_eq!(color, RgbaColor::rgba(255, 0, 0, 1.0));

        let color = RgbaColor::parse("#f008").unwrap();
        assert_eq!(color, RgbaColor::rgba(255, 0, 0, 0x88 as f32 / 255.0));

        let color = RgbaColor::parse("#0000").unwrap();
        assert_eq!(color, RgbaColor::rgba(0, 0, 0, 0.0));
    }

    #[test]
    fn test_hex_6_digit() {
        let color = RgbaColor::parse("#ff0000").unwrap();
        assert_eq!(color, RgbaColor::rgb(255, 0, 0));

        let color = RgbaColor::parse("#00ff00").unwrap();
        assert_eq!(color, RgbaColor::rgb(0, 255, 0));

        let color = RgbaColor::parse("#0000ff").unwrap();
        assert_eq!(color, RgbaColor::rgb(0, 0, 255));

        let color = RgbaColor::parse("#9932CC").unwrap();
        assert_eq!(color, RgbaColor::rgb(0x99, 0x32, 0xCC));
    }

    #[test]
    fn test_hex_8_digit() {
        let color = RgbaColor::parse("#ff0000ff").unwrap();
        assert_eq!(color, RgbaColor::rgba(255, 0, 0, 1.0));

        let color = RgbaColor::parse("#ff000080").unwrap();
        assert_eq!(color, RgbaColor::rgba(255, 0, 0, 0x80 as f32 / 255.0));

        let color = RgbaColor::parse("#00000000").unwrap();
        assert_eq!(color, RgbaColor::rgba(0, 0, 0, 0.0));
    }

    #[test]
    fn test_hex_case_insensitive() {
        let lower = RgbaColor::parse("#aabbcc").unwrap();
        let upper = RgbaColor::parse("#AABBCC").unwrap();
        let mixed = RgbaColor::parse("#AaBbCc").unwrap();
        assert_eq!(lower, upper);
        assert_eq!(lower, mixed);
    }

    // ==================== RGB FORMAT TESTS ====================

    #[test]
    fn test_rgb_basic() {
        let color = RgbaColor::parse("rgb(255, 0, 0)").unwrap();
        assert_eq!(color, RgbaColor::rgb(255, 0, 0));

        let color = RgbaColor::parse("rgb(0, 255, 0)").unwrap();
        assert_eq!(color, RgbaColor::rgb(0, 255, 0));

        let color = RgbaColor::parse("rgb(0, 0, 255)").unwrap();
        assert_eq!(color, RgbaColor::rgb(0, 0, 255));
    }

    #[test]
    fn test_rgb_no_spaces() {
        let color = RgbaColor::parse("rgb(255,128,64)").unwrap();
        assert_eq!(color, RgbaColor::rgb(255, 128, 64));
    }

    #[test]
    fn test_rgb_with_alpha() {
        let color = RgbaColor::parse("rgb(255, 0, 0, 0.5)").unwrap();
        assert_eq!(color, RgbaColor::rgba(255, 0, 0, 0.5));

        let color = RgbaColor::parse("rgb(255, 0, 0, 1)").unwrap();
        assert_eq!(color, RgbaColor::rgba(255, 0, 0, 1.0));

        let color = RgbaColor::parse("rgb(255, 0, 0, 0)").unwrap();
        assert_eq!(color, RgbaColor::rgba(255, 0, 0, 0.0));
    }

    #[test]
    fn test_rgba_function() {
        let color = RgbaColor::parse("rgba(255, 0, 0, 0.5)").unwrap();
        assert_eq!(color, RgbaColor::rgba(255, 0, 0, 0.5));
    }

    // ==================== HSL FORMAT TESTS ====================

    #[test]
    fn test_hsl_basic() {
        // Red: hsl(0, 100%, 50%)
        let color = RgbaColor::parse("hsl(0, 100%, 50%)").unwrap();
        assert_eq!(color, RgbaColor::rgb(255, 0, 0));

        // Green: hsl(120, 100%, 50%)
        let color = RgbaColor::parse("hsl(120, 100%, 50%)").unwrap();
        assert_eq!(color, RgbaColor::rgb(0, 255, 0));

        // Blue: hsl(240, 100%, 50%)
        let color = RgbaColor::parse("hsl(240, 100%, 50%)").unwrap();
        assert_eq!(color, RgbaColor::rgb(0, 0, 255));
    }

    #[test]
    fn test_hsl_grayscale() {
        // Black: hsl(0, 0%, 0%)
        let color = RgbaColor::parse("hsl(0, 0%, 0%)").unwrap();
        assert_eq!(color, RgbaColor::rgb(0, 0, 0));

        // White: hsl(0, 0%, 100%)
        let color = RgbaColor::parse("hsl(0, 0%, 100%)").unwrap();
        assert_eq!(color, RgbaColor::rgb(255, 255, 255));

        // Gray: hsl(0, 0%, 50%)
        let color = RgbaColor::parse("hsl(0, 0%, 50%)").unwrap();
        // Should be approximately 128
        assert!(color.r >= 127 && color.r <= 128);
        assert!(color.g >= 127 && color.g <= 128);
        assert!(color.b >= 127 && color.b <= 128);
    }

    #[test]
    fn test_hsla_with_alpha() {
        let color = RgbaColor::parse("hsla(0, 100%, 50%, 0.5)").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert!((color.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_hsl_no_spaces() {
        let color = RgbaColor::parse("hsl(0,100%,50%)").unwrap();
        assert_eq!(color, RgbaColor::rgb(255, 0, 0));
    }

    // ==================== NAMED COLORS TESTS ====================

    #[test]
    fn test_named_basic_colors() {
        assert_eq!(RgbaColor::parse("red").unwrap(), RgbaColor::rgb(255, 0, 0));
        assert_eq!(RgbaColor::parse("green").unwrap(), RgbaColor::rgb(0, 128, 0));
        assert_eq!(RgbaColor::parse("blue").unwrap(), RgbaColor::rgb(0, 0, 255));
        assert_eq!(RgbaColor::parse("white").unwrap(), RgbaColor::rgb(255, 255, 255));
        assert_eq!(RgbaColor::parse("black").unwrap(), RgbaColor::rgb(0, 0, 0));
    }

    #[test]
    fn test_named_extended_colors() {
        assert_eq!(RgbaColor::parse("aliceblue").unwrap(), RgbaColor::rgb(240, 248, 255));
        assert_eq!(RgbaColor::parse("coral").unwrap(), RgbaColor::rgb(255, 127, 80));
        assert_eq!(RgbaColor::parse("darkorchid").unwrap(), RgbaColor::rgb(153, 50, 204));
        assert_eq!(RgbaColor::parse("crimson").unwrap(), RgbaColor::rgb(220, 20, 60));
    }

    #[test]
    fn test_named_case_insensitive() {
        assert_eq!(RgbaColor::parse("Red").unwrap(), RgbaColor::parse("red").unwrap());
        assert_eq!(RgbaColor::parse("RED").unwrap(), RgbaColor::parse("red").unwrap());
        assert_eq!(RgbaColor::parse("AliceBlue").unwrap(), RgbaColor::parse("aliceblue").unwrap());
    }

    // ==================== SPECIAL VALUES TESTS ====================

    #[test]
    fn test_auto() {
        let color = RgbaColor::parse("auto").unwrap();
        assert!(color.auto);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_transparent() {
        let color = RgbaColor::parse("transparent").unwrap();
        assert_eq!(color.a, 0.0);
        assert!(color.is_transparent());
    }

    // ==================== WHITESPACE HANDLING ====================

    #[test]
    fn test_whitespace_trimming() {
        assert_eq!(
            RgbaColor::parse("  #ff0000  ").unwrap(),
            RgbaColor::rgb(255, 0, 0)
        );
        assert_eq!(
            RgbaColor::parse("  red  ").unwrap(),
            RgbaColor::rgb(255, 0, 0)
        );
    }

    // ==================== ERROR CASES ====================

    #[test]
    fn test_invalid_hex() {
        assert!(RgbaColor::parse("#gg0000").is_err()); // invalid hex digits
        assert!(RgbaColor::parse("#ff00000").is_err()); // 7 digits (invalid length)
        assert!(RgbaColor::parse("#ff").is_err()); // 2 digits (invalid length)
        assert!(RgbaColor::parse("#f").is_err()); // 1 digit (invalid length)
    }

    #[test]
    fn test_invalid_rgb() {
        assert!(RgbaColor::parse("rgb(256, 0, 0)").is_err()); // out of range
        assert!(RgbaColor::parse("rgb(-1, 0, 0)").is_err()); // negative
        assert!(RgbaColor::parse("rgb(255, 0)").is_err()); // missing component
    }

    #[test]
    fn test_invalid_named() {
        assert!(RgbaColor::parse("notacolor").is_err());
        assert!(RgbaColor::parse("redd").is_err());
    }

    #[test]
    fn test_empty_input() {
        assert!(RgbaColor::parse("").is_err());
        assert!(RgbaColor::parse("   ").is_err());
    }
}
