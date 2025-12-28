use tcss::{ComputedStyle, WidgetMeta, WidgetStates};
use tcss::types::RgbaColor;

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

    /// Get the effective foreground color (from CSS or default).
    fn effective_foreground(&self) -> Option<RgbaColor> {
        self.style.color.clone()
    }

    /// Get the effective background color (from CSS or default).
    fn effective_background(&self) -> Option<RgbaColor> {
        self.style.background.clone()
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<M> Widget<M> for Label {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        let text = self.text();
        let fg = self.effective_foreground();
        let bg = self.effective_background();

        // Fill background if specified
        if let Some(ref bg_color) = bg {
            for y in region.y..(region.y + region.height) {
                for x in region.x..(region.x + region.width) {
                    canvas.put_char(x, y, ' ', None, Some(bg_color.clone()));
                }
            }
        }

        // Render text at top-left of region
        // TODO: Support text-align for horizontal alignment
        // TODO: Support multi-line text with word wrapping
        let x = region.x;
        let y = region.y;

        // Truncate text if it exceeds region width
        let max_chars = region.width as usize;
        let display_text: String = text.chars().take(max_chars).collect();

        canvas.put_str(x, y, &display_text, fg, bg);
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
