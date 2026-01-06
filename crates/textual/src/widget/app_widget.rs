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
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

/// The root DOM node that wraps Screen.
///
/// `AppWidget` provides:
/// 1. A CSS-targetable root (type name "App")
/// 2. Passthrough behavior for all widget operations
///
/// This matches Python Textual's architecture where App is a DOMNode
/// at the root of the widget tree.
pub struct AppWidget<M> {
    children: Vec<Box<dyn Widget<M>>>,
    style: ComputedStyle,
    inline_style: StyleOverride,
    is_dirty: bool,
}

impl<M> AppWidget<M> {
    /// Create a new AppWidget wrapping the given children (typically Screen + overlays).
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            style: ComputedStyle::default(),
            inline_style: StyleOverride::default(),
            is_dirty: true,
        }
    }

    /// Convenience helper to wrap a single child.
    pub fn new_single(child: Box<dyn Widget<M>>) -> Self {
        Self::new(vec![child])
    }
}

impl<M> Widget<M> for AppWidget<M> {
    fn default_css(&self) -> &'static str {
        // Match Python Textual's App DEFAULT_CSS (flattened selectors).
        r#"
App {
    background: $background;
    color: $foreground;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        // Render children in order (later children overlay earlier ones)
        for child in &self.children {
            child.render(canvas, region);
        }
    }

    fn desired_size(&self) -> Size {
        // App fills available space (use the max child size)
        self.children.iter().fold(Size::new(0, 0), |acc, child| {
            let size = child.desired_size();
            Size::new(acc.width.max(size.width), acc.height.max(size.height))
        })
    }

    fn on_resize(&mut self, size: Size) {
        for child in &mut self.children {
            child.on_resize(size);
        }
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "App",
            type_names: vec!["App", "DOMNode"],
            classes: vec![],
            states: WidgetStates::empty(),
            id: None,
        }
    }

    // Delegate hierarchy traversal to child
    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        for child in &mut self.children {
            f(child.as_mut());
        }
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        match self.children.get_mut(index) {
            Some(child) => Some(child.as_mut()),
            None => None,
        }
    }

    // Delegate state management
    fn is_dirty(&self) -> bool {
        self.is_dirty || self.children.iter().any(|child| child.is_dirty())
    }

    fn mark_dirty(&mut self) {
        self.is_dirty = true;
        for child in &mut self.children {
            child.mark_dirty();
        }
    }

    fn mark_clean(&mut self) {
        self.is_dirty = false;
        for child in &mut self.children {
            child.mark_clean();
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
        self.is_dirty = true;
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
        self.is_dirty = true;
    }

    // Delegate event handling
    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        // Forward to the top-most child that handles the event
        for child in self.children.iter_mut().rev() {
            if let Some(msg) = child.on_event(key) {
                return Some(msg);
            }
        }
        None
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        for child in self.children.iter_mut().rev() {
            if let Some(msg) = child.on_mouse(event, region) {
                return Some(msg);
            }
        }
        None
    }

    fn on_mouse_with_sender(
        &mut self,
        event: MouseEvent,
        region: Region,
    ) -> Option<(M, crate::widget::SenderInfo)> {
        for child in self.children.iter_mut().rev() {
            if let Some(result) = child.on_mouse_with_sender(event, region) {
                return Some(result);
            }
        }
        None
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        let mut any_changed = false;
        for child in &mut self.children {
            any_changed |= child.set_hover(is_hovered);
        }
        any_changed
    }

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            child.clear_hover();
        }
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        let mut any_changed = false;
        for child in &mut self.children {
            any_changed |= child.set_active(is_active);
        }
        any_changed
    }

    fn handle_message(&mut self, envelope: &mut crate::MessageEnvelope<M>) -> Option<M> {
        let mut handled = None;
        for child in &mut self.children {
            handled = child.handle_message(envelope);
        }
        handled
    }

    // Focus delegation
    fn count_focusable(&self) -> usize {
        self.children
            .iter()
            .filter(|child| child.participates_in_layout())
            .map(|child| child.count_focusable())
            .sum()
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        let mut remaining = n;
        for child in &mut self.children {
            if !child.participates_in_layout() {
                continue;
            }
            let count = child.count_focusable();
            if remaining < count {
                return child.focus_nth(remaining);
            }
            remaining -= count;
        }
        false
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            if child.participates_in_layout() {
                child.clear_focus();
            }
        }
    }
}
