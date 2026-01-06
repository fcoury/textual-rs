//! Loading indicator widget (spinner).

use std::marker::PhantomData;

use tcss::types::Visibility;
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region};
use crate::{KeyCode, MouseEvent, Size, Widget};

const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// A simple loading spinner.
pub struct LoadingIndicator<M> {
    frame: usize,
    visible: bool,
    style: ComputedStyle,
    inline_style: StyleOverride,
    dirty: bool,
    id: Option<String>,
    classes: Vec<String>,
    _phantom: PhantomData<M>,
}

impl<M> LoadingIndicator<M> {
    pub fn new() -> Self {
        Self {
            frame: 0,
            visible: true,
            style: ComputedStyle::default(),
            inline_style: StyleOverride::default(),
            dirty: true,
            id: None,
            classes: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        self.classes = classes
            .into()
            .split_whitespace()
            .map(String::from)
            .collect();
        self
    }

    pub fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.dirty = true;
        }
    }

    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
        self.dirty = true;
    }
}

impl<M> Default for LoadingIndicator<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: 'static> Widget<M> for LoadingIndicator<M> {
    fn default_css(&self) -> &'static str {
        r#"
LoadingIndicator {
    height: auto;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if !self.visible || self.style.visibility == Visibility::Hidden {
            return;
        }
        let frame = SPINNER_FRAMES[self.frame];
        canvas.put_char(
            region.x,
            region.y,
            frame,
            self.style.color.clone(),
            self.style.background.clone(),
            Default::default(),
        );
    }

    fn desired_size(&self) -> Size {
        Size::new(1, 1)
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "LoadingIndicator",
            type_names: vec!["LoadingIndicator", "Widget", "DOMNode"],
            id: self.id.clone(),
            classes: self.classes.clone(),
            states: self.get_state(),
        }
    }

    fn get_state(&self) -> WidgetStates {
        if self.is_disabled() {
            WidgetStates::DISABLED
        } else {
            WidgetStates::empty()
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

    fn on_event(&mut self, _key: KeyCode) -> Option<M> {
        None
    }

    fn on_mouse(&mut self, _event: MouseEvent, _region: Region) -> Option<M> {
        None
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn is_loading(&self) -> bool {
        false
    }

    fn set_loading(&mut self, _loading: bool) {}

    fn is_disabled(&self) -> bool {
        false
    }

    fn set_disabled(&mut self, _disabled: bool) {}

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    fn type_name(&self) -> &'static str {
        "LoadingIndicator"
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
