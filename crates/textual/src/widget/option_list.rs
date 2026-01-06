//! OptionList widget for displaying selectable lists.

use std::marker::PhantomData;

use tcss::types::Visibility;
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region};
use crate::widget::static_widget::Static;
use crate::{KeyCode, MouseEvent, Size, Widget};

fn escape_markup(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '[' | ']' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// A simple list widget that highlights the selected item.
pub struct OptionList<M> {
    inner: Static<M>,
    items: Vec<String>,
    selected: usize,
    focused: bool,
    dirty: bool,
    allow_markup: bool,
    last_width: Option<u16>,
    _phantom: PhantomData<M>,
}

impl<M: 'static> OptionList<M> {
    pub fn new(items: Vec<String>) -> Self {
        let mut inner = Static::new("");
        inner = inner.with_markup(true);
        let mut list = Self {
            inner,
            items,
            selected: 0,
            focused: false,
            dirty: true,
            allow_markup: false,
            last_width: None,
            _phantom: PhantomData,
        };
        list.refresh_display();
        list
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner = self.inner.with_id(id);
        self
    }

    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        self.inner = self.inner.with_classes(classes);
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.inner = self.inner.with_name(name);
        self
    }

    pub fn with_items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self.selected = 0;
        self.refresh_display();
        self
    }

    /// Allow items to include markup tags (default: false).
    pub fn with_markup(mut self, allow: bool) -> Self {
        self.allow_markup = allow;
        self.refresh_display();
        self
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        if self.items.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.items.len() - 1);
        }
        self.refresh_display();
    }

    /// Toggle whether item strings contain markup.
    pub fn set_markup(&mut self, allow: bool) {
        if self.allow_markup != allow {
            self.allow_markup = allow;
            self.refresh_display();
        }
    }

    pub fn set_selected(&mut self, selected: usize) {
        if self.items.is_empty() {
            self.selected = 0;
            self.refresh_display();
            return;
        }
        let clamped = selected.min(self.items.len() - 1);
        if self.selected != clamped {
            self.selected = clamped;
            self.refresh_display();
        }
    }

    pub fn selected(&self) -> Option<&str> {
        self.items.get(self.selected).map(|s| s.as_str())
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    fn refresh_display(&mut self) {
        let mut lines = Vec::with_capacity(self.items.len().max(1));
        let highlight_style = self.highlight_style_markup();
        for (index, item) in self.items.iter().enumerate() {
            let content = if self.allow_markup {
                item.clone()
            } else {
                escape_markup(item)
            };
            if index == self.selected {
                // Apply highlight style - let the renderer handle background fill
                if let Some(style) = &highlight_style {
                    lines.push(format!("[{}]{}[/]", style, content));
                } else {
                    lines.push(format!("[reverse]{}[/]", content));
                }
            } else {
                lines.push(content);
            }
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        let content = lines.join("\n");
        self.inner.update(content);
        self.dirty = false;
    }

    fn highlight_style_markup(&self) -> Option<String> {
        let style = self.inner.get_style();
        let link = &style.link;

        let has_link_style = link.color.is_some()
            || link.background.is_some()
            || !link.style.is_none()
            || link.color_hover.is_some()
            || link.background_hover.is_some()
            || !link.style_hover.is_none();
        if !has_link_style {
            return None;
        }

        let mut parts: Vec<String> = Vec::new();

        if link.style.bold {
            parts.push("bold".to_string());
        }
        if link.style.dim {
            parts.push("dim".to_string());
        }
        if link.style.italic {
            parts.push("italic".to_string());
        }
        if link.style.underline {
            parts.push("underline".to_string());
        }
        if link.style.strike {
            parts.push("strike".to_string());
        }
        if link.style.reverse {
            parts.push("reverse".to_string());
        }
        if link.style.blink {
            parts.push("blink".to_string());
        }

        if let Some(fg) = link.color.clone().or_else(|| style.color.clone()) {
            parts.push(color_to_markup(&fg));
        }

        if let Some(mut bg) = link.background.clone() {
            if bg.a < 1.0 {
                if let Some(base) = style.effective_background() {
                    bg = bg.blend_over(&base);
                }
            }
            parts.push(format!("on {}", color_to_markup(&bg)));
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" "))
        }
    }
}

fn color_to_markup(color: &tcss::types::RgbaColor) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
}

impl<M: 'static> Widget<M> for OptionList<M> {
    fn default_css(&self) -> &'static str {
        r#"
OptionList {
    height: auto;
    text-wrap: nowrap;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if self.get_style().visibility == Visibility::Hidden {
            return;
        }
        self.inner.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn intrinsic_height_for_width(&self, width: u16) -> u16 {
        self.inner.intrinsic_height_for_width(width)
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        if self.is_disabled() || !self.is_visible() {
            return None;
        }
        match key {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.refresh_display();
                }
            }
            KeyCode::Down => {
                if !self.items.is_empty() && self.selected + 1 < self.items.len() {
                    self.selected += 1;
                    self.refresh_display();
                }
            }
            _ => {}
        }
        None
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.inner.on_mouse(event, region)
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "OptionList";
        meta.type_names = vec!["OptionList", "Static", "Widget", "DOMNode"];
        meta.states = self.get_state();
        meta
    }

    fn get_state(&self) -> WidgetStates {
        let mut states = WidgetStates::empty();
        if self.focused {
            states |= WidgetStates::FOCUS;
        }
        if self.is_disabled() {
            states |= WidgetStates::DISABLED;
        }
        states
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.inner.set_style(style);
        self.refresh_display();
    }

    fn get_style(&self) -> ComputedStyle {
        self.inner.get_style()
    }

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inner.set_inline_style(style);
        self.dirty = true;
    }

    fn inline_style(&self) -> Option<&StyleOverride> {
        self.inner.inline_style()
    }

    fn clear_inline_style(&mut self) {
        self.inner.clear_inline_style();
        self.dirty = true;
    }

    fn is_dirty(&self) -> bool {
        self.dirty || self.inner.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.inner.mark_dirty();
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
        self.inner.mark_clean();
    }

    fn is_focusable(&self) -> bool {
        self.is_visible() && !self.is_disabled()
    }

    fn set_focus(&mut self, is_focused: bool) {
        if self.focused != is_focused {
            self.focused = is_focused;
            self.refresh_display();
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.inner.set_visible(visible);
    }

    fn is_loading(&self) -> bool {
        self.inner.is_loading()
    }

    fn set_loading(&mut self, loading: bool) {
        self.inner.set_loading(loading);
    }

    fn is_disabled(&self) -> bool {
        self.inner.is_disabled()
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.inner.set_disabled(disabled);
    }

    fn handle_message(&mut self, envelope: &mut crate::MessageEnvelope<M>) -> Option<M> {
        self.inner.handle_message(envelope)
    }

    fn id(&self) -> Option<&str> {
        self.inner.id()
    }

    fn type_name(&self) -> &'static str {
        "OptionList"
    }

    fn on_resize(&mut self, size: Size) {
        if self.last_width != Some(size.width) {
            self.last_width = Some(size.width);
            self.refresh_display();
        }
        self.inner.on_resize(size);
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        self.inner.for_each_child(f);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn add_class(&mut self, class: &str) {
        self.inner.add_class(class);
    }

    fn remove_class(&mut self, class: &str) {
        self.inner.remove_class(class);
    }

    fn has_class(&self, class: &str) -> bool {
        self.inner.has_class(class)
    }

    fn set_classes(&mut self, classes: &str) {
        self.inner.set_classes(classes);
    }

    fn classes(&self) -> Vec<String> {
        self.inner.classes()
    }
}
