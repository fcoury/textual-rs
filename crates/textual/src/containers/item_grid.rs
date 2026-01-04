//! ItemGrid container - a grid with automatic column calculation.
//!
//! ItemGrid extends Grid with runtime-configurable properties via pre_layout:
//! - `min_column_width`: Auto-calculate column count to fit minimum width
//! - `max_column_width`: Limit column widths
//! - `stretch_height`: Force all cells in a row to equal height
//! - `regular`: Ensure no partial rows (even distribution)

use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates, types::Visibility};

use crate::canvas::{Canvas, Region, Size};
use crate::layouts::{self, GridLayout, Layout};
use crate::widget::Widget;
use crate::{KeyCode, MouseEvent};

/// A grid container with automatic column calculation.
///
/// ItemGrid uses the `pre_layout` hook to configure GridLayout at runtime,
/// allowing properties like `min_column_width` to dynamically determine
/// the number of columns based on available width.
///
/// ## Example
///
/// ```ignore
/// use textual::containers::ItemGrid;
///
/// // Creates a grid where columns are at least 20 cells wide
/// let grid = ItemGrid::new(children)
///     .with_min_column_width(20)
///     .with_stretch_height(true);
/// ```
pub struct ItemGrid<M> {
    children: Vec<Box<dyn Widget<M>>>,
    style: ComputedStyle,
    inline_style: StyleOverride,
    dirty: bool,
    id: Option<String>,

    // Runtime-configurable properties (passed to GridLayout via pre_layout)
    // Use the builder methods (with_min_column_width, etc.) to set these
    min_column_width: Option<u16>,
    max_column_width: Option<u16>,
    stretch_height: bool,
    regular: bool,
}

impl<M> ItemGrid<M> {
    /// Create a new ItemGrid with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            style: ComputedStyle::default(),
            inline_style: StyleOverride::default(),
            dirty: true,
            id: None,
            min_column_width: None,
            max_column_width: None,
            stretch_height: true,
            regular: false,
        }
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the minimum column width.
    ///
    /// When set, the grid automatically calculates the number of columns
    /// that fit within the available width while maintaining this minimum.
    pub fn with_min_column_width(mut self, width: u16) -> Self {
        self.min_column_width = Some(width);
        self
    }

    /// Set the maximum column width.
    pub fn with_max_column_width(mut self, width: u16) -> Self {
        self.max_column_width = Some(width);
        self
    }

    /// Set whether to stretch cell heights to match row height.
    pub fn with_stretch_height(mut self, stretch: bool) -> Self {
        self.stretch_height = stretch;
        self
    }

    /// Set whether the grid should be regular (no partial rows).
    pub fn with_regular(mut self, regular: bool) -> Self {
        self.regular = regular;
        self
    }

    /// Count visible children that participate in layout.
    fn visible_children(&self) -> usize {
        self.children
            .iter()
            .filter(|c| c.participates_in_layout())
            .count()
    }

    /// Compute child placements using GridLayout with pre_layout configuration.
    fn compute_child_placements(
        &self,
        region: Region,
        viewport: layouts::Viewport,
    ) -> Vec<layouts::WidgetPlacement> {
        // Collect visible children with their styles and desired sizes
        let children_with_styles: Vec<layouts::LayoutChild> = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.participates_in_layout())
            .map(|(i, c)| layouts::LayoutChild {
                index: i,
                style: c.get_style(),
                desired_size: c.desired_size(),
                node: c,
            })
            .collect();

        // Create and configure GridLayout
        let mut layout = GridLayout::default();
        layout.min_column_width = self.min_column_width;
        layout.max_column_width = self.max_column_width;
        layout.stretch_height = self.stretch_height;
        layout.regular = self.regular;

        let mut placements = layout.arrange(&self.style, &children_with_styles, region, viewport);
        layouts::apply_alignment(&mut placements, &children_with_styles, &self.style, region);
        placements
    }
}

impl<M> Widget<M> for ItemGrid<M> {
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
            let child = &self.children[placement.child_index];
            // Skip rendering if visibility is hidden (but widget still occupies space)
            if child.get_style().visibility == Visibility::Hidden {
                continue;
            }
            child.render(canvas, placement.region);
        }
        canvas.pop_clip();
    }

    fn desired_size(&self) -> Size {
        // Return a reasonable minimum based on visible children
        let visible = self.visible_children() as u16;
        Size::new(visible.max(1) * 10, visible.max(1) * 3)
    }

    fn get_meta(&self) -> WidgetMeta {
        WidgetMeta {
            type_name: "ItemGrid",
            type_names: vec!["ItemGrid", "Widget", "DOMNode"],
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

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inline_style = style;
        self.dirty = true;
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
        self.dirty = true;
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

        // For mouse handling, approximate viewport as region
        let viewport = layouts::Viewport::from(region);
        let placements = self.compute_child_placements(region, viewport);

        for placement in placements {
            if placement.region.contains_point(mx, my) {
                if let Some(msg) =
                    self.children[placement.child_index].on_mouse(event, placement.region)
                {
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

    fn pre_layout(&mut self, layout: &mut dyn Layout) {
        // Configure GridLayout with our runtime properties
        if let Some(grid) = layout.as_grid_mut() {
            grid.min_column_width = self.min_column_width;
            grid.max_column_width = self.max_column_width;
            grid.stretch_height = self.stretch_height;
            grid.regular = self.regular;
        }
    }
}
