//! Ruler widget for visual measurement.
//!
//! Ruler displays measurement markers along a vertical or horizontal axis,
//! useful for debugging layout issues and visualizing widget sizes.

use std::marker::PhantomData;

use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

use crate::segment::{Segment, Style};
use crate::strip::Strip;
use crate::{Canvas, KeyCode, MouseEvent, Region, Size, Widget};

/// Orientation of the ruler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RulerOrientation {
    /// Vertical ruler (measures height, typically docked to right edge).
    #[default]
    Vertical,
    /// Horizontal ruler (measures width, typically docked to bottom edge).
    Horizontal,
}

/// A visual measurement widget for debugging layout.
///
/// Ruler displays tick marks along its axis matching Python Textual's pattern:
/// - Minor ticks (`·`) for most positions
/// - Major ticks (`•`) every 5th position (5, 10, 15...)
///
/// # Default CSS
///
/// ```css
/// Ruler.-vertical {
///     dock: right;
///     width: 1;
/// }
/// Ruler.-horizontal {
///     dock: bottom;
///     height: 1;
/// }
/// ```
///
/// # Example
///
/// ```ignore
/// ui! {
///     VerticalScroll {
///         // ... content ...
///     }
///     Ruler {}  // Vertical ruler docked to right
/// }
/// ```
pub struct Ruler<M> {
    orientation: RulerOrientation,
    id: Option<String>,
    classes: Vec<String>,
    style: ComputedStyle,
    dirty: bool,
    _phantom: PhantomData<M>,
}

impl<M> Default for Ruler<M> {
    fn default() -> Self {
        Self {
            orientation: RulerOrientation::Vertical,
            id: None,
            classes: vec!["-vertical".to_string()],
            style: ComputedStyle::default(),
            dirty: true,
            _phantom: PhantomData,
        }
    }
}

impl<M> Ruler<M> {
    /// Create a new vertical ruler (default).
    ///
    /// The `_children` parameter is ignored - Ruler is a leaf widget
    /// with no children. The parameter exists for compatibility with
    /// the `ui!` macro which passes `vec![]` for `Ruler {}` syntax.
    pub fn new(_children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self::default()
    }

    /// Create a vertical ruler.
    pub fn vertical() -> Self {
        Self::default()
    }

    /// Create a horizontal ruler.
    pub fn horizontal() -> Self {
        Self {
            orientation: RulerOrientation::Horizontal,
            classes: vec!["-horizontal".to_string()],
            ..Default::default()
        }
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set CSS classes (space-separated).
    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        // Keep orientation class, add user classes
        let orientation_class = if self.orientation == RulerOrientation::Vertical {
            "-vertical"
        } else {
            "-horizontal"
        };
        self.classes = vec![orientation_class.to_string()];
        for class in classes.into().split_whitespace() {
            self.classes.push(class.to_string());
        }
        self
    }

    /// Get the character for a given position.
    /// Uses bullet (•) for every 5th position, middle dot (·) for others.
    /// Matches Python Textual's ruler pattern: "·\n·\n·\n·\n•\n" repeated.
    fn char_at(&self, pos: usize) -> char {
        if (pos + 1) % 5 == 0 {
            // Major tick at positions 5, 10, 15... (1-indexed)
            '•' // U+2022 BULLET
        } else {
            // Minor tick
            '·' // U+00B7 MIDDLE DOT
        }
    }

    /// Get the rendering style from computed CSS.
    fn rendering_style(&self) -> Style {
        Style {
            fg: self.style.color.clone(),
            bg: self.style.background.clone(),
            bold: self.style.text_style.bold,
            dim: self.style.text_style.dim,
            italic: self.style.text_style.italic,
            underline: self.style.text_style.underline,
            strike: self.style.text_style.strike,
            reverse: self.style.text_style.reverse,
        }
    }
}

impl<M: 'static> Widget<M> for Ruler<M> {
    fn default_css(&self) -> &'static str {
        r#"
