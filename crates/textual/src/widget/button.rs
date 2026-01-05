use std::cell::RefCell;

use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

use crate::{
    Canvas, KeyCode, MouseEvent, MouseEventKind, Region, Size, Widget,
    widget::static_widget::Static,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Primary,
    Success,
    Warning,
    Error,
}

impl ButtonVariant {
    fn class_name(self) -> &'static str {
        match self {
            ButtonVariant::Default => "-default",
            ButtonVariant::Primary => "-primary",
            ButtonVariant::Success => "-success",
            ButtonVariant::Warning => "-warning",
            ButtonVariant::Error => "-error",
        }
    }

    fn parse(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().as_str() {
            "default" => Some(ButtonVariant::Default),
            "primary" => Some(ButtonVariant::Primary),
            "success" => Some(ButtonVariant::Success),
            "warning" => Some(ButtonVariant::Warning),
            "error" => Some(ButtonVariant::Error),
            _ => None,
        }
    }
}

/// A clickable button widget.
pub struct Button<M> {
    inner: Static<M>,
    focused: bool,
    hovered: bool,
    active: bool,
    dirty: bool,
    variant: ButtonVariant,
    flat: bool,
    compact: bool,
    action: Option<String>,
    on_press: Option<Box<dyn Fn() -> M>>,
    pending_action: RefCell<Option<String>>,
}

impl<M: 'static> Button<M> {
    pub fn new(label: impl Into<String>) -> Self {
        let mut inner = Static::new(label);
        inner = inner.with_markup(false);
        inner.add_class("-style-default");
        inner.add_class(ButtonVariant::Default.class_name());

        Self {
            inner,
            focused: false,
            hovered: false,
            active: false,
            dirty: true,
            variant: ButtonVariant::Default,
            flat: false,
            compact: false,
            action: None,
            on_press: None,
            pending_action: RefCell::new(None),
        }
    }

    /// Set a unique ID for this button.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner = self.inner.with_id(id);
        self
    }

    /// Set CSS classes (space-separated).
    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        self.inner = self.inner.with_classes(classes);
        self
    }

    /// Set the button variant.
    pub fn with_variant(mut self, variant: impl AsRef<str>) -> Self {
        self.set_variant(variant);
        self
    }

    /// Enable compact styling (no borders for default style).
    pub fn with_compact(mut self, compact: bool) -> Self {
        self.set_compact(compact);
        self
    }

    /// Enable flat button style.
    pub fn with_flat(mut self, flat: bool) -> Self {
        self.set_flat(flat);
        self
    }

    /// Set the button action to dispatch on press.
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Set a press handler that produces a message.
    pub fn with_on_press<F>(mut self, on_press: F) -> Self
    where
        F: Fn() -> M + 'static,
    {
        self.on_press = Some(Box::new(on_press));
        self
    }

    /// Set a message to emit when pressed (cloned for each press).
    pub fn with_message(mut self, message: M) -> Self
    where
        M: Clone + 'static,
    {
        self.on_press = Some(Box::new(move || message.clone()));
        self
    }

    /// Set the disabled state.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.inner.set_disabled(disabled);
        self
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.inner.update(label);
    }

    pub fn set_variant(&mut self, variant: impl AsRef<str>) {
        let Some(parsed) = ButtonVariant::parse(variant.as_ref()) else {
            panic!(
                "Invalid button variant '{}'. Valid variants: default, primary, success, warning, error",
                variant.as_ref()
            );
        };
        if self.variant != parsed {
            self.inner.remove_class(self.variant.class_name());
            self.variant = parsed;
            self.inner.add_class(self.variant.class_name());
        }
    }

    pub fn set_flat(&mut self, flat: bool) {
        if self.flat == flat {
            return;
        }
        self.flat = flat;
        if flat {
            self.inner.remove_class("-style-default");
            self.inner.add_class("-style-flat");
        } else {
            self.inner.remove_class("-style-flat");
            self.inner.add_class("-style-default");
        }
    }

    pub fn set_compact(&mut self, compact: bool) {
        if self.compact == compact {
            return;
        }
        self.compact = compact;
        if compact {
            self.inner.add_class("-textual-compact");
        } else {
            self.inner.remove_class("-textual-compact");
        }
    }

    fn press(&mut self) -> Option<M> {
        if self.is_disabled() {
            return None;
        }
        if let Some(action) = &self.action {
            *self.pending_action.borrow_mut() = Some(action.clone());
            return None;
        }
        self.on_press.as_ref().map(|f| f())
    }
}

impl<M: 'static> Widget<M> for Button<M> {
    fn default_css(&self) -> &'static str {
        r#"
