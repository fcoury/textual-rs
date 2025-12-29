//! Placeholder widget for prototyping layouts.
//!
//! A simple widget that displays a colored background with a label.
//! Colors auto-cycle through a harmonious palette, matching Python Textual's behavior.
//!
//! ## Example
//!
//! ```ignore
//! let p = Placeholder::new("Item 1");
//! // Colors auto-assigned from palette
//! ```
//!
//! ## CSS
//!
//! ```css
//! Placeholder {
//!     background: #ff6b6b;  /* Override palette color */
//!     color: white;         /* Label color */
//!     padding: 2;
//! }
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};

use tcss::{ComputedStyle, WidgetMeta, WidgetStates};
use tcss::types::RgbaColor;

use crate::canvas::{Canvas, Region, TextAttributes};
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent, Size};

/// Global counter for auto-assigning palette indices.
static PLACEHOLDER_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 8-color harmonious palette (matches Python Textual style).
const PALETTE: [(u8, u8, u8); 8] = [
    (255, 107, 107), // Coral
    (255, 159, 64),  // Orange
    (255, 217, 61),  // Yellow
    (72, 207, 173),  // Teal
    (77, 171, 247),  // Blue
    (127, 90, 240),  // Purple
    (210, 82, 127),  // Pink
    (113, 128, 150), // Slate
];

/// A placeholder widget for prototyping layouts.
///
/// Displays a colored background with a centered label.
/// Colors auto-cycle through a harmonious palette unless
/// overridden via CSS.
pub struct Placeholder {
    label: String,
    palette_index: usize,
    style: ComputedStyle,
    dirty: bool,
    id: Option<String>,
}

impl Placeholder {
    /// Create a new Placeholder with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        let index = PLACEHOLDER_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self {
            label: label.into(),
            palette_index: index % PALETTE.len(),
            style: ComputedStyle::default(),
            dirty: true,
            id: None,
        }
    }

    /// Set the widget ID for CSS targeting and message tracking.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Get the background color (CSS or palette).
    fn effective_background(&self) -> RgbaColor {
        self.style.background.clone().unwrap_or_else(|| {
            let (r, g, b) = PALETTE[self.palette_index];
            RgbaColor::rgb(r, g, b)
        })
    }

    /// Get the foreground color (CSS or contrasting white).
    fn effective_foreground(&self) -> RgbaColor {
        self.style.color.clone().unwrap_or_else(|| {
            // White text for contrast on colored backgrounds
            RgbaColor::rgb(255, 255, 255)
        })
    }
}

impl<M> Widget<M> for Placeholder {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        let bg = self.effective_background();
        let fg = self.effective_foreground();

        // Fill background
        for y in region.y..(region.y + region.height) {
            for x in region.x..(region.x + region.width) {
                canvas.put_char(x, y, ' ', None, Some(bg.clone()), TextAttributes::default());
            }
        }

        // Center label
        let label_len = self.label.len() as i32;
        let x = region.x + (region.width - label_len).max(0) / 2;
        let y = region.y + region.height / 2;

        canvas.put_str(x, y, &self.label, Some(fg), Some(bg), TextAttributes::default());
    }

    fn desired_size(&self) -> Size {
        // Reasonable default size for prototyping
        Size::new(20, 3)
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Placeholder".to_string(),
            id: self.id.clone(),
            classes: Vec::new(),
            states: WidgetStates::empty(),
        }
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
}
