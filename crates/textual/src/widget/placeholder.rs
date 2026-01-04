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

use std::cell::Cell;

use tcss::types::{RgbaColor, Visibility};
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region, TextAttributes};
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent, MouseEventKind, Size};

/// Display variants for the Placeholder widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaceholderVariant {
    /// Shows the label or widget ID (e.g., #p1)
    #[default]
    Default,
    /// Shows the widget's dimensions (e.g., 80 x 24)
    Size,
    /// Shows Lorem Ipsum text
    Text,
}

impl PlaceholderVariant {
    /// Get the next variant in the cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Default => Self::Size,
            Self::Size => Self::Text,
            Self::Text => Self::Default,
        }
    }
}

/// Lorem ipsum paragraph for the Text variant.
const LOREM_IPSUM_PARAGRAPH: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam feugiat ac elit sit amet accumsan. Suspendisse bibendum nec libero quis gravida. Phasellus id eleifend ligula. Nullam imperdiet sem tellus, sed vehicula nisl faucibus sit amet. Praesent iaculis tempor ultricies. Sed lacinia, tellus id rutrum lacinia, sapien sapien congue mauris, sit amet pellentesque quam quam vel nisl. Curabitur vulputate erat pellentesque mauris posuere, non dictum risus mattis.";

thread_local! {
    /// Thread-local counter for auto-assigning palette indices.
    /// Using thread-local ensures deterministic colors in tests (each test thread starts at 0).
    static PLACEHOLDER_COUNTER: Cell<usize> = const { Cell::new(0) };
}

/// Reset the placeholder counter to zero.
/// Used in tests to ensure deterministic color assignment.
#[doc(hidden)]
pub fn reset_placeholder_counter() {
    PLACEHOLDER_COUNTER.with(|c| c.set(0));
}

/// 12-color palette matching Python Textual's _PLACEHOLDER_BACKGROUND_COLORS.
const PALETTE: [(u8, u8, u8); 12] = [
    (136, 17, 119),  // #881177 - Purple
    (170, 51, 85),   // #aa3355 - Maroon
    (204, 102, 102), // #cc6666 - Rose
    (238, 153, 68),  // #ee9944 - Orange
    (238, 221, 0),   // #eedd00 - Yellow
    (153, 221, 85),  // #99dd55 - Lime
    (68, 221, 136),  // #44dd88 - Teal
    (34, 204, 187),  // #22ccbb - Cyan-teal
    (0, 187, 204),   // #00bbcc - Cyan
    (0, 153, 204),   // #0099cc - Blue-cyan
    (51, 102, 187),  // #3366bb - Blue
    (102, 51, 153),  // #663399 - Violet
];

/// A placeholder widget for prototyping layouts.
///
/// Displays a colored background with a centered label.
/// Colors auto-cycle through a harmonious palette unless
/// overridden via CSS.
///
/// Supports three display variants:
/// - `Default`: Shows the label or widget ID (e.g., #p1)
/// - `Size`: Shows the widget's dimensions (e.g., 80 x 24)
/// - `Text`: Shows Lorem Ipsum text
///
/// Click the placeholder to cycle through variants.
pub struct Placeholder {
    label: Option<String>,
    palette_index: usize,
    style: ComputedStyle,
    inline_style: StyleOverride,
    dirty: bool,
    id: Option<String>,
    classes: Vec<String>,
    variant: PlaceholderVariant,
}