Button {
    width: auto;
    min-width: 16;
    height: auto;
    text-align: center;
    content-align: center middle;

    &.-style-flat {
        text-style: bold;
        color: auto 90%;
        background: $surface;
        border: block $surface;
        &:hover {
            background: $primary;
            border: block $primary;
        }
        &:focus {
            text-style: $button-focus-text-style;
        }
        &.-active {
            background: $surface;
            border: block $surface;
            tint: $background 30%;
        }
        &:disabled {
            color: auto 50%;
        }

        &.-primary {
            background: $primary-muted;
            border: block $primary-muted;
            color: $text-primary;
            &:hover {
                color: $text;
                background: $primary;
                border: block $primary;
            }
        }
        &.-success {
            background: $success-muted;
            border: block $success-muted;
            color: $text-success;
            &:hover {
                color: $text;
                background: $success;
                border: block $success;
            }
        }
        &.-warning {
            background: $warning-muted;
            border: block $warning-muted;
            color: $text-warning;
            &:hover {
                color: $text;
                background: $warning;
                border: block $warning;
            }
        }
        &.-error {
            background: $error-muted;
            border: block $error-muted;
            color: $text-error;
            &:hover {
                color: $text;
                background: $error;
                border: block $error;
            }
        }
    }
    &.-style-default {
        text-style: bold;
        color: $button-foreground;
        background: $surface;
        border: none;
        border-top: tall $surface-lighten-1;
        border-bottom: tall $surface-darken-1;

        &.-textual-compact {
            border: none !important;
        }

        &:disabled {
            text-opacity: 0.6;
        }

        &:focus {
            text-style: $button-focus-text-style;
            background-tint: $foreground 5%;
        }
        &:hover {
            border-top: tall $surface;
            background: $surface-darken-1;
        }

        &.-active {
            background: $surface;
            border-bottom: tall $surface-lighten-1;
            border-top: tall $surface-darken-1;
            tint: $background 30%;
        }

        &.-primary {
            color: $button-color-foreground;
            background: $primary;
            border-top: tall $primary-lighten-3;
            border-bottom: tall $primary-darken-3;

            &:hover {
                background: $primary-darken-2;
                border-top: tall $primary;
            }

            &.-active {
                background: $primary;
                border-bottom: tall $primary-lighten-3;
                border-top: tall $primary-darken-3;
            }
        }

        &.-success {
            color: $button-color-foreground;
            background: $success;
            border-top: tall $success-lighten-2;
            border-bottom: tall $success-darken-3;

            &:hover {
                background: $success-darken-2;
                border-top: tall $success;
            }

            &.-active {
                background: $success;
                border-bottom: tall $success-lighten-2;
                border-top: tall $success-darken-2;
            }
        }

        &.-warning{
            color: $button-color-foreground;
            background: $warning;
            border-top: tall $warning-lighten-2;
            border-bottom: tall $warning-darken-3;

            &:hover {
                background: $warning-darken-2;
                border-top: tall $warning;
            }

            &.-active {
                background: $warning;
                border-bottom: tall $warning-lighten-2;
                border-top: tall $warning-darken-2;
            }
        }

        &.-error {
            color: $button-color-foreground;
            background: $error;
            border-top: tall $error-lighten-2;
            border-bottom: tall $error-darken-3;

            &:hover {
                background: $error-darken-1;
                border-top: tall $error;
            }

            &.-active {
                background: $error;
                border-bottom: tall $error-lighten-2;
                border-top: tall $error-darken-2;
            }
        }
    }
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.inner.render(canvas, region)
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn intrinsic_height_for_width(&self, width: u16) -> u16 {
        self.inner.intrinsic_height_for_width(width)
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "Button";
        meta.type_names = vec!["Button", "Widget", "DOMNode"];
        meta.states = self.get_state();
        meta
    }

    fn get_state(&self) -> WidgetStates {
        let mut states = WidgetStates::empty();
        if self.focused {
            states |= WidgetStates::FOCUS;
        }
        if self.hovered {
            states |= WidgetStates::HOVER;
        }
        if self.active {
            states |= WidgetStates::ACTIVE;
        }
        if self.inner.is_disabled() {
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

    fn id(&self) -> Option<&str> {
        self.inner.id()
    }

    fn type_name(&self) -> &'static str {
        "Button"
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        if self.is_disabled() {
            return None;
        }
        match key {
            KeyCode::Enter | KeyCode::Char(' ') => self.press(),
            _ => None,
        }
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;
        let in_bounds = region.contains_point(mx, my);

        match event.kind {
            MouseEventKind::Moved => {
                if in_bounds != self.hovered {
                    self.hovered = in_bounds;
                    self.dirty = true;
                }
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                if in_bounds && !self.is_disabled() {
                    if !self.active {
                        self.active = true;
                        self.dirty = true;
                    }
                }
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                if self.active {
                    self.active = false;
                    self.dirty = true;
                    if in_bounds && !self.is_disabled() {
                        return self.press();
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        if self.hovered != is_hovered {
            self.hovered = is_hovered;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    fn clear_hover(&mut self) {
        if self.hovered {
            self.hovered = false;
            self.dirty = true;
        }
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        if self.active != is_active {
            self.active = is_active;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    fn is_focusable(&self) -> bool {
        !self.inner.is_disabled()
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        if n == 0 {
            self.set_focus(true);
            true
        } else {
            false
        }
    }

    fn set_focus(&mut self, is_focused: bool) {
        if self.focused != is_focused {
            self.focused = is_focused;
            self.dirty = true;
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn is_disabled(&self) -> bool {
        self.inner.is_disabled()
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.inner.set_disabled(disabled);
        self.dirty = true;
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

    fn take_pending_action(&self) -> Option<String> {
        self.pending_action.borrow_mut().take()
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
