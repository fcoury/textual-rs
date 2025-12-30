//! AppWidget - The root DOM node for a widget tree.
//!
//! AppWidget wraps Screen to provide Python Textual-compatible DOM hierarchy:
//!
//! ```text
//! App (type_name = "App")
//! └── Screen (type_name = "Screen")
//!     └── ... children from compose()
//! ```
//!
//! This enables CSS selectors like `App Static` to work correctly, matching
//! Python Textual's behavior where App is the root of the DOM tree.

use crate::canvas::{Canvas, Region, Size};
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent};
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

/// The root DOM node that wraps Screen.
///
/// `AppWidget` provides:
/// 1. A CSS-targetable root (type name "App")
/// 2. Passthrough behavior for all widget operations
///
/// This matches Python Textual's architecture where App is a DOMNode
/// at the root of the widget tree.
pub struct AppWidget<M> {
    child: Box<dyn Widget<M>>,
    style: ComputedStyle,
    is_dirty: bool,
}

impl<M> AppWidget<M> {
    /// Create a new AppWidget wrapping the given child (typically Screen).
    pub fn new(child: Box<dyn Widget<M>>) -> Self {
        Self {
            child,
            style: ComputedStyle::default(),
            is_dirty: true,
        }
    }
}

impl<M> Widget<M> for AppWidget<M> {
    fn default_css(&self) -> &'static str {
        // App doesn't need special default styling - it's purely structural
        ""
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        // Delegate rendering entirely to child
        self.child.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        // App fills available space (delegates to Screen which also fills)
        self.child.desired_size()
    }

    fn on_resize(&mut self, size: Size) {
        self.child.on_resize(size);
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "App".to_string(),
            classes: vec![],
            states: WidgetStates::empty(),
            id: None,
        }
    }

    // Delegate hierarchy traversal to child
    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        f(self.child.as_mut());
    }

    fn child_count(&self) -> usize {
        1
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        if index == 0 {
            Some(self.child.as_mut())
        } else {
            None
        }
    }

    // Delegate state management
    fn is_dirty(&self) -> bool {
        self.is_dirty || self.child.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.is_dirty = true;
        self.child.mark_dirty();
    }

    fn mark_clean(&mut self) {
        self.is_dirty = false;
        self.child.mark_clean();
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    // Delegate event handling
    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.child.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.child.on_mouse(event, region)
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        self.child.set_hover(is_hovered)
    }

    fn clear_hover(&mut self) {
        self.child.clear_hover();
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        self.child.set_active(is_active)
    }

    fn handle_message(&mut self, envelope: &mut crate::MessageEnvelope<M>) -> Option<M> {
        self.child.handle_message(envelope)
    }

    // Focus delegation
    fn count_focusable(&self) -> usize {
        self.child.count_focusable()
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.child.focus_nth(n)
    }

    fn clear_focus(&mut self) {
        self.child.clear_focus();
    }
}
