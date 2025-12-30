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
    /// Title displayed in the top border (supports markup).
    border_title: Option<String>,
    /// Subtitle displayed in the bottom border (supports markup).
    border_subtitle: Option<String>,
    _phantom: PhantomData<M>,
}

impl<M> Default for Static<M> {
    fn default() -> Self {
        Self {
            content: VisualType::Text(String::new()),
            expand: false,
            shrink: false,
            markup: true,
            name: None,
            id: None,
            classes: Vec::new(),
            disabled: false,
            style: ComputedStyle::default(),
            dirty: true,
            border_title: None,
            border_subtitle: None,
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

    /// Set the border title (displayed in the top border).
    ///
    /// The title supports markup for styling (e.g., `[b]Bold Title[/]`).
    pub fn with_border_title(mut self, title: impl Into<String>) -> Self {
        self.border_title = Some(title.into());
        self
    }

    /// Set the border subtitle (displayed in the bottom border).
    ///
    /// The subtitle supports markup for styling (e.g., `[i]Italic Subtitle[/]`).
    pub fn with_border_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.border_subtitle = Some(subtitle.into());
        self
    }

    /// Set the border title at runtime.
    pub fn set_border_title(&mut self, title: impl Into<String>) {
        self.border_title = Some(title.into());
        self.dirty = true;
    }

    /// Set the border subtitle at runtime.
    pub fn set_border_subtitle(&mut self, subtitle: impl Into<String>) {
        self.border_subtitle = Some(subtitle.into());
        self.dirty = true;
    }

    /// Get the border title.
    pub fn border_title(&self) -> Option<&str> {
        self.border_title.as_deref()
    }

    /// Get the border subtitle.
    pub fn border_subtitle(&self) -> Option<&str> {
        self.border_subtitle.as_deref()
    }

    /// Get the text content.
    fn text(&self) -> &str {
        match &self.content {
            VisualType::Text(s) => s,
        }
    }

    /// Convert ComputedStyle to rendering Style.
    fn rendering_style(&self) -> Style {
        // Compute effective foreground color
        let fg = if self.style.auto_color {
            // For auto color, compute contrasting color against effective background
            self.effective_background().map(|bg| {
                // Get contrast ratio from the color's alpha (e.g., "auto 90%" has a=0.9)
                let ratio = self.style.color.as_ref().map(|c| c.a).unwrap_or(1.0);
                bg.get_contrasting_color(ratio)
            })
        } else {
            self.style.color.clone()
        };

        Style {
            fg,
            bg: self.effective_background(),
            bold: self.style.text_style.bold,
            dim: self.style.text_style.dim,
            italic: self.style.text_style.italic,
            underline: self.style.text_style.underline,
            strike: self.style.text_style.strike,
            reverse: self.style.text_style.reverse,
        }
    }

    /// Get the effective background color (with alpha compositing and background-tint applied).
    /// Falls back to inherited background from parent if this widget is transparent.
    fn effective_background(&self) -> Option<tcss::types::RgbaColor> {
        match (&self.style.background, &self.style.inherited_background) {
            (Some(bg), Some(inherited)) if bg.a < 1.0 => {
                // Composite semi-transparent background over inherited
                let composited = bg.blend_over(inherited);
                // Then apply tint if present
                match &self.style.background_tint {
                    Some(tint) => Some(composited.tint(tint)),
                    None => Some(composited),
                }
            }
            (Some(bg), _) => {
                // Opaque background or no inherited - just apply tint
                match &self.style.background_tint {
                    Some(tint) => Some(bg.tint(tint)),
                    None => Some(bg.clone()),
                }
            }
            (None, Some(inherited)) => {
                // No background specified, inherit from parent
                Some(inherited.clone())
            }
            (None, None) => None,
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

impl<M: 'static> Widget<M> for Static<M> {
    fn default_css(&self) -> &'static str {
        r#"
Static {
    height: auto;
}
"#
    }

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

        // 3. Parse content into strips (with markup if enabled)
        let content = if self.markup {
            Content::from_markup(self.text())
                .unwrap_or_else(|_| Content::new(self.text()))
                .with_style(style.clone())
        } else {
            Content::new(self.text()).with_style(style.clone())
        };
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

        // 6. Parse border titles as markup (only if we have a border)
        // Title/subtitle color inheritance:
        // 1. border_title_color / border_subtitle_color if explicitly set
        // 2. border.top.color / border.bottom.color (border color)
        // 3. style.color (text color) as fallback
        let title_fg = self.style.border_title_color.clone()
            .or_else(|| self.style.border.top.color.clone())
            .or_else(|| self.style.color.clone());
        let subtitle_fg = self.style.border_subtitle_color.clone()
            .or_else(|| self.style.border.bottom.color.clone())
            .or_else(|| self.style.color.clone());

        // Use border-title-background if set, otherwise fall back to widget background
        let title_bg = self.style.border_title_background.clone()
            .or_else(|| self.style.background.clone());
        let subtitle_bg = self.style.border_subtitle_background.clone()
            .or_else(|| self.style.background.clone());

        let title_style = Style {
            fg: title_fg,
            bg: title_bg,
            bold: self.style.border_title_style.bold,
            dim: self.style.border_title_style.dim,
            italic: self.style.border_title_style.italic,
            underline: self.style.border_title_style.underline,
            strike: self.style.border_title_style.strike,
            reverse: self.style.border_title_style.reverse,
        };
        let subtitle_style = Style {
            fg: subtitle_fg,
            bg: subtitle_bg,
            bold: self.style.border_subtitle_style.bold,
            dim: self.style.border_subtitle_style.dim,
            italic: self.style.border_subtitle_style.italic,
            underline: self.style.border_subtitle_style.underline,
            strike: self.style.border_subtitle_style.strike,
            reverse: self.style.border_subtitle_style.reverse,
        };

        // For border titles, don't wrap - let render_label_in_row handle truncation with ellipsis
        let title_strip = if cache.has_border() {
            self.border_title.as_ref().and_then(|t| {
                if t.is_empty() {
                    None
                } else {
                    let content = Content::from_markup(t)
                        .unwrap_or_else(|_| Content::new(t))
                        .with_style(title_style.clone());
                    // Use very large width to prevent word-wrapping; truncation happens in render_label_in_row
                    content.wrap(usize::MAX).into_iter().next()
                }
            })
        } else {
            None
        };

        let subtitle_strip = if cache.has_border() {
            self.border_subtitle.as_ref().and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    let content = Content::from_markup(s)
                        .unwrap_or_else(|_| Content::new(s))
                        .with_style(subtitle_style.clone());
                    // Use very large width to prevent word-wrapping; truncation happens in render_label_in_row
                    content.wrap(usize::MAX).into_iter().next()
                }
            })
        } else {
            None
        };

        // 7. Render each line with borders and padding
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

            let strip = cache.render_line(
                y,
                height,
                width,
                content_line,
                title_strip.as_ref(),
                subtitle_strip.as_ref(),
            );

            // 8. Apply tint as post-processing (tints both fg and bg colors)
            let strip = if let Some(tint) = &self.style.tint {
                strip.apply_tint(tint)
            } else {
                strip
            };

            canvas.render_strip(&strip, region.x, region.y + y as i32);
        }
    }

    fn desired_size(&self) -> Size {
        // Check CSS dimensions first, fall back to content size
        // Account for box-sizing: border-box vs content-box
        let style = self.get_style();
        use tcss::types::border::BorderKind;
        use tcss::types::BoxSizing;

        // Calculate border contribution (each visible edge adds 1 cell)
        let has_top = !matches!(style.border.top.kind, BorderKind::None | BorderKind::Hidden);
        let has_bottom = !matches!(style.border.bottom.kind, BorderKind::None | BorderKind::Hidden);
        let has_left = !matches!(style.border.left.kind, BorderKind::None | BorderKind::Hidden);
        let has_right = !matches!(style.border.right.kind, BorderKind::None | BorderKind::Hidden);

        let border_width = (if has_left { 1 } else { 0 }) + (if has_right { 1 } else { 0 });
        let border_height = (if has_top { 1 } else { 0 }) + (if has_bottom { 1 } else { 0 });

        // Calculate padding contribution
        let padding_width = style.padding.left.value as u16 + style.padding.right.value as u16;
        let padding_height = style.padding.top.value as u16 + style.padding.bottom.value as u16;

        // Chrome (border + padding) to add for content-box
        let chrome_width = border_width + padding_width;
        let chrome_height = border_height + padding_height;

        let width = if let Some(w) = &style.width {
            use tcss::types::Unit;
            match w.unit {
                Unit::Cells => {
                    // Apply box-sizing: border-box returns as-is, content-box adds chrome
                    match style.box_sizing {
                        BoxSizing::BorderBox => w.value as u16,
                        BoxSizing::ContentBox => w.value as u16 + chrome_width,
                    }
                }
                // For other units (auto, percent, fr), fall back to content width + chrome
                _ => {
                    let text = self.text();
                    let content_width = text.lines().map(|l| l.width()).max().unwrap_or(0) as u16;
                    content_width + chrome_width
                }
            }
        } else {
            let text = self.text();
            let content_width = text.lines().map(|l| l.width()).max().unwrap_or(0) as u16;
            content_width + chrome_width
        };

        let height = if let Some(h) = &style.height {
            use tcss::types::Unit;
            match h.unit {
                Unit::Cells => {
                    // Apply box-sizing: border-box returns as-is, content-box adds chrome
                    match style.box_sizing {
                        BoxSizing::BorderBox => h.value as u16,
                        BoxSizing::ContentBox => h.value as u16 + chrome_height,
                    }
                }
                // For other units (auto, percent, fr), fall back to content height + chrome
                _ => {
                    let text = self.text();
                    let content_height = text.lines().count().max(1) as u16;
                    content_height + chrome_height
                }
            }
        } else {
            let text = self.text();
            let content_height = text.lines().count().max(1) as u16;
            content_height + chrome_height
        };

        Size::new(width, height)
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

    fn set_border_title(&mut self, title: &str) {
        self.border_title = Some(title.to_string());
        self.dirty = true;
    }

    fn set_border_subtitle(&mut self, subtitle: &str) {
        self.border_subtitle = Some(subtitle.to_string());
        self.dirty = true;
    }

    fn border_title(&self) -> Option<&str> {
        self.border_title.as_deref()
    }

    fn border_subtitle(&self) -> Option<&str> {
        self.border_subtitle.as_deref()
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
        self.classes = classes.split_whitespace().map(String::from).collect();
        self.dirty = true;
    }

    fn classes(&self) -> Vec<String> {
        self.classes.clone()
    }
}