Ruler {
    width: 1;
    height: 1fr;
}
Ruler.-vertical {
    dock: right;
    width: 1;
    height: 1fr;
}
Ruler.-horizontal {
    dock: bottom;
    width: 1fr;
    height: 1;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        let style = self.rendering_style();

        match self.orientation {
            RulerOrientation::Vertical => {
                // Render vertical ruler (one character per row)
                for y in 0..region.height as usize {
                    let ch = self.char_at(y);
                    let segment = Segment::styled(ch.to_string(), style.clone());
                    let strip = Strip::from_segment(segment);
                    canvas.render_strip(&strip, region.x, region.y + y as i32);
                }
            }
            RulerOrientation::Horizontal => {
                // Render horizontal ruler (one character per column)
                let mut chars = String::with_capacity(region.width as usize);
                for x in 0..region.width as usize {
                    chars.push(self.char_at(x));
                }
                let segment = Segment::styled(chars, style);
                let strip = Strip::from_segment(segment);
                canvas.render_strip(&strip, region.x, region.y);
            }
        }
    }

    fn desired_size(&self) -> Size {
        match self.orientation {
            RulerOrientation::Vertical => Size::new(1, u16::MAX),
            RulerOrientation::Horizontal => Size::new(u16::MAX, 1),
        }
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Ruler",
            id: self.id.clone(),
            classes: self.classes.clone(),
            states: WidgetStates::empty(),
        }
    }

    fn get_state(&self) -> WidgetStates {
        WidgetStates::empty()
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    fn on_event(&mut self, _key: KeyCode) -> Option<M> {
        None
    }

    fn on_mouse(&mut self, _event: MouseEvent, _region: Region) -> Option<M> {
        None
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn add_class(&mut self, class: &str) {
        if !self.classes.iter().any(|c| c == class) {
            self.classes.push(class.to_string());
            self.dirty = true;
        }
    }

    fn remove_class(&mut self, class: &str) {
        if let Some(pos) = self.classes.iter().position(|c| c == class) {
            self.classes.remove(pos);
            self.dirty = true;
        }
    }

    fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c == class)
    }

    fn set_classes(&mut self, classes: &str) {
        // Keep orientation class
        let orientation_class = if self.orientation == RulerOrientation::Vertical {
            "-vertical"
        } else {
            "-horizontal"
        };
        self.classes = vec![orientation_class.to_string()];
        for class in classes.split_whitespace() {
            self.classes.push(class.to_string());
        }
        self.dirty = true;
    }

    fn classes(&self) -> Vec<String> {
        self.classes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that char_at produces the correct pattern matching Python Textual.
    /// Pattern: middle dot (·) for most positions, bullet (•) every 5th position.
    /// Positions are 0-indexed, so bullets appear at 4, 9, 14... (1-indexed: 5, 10, 15...)
    #[test]
    fn test_ruler_char_pattern() {
        let ruler: Ruler<()> = Ruler::vertical();

        // Test first 15 positions
        let expected = [
            '·', '·', '·', '·', '•', '·', '·', '·', '·', '•', '·', '·', '·', '·', '•',
        ];

        for (i, &expected_char) in expected.iter().enumerate() {
            assert_eq!(
                ruler.char_at(i),
                expected_char,
                "Position {} should be '{}', got '{}'",
                i,
                expected_char,
                ruler.char_at(i)
            );
        }
    }

    /// Test that bullet (•) appears at every 5th position (1-indexed).
    #[test]
    fn test_ruler_major_ticks_at_multiples_of_five() {
        let ruler: Ruler<()> = Ruler::vertical();

        // Bullets at positions 4, 9, 14, 19... (0-indexed)
        // Which is 5, 10, 15, 20... (1-indexed)
        for i in 0..100 {
            let ch = ruler.char_at(i);
            if (i + 1) % 5 == 0 {
                assert_eq!(
                    ch,
                    '•',
                    "Position {} (1-indexed: {}) should be bullet",
                    i,
                    i + 1
                );
            } else {
                assert_eq!(ch, '·', "Position {} should be middle dot", i);
            }
        }
    }

    /// Test default orientation is vertical.
    #[test]
    fn test_ruler_default_orientation() {
        let ruler: Ruler<()> = Ruler::default();
        assert_eq!(ruler.orientation, RulerOrientation::Vertical);
        assert!(ruler.classes.contains(&"-vertical".to_string()));
    }

    /// Test horizontal ruler creation.
    #[test]
    fn test_ruler_horizontal() {
        let ruler: Ruler<()> = Ruler::horizontal();
        assert_eq!(ruler.orientation, RulerOrientation::Horizontal);
        assert!(ruler.classes.contains(&"-horizontal".to_string()));
    }
}
