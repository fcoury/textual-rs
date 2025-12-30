//! Screen - The root container for a widget tree.
//!
//! Screen is an implicit wrapper that provides:
//! - A CSS-targetable root container (type name "Screen")
//! - CSS-driven layout dispatch (grid, vertical, horizontal)
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
use crate::layouts;
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
/// 3. Dispatching to layout algorithms based on the `layout` CSS property.
///
/// It mimics Textual's Screen behavior where the app's content is implicitly
/// wrapped in a Screen widget.
pub struct Screen<M> {
    children: Vec<Box<dyn Widget<M>>>,
    /// Responsive classes are static strings from breakpoints, avoiding allocations.
    responsive_classes: Vec<&'static str>,
    style: ComputedStyle,
    is_dirty: bool,
    horizontal_breakpoints: &'static [Breakpoint],
    vertical_breakpoints: &'static [Breakpoint],
}

impl<M> Screen<M> {
    /// Create a new Screen with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
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

    /// Compute child placements using the appropriate layout algorithm.
    fn compute_child_placements(&self, region: Region) -> Vec<layouts::WidgetPlacement> {
        // Collect visible children with their styles and desired sizes
        let children_with_styles: Vec<_> = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.participates_in_layout())
            .map(|(i, c)| (i, c.get_style(), c.desired_size()))
            .collect();

        // Dispatch to layout based on CSS
        layouts::arrange_children(&self.style, &children_with_styles, region)
    }
}

impl<M> Widget<M> for Screen<M> {
    fn default_css(&self) -> &'static str {
        // Match Python Textual's Screen DEFAULT_CSS
        r#"
Screen {
    layout: vertical;
    overflow-y: auto;
    background: $background;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        // Render background/border and get inner region
        let inner_region =
            crate::containers::render_container_chrome(canvas, region, &self.style);

        for placement in self.compute_child_placements(inner_region) {
            if let Some(child) = self.children.get(placement.child_index) {
                child.render(canvas, placement.region);
            }
        }
    }

    fn desired_size(&self) -> Size {
        // Screen fills available space
        Size::new(u16::MAX, u16::MAX)
    }

    fn on_resize(&mut self, size: Size) {
        self.update_breakpoints(size.width, size.height);
        // Propagate resize to children
        for child in &mut self.children {
            child.on_resize(size);
        }
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
        for child in &mut self.children {
            f(child.as_mut());
        }
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        if index < self.children.len() {
            Some(self.children[index].as_mut())
        } else {
            None
        }
    }

    // Delegate state management
    fn is_dirty(&self) -> bool {
        self.is_dirty || self.children.iter().any(|c| c.is_dirty())
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

    // Delegate event handling
    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        for child in &mut self.children {
            if !child.participates_in_layout() {
                continue;
            }
            if let Some(msg) = child.on_event(key) {
                return Some(msg);
            }
        }
        None
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;

        if !region.contains_point(mx, my) {
            return None;
        }

        // Compute placements and dispatch mouse events
        let placements = self.compute_child_placements(region);

        for placement in placements {
            if placement.region.contains_point(mx, my) {
                if let Some(child) = self.children.get_mut(placement.child_index) {
                    if let Some(msg) = child.on_mouse(event, placement.region) {
                        return Some(msg);
                    }
                }
            }
        }

        None
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        let mut changed = false;
        for child in &mut self.children {
            if child.set_hover(is_hovered) {
                changed = true;
            }
        }
        changed
    }

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            if child.participates_in_layout() {
                child.clear_hover();
            }
        }
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        let mut changed = false;
        for child in &mut self.children {
            if child.set_active(is_active) {
                changed = true;
            }
        }
        changed
    }

    fn handle_message(&mut self, envelope: &mut crate::MessageEnvelope<M>) -> Option<M> {
        for child in &mut self.children {
            if let Some(msg) = child.handle_message(envelope) {
                return Some(msg);
            }
        }
        None
    }

    // Focus delegation
    fn count_focusable(&self) -> usize {
        self.children
            .iter()
            .filter(|c| c.participates_in_layout())
            .map(|c| c.count_focusable())
            .sum()
    }

    fn focus_nth(&mut self, mut n: usize) -> bool {
        for child in &mut self.children {
            if !child.participates_in_layout() {
                continue;
            }
            let count = child.count_focusable();
            if n < count {
                return child.focus_nth(n);
            }
            n -= count;
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
