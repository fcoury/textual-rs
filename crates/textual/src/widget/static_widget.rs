//! Static widget for displaying text content.
//!
//! Static is the base widget for displaying text. It handles:
//! - Content storage and rendering
//! - Visual caching for performance
//! - Content alignment (horizontal and vertical)
//! - The `update()` method for dynamic content
//!
//! Label and other text widgets wrap Static to add specialized behavior.

use std::marker::PhantomData;

use tcss::types::{AlignHorizontal, AlignVertical};
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};
use unicode_width::UnicodeWidthStr;

use crate::content::Content;
use crate::render_cache::RenderCache;
use crate::segment::Style;
use crate::strip::Strip;
use crate::{Canvas, KeyCode, MouseEvent, Region, Size, VisualType, Widget};

/// A widget that displays static or updateable text content.
///
/// Static is the foundation for text-displaying widgets. It handles content
/// storage, rendering with borders and alignment, and provides an `update()`
/// method for changing content dynamically.
///
/// # Example
///
/// ```ignore
/// use textual::Static;
///
/// // Create a static text display
/// let status: Static<MyMessage> = Static::new("Ready");
///
/// // With builder methods
/// let label: Static<MyMessage> = Static::new("Hello")
///     .with_id("greeting")
///     .with_classes("highlight bold");
///
/// // Update content later
/// status.update("Processing...");
/// ```
#[derive(Debug, Clone)]
pub struct Static<M> {
    content: VisualType,
    expand: bool,
    shrink: bool,
    markup: bool,
    name: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    disabled: bool,
    style: ComputedStyle,
    dirty: bool,
    _phantom: PhantomData<M>,
}

impl<M> Default for Static<M> {
    fn default() -> Self {
        Self {
            content: VisualType::Text(String::new()),
            expand: false,
            shrink: false,
            markup: false,
            name: None,
            id: None,
            classes: Vec::new(),
            disabled: false,
            style: ComputedStyle::default(),
            dirty: true,
            _phantom: PhantomData,
        }
    }
}

impl<M> Static<M> {
    /// Create a new Static widget with the given text content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: VisualType::from(content.into()),
            ..Default::default()
        }
    }

    /// Update the content and mark the widget as dirty.
    ///
    /// This is the primary way to change a Static's content after creation.
    pub fn update(&mut self, content: impl Into<String>) {
        self.content = VisualType::from(content.into());
        self.dirty = true;
    }

    /// Set the widget ID for CSS targeting.
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

    /// Add a single CSS class.
    pub fn add_class(&mut self, class: impl Into<String>) {
        self.classes.push(class.into());
    }

    /// Set whether the widget expands to fill available space.
    pub fn with_expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Set whether the widget shrinks to fit content.
    pub fn with_shrink(mut self, shrink: bool) -> Self {
        self.shrink = shrink;
        self
    }

    /// Set whether content should be parsed as markup.
    pub fn with_markup(mut self, markup: bool) -> Self {
        self.markup = markup;
        self
    }

    /// Set the widget name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the disabled state.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Get the text content.
    fn text(&self) -> &str {
        match &self.content {
            VisualType::Text(s) => s,
        }
    }

    /// Convert ComputedStyle to rendering Style.
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

    /// Apply content alignment to lines.
    fn align_content(
        &self,
        lines: &[Strip],
        width: usize,
        height: usize,
        style: Style,
    ) -> Vec<Strip> {
        if width == 0 || height == 0 {
            return vec![];
        }

        let h_align = self.style.content_align_horizontal;
        let v_align = self.style.content_align_vertical;

        // Calculate vertical offset
        let content_height = lines.len();
        let v_offset = match v_align {
            AlignVertical::Top => 0,
            AlignVertical::Middle => height.saturating_sub(content_height) / 2,
            AlignVertical::Bottom => height.saturating_sub(content_height),
        };

        // Build aligned lines with vertical padding
        let mut result = Vec::with_capacity(height);
        let pad_style = Some(style);

        // Add top padding
        for _ in 0..v_offset {
            result.push(Strip::blank(width, pad_style.clone()));
        }

        // Add content lines with horizontal alignment
        for line in lines.iter().take(height - v_offset) {
            let aligned = match h_align {
                AlignHorizontal::Left => line.adjust_cell_length(width, pad_style.clone()),
                AlignHorizontal::Center => {
                    let line_len = line.cell_length();
                    let left_pad = width.saturating_sub(line_len) / 2;
                    if left_pad > 0 {
                        let left = Strip::blank(left_pad, pad_style.clone());
                        Strip::join(vec![left, line.clone()])
                            .adjust_cell_length(width, pad_style.clone())
                    } else {
                        line.adjust_cell_length(width, pad_style.clone())
                    }
                }
                AlignHorizontal::Right => {
                    let line_len = line.cell_length();
                    let left_pad = width.saturating_sub(line_len);
                    if left_pad > 0 {
                        let left = Strip::blank(left_pad, pad_style.clone());
                        Strip::join(vec![left, line.clone()])
                            .adjust_cell_length(width, pad_style.clone())
                    } else {
                        line.adjust_cell_length(width, pad_style.clone())
                    }
                }
            };
            result.push(aligned);
        }

        // Add bottom padding
        while result.len() < height {
            result.push(Strip::blank(width, pad_style.clone()));
        }

        result
    }
}

