//! Grid container for CSS Grid-like layouts.
//!
//! Implements a 2D grid layout with support for:
//! - Fixed column/row counts via `grid-size`
//! - Flexible column/row sizes via `grid-columns` and `grid-rows`
//! - Gutter spacing via `grid-gutter`
//! - Column/row spanning for children
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
//! ```

use tcss::{ComputedStyle, WidgetMeta, WidgetStates};
use tcss::types::{Scalar, Unit};

use crate::{Canvas, KeyCode, MouseEvent, Region, Size, Widget};

/// A grid container that arranges children in a 2D grid.
///
/// Children are placed left-to-right, top-to-bottom.
/// Grid size and layout are controlled via CSS.
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

    /// Get the number of columns from style, defaulting to 1.
    fn column_count(&self) -> usize {
        self.style.grid.columns.unwrap_or(1) as usize
    }

    /// Get the number of rows, either from style or calculated.
    fn row_count(&self, visible_children: usize) -> usize {
        if let Some(rows) = self.style.grid.rows {
            rows as usize
        } else {
            let cols = self.column_count();
            if cols == 0 {
                1
            } else {
                (visible_children + cols - 1) / cols
            }
        }
    }

    /// Count visible children.
    fn visible_children(&self) -> usize {
        self.children.iter().filter(|c| c.is_visible()).count()
    }

    /// Resolve a scalar value to cells given the available space.
    fn resolve_scalar(&self, scalar: &Scalar, available: i32) -> i32 {
        match scalar.unit {
            Unit::Cells => scalar.value as i32,
            Unit::Percent => ((scalar.value / 100.0) * available as f64) as i32,
            Unit::Auto => available, // Auto fills available space
            Unit::Fraction => available, // fr handled specially in distribution
            _ => scalar.value as i32, // Default to treating as cells
        }
    }

    /// Distribute available space among tracks (columns or rows).
    ///
    /// Handles fr units, fixed sizes, and auto sizing.
    fn distribute_space(&self, specs: &[Scalar], count: usize, available: i32, gutter: i32) -> Vec<i32> {
        if count == 0 {
            return vec![];
        }

        // Total gutter space
        let total_gutter = (count.saturating_sub(1)) as i32 * gutter;
        let available_for_tracks = (available - total_gutter).max(0);

        // Get specs for each track (cycling if needed)
        let track_specs: Vec<Scalar> = (0..count)
            .map(|i| {
                if specs.is_empty() {
                    Scalar::cells(1.0) // Default: equal distribution
                } else {
                    specs[i % specs.len()]
                }
            })
            .collect();

        // First pass: calculate fixed sizes and sum fr values
        let mut sizes = vec![0i32; count];
        let mut total_fr = 0.0;
        let mut remaining = available_for_tracks;

        for (i, spec) in track_specs.iter().enumerate() {
            match spec.unit {
                Unit::Fraction => {
                    total_fr += spec.value;
                }
                Unit::Auto => {
                    // Auto gets equal share of remaining space
                    // Will be calculated after fixed sizes
                }
                _ => {
                    let size = self.resolve_scalar(spec, available_for_tracks);
                    sizes[i] = size;
                    remaining -= size;
                }
            }
        }

        // Count auto tracks
        let auto_count = track_specs.iter().filter(|s| s.unit == Unit::Auto).count();

        // Second pass: distribute remaining space to fr and auto
        if total_fr > 0.0 {
            let fr_unit = remaining as f64 / total_fr;
            for (i, spec) in track_specs.iter().enumerate() {
                if spec.unit == Unit::Fraction {
                    sizes[i] = (spec.value * fr_unit) as i32;
                }
            }
        } else if auto_count > 0 {
            // No fr units, distribute to auto
            let auto_size = remaining / auto_count as i32;
            for (i, spec) in track_specs.iter().enumerate() {
                if spec.unit == Unit::Auto {
                    sizes[i] = auto_size;
                }
            }
        } else if specs.is_empty() {
            // No specs at all: equal distribution
            let per_track = available_for_tracks / count as i32;
            for size in &mut sizes {
                *size = per_track;
            }
        }

        // Ensure minimum size of 1 for each track
        for size in &mut sizes {
            if *size < 1 {
                *size = 1;
            }
        }

        sizes
    }

}

/// Calculate the region for a child at the given grid position.
fn child_region(
    col: usize,
    row: usize,
    col_widths: &[i32],
    row_heights: &[i32],
    region: Region,
    gutter_h: i32,
    gutter_v: i32,
) -> Region {
    // Calculate x position
    let mut x = region.x;
    for (i, &w) in col_widths.iter().enumerate() {
        if i >= col {
            break;
        }
        x += w + gutter_h;
    }

    // Calculate y position
    let mut y = region.y;
    for (i, &h) in row_heights.iter().enumerate() {
        if i >= row {
            break;
        }
        y += h + gutter_v;
    }

    let width = col_widths.get(col).copied().unwrap_or(1);
    let height = row_heights.get(row).copied().unwrap_or(1);

    Region { x, y, width, height }
}

impl<M> Widget<M> for Grid<M> {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        canvas.push_clip(region);

        let visible_count = self.visible_children();
        if visible_count == 0 {
            canvas.pop_clip();
            return;
        }

        let cols = self.column_count();
        let rows = self.row_count(visible_count);

        // Get gutter values
        let gutter_v = self.resolve_scalar(&self.style.grid.gutter.0, region.height);
        let gutter_h = self.resolve_scalar(&self.style.grid.gutter.1, region.width);

        // Distribute space
        let col_widths = self.distribute_space(
            &self.style.grid.column_widths,
            cols,
            region.width,
            gutter_h,
        );
        let row_heights = self.distribute_space(
            &self.style.grid.row_heights,
            rows,
            region.height,
            gutter_v,
        );

        // Render children
        let mut col = 0;
        let mut row = 0;

        for child in &self.children {
            if !child.is_visible() {
                continue;
            }

            if row >= rows {
                break; // No more space
            }

            let cell_region = child_region(
                col, row,
                &col_widths, &row_heights,
                region, gutter_h, gutter_v,
            );

            child.render(canvas, cell_region);

            // Advance position
            col += 1;
            if col >= cols {
                col = 0;
                row += 1;
            }
        }

        canvas.pop_clip();
    }

    fn desired_size(&self) -> Size {
        // Grid fills available space; return reasonable minimum
        Size::new(
            self.column_count() as u16 * 10,
            self.row_count(self.visible_children()) as u16 * 3,
        )
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

        let visible_count = self.visible_children();
        if visible_count == 0 {
            return None;
        }

        let cols = self.column_count();
        let rows = self.row_count(visible_count);

        let gutter_v = self.resolve_scalar(&self.style.grid.gutter.0, region.height);
        let gutter_h = self.resolve_scalar(&self.style.grid.gutter.1, region.width);

        let col_widths = self.distribute_space(
            &self.style.grid.column_widths,
            cols,
            region.width,
            gutter_h,
        );
        let row_heights = self.distribute_space(
            &self.style.grid.row_heights,
            rows,
            region.height,
            gutter_v,
        );

        let mut col = 0;
        let mut row = 0;

        for child in &mut self.children {
            if !child.is_visible() {
                continue;
            }

            if row >= rows {
                break;
            }

            let cell_region = child_region(
                col, row,
                &col_widths, &row_heights,
                region, gutter_h, gutter_v,
            );

            if cell_region.contains_point(mx, my) {
                if let Some(msg) = child.on_mouse(event, cell_region) {
                    return Some(msg);
                }
            }

            col += 1;
            if col >= cols {
                col = 0;
                row += 1;
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
}
