pub mod horizontal;
pub mod vertical;

use crate::KeyCode;
use crate::MouseEvent;
use crate::canvas::{Canvas, Region, Size};
use crate::widget::Widget;

/// Centered horizontally
pub struct Center<M> {
    child: Box<dyn Widget<M>>,
    dirty: bool,
}

impl<M> Center<M> {
    pub fn new(child: Box<dyn Widget<M>>) -> Self {
        Self {
            child,
            dirty: true, // Start dirty so initial styles are computed
        }
    }
}

impl<M> Widget<M> for Center<M> {
    fn desired_size(&self) -> Size {
        self.child.desired_size()
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let child_size = self.child.desired_size();

        let x_offset = region.width.saturating_sub(child_size.width) / 2;

        let centered_region = Region {
            x: region.x + x_offset,
            y: region.y,
            width: child_size.width,
            height: region.height,
        };

        self.child.render(canvas, centered_region);
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        f(self.child.as_mut());
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
        self.child.on_event(key)
    }

    fn count_focusable(&self) -> usize {
        self.child.count_focusable()
    }

    fn clear_focus(&mut self) {
        self.child.clear_focus();
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.child.focus_nth(n)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        // Compute child region (same as render) and delegate
        let child_size = self.child.desired_size();
        let x_offset = region.width.saturating_sub(child_size.width) / 2;
        let centered_region = Region {
            x: region.x + x_offset,
            y: region.y,
            width: child_size.width,
            height: region.height,
        };
        self.child.on_mouse(event, centered_region)
    }

    fn clear_hover(&mut self) {
        self.child.clear_hover();
    }
}

/// Centered vertically
pub struct Middle<M> {
    pub child: Box<dyn Widget<M>>,
    dirty: bool,
}

impl<M> Middle<M> {
    pub fn new(child: Box<dyn Widget<M>>) -> Self {
        Self {
            child,
            dirty: true, // Start dirty so initial styles are computed
        }
    }
}

impl<M> Widget<M> for Middle<M> {
    fn desired_size(&self) -> Size {
        self.child.desired_size()
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let child_size = self.child.desired_size();

        let y_offset = region.height.saturating_sub(child_size.height) / 2;

        let middled_region = Region {
            x: region.x,
            y: region.y + y_offset,
            width: region.width,
            height: child_size.height,
        };

        self.child.render(canvas, middled_region);
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        f(self.child.as_mut());
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
        self.child.on_event(key)
    }

    fn count_focusable(&self) -> usize {
        self.child.count_focusable()
    }

    fn clear_focus(&mut self) {
        self.child.clear_focus();
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.child.focus_nth(n)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        // Compute child region (same as render) and delegate
        let child_size = self.child.desired_size();
        let y_offset = region.height.saturating_sub(child_size.height) / 2;
        let middled_region = Region {
            x: region.x,
            y: region.y + y_offset,
            width: region.width,
            height: child_size.height,
        };
        self.child.on_mouse(event, middled_region)
    }

    fn clear_hover(&mut self) {
        self.child.clear_hover();
    }
}
