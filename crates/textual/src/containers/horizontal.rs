use crate::{Canvas, KeyCode, MouseEvent, Region, Size, Widget};

pub struct Horizontal<M> {
    pub children: Vec<Box<dyn Widget<M>>>,
    dirty: bool,
}

impl<M> Horizontal<M> {
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            dirty: true, // Start dirty so initial styles are computed
        }
    }
}

impl<M> Widget<M> for Horizontal<M> {
    fn desired_size(&self) -> Size {
        let mut width = 0;
        let mut height = 0;
        for child in &self.children {
            let size = child.desired_size();
            width += size.width;
            height = height.max(size.height);
        }
        Size { width, height }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let mut current_x = region.x;
        for child in &self.children {
            let size = child.desired_size();
            let child_region = Region {
                x: current_x,
                y: region.y,
                width: size.width,
                height: region.height,
            };
            child.render(canvas, child_region);
            current_x += size.width;
        }
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        for child in &mut self.children {
            f(child.as_mut());
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
        for child in &mut self.children {
            if let Some(msg) = child.on_event(key) {
                return Some(msg);
            }
        }
        None
    }

    fn count_focusable(&self) -> usize {
        self.children.iter().map(|c| c.count_focusable()).sum()
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            child.clear_focus();
        }
    }

    fn focus_nth(&mut self, mut n: usize) -> bool {
        for child in &mut self.children {
            let count = child.count_focusable();
            if n < count {
                return child.focus_nth(n);
            }
            n -= count;
        }
        false
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        // Compute child regions (same logic as render) and delegate
        let mut current_x = region.x;
        for child in &mut self.children {
            let size = child.desired_size();
            let child_region = Region {
                x: current_x,
                y: region.y,
                width: size.width,
                height: region.height,
            };

            // Delegate to child - it will handle its own hit testing
            if let Some(msg) = child.on_mouse(event, child_region) {
                return Some(msg);
            }
            current_x += size.width;
        }
        None
    }

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            child.clear_hover();
        }
    }
}
