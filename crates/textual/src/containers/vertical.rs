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
        let mut width = 0;
        let mut height = 0;
        for child in &self.children {
            if !child.is_visible() {
                continue;
            }
            let size = child.desired_size();
            width = width.max(size.width);
            height += size.height;
        }
        Size { width, height }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        canvas.push_clip(region);

        let mut current_y = region.y;
        for child in &self.children {
            if !child.is_visible() {
                continue;
            }

            let size = child.desired_size();
            let child_height = size.height as i32;

            let child_region = Region {
                x: region.x,
                y: current_y,
                width: region.width,
                height: child_height,
            };

            child.render(canvas, child_region);
            current_y += child_height;
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
        for child in &mut self.children {
            if !child.is_visible() {
                continue;
            }

            let size = child.desired_size();
            let child_height = size.height as i32;

            let child_region = Region {
                x: region.x,
                y: current_y,
                width: region.width,
                height: child_height,
            };

            if child_region.contains_point(mx, my) {
                if let Some(msg) = child.on_mouse(event, child_region) {
                    return Some(msg);
                }
            }
            current_y += child_height;
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
