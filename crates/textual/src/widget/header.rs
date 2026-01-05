//! Header widget for displaying a title and optional subtitle.

use std::cell::RefCell;
use std::marker::PhantomData;

use tcss::types::Visibility;
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region};
use crate::containers::container::Container;
use crate::widget::static_widget::Static;
use crate::{KeyCode, MouseEvent, MouseEventKind, Size, Widget};

fn format_title(title: &str, subtitle: Option<&str>) -> String {
    let subtitle = subtitle.filter(|value| !value.is_empty());
    match subtitle {
        Some(value) => format!("{title} [dim]— {value}[/]"),
        None => title.to_string(),
    }
}

#[derive(Debug, Clone)]
struct HeaderTitle<M> {
    inner: Static<M>,
}

impl<M> HeaderTitle<M> {
    fn new(content: String) -> Self {
        let mut inner = Static::new(content);
        inner = inner.with_markup(true);
        Self { inner }
    }

    fn update(&mut self, content: String) {
        self.inner.update(content);
    }
}

impl<M: 'static> Widget<M> for HeaderTitle<M> {
    fn default_css(&self) -> &'static str {
        r#"
HeaderTitle {
    text-wrap: nowrap;
    text-overflow: ellipsis;
    content-align: center middle;
    width: 100%;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.inner.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn intrinsic_height_for_width(&self, width: u16) -> u16 {
        self.inner.intrinsic_height_for_width(width)
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "HeaderTitle";
        meta.type_names = vec!["HeaderTitle", "Static", "Widget", "DOMNode"];
        meta
    }

    fn get_state(&self) -> WidgetStates {
        self.inner.get_state()
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.inner.set_style(style);
    }

    fn get_style(&self) -> ComputedStyle {
        self.inner.get_style()
    }

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inner.set_inline_style(style);
    }

    fn inline_style(&self) -> Option<&StyleOverride> {
        self.inner.inline_style()
    }

    fn clear_inline_style(&mut self) {
        self.inner.clear_inline_style();
    }

    fn is_dirty(&self) -> bool {
        self.inner.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.inner.mark_dirty();
    }

    fn mark_clean(&mut self) {
        self.inner.mark_clean();
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.inner.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.inner.on_mouse(event, region)
    }

    fn is_focusable(&self) -> bool {
        self.inner.is_focusable()
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
        "HeaderTitle"
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

struct HeaderClockSpace<M> {
    inner: Static<M>,
}

impl<M> HeaderClockSpace<M> {
    fn new() -> Self {
        Self {
            inner: Static::new(""),
        }
    }
}

impl<M: 'static> Widget<M> for HeaderClockSpace<M> {
    fn default_css(&self) -> &'static str {
        r#"
HeaderClockSpace {
    dock: right;
    width: 10;
    padding: 0 1;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.inner.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "HeaderClockSpace";
        meta.type_names = vec!["HeaderClockSpace", "Static", "Widget", "DOMNode"];
        meta
    }

    fn get_state(&self) -> WidgetStates {
        self.inner.get_state()
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.inner.set_style(style);
    }

    fn get_style(&self) -> ComputedStyle {
        self.inner.get_style()
    }

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inner.set_inline_style(style);
    }

    fn inline_style(&self) -> Option<&StyleOverride> {
        self.inner.inline_style()
    }

    fn clear_inline_style(&mut self) {
        self.inner.clear_inline_style();
    }

    fn is_dirty(&self) -> bool {
        self.inner.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.inner.mark_dirty();
    }

    fn mark_clean(&mut self) {
        self.inner.mark_clean();
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.inner.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.inner.on_mouse(event, region)
    }

    fn is_focusable(&self) -> bool {
        self.inner.is_focusable()
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
        "HeaderClockSpace"
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

struct HeaderIcon<M> {
    inner: Static<M>,
    hovered: bool,
    active: bool,
    pending_action: RefCell<Option<String>>,
}

impl<M> HeaderIcon<M> {
    fn new() -> Self {
        let mut inner = Static::new("⭘");
        inner = inner.with_markup(false);
        Self {
            inner,
            hovered: false,
            active: false,
            pending_action: RefCell::new(None),
        }
    }

    fn press(&self) {
        *self.pending_action.borrow_mut() = Some("app.command_palette".to_string());
    }
}

impl<M: 'static> Widget<M> for HeaderIcon<M> {
    fn default_css(&self) -> &'static str {
        r#"
HeaderIcon {
    dock: left;
    padding: 0 1;
    width: 8;
    content-align: left middle;
}

HeaderIcon:hover {
    background: $foreground 10%;
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

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "HeaderIcon";
        meta.type_names = vec!["HeaderIcon", "Static", "Widget", "DOMNode"];
        meta.states = self.get_state();
        meta
    }

    fn get_state(&self) -> WidgetStates {
        let mut states = WidgetStates::empty();
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
    }

    fn inline_style(&self) -> Option<&StyleOverride> {
        self.inner.inline_style()
    }

    fn clear_inline_style(&mut self) {
        self.inner.clear_inline_style();
    }

    fn is_dirty(&self) -> bool {
        self.inner.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.inner.mark_dirty();
    }

    fn mark_clean(&mut self) {
        self.inner.mark_clean();
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.inner.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;
        let in_bounds = region.contains_point(mx, my);

        match event.kind {
            MouseEventKind::Moved => {
                if in_bounds != self.hovered {
                    self.hovered = in_bounds;
                    self.inner.mark_dirty();
                }
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                if in_bounds && !self.inner.is_disabled() {
                    if !self.active {
                        self.active = true;
                        self.inner.mark_dirty();
                    }
                }
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                if self.active {
                    self.active = false;
                    self.inner.mark_dirty();
                    if in_bounds && !self.inner.is_disabled() {
                        self.press();
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn on_mouse_with_sender(
        &mut self,
        event: MouseEvent,
        region: Region,
    ) -> Option<(M, crate::widget::SenderInfo)> {
        self.on_mouse(event, region).map(|message| {
            let sender = crate::widget::SenderInfo {
                id: self.id().map(|s| s.to_string()),
                type_name: self.type_name(),
            };
            (message, sender)
        })
    }

    fn take_pending_action(&self) -> Option<String> {
        self.pending_action.borrow_mut().take()
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        if self.hovered != is_hovered {
            self.hovered = is_hovered;
            self.inner.mark_dirty();
            true
        } else {
            false
        }
    }

    fn clear_hover(&mut self) {
        if self.hovered {
            self.hovered = false;
            self.inner.mark_dirty();
        }
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        if self.active != is_active {
            self.active = is_active;
            self.inner.mark_dirty();
            true
        } else {
            false
        }
    }

    fn is_focusable(&self) -> bool {
        false
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
        "HeaderIcon"
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

/// A header widget that displays a title and optional subtitle.
pub struct Header<M: 'static> {
    inner: Container<M>,
    title: String,
    subtitle: Option<String>,
    tall: bool,
    title_index: usize,
    _phantom: PhantomData<M>,
}

impl<M: 'static> Header<M> {
    /// Create a new Header with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        let title_widget = HeaderTitle::new(format_title(&title, None));
        let icon = HeaderIcon::new();
        let clock_space = HeaderClockSpace::new();

        let children: Vec<Box<dyn Widget<M>>> = vec![
            Box::new(icon),
            Box::new(title_widget),
            Box::new(clock_space),
        ];

        let inner = Container::new(children);

        Self {
            inner,
            title,
            subtitle: None,
            tall: false,
            title_index: 1,
            _phantom: PhantomData,
        }
    }

    /// Set the subtitle shown after the title.
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self.refresh_title();
        self
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner = self.inner.with_id(id);
        self
    }

    /// Set CSS classes (space-separated).
    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        self.inner = self.inner.with_classes(classes);
        self
    }

    /// Toggle the tall header style (adds/removes the `-tall` class).
    pub fn with_tall(mut self, tall: bool) -> Self {
        self.set_tall(tall);
        self
    }

    /// Set the title at runtime.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.refresh_title();
    }

    /// Set the subtitle at runtime.
    pub fn set_subtitle(&mut self, subtitle: impl Into<String>) {
        self.subtitle = Some(subtitle.into());
        self.refresh_title();
    }

    /// Clear the subtitle.
    pub fn clear_subtitle(&mut self) {
        self.subtitle = None;
        self.refresh_title();
    }

    /// Get the current title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the current subtitle, if any.
    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    /// Return whether the header is in tall mode.
    pub fn is_tall(&self) -> bool {
        self.tall
    }

    fn refresh_title(&mut self) {
        let content = format_title(&self.title, self.subtitle.as_deref());
        if let Some(title) = self.title_mut() {
            title.update(content);
        }
    }

    fn title_mut(&mut self) -> Option<&mut HeaderTitle<M>> {
        self.inner
            .get_child_mut(self.title_index)
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<HeaderTitle<M>>())
    }

    fn set_tall(&mut self, tall: bool) {
        if self.tall == tall {
            return;
        }
        self.tall = tall;
        if tall {
            self.inner.add_class("-tall");
        } else {
            self.inner.remove_class("-tall");
        }
    }
}