impl Placeholder {
    /// Create a new Placeholder with the given label.
    pub fn new() -> Self {
        let index = PLACEHOLDER_COUNTER.with(|c| {
            let current = c.get();
            c.set(current + 1);
            current
        });
        Self {
            label: None,
            palette_index: index % PALETTE.len(),
            style: ComputedStyle::default(),
            inline_style: StyleOverride::default(),
            dirty: true,
            id: None,
            classes: Vec::new(),
            variant: PlaceholderVariant::default(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the widget ID for CSS targeting and message tracking.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set CSS classes (space-separated).
    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        self.classes = classes
            .into()
            .split_whitespace()
            .map(String::from)
            .collect();
        self
    }

    /// Set the display variant.
    pub fn with_variant(mut self, variant: PlaceholderVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Get the current display variant.
    pub fn variant(&self) -> PlaceholderVariant {
        self.variant
    }

    /// Set the variant (for cycling on click).
    pub fn set_variant(&mut self, variant: PlaceholderVariant) {
        self.variant = variant;
        self.dirty = true;
    }

    /// Get the display content based on the current variant.
    fn get_display_content(&self, region: &Region) -> Option<String> {
        match self.variant {
            PlaceholderVariant::Default => {
                // Explicit label > #id > "Placeholder" (matching Python Textual)
                self.label
                    .clone()
                    .or_else(|| self.id.as_ref().map(|id| format!("#{}", id)))
                    .or_else(|| Some("Placeholder".to_string()))
            }
            PlaceholderVariant::Size => Some(format!("{} x {}", region.width, region.height)),
            PlaceholderVariant::Text => {
                // 5 paragraphs joined by double newlines (matching Python Textual)
                let paragraphs: Vec<&str> = (0..5).map(|_| LOREM_IPSUM_PARAGRAPH).collect();
                Some(paragraphs.join("\n\n"))
            }
        }
    }

    /// Word-wrap text to fit within a given width.
    /// Handles newlines in the input and wraps at word boundaries.
    fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();

        for paragraph in text.split('\n') {
            if paragraph.is_empty() {
                lines.push(String::new());
                continue;
            }

            let mut current_line = String::new();

            for word in paragraph.split_whitespace() {
                if current_line.is_empty() {
                    // First word on line
                    if word.len() > max_width {
                        // Word is longer than max_width, break it up
                        let mut remaining = word;
                        while remaining.len() > max_width {
                            lines.push(remaining[..max_width].to_string());
                            remaining = &remaining[max_width..];
                        }
                        current_line = remaining.to_string();
                    } else {
                        current_line = word.to_string();
                    }
                } else if current_line.len() + 1 + word.len() <= max_width {
                    // Word fits on current line with a space
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    // Word doesn't fit, start a new line
                    lines.push(current_line);
                    if word.len() > max_width {
                        // Word is longer than max_width, break it up
                        let mut remaining = word;
                        while remaining.len() > max_width {
                            lines.push(remaining[..max_width].to_string());
                            remaining = &remaining[max_width..];
                        }
                        current_line = remaining.to_string();
                    } else {
                        current_line = word.to_string();
                    }
                }
            }

            if !current_line.is_empty() {
                lines.push(current_line);
            }
        }

        lines
    }

    /// Get the background color, composited with inherited background as needed.
    fn effective_background(&self) -> RgbaColor {
        if self.style.background.is_some() {
            return self
                .style
                .effective_background()
                .unwrap_or_else(|| self.style.background.clone().unwrap());
        }

        let (r, g, b) = PALETTE[self.palette_index];
        let overlay = RgbaColor::rgba(r, g, b, 0.5);
        match &self.style.inherited_background {
            Some(bg) => overlay.blend_over(bg),
            None => overlay,
        }
    }

    /// Get the foreground color, respecting auto color and opacity.
    fn effective_foreground(&self, effective_bg: Option<&RgbaColor>) -> Option<RgbaColor> {
        if self.style.auto_color {
            effective_bg.map(|bg| {
                let ratio = self.style.color.as_ref().map(|c| c.a).unwrap_or(1.0);
                let contrasting = bg.get_contrasting_color(ratio);
                match &self.style.inherited_background {
                    Some(inherited_bg) => {
                        contrasting.blend_toward(inherited_bg, self.style.opacity)
                    }
                    None => contrasting.with_opacity(self.style.opacity),
                }
            })
        } else {
            match (&self.style.color, &self.style.inherited_background) {
                (Some(color), Some(bg)) => Some(color.blend_toward(bg, self.style.opacity)),
                (Some(color), None) => Some(color.with_opacity(self.style.opacity)),
                (None, Some(bg)) if self.style.opacity < 1.0 => {
                    let white = RgbaColor::rgba(255, 255, 255, 1.0);
                    Some(white.blend_toward(bg, self.style.opacity))
                }
                _ => None,
            }
        }
    }
}

impl<M> Widget<M> for Placeholder {
    fn default_css(&self) -> &'static str {
        // Match Python Textual: width fills, height unspecified (fills in horizontal, intrinsic in vertical)
        r#"
Placeholder {
    content-align: center middle;
    overflow: hidden;
    color: $text;
    width: 1fr;

    &:disabled {
        opacity: 0.7;
    }
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if self.style.visibility == Visibility::Hidden {
            return;
        }
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        let padding_top = self.style.padding.top.value as i32;
        let padding_right = self.style.padding.right.value as i32;
        let padding_bottom = self.style.padding.bottom.value as i32;
        let padding_left = self.style.padding.left.value as i32;

        let content_x = region.x + padding_left;
        let content_y = region.y + padding_top;
        let content_width = (region.width - padding_left - padding_right).max(0);
        let content_height = (region.height - padding_top - padding_bottom).max(0);

        let bg = self.effective_background();
        let fg = self
            .effective_foreground(Some(&bg))
            .unwrap_or_else(|| RgbaColor::rgb(255, 255, 255));

        // Fill background
        for y in region.y..(region.y + region.height) {
            for x in region.x..(region.x + region.width) {
                canvas.put_char(x, y, ' ', None, Some(bg.clone()), TextAttributes::default());
            }
        }

        // Get display content based on variant
        let display_content = self.get_display_content(&region);

        if let Some(content) = &display_content {
            if content_width <= 0 || content_height <= 0 {
                return;
            }
            match self.variant {
                PlaceholderVariant::Text => {
                    // Text variant: 1-cell padding on all sides, word wrap, block-centered
                    let padding = 1;
                    let content_x = content_x + padding;
                    let content_y = content_y + padding;
                    let content_width = (content_width - 2 * padding).max(1) as usize;
                    let content_height = (content_height - 2 * padding).max(1) as usize;

                    log::info!(
                        "Placeholder Text variant='#{}' content area={}x{} at ({},{})",
                        self.id.as_deref().unwrap_or(""),
                        content_width,
                        content_height,
                        content_x,
                        content_y
                    );

                    // Word-wrap the text into lines
                    let lines = Self::word_wrap(&content, content_width);

                    // Find the width of the text block (longest line among RENDERED lines only)
                    let rendered_lines: Vec<_> = lines.iter().take(content_height).collect();
                    let max_line_len =
                        rendered_lines.iter().map(|l| l.len()).max().unwrap_or(0) as i32;

                    // Calculate horizontal centering for the ENTIRE BLOCK
                    let gap = content_width as i32 - max_line_len;
                    let x_offset = (gap / 2).max(0);

                    // Calculate vertical centering
                    let total_lines = rendered_lines.len();
                    let y_offset = ((content_height as i32 - total_lines as i32) / 2).max(0);

                    // Render each line, left-aligned WITHIN the centered block
                    for (i, line) in rendered_lines.iter().enumerate() {
                        let y = content_y + y_offset + i as i32;

                        canvas.put_str(
                            content_x + x_offset, // Same X for ALL lines
                            y,
                            line,
                            Some(fg.clone()),
                            Some(bg.clone()),
                            TextAttributes::default(),
                        );
                    }
                }
                PlaceholderVariant::Size => {
                    // Size: center the label, bold text
                    let label_len = content.len() as i32;
                    let x = content_x + (content_width - label_len).max(0) / 2;
                    let y = content_y + (content_height - 1) / 2;

                    canvas.put_str(
                        x,
                        y,
                        content,
                        Some(fg),
                        Some(bg),
                        TextAttributes {
                            bold: true,
                            ..Default::default()
                        },
                    );
                }
                PlaceholderVariant::Default => {
                    // Default: center the label with word-wrapping if needed
                    let content_width = content_width.max(1) as usize;

                    // Word-wrap the label if it's wider than the container
                    let lines = Self::word_wrap(content, content_width);
                    let num_lines = lines.len() as i32;

                    // Find the width of the text block (longest line)
                    let max_line_len = lines.iter().map(|l| l.len()).max().unwrap_or(0) as i32;

                    // Calculate vertical centering
                    // Use (height - num_lines) / 2 to center the block
                    let y_offset = ((content_height - num_lines) / 2).max(0);

                    // Calculate horizontal centering for the block
                    let x_offset = ((content_width as i32 - max_line_len) / 2).max(0);

                    for (i, line) in lines.iter().enumerate() {
                        let y = content_y + y_offset + i as i32;
                        canvas.put_str(
                            content_x + x_offset,
                            y,
                            line,
                            Some(fg.clone()),
                            Some(bg.clone()),
                            TextAttributes::default(),
                        );
                    }
                }
            }
        }
    }

    fn desired_size(&self) -> Size {
        use tcss::types::Unit;

        // Calculate chrome (padding) for auto sizing
        // When height/width is auto, the widget should include padding in its desired size
        let padding_h =
            self.style.padding.left.value as u16 + self.style.padding.right.value as u16;
        let padding_v =
            self.style.padding.top.value as u16 + self.style.padding.bottom.value as u16;

        let content = self
            .get_display_content(&Region::default())
            .unwrap_or_default();
        let content_width = content
            .lines()
            .map(|line| line.chars().count() as u16)
            .max()
            .unwrap_or(0);
        let content_height = content.lines().count().max(1) as u16;

        // Check CSS dimensions - return u16::MAX for flexible units
        let width = if let Some(w) = &self.style.width {
            match w.unit {
                Unit::Cells => w.value as u16,
                Unit::Percent
                | Unit::ViewWidth
                | Unit::ViewHeight
                | Unit::Fraction
                | Unit::Width
                | Unit::Height => {
                    u16::MAX // Signal "fill available space"
                }
                Unit::Auto => content_width + padding_h, // Content width + horizontal padding
            }
        } else {
            content_width + padding_h // Default width + padding
        };

        // For height, match Python Textual behavior:
        // - auto returns content height (1 for label) + padding
        // - This allows vertical layout to size based on actual content
        let height = if let Some(h) = &self.style.height {
            match h.unit {
                Unit::Cells => h.value as u16,
                Unit::Percent
                | Unit::ViewWidth
                | Unit::ViewHeight
                | Unit::Fraction
                | Unit::Width
                | Unit::Height => {
                    u16::MAX // Signal "fill available space"
                }
                Unit::Auto => content_height + padding_v, // Content height + vertical padding
            }
        } else {
            content_height + padding_v // Default: content height + padding
        };

        Size::new(width, height)
    }

    fn intrinsic_height_for_width(&self, width: u16) -> u16 {
        let padding_top = self.style.padding.top.value as u16;
        let padding_bottom = self.style.padding.bottom.value as u16;
        let padding_left = self.style.padding.left.value as u16;
        let padding_right = self.style.padding.right.value as u16;

        let content_width = width.saturating_sub(padding_left + padding_right).max(1);
        let content = self
            .get_display_content(&Region::default())
            .unwrap_or_default();
        let lines = Self::word_wrap(&content, content_width as usize);
        let content_height = lines.len().max(1) as u16;

        content_height + padding_top + padding_bottom
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Placeholder",
            type_names: vec!["Placeholder", "Widget", "DOMNode"],
            id: self.id.clone(),
            classes: self.classes.clone(),
            states: WidgetStates::empty(),
        }
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inline_style = style;
        self.dirty = true;
    }

    fn inline_style(&self) -> Option<&StyleOverride> {
        if self.inline_style.is_empty() {
            None
        } else {
            Some(&self.inline_style)
        }
    }

    fn clear_inline_style(&mut self) {
        self.inline_style = StyleOverride::default();
        self.dirty = true;
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

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        // Check if click is within our region
        let col = event.column as i32;
        let row = event.row as i32;
        let in_bounds = col >= region.x
            && col < region.x + region.width
            && row >= region.y
            && row < region.y + region.height;

        if let MouseEventKind::Down(_) = event.kind {
            if in_bounds {
                // Cycle to next variant
                self.variant = self.variant.next();
                self.dirty = true;
            }
        }

        None
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
        self.classes = classes.split_whitespace().map(String::from).collect();
        self.dirty = true;
    }

    fn classes(&self) -> Vec<String> {
        self.classes.clone()
    }
}
