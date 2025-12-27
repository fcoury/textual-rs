#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub underline2: bool, // Double underline
    pub blink: bool,
    pub blink2: bool, // Rapid blink
    pub reverse: bool,
    pub strike: bool,
    pub overline: bool,
}

impl TextStyle {
    /// Creates a default style with all modifiers disabled.
    pub fn none() -> Self {
        Self::default()
    }

    /// Check if no styles are applied.
    pub fn is_none(&self) -> bool {
        *self == Self::default()
    }

    /// Merges another style into this one.
    /// Used during the CSS cascade where multiple rules apply to one widget.
    pub fn merge(&mut self, other: TextStyle) {
        if other.bold {
            self.bold = true;
        }
        if other.dim {
            self.dim = true;
        }
        if other.italic {
            self.italic = true;
        }
        if other.underline {
            self.underline = true;
        }
        if other.underline2 {
            self.underline2 = true;
        }
        if other.blink {
            self.blink = true;
        }
        if other.blink2 {
            self.blink2 = true;
        }
        if other.reverse {
            self.reverse = true;
        }
        if other.strike {
            self.strike = true;
        }
        if other.overline {
            self.overline = true;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignHorizontal {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignVertical {
    #[default]
    Top,
    Middle,
    Bottom,
}
