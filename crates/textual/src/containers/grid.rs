//! Grid container for CSS Grid-like layouts.
//!
//! Implements a 2D grid layout with support for:
//! - Fixed column/row counts via `grid-size`
//! - Flexible column/row sizes via `grid-columns` and `grid-rows`
//! - Gutter spacing via `grid-gutter`
//! - Column/row spanning for children via `row-span` and `column-span`
//!
//! ## CSS Properties
//!
//! ```css
//! Grid {
//!     grid-size: 3;              /* 3 columns, auto rows */
//!     grid-size: 3 2;            /* 3 columns, 2 rows */
//!     grid-columns: 1fr 2fr 1fr; /* flexible widths */
//!     grid-rows: 5 auto;         /* fixed + auto heights */
//!     grid-gutter: 1;            /* 1 cell spacing */
//!     grid-gutter: 1 2;          /* vertical horizontal */
//! }
//!
//! /* Child spanning */
//! #my-widget {
//!     row-span: 2;               /* span 2 rows */
//!     column-span: 3;            /* span 3 columns */
//! }
//! ```

use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

use crate::layouts::{self, Layout};
use crate::{Canvas, KeyCode, MouseEvent, Region, Size, Widget};

/// A grid container that arranges children in a 2D grid.
///
/// Children are placed left-to-right, top-to-bottom using Tetris-style
/// placement. Grid size and layout are controlled via CSS.
///
/// This is a convenience wrapper that uses the `layouts::GridLayout` algorithm.
pub struct Grid<M> {
    children: Vec<Box<dyn Widget<M>>>,
    style: ComputedStyle,
    dirty: bool,
    id: Option<String>,
}

impl<M> Grid<M> {
    /// Create a new Grid with the given children.
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

    /// Compute child placements using GridLayout.
    fn compute_child_placements(
        &self,
        region: Region,
        viewport: layouts::Viewport,
    ) -> Vec<layouts::WidgetPlacement> {
        // Collect visible children with their styles and desired sizes
        let children_with_styles: Vec<_> = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.participates_in_layout())
            .map(|(i, c)| (i, c.get_style(), c.desired_size()))
            .collect();

        // Force grid layout regardless of CSS
        let mut layout = layouts::GridLayout::default();
        layout.arrange(&self.style, &children_with_styles, region, viewport)
    }
}

impl<M> Widget<M> for Grid<M> {
    fn default_css(&self) -> &'static str {
        // Match Python Textual's Grid DEFAULT_CSS
        r#"
Grid {
    width: 1fr;
    height: 1fr;
    layout: grid;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        // 1. Render background/border and get inner region
        let inner_region = crate::containers::render_container_chrome(canvas, region, &self.style);

        // 2. Render children in inner region using the canvas viewport
        let viewport = canvas.viewport();
        canvas.push_clip(inner_region);
        for placement in self.compute_child_placements(inner_region, viewport) {
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
            u16::MAX // Grid expands to fill available space by default
        };

        let height = if let Some(h) = &self.style.height {
            use tcss::types::Unit;
            match h.unit {
                Unit::Cells => h.value as u16,
                _ => u16::MAX, // Fill available space
            }
        } else {
            u16::MAX // Grid expands to fill available space by default
        };

        Size::new(width, height)
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "Grid".to_string(),
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

        // Compute placements first (borrows self immutably)
        // For mouse handling, approximate viewport as region
        let viewport = layouts::Viewport::from(region);
        let placements = self.compute_child_placements(region, viewport);

        // Then iterate and dispatch mouse events (borrows self mutably)
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
            .filter(|c| c.participates_in_layout())
            .map(|c| c.count_focusable())
            .sum()
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            if child.participates_in_layout() {
                child.clear_focus();
            }
        }
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

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            if child.participates_in_layout() {
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
        // Grid container doesn't configure layout at runtime
        // Override in ItemGrid for min_column_width, etc.
    }
}

// Tests for OccupancyGrid, ResolvedTrack, and child_region have been moved to
// crates/textual/src/layouts/grid.rs where the layout algorithm is now implemented.