impl<M> Widget<M> for Static<M> {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        let width = region.width as usize;
        let height = region.height as usize;

        // 1. Create rendering style from computed CSS
        let style = self.rendering_style();

        // 2. Create render cache for border handling
        let cache = RenderCache::new(&self.style);
        let (inner_width, inner_height) = cache.inner_size(width, height);

        // 3. Parse content into strips
        let content = Content::new(self.text()).with_style(style.clone());
        let lines = if inner_width > 0 {
            content.wrap(inner_width)
        } else {
            vec![]
        };

        // 4. Apply content alignment
        let aligned_lines = self.align_content(&lines, inner_width, inner_height, style.clone());

        // 5. Calculate content region boundaries (accounting for borders and padding)
        let border_offset = if cache.has_border() { 1 } else { 0 };
        let content_start = border_offset + cache.padding_top();
        let content_end = height.saturating_sub(border_offset + cache.padding_bottom());

        // 6. Render each line with borders and padding
        for y in 0..height {
            // Determine if this row should have content or be blank (padding row)
            let content_line = if y >= content_start && y < content_end {
                let content_y = y - content_start;
                aligned_lines.get(content_y)
            } else if y >= border_offset && y < height - border_offset {
                // This is a padding row (inside borders but outside content area)
                None
            } else {
                // Border row - no content
                None
            };

            let strip = cache.render_line(y, height, width, content_line, None);

            // 7. Apply tint as post-processing (tints both fg and bg colors)
            let strip = if let Some(tint) = &self.style.tint {
                strip.apply_tint(tint)
            } else {
                strip
            };

            canvas.render_strip(&strip, region.x, region.y + y as i32);
        }
    }

    fn desired_size(&self) -> Size {
        let text = self.text();
        // Width = cell width (Unicode-aware), height = line count
        let width = text.lines().map(|l| l.width()).max().unwrap_or(0);
        let height = text.lines().count().max(1);
        Size::new(width as u16, height as u16)
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut states = WidgetStates::empty();
        if self.disabled {
            states |= WidgetStates::DISABLED;
        }
        WidgetMeta {
            type_name: "Static".to_string(),
            id: self.id.clone(),
            classes: self.classes.clone(),
            states,
        }
    }

    fn get_state(&self) -> WidgetStates {
        let mut states = WidgetStates::empty();
        if self.disabled {
            states |= WidgetStates::DISABLED;
        }
        states
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

    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
        self.dirty = true;
    }

    fn on_event(&mut self, _key: KeyCode) -> Option<M> {
        None
    }

    fn on_mouse(&mut self, _event: MouseEvent, _region: Region) -> Option<M> {
        None
    }
}
