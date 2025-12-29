use crate::layouts::{resolve_height_fixed, resolve_width_fill};
use crate::{Canvas, KeyCode, MouseEvent, Region, Size, Widget};

pub struct Vertical<M> {
    pub children: Vec<Box<dyn Widget<M>>>,
    dirty: bool,
}

impl<M> Vertical<M> {
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            dirty: true, // Start dirty so initial styles are computed
        }
    }
}

impl<M> Widget<M> for Vertical<M> {
    fn desired_size(&self) -> Size {
        let mut width: u16 = 0;
        let mut height: u16 = 0;
        let mut prev_margin_bottom: u16 = 0;
        let mut is_first = true;

        for child in &self.children {
            if !child.is_visible() {
                continue;
            }
            let size = child.desired_size();
            let style = child.get_style();

            // Account for margins
            let margin_top = style.margin.top.value as u16;
            let margin_bottom = style.margin.bottom.value as u16;
            let margin_left = style.margin.left.value as u16;
            let margin_right = style.margin.right.value as u16;

            width = width.max(size.width.saturating_add(margin_left).saturating_add(margin_right));

            // CSS margin collapsing: use max of adjacent margins, not sum
            let effective_top_margin = if is_first {
                margin_top
            } else {
                // Collapsed gap = max(prev_bottom, current_top)
                // We already added prev_margin_bottom, so add the difference if current is larger
                margin_top.saturating_sub(prev_margin_bottom)
            };

            height = height.saturating_add(size.height.saturating_add(effective_top_margin).saturating_add(margin_bottom));
            prev_margin_bottom = margin_bottom;
            is_first = false;
        }
        Size { width, height }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        canvas.push_clip(region);

        let mut current_y = region.y;
        let mut prev_margin_bottom: i32 = 0;
        let mut is_first = true;

        for child in &self.children {
            if !child.is_visible() {
                continue;
            }

            let child_style = child.get_style();

            // Resolve dimensions from child's CSS style
            // Vertical container: children fill width, have fixed/auto height
            let child_height = resolve_height_fixed(&child_style, region.height);
            let child_width = resolve_width_fill(&child_style, region.width);

            // Get margin (Scalar.value is f64)
            let margin_top = child_style.margin.top.value as i32;
            let margin_left = child_style.margin.left.value as i32;
            let margin_right = child_style.margin.right.value as i32;
            let margin_bottom = child_style.margin.bottom.value as i32;

            // CSS margin collapsing: use max of adjacent margins, not sum
            let effective_top_margin = if is_first {
                margin_top
            } else {
                // Collapsed gap = max(prev_bottom, current_top)
                // We already advanced by prev_margin_bottom, so add the difference if current is larger
                (margin_top - prev_margin_bottom).max(0)
            };

            current_y += effective_top_margin;

            // Reduce child width by horizontal margins to prevent overflow
            let adjusted_width = (child_width - margin_left - margin_right).max(0);

            let child_region = Region {
                x: region.x + margin_left,
                y: current_y,
                width: adjusted_width,
                height: child_height,
            };

            child.render(canvas, child_region);
            current_y += child_height + margin_bottom;
            prev_margin_bottom = margin_bottom;
            is_first = false;
        }

        canvas.pop_clip();
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

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        // Pass event to visible children until one handles it
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

    fn count_focusable(&self) -> usize {
        self.children
            .iter()
            .filter(|c| c.is_visible())
            .map(|c| c.count_focusable())
            .sum()
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            if !child.is_visible() {
                continue;
            }
            child.clear_focus();
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

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;

        if !region.contains_point(mx, my) {
            return None;
        }

        let mut current_y = region.y;
        let mut prev_margin_bottom: i32 = 0;
        let mut is_first = true;

        for child in &mut self.children {
            if !child.is_visible() {
                continue;
            }

            let child_style = child.get_style();

            // Resolve dimensions from child's CSS style
            // Vertical container: children fill width, have fixed/auto height
            let child_height = resolve_height_fixed(&child_style, region.height);
            let child_width = resolve_width_fill(&child_style, region.width);

            // Get margin (Scalar.value is f64)
            let margin_top = child_style.margin.top.value as i32;
            let margin_left = child_style.margin.left.value as i32;
            let margin_right = child_style.margin.right.value as i32;
            let margin_bottom = child_style.margin.bottom.value as i32;

            // CSS margin collapsing: use max of adjacent margins, not sum
            let effective_top_margin = if is_first {
                margin_top
            } else {
                (margin_top - prev_margin_bottom).max(0)
            };

            current_y += effective_top_margin;

            // Reduce child width by horizontal margins to prevent overflow
            let adjusted_width = (child_width - margin_left - margin_right).max(0);

            let child_region = Region {
                x: region.x + margin_left,
                y: current_y,
                width: adjusted_width,
                height: child_height,
            };

            if child_region.contains_point(mx, my) {
                if let Some(msg) = child.on_mouse(event, child_region) {
                    return Some(msg);
                }
            }
            current_y += child_height + margin_bottom;
            prev_margin_bottom = margin_bottom;
            is_first = false;
        }
        None
    }

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            if !child.is_visible() {
                continue;
            }
            child.clear_hover();
        }
    }

    // Note: child_count and get_child_mut return ALL children for tree traversal
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
