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

use tcss::{
    ComputedStyle, StyleOverride, WidgetMeta, WidgetStates, types::Visibility,
    types::keyline::KeylineStyle,
};

use crate::keyline_canvas::KeylineCanvas;
use crate::layouts::{self, GridLayout, Layout};
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
    inline_style: StyleOverride,
    dirty: bool,
    id: Option<String>,
}

impl<M> Grid<M> {
    /// Create a new Grid with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            style: ComputedStyle::default(),
            inline_style: StyleOverride::default(),
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

        // When keylines are enabled, reduce layout area by (2, 2) to make room for border
        // This matches Python Textual's behavior: size -= (2, 2); offset = (1, 1)
        let (layout_region, keyline_offset) = if self.style.keyline.style != KeylineStyle::None {
            let reduced_region = Region {
                x: region.x,
                y: region.y,
                width: (region.width - 2).max(0),
                height: (region.height - 2).max(0),
            };
            (reduced_region, (1, 1))
        } else {
            (region, (0, 0))
        };

        // Force grid layout regardless of CSS
        let mut layout = layouts::GridLayout::default();
        let mut placements =
            layout.arrange(&self.style, &children_with_styles, layout_region, viewport);

        // Offset all placements when keylines are enabled
        if keyline_offset != (0, 0) {
            for placement in &mut placements {
                placement.region.x += keyline_offset.0;
                placement.region.y += keyline_offset.1;
            }
        }

        // Apply post-layout alignment to match Textual's container-level align behavior.
        layouts::apply_alignment(&mut placements, &children_with_styles, &self.style, region);

        placements
    }

    /// Render keylines for the grid layout.
    fn render_keylines(&self, canvas: &mut Canvas, region: Region) {
        // Collect children info for track computation
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

        if children_with_styles.is_empty() {
            return;
        }

        // Compute tracks using reduced region (same as compute_child_placements)
        // This matches Python's approach where layout is computed in reduced space
        let layout_region = Region {
            x: region.x,
            y: region.y,
            width: (region.width - 2).max(0),
            height: (region.height - 2).max(0),
        };

        // Get track info from GridLayout (includes cell occupancy for span-aware rendering)
        let layout = GridLayout::default();
        let track_info =
            layout.compute_track_info(&self.style, &children_with_styles, layout_region);

        if track_info.col_positions.len() < 2 || track_info.row_positions.len() < 2 {
            return;
        }

        // Create keyline canvas using full region (keylines draw at edges)
        let line_type = self.style.keyline.style.line_type();
        let mut keyline_canvas = KeylineCanvas::new(
            region.width as usize,
            region.height as usize,
            line_type,
            self.style.keyline.color.clone(),
        );

        // Convert track positions for keyline rendering:
        // - Outer borders are at edges: 0 and (region_size - 1)
        // - Interior lines are at gutter positions (track start in reduced region maps
        //   to gutter position in full region because outer border takes position 0)
        let col_positions: Vec<usize> = track_info
            .col_positions
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                if i == 0 {
                    0 // Left border at edge
                } else if i == track_info.col_positions.len() - 1 {
                    (region.width - 1).max(0) as usize // Right border at edge
                } else {
                    // Interior keylines: track position in reduced space = gutter position
                    // Add 1 to account for outer border (but gutter is at end of previous cell)
                    // track_pos is start of next column, so track_pos in full space = track_pos + 1
                    // But keyline should be at gutter (end of prev col) = track_pos - 1 + 1 = track_pos
                    x.max(0) as usize
                }
            })
            .collect();
        let row_positions: Vec<usize> = track_info
            .row_positions
            .iter()
            .enumerate()
            .map(|(i, &y)| {
                if i == 0 {
                    0 // Top border at edge
                } else if i == track_info.row_positions.len() - 1 {
                    (region.height - 1).max(0) as usize // Bottom border at edge
                } else {
                    // Interior keylines at gutter positions
                    y.max(0) as usize
                }
            })
            .collect();

        // Use span-aware grid rendering (respects column-span/row-span)
        keyline_canvas.add_grid_with_occupancy(
            &col_positions,
            &row_positions,
            &track_info.cell_occupancy,
        );
        keyline_canvas.render(canvas, region);
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
            let child = &self.children[placement.child_index];
            // Skip rendering if visibility is hidden (but widget still occupies space)
            if child.get_style().visibility == Visibility::Hidden {
                continue;
            }
            child.render(canvas, placement.region);
        }

        // 3. Render keylines on top of children (if enabled)
        if self.style.keyline.style != KeylineStyle::None {
            self.render_keylines(canvas, inner_region);
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
            type_name: "Grid",
            type_names: vec!["Grid", "Widget", "DOMNode"],
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

        // Compute placements first (borrows self immutably)
        // For mouse handling, approximate viewport as region
        let viewport = layouts::Viewport::from(region);
        let placements = self.compute_child_placements(region, viewport);

        // Then iterate and dispatch mouse events (borrows self mutably)
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

    fn pre_layout(&mut self, _layout: &mut dyn Layout) {
        // Grid container doesn't configure layout at runtime
        // Override in ItemGrid for min_column_width, etc.
    }
}

// Tests for OccupancyGrid, ResolvedTrack, and child_region have been moved to
// crates/textual/src/layouts/grid.rs where the layout algorithm is now implemented.
