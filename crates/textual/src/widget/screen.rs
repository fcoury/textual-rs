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
    /// Responsive classes are static strings from breakpoints, avoiding allocations.
    responsive_classes: Vec<&'static str>,
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
        if let Some((_, class)) = self
            .horizontal_breakpoints
            .iter()
            .filter(|(threshold, _)| width >= *threshold)
            .last()
        {
            self.responsive_classes.push(*class);
        }

        // Find matching vertical breakpoint (last one where height >= threshold)
        if let Some((_, class)) = self
            .vertical_breakpoints
            .iter()
            .filter(|(threshold, _)| height >= *threshold)
            .last()
        {
            self.responsive_classes.push(*class);
        }

        if old_classes != self.responsive_classes {
            self.is_dirty = true;
        }
    }

    /// Calculate the aligned child region within the available region.
    ///
    /// Handles alignment (left/center/right, top/middle/bottom) and ensures
    /// the child doesn't exceed the available space.
    fn child_region(&self, region: Region) -> Region {
        use tcss::types::{AlignHorizontal, AlignVertical};

        let child_size = self.child.desired_size();
        let child_width = (child_size.width as i32).min(region.width);
        let child_height = (child_size.height as i32).min(region.height);

        // Clamp offsets to 0 to prevent negative offsets when content > container
        let offset_x = match self.style.align_horizontal {
            AlignHorizontal::Left => 0,
            AlignHorizontal::Center => (region.width - child_width).max(0) / 2,
            AlignHorizontal::Right => (region.width - child_width).max(0),
        };

        let offset_y = match self.style.align_vertical {
            AlignVertical::Top => 0,
            AlignVertical::Middle => (region.height - child_height).max(0) / 2,
            AlignVertical::Bottom => (region.height - child_height).max(0),
        };

        Region {
            x: region.x + offset_x,
            y: region.y + offset_y,
            width: child_width,
            height: child_height,
        }
    }
}

impl<M> Widget<M> for Screen<M> {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        let child_region = self.child_region(region);
        self.child.render(canvas, child_region);
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
            // Convert &'static str to String only when metadata is requested
            classes: self.responsive_classes.iter().map(|s| s.to_string()).collect(),
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
        let child_region = self.child_region(region);
        self.child.on_mouse(event, child_region)
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
