//! Screen - The root container for a widget tree.
//!
//! Screen is an implicit wrapper that provides:
//! - A CSS-targetable root container (type name "Screen")
//! - Responsive breakpoint classes based on terminal size
//! - Resize event propagation to children
//!
//! ## Custom Breakpoints
//!
//! Apps can define custom breakpoints by implementing `horizontal_breakpoints`
//! and `vertical_breakpoints` on the App trait. Breakpoints are (threshold, class_name)
//! pairs where the class is applied when the dimension >= threshold.
//!
//! The last matching breakpoint wins (iterate in order).

use crate::canvas::{Canvas, Region, Size};
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent};
use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

/// Breakpoint configuration: threshold and class name to apply.
pub type Breakpoint = (u16, &'static str);

/// Default horizontal breakpoints (matches Textual).
pub const DEFAULT_HORIZONTAL_BREAKPOINTS: &[Breakpoint] = &[
    (0, "-narrow"),
    (80, "-wide"),
];

/// Default vertical breakpoints (matches Textual).
pub const DEFAULT_VERTICAL_BREAKPOINTS: &[Breakpoint] = &[
    (0, "-short"),
    (24, "-tall"),
];

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
    horizontal_breakpoints: &'static [Breakpoint],
    vertical_breakpoints: &'static [Breakpoint],
}

impl<M> Screen<M> {
    /// Wraps a widget in a new Screen with default breakpoints.
    pub fn new(child: Box<dyn Widget<M>>) -> Self {
        Self {
            child,
            responsive_classes: Vec::new(),
            style: ComputedStyle::default(),
            is_dirty: true,
            horizontal_breakpoints: DEFAULT_HORIZONTAL_BREAKPOINTS,
            vertical_breakpoints: DEFAULT_VERTICAL_BREAKPOINTS,
        }
    }

    /// Set custom horizontal breakpoints.
    pub fn with_horizontal_breakpoints(mut self, breakpoints: &'static [Breakpoint]) -> Self {
        self.horizontal_breakpoints = breakpoints;
        self
    }

    /// Set custom vertical breakpoints.
    pub fn with_vertical_breakpoints(mut self, breakpoints: &'static [Breakpoint]) -> Self {
        self.vertical_breakpoints = breakpoints;
        self
    }

    /// Updates the responsive classes based on dimensions.
    ///
    /// For each axis, finds the last matching breakpoint (threshold <= dimension)
    /// and applies that class.
    fn update_breakpoints(&mut self, width: u16, height: u16) {
        let old_classes = self.responsive_classes.clone();
        self.responsive_classes.clear();

        // Find matching horizontal breakpoint (last one where width >= threshold)
        if let Some((_, class)) = self.horizontal_breakpoints
            .iter()
            .filter(|(threshold, _)| width >= *threshold)
            .last()
        {
            self.responsive_classes.push(class.to_string());
        }

        // Find matching vertical breakpoint (last one where height >= threshold)
        if let Some((_, class)) = self.vertical_breakpoints
            .iter()
            .filter(|(threshold, _)| height >= *threshold)
            .last()
        {
            self.responsive_classes.push(class.to_string());
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
