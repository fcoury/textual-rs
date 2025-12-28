use tcss::types::{AlignHorizontal, AlignVertical};
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

use crate::content::Content;
use crate::render_cache::RenderCache;
use crate::segment::Style;
use crate::strip::Strip;
use crate::{Canvas, KeyCode, MouseEvent, Region, Size, VisualType, Widget};

#[derive(Debug, Clone)]
pub enum LabelVariant {
    Success,
    Error,
    Warning,
    Primary,
    Secondary,
    Accent,
}

#[derive(Debug, Clone)]
pub struct Label {
    content: VisualType,
    variant: Option<LabelVariant>,
    expand: bool,
    shrink: bool,
    markup: bool,
    name: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    disabled: bool,
    style: ComputedStyle,
    dirty: bool,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            content: VisualType::Text(String::new()),
            variant: None,
            expand: false,
            shrink: false,
            markup: false,
            name: None,
            id: None,
            classes: Vec::new(),
            disabled: false,
            style: ComputedStyle::default(),
            dirty: true,
        }
    }
}

impl Label {
    pub fn new<S: Into<String>>(content: S) -> Self {
        Self {
            content: VisualType::from(content.into()),
            ..Default::default()
        }
    }

    pub fn variant(mut self, variant: LabelVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    pub fn shrink(mut self, shrink: bool) -> Self {
        self.shrink = shrink;
        self
    }

    pub fn markup(mut self, markup: bool) -> Self {
        self.markup = markup;
        self
    }

    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn classes<S: Into<String>>(mut self, classes: S) -> Self {
        self.classes = classes.into().split_whitespace().map(String::from).collect();
        self
    }

    /// Get the text content of the label.
    fn text(&self) -> &str {
        match &self.content {
            VisualType::Text(s) => s,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
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
                        Strip::join(vec![left, line.clone()]).adjust_cell_length(width, pad_style.clone())
                    } else {
                        line.adjust_cell_length(width, pad_style.clone())
                    }
                }
                AlignHorizontal::Right => {
                    let line_len = line.cell_length();
                    let left_pad = width.saturating_sub(line_len);
                    if left_pad > 0 {
                        let left = Strip::blank(left_pad, pad_style.clone());
                        Strip::join(vec![left, line.clone()]).adjust_cell_length(width, pad_style.clone())
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

impl<M> Widget<M> for Label {
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

        // 5. Render each line with borders
        for y in 0..height {
            // Get content line index (accounting for borders)
            let content_y = if cache.has_border() { y.saturating_sub(1) } else { y };
            let content_line = aligned_lines.get(content_y);

            let strip = cache.render_line(y, height, width, content_line, None);
            canvas.render_strip(&strip, region.x, region.y + y as i32);
        }
    }

    fn desired_size(&self) -> Size {
        let text = self.text();
        // For now: width = text length, height = 1 line
        // TODO: Support multi-line content
        Size::new(text.len() as u16, 1)
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut states = WidgetStates::empty();
        if self.disabled {
            states |= WidgetStates::DISABLED;
        }
        WidgetMeta {
            type_name: "Label".to_string(),
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