impl<M: 'static> Widget<M> for Header<M> {
    fn default_css(&self) -> &'static str {
        r#"
Header {
    dock: top;
    width: 100%;
    background: $panel;
    color: $foreground;
    height: 1;
    layout: horizontal;
}

Header.-tall {
    height: 3;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.inner.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "Header";
        meta.type_names = vec!["Header", "Container", "Widget", "DOMNode"];
        meta
    }

    fn get_state(&self) -> WidgetStates {
        self.inner.get_state()
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.inner.set_style(style);
    }

    fn get_style(&self) -> ComputedStyle {
        self.inner.get_style()
    }

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inner.set_inline_style(style);
    }

    fn inline_style(&self) -> Option<&StyleOverride> {
        self.inner.inline_style()
    }

    fn clear_inline_style(&mut self) {
        self.inner.clear_inline_style();
    }

    fn is_dirty(&self) -> bool {
        self.inner.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.inner.mark_dirty();
    }

    fn mark_clean(&mut self) {
        self.inner.mark_clean();
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.inner.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.inner.on_mouse(event, region)
    }

    fn on_mouse_with_sender(
        &mut self,
        event: MouseEvent,
        region: Region,
    ) -> Option<(M, crate::widget::SenderInfo)> {
        self.inner.on_mouse_with_sender(event, region)
    }

    fn is_focusable(&self) -> bool {
        self.inner.is_focusable()
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
        "Header"
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
