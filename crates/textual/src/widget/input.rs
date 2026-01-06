//! Input widget for editable text.
//!
//! This is a minimal single-line input widget used by the command palette.

use std::marker::PhantomData;

use tcss::types::Visibility;
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region};
use crate::grapheme::{grapheme_byte_index, grapheme_byte_range, grapheme_count, graphemes};
use crate::widget::static_widget::Static;
use crate::{KeyCode, MouseEvent, Size, Widget};

/// Escape markup control characters so user input is rendered literally.
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

/// A single-line input widget.
pub struct Input<M> {
    inner: Static<M>,
    value: String,
    placeholder: Option<String>,
    cursor: usize,
    focused: bool,
    dirty: bool,
    _phantom: PhantomData<M>,
}

impl<M> Input<M> {
    pub fn new() -> Self {
        let mut inner = Static::new("");
        inner = inner.with_markup(true);
        Self {
            inner,
            value: String::new(),
            placeholder: None,
            cursor: 0,
            focused: false,
            dirty: true,
            _phantom: PhantomData,
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self.refresh_display();
        self
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

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.with_disabled(disabled);
        self
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.cursor.min(grapheme_count(&self.value));
        self.refresh_display();
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = Some(placeholder.into());
        self.refresh_display();
    }

    pub fn clear_placeholder(&mut self) {
        self.placeholder = None;
        self.refresh_display();
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor.min(grapheme_count(&self.value));
        self.refresh_display();
    }

    fn refresh_display(&mut self) {
        let display = self.build_display();
        self.inner.update(display);
        self.dirty = false;
    }

    fn build_display(&self) -> String {
        // Cursor style: light background with dark text (inverted from default)
        let cursor_style = "#121212 on #e0e0e0";
        // Placeholder style: muted gray text
        let placeholder_style = "#6d7479";

        if self.value.is_empty() {
            if let Some(placeholder) = &self.placeholder {
                if !self.focused {
                    return format!("[{placeholder_style}]{}[/]", escape_markup(placeholder));
                }
                if placeholder.is_empty() {
                    return format!("[{cursor_style}] [/]");
                }
                let mut placeholder_graphemes = graphemes(placeholder);
                let first = placeholder_graphemes.next().unwrap_or(" ");
                let rest: String = placeholder_graphemes.collect();
                return format!(
                    "[{cursor_style}]{}[/][{placeholder_style}]{}[/]",
                    escape_markup(first),
                    escape_markup(&rest)
                );
            }
            if !self.focused {
                return String::new();
            }
            return format!("[{cursor_style}] [/]");
        }

        let escaped = escape_markup(&self.value);
        if !self.focused {
            return escaped;
        }

        let value_graphemes: Vec<&str> = graphemes(&self.value).collect();
        let cursor = self.cursor.min(value_graphemes.len());

        if cursor >= value_graphemes.len() {
            format!("{escaped}[{cursor_style}] [/]")
        } else {
            let prefix: String = value_graphemes[..cursor].concat();
            let cursor_grapheme = value_graphemes[cursor];
            let suffix: String = value_graphemes[cursor + 1..].concat();
            format!(
                "{}[{cursor_style}]{}[/]{}",
                escape_markup(&prefix),
                escape_markup(cursor_grapheme),
                escape_markup(&suffix)
            )
        }
    }

    fn insert_char(&mut self, ch: char) {
        let cursor = self.cursor.min(grapheme_count(&self.value));
        let byte_index = grapheme_byte_index(&self.value, cursor);
        self.value.insert(byte_index, ch);
        self.cursor = cursor + 1;
        self.refresh_display();
    }

    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let cursor = self.cursor.min(grapheme_count(&self.value));
        let remove_index = cursor.saturating_sub(1);
        if let Some((start, end)) = grapheme_byte_range(&self.value, remove_index) {
            self.value.replace_range(start..end, "");
            self.cursor = remove_index;
        }
        self.refresh_display();
    }

    fn delete(&mut self) {
        let cursor = self.cursor.min(grapheme_count(&self.value));
        if let Some((start, end)) = grapheme_byte_range(&self.value, cursor) {
            self.value.replace_range(start..end, "");
        }
        self.refresh_display();
    }
}

impl<M> Default for Input<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: 'static> Widget<M> for Input<M> {
    fn default_css(&self) -> &'static str {
        r#"
Input {
    width: 1fr;
    height: auto;
    border: blank;
    background: transparent;
    padding-left: 0;
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
            KeyCode::Char(ch) => {
                self.insert_char(ch);
            }
            KeyCode::Backspace => {
                self.backspace();
            }
            KeyCode::Delete => {
                self.delete();
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.refresh_display();
                }
            }
            KeyCode::Right => {
                let max = grapheme_count(&self.value);
                if self.cursor < max {
                    self.cursor += 1;
                    self.refresh_display();
                }
            }
            KeyCode::Home => {
                self.cursor = 0;
                self.refresh_display();
            }
            KeyCode::End => {
                self.cursor = grapheme_count(&self.value);
                self.refresh_display();
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
        meta.type_name = "Input";
        meta.type_names = vec!["Input", "Static", "Widget", "DOMNode"];
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
        "Input"
    }

    fn on_resize(&mut self, size: Size) {
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
