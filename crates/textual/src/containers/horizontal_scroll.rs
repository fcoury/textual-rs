//! HorizontalScroll container that arranges children horizontally with scrolling.
//!
//! This container wraps a `Horizontal` inside a `ScrollableContainer`, providing
//! horizontal scrolling for content that exceeds the viewport.

use crate::canvas::{Canvas, Region, Size};
use crate::containers::horizontal::Horizontal;
use crate::containers::scrollable::ScrollableContainer;
use crate::widget::Widget;
use crate::{KeyCode, MessageEnvelope, MouseEvent};
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

/// A scrollable container that arranges children horizontally (left-to-right).
///
/// This is equivalent to a `Horizontal` container wrapped in a `ScrollableContainer`,
/// providing horizontal scrolling when content exceeds the viewport width.
///
/// # Example
///
/// ```ignore
/// use textual::HorizontalScroll;
///
/// ui! {
///     HorizontalScroll {
///         Label("Item 1")
///         Label("Item 2")
///         Label("Item 3")
///         // ... more items that may exceed viewport
///     }
/// }
/// ```
pub struct HorizontalScroll<M: 'static> {
    inner: ScrollableContainer<M>,
}

impl<M: 'static> HorizontalScroll<M> {
    /// Create a new HorizontalScroll container with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        let horizontal = Box::new(Horizontal::new(children)) as Box<dyn Widget<M>>;
        Self {
            inner: ScrollableContainer::from_child(horizontal),
        }
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        // Get the inner Horizontal and set its ID
        if let Some(child) = self.inner.get_child_mut(0) {
            child.add_class(&format!("#{}", id.into()));
        }
        self
    }

    /// Set CSS classes.
    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        let classes_str: String = classes.into();
        if let Some(child) = self.inner.get_child_mut(0) {
            for class in classes_str.split_whitespace() {
                child.add_class(class);
            }
        }
        self
    }

    /// Set the border title.
    pub fn with_border_title(mut self, title: impl Into<String>) -> Self {
        if let Some(child) = self.inner.get_child_mut(0) {
            child.set_border_title(&title.into());
        }
        self
    }

    /// Set the border subtitle.
    pub fn with_border_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        if let Some(child) = self.inner.get_child_mut(0) {
            child.set_border_subtitle(&subtitle.into());
        }
        self
    }
}

impl<M: 'static> Widget<M> for HorizontalScroll<M> {
    fn default_css(&self) -> &'static str {
        r#"
HorizontalScroll {
    width: 1fr;
    height: 1fr;
    overflow-x: auto;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.inner.render(canvas, region)
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "HorizontalScroll".to_string();
        meta
    }

    fn get_state(&self) -> WidgetStates {
        self.inner.get_state()
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.inner.set_style(style)
    }

    fn get_style(&self) -> ComputedStyle {
        self.inner.get_style()
    }

    fn is_dirty(&self) -> bool {
        self.inner.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.inner.mark_dirty()
    }

    fn mark_clean(&mut self) {
        self.inner.mark_clean()
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.inner.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.inner.on_mouse(event, region)
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        self.inner.set_hover(is_hovered)
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        self.inner.set_active(is_active)
    }

    fn clear_hover(&mut self) {
        self.inner.clear_hover()
    }

    fn is_focusable(&self) -> bool {
        self.inner.is_focusable()
    }

    fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.inner.set_visible(visible)
    }

    fn is_loading(&self) -> bool {
        self.inner.is_loading()
    }

    fn set_loading(&mut self, loading: bool) {
        self.inner.set_loading(loading)
    }

    fn is_disabled(&self) -> bool {
        self.inner.is_disabled()
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.inner.set_disabled(disabled)
    }

    fn count_focusable(&self) -> usize {
        self.inner.count_focusable()
    }

    fn clear_focus(&mut self) {
        self.inner.clear_focus()
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.inner.focus_nth(n)
    }

    fn set_focus(&mut self, is_focused: bool) {
        self.inner.set_focus(is_focused)
    }

    fn is_focused(&self) -> bool {
        self.inner.is_focused()
    }

    fn child_count(&self) -> usize {
        self.inner.child_count()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        self.inner.get_child_mut(index)
    }

    fn handle_message(&mut self, envelope: &mut MessageEnvelope<M>) -> Option<M> {
        self.inner.handle_message(envelope)
    }

    fn id(&self) -> Option<&str> {
        self.inner.id()
    }

    fn type_name(&self) -> &'static str {
        "HorizontalScroll"
    }

    fn on_resize(&mut self, size: Size) {
        self.inner.on_resize(size)
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        self.inner.for_each_child(f)
    }
}
