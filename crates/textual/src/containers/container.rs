//! Generic container widget with CSS-driven layout dispatch.
//!
//! Container is the base for all layout containers. It dispatches to the
//! appropriate layout algorithm based on the `layout` CSS property.

use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region, Size};
use crate::layouts::{self, Layout};
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent};

/// A generic container that arranges children using CSS-driven layout.
///
/// The layout algorithm is determined by the `layout` CSS property:
/// - `layout: vertical` - stacks children top-to-bottom (default)
/// - `layout: horizontal` - stacks children left-to-right
/// - `layout: grid` - CSS Grid-like 2D layout
///
/// Containers are the building blocks for complex layouts. Use the
/// type aliases (`Grid`, `Vertical`, `Horizontal`) for semantic clarity.
pub struct Container<M> {
    children: Vec<Box<dyn Widget<M>>>,
    style: ComputedStyle,
    dirty: bool,
    id: Option<String>,
}

impl<M> Container<M> {
    /// Create a new Container with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            style: ComputedStyle::default(),
            dirty: true,
            id: None,
        }
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Compute child placements using the appropriate layout algorithm.
    fn compute_child_placements(&self, region: Region) -> Vec<layouts::WidgetPlacement> {
        // Collect visible children with their styles and desired sizes
        let children_with_styles: Vec<_> = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_visible())
            .map(|(i, c)| (i, c.get_style(), c.desired_size()))
            .collect();

        // Dispatch to layout based on CSS
        layouts::arrange_children(&self.style, &children_with_styles, region)
    }
}

impl<M> Widget<M> for Container<M> {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        // 1. Render background/border and get inner region
        let inner_region = crate::containers::render_container_chrome(canvas, region, &self.style);

        // 2. Render children in inner region
        canvas.push_clip(inner_region);
        for placement in self.compute_child_placements(inner_region) {
            self.children[placement.child_index].render(canvas, placement.region);
        }
        canvas.pop_clip();
    }

    fn desired_size(&self) -> Size {
        // Check CSS dimensions first
        let width = if let Some(w) = &self.style.width {
            use tcss::types::Unit;
            match w.unit {
                Unit::Cells => w.value as u16,
                _ => u16::MAX, // Fill available space
            }
        } else {
            u16::MAX // Container expands to fill available space by default
        };

        let height = if let Some(h) = &self.style.height {
            use tcss::types::Unit;
            match h.unit {
                Unit::Cells => h.value as u16,
                _ => u16::MAX, // Fill available space
            }
        } else {
            u16::MAX // Container expands to fill available space by default
        };

        Size::new(width, height)
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Container".to_string(),
            id: self.id.clone(),
            classes: Vec::new(),
            states: WidgetStates::empty(),
        }
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        for child in &mut self.children {
            f(child.as_mut());
        }
    }

    fn on_resize(&mut self, size: Size) {
        for child in &mut self.children {
            child.on_resize(size);
        }
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        for child in &mut self.children {
            if !child.is_visible() {
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
                if let Some(msg) = self.children[placement.child_index].on_mouse(event, placement.region) {
                    return Some(msg);
                }
            }
        }

        None
    }

    fn count_focusable(&self) -> usize {
        self.children
            .iter()
            .filter(|c| c.is_visible())
            .map(|c| c.count_focusable())
            .sum()
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            if child.is_visible() {
                child.clear_focus();
            }
        }
    }

    fn focus_nth(&mut self, mut n: usize) -> bool {
        for child in &mut self.children {
            if !child.is_visible() {
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

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            if child.is_visible() {
                child.clear_hover();
            }
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

    fn pre_layout(&mut self, _layout: &mut dyn Layout) {
        // Default container doesn't configure layout
        // Override in ItemGrid to set min_column_width, etc.
    }
}
