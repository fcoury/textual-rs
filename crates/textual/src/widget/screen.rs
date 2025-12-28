//! Screen - The root container for a widget tree.
//!
//! Screen is an implicit wrapper that provides:
//! - A CSS-targetable root container (type name "Screen")
//! - Responsive breakpoint classes based on terminal size
//! - Resize event propagation to children

use crate::canvas::{Canvas, Region, Size};
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent};
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

/// The root container for a widget tree.
///
/// `Screen` is responsible for:
/// 1. Providing the root context for CSS matching (type name "Screen").
/// 2. Managing responsive breakpoint classes based on terminal size.
///
/// It mimics Textual's Screen behavior where the app's content is implicitly
/// wrapped in a Screen widget.
pub struct Screen<M> {
    child: Box<dyn Widget<M>>,
    responsive_classes: Vec<String>,
    style: ComputedStyle,
    is_dirty: bool,
}

impl<M> Screen<M> {
    /// Wraps a widget in a new Screen.
    pub fn new(child: Box<dyn Widget<M>>) -> Self {
        Self {
            child,
            responsive_classes: Vec::new(),
            style: ComputedStyle::default(),
            is_dirty: true,
        }
    }

    /// Updates the responsive classes based on dimensions.
    ///
    /// Breakpoints match Textual's defaults:
    /// - width < 80: "-narrow"
    /// - width >= 80: "-wide"
    /// - height < 24: "-short"
    /// - height >= 24: "-tall"
    fn update_breakpoints(&mut self, width: u16, height: u16) {
        let old_classes = self.responsive_classes.clone();
        self.responsive_classes.clear();

        // Textual standard breakpoints
        if width < 80 {
            self.responsive_classes.push("-narrow".to_string());
        } else {
            self.responsive_classes.push("-wide".to_string());
        }

        if height < 24 {
            self.responsive_classes.push("-short".to_string());
        } else {
            self.responsive_classes.push("-tall".to_string());
        }

        if old_classes != self.responsive_classes {
            self.is_dirty = true;
        }
    }
}

impl<M> Widget<M> for Screen<M> {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        // Screen simply delegates rendering to its child, filling the region
        self.child.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        // Screen always takes available space, but reports child's desire
        self.child.desired_size()
    }

    fn on_resize(&mut self, size: Size) {
        self.update_breakpoints(size.width, size.height);
        // Propagate resize to children
        self.child.on_resize(size);
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Screen".to_string(),
            classes: self.responsive_classes.clone(),
            states: WidgetStates::empty(), // Screen typically doesn't have focus/hover itself
            id: None,
        }
    }

    // Delegate hierarchy traversal
    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        f(&mut *self.child);
    }

    fn child_count(&self) -> usize {
        1
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        if index == 0 {
            Some(&mut *self.child)
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
