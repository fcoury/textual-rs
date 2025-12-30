use crate::{Canvas, KeyCode, MouseEvent, Region, Size, Widget};
use tcss::ComputedStyle;

pub struct Horizontal<M> {
    pub children: Vec<Box<dyn Widget<M>>>,
    style: ComputedStyle,
    dirty: bool,
    id: Option<String>,
}

impl<M> Horizontal<M> {
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            children,
            style: ComputedStyle::default(),
            dirty: true, // Start dirty so initial styles are computed
            id: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl<M> Widget<M> for Horizontal<M> {
    fn desired_size(&self) -> Size {
        let mut width = 0;
        let mut height = 0;
        for child in &self.children {
            if !child.participates_in_layout() {
                continue;
            }
            let size = child.desired_size();
            width += size.width;
            height = height.max(size.height);
        }
        Size { width, height }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if region.width <= 0 || region.height <= 0 {
            return;
        }

        // 1. Render background/border and get inner region
        let inner_region = crate::containers::render_container_chrome(canvas, region, &self.style);

        canvas.push_clip(inner_region);

        let mut current_x = inner_region.x;
        for child in &self.children {
            if !child.participates_in_layout() {
                continue;
            }

            let size = child.desired_size();
            let child_width = size.width as i32;

            let child_region = Region {
                x: current_x,
                y: inner_region.y,
                width: child_width,
                height: inner_region.height,
            };

            child.render(canvas, child_region);
            current_x += child_width;
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

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        // Pass event to visible children until one handles it
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

    fn count_focusable(&self) -> usize {
        self.children
            .iter()
            .filter(|c| c.participates_in_layout())
            .map(|c| c.count_focusable())
            .sum()
    }

    fn clear_focus(&mut self) {
        for child in &mut self.children {
            if !child.participates_in_layout() {
                continue;
            }
            child.clear_focus();
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

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        let mx = event.column as i32;
        let my = event.row as i32;

        if !region.contains_point(mx, my) {
            return None;
        }

        let mut current_x = region.x;
        for child in &mut self.children {
            if !child.participates_in_layout() {
                continue;
            }

            let size = child.desired_size();
            let child_width = size.width as i32;

            let child_region = Region {
                x: current_x,
                y: region.y,
                width: child_width,
                height: region.height,
            };

            if child_region.contains_point(mx, my) {
                if let Some(msg) = child.on_mouse(event, child_region) {
                    return Some(msg);
                }
            }
            current_x += child_width;
        }
        None
    }

    fn clear_hover(&mut self) {
        for child in &mut self.children {
            if !child.participates_in_layout() {
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

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}
