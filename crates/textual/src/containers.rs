pub mod horizontal;
pub mod scrollable;
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

    /// Calculate the centered region for the child within the given region.
    fn centered_region(&self, region: Region) -> Region {
        let child_size = self.child.desired_size();
        let child_width = child_size.width as i32;
        let x_offset = (region.width - child_width).max(0) / 2;
        Region {
            x: region.x + x_offset,
            y: region.y,
            width: child_width,
            height: region.height,
        }
    }
}

impl<M> Widget<M> for Center<M> {
    fn desired_size(&self) -> Size {
        if !self.child.is_visible() {
            return Size { width: 0, height: 0 };
        }
        self.child.desired_size()
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if !self.child.is_visible() {
            return;
        }
        self.child.render(canvas, self.centered_region(region));
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
        if !self.child.is_visible() {
            return None;
        }
        self.child.on_event(key)
    }

    fn count_focusable(&self) -> usize {
        if !self.child.is_visible() {
            return 0;
        }
        self.child.count_focusable()
    }

    fn clear_focus(&mut self) {
        if self.child.is_visible() {
            self.child.clear_focus();
        }
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        if !self.child.is_visible() {
            return false;
        }
        self.child.focus_nth(n)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        if !self.child.is_visible() {
            return None;
        }
        self.child.on_mouse(event, self.centered_region(region))
    }

    fn clear_hover(&mut self) {
        if self.child.is_visible() {
            self.child.clear_hover();
        }
    }

    // Note: child_count and get_child_mut return child for tree traversal
    fn child_count(&self) -> usize {
        1
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        if index == 0 {
            Some(self.child.as_mut())
        } else {
            None
        }
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

    /// Calculate the vertically centered region for the child within the given region.
    fn middled_region(&self, region: Region) -> Region {
        let child_size = self.child.desired_size();
        let child_height = child_size.height as i32;
        let y_offset = (region.height - child_height).max(0) / 2;
        Region {
            x: region.x,
            y: region.y + y_offset,
            width: region.width,
            height: child_height,
        }
    }
}

impl<M> Widget<M> for Middle<M> {
    fn desired_size(&self) -> Size {
        if !self.child.is_visible() {
            return Size { width: 0, height: 0 };
        }
        self.child.desired_size()
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if !self.child.is_visible() {
            return;
        }
        self.child.render(canvas, self.middled_region(region));
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
        if !self.child.is_visible() {
            return None;
        }
        self.child.on_event(key)
    }

    fn count_focusable(&self) -> usize {
        if !self.child.is_visible() {
            return 0;
        }
        self.child.count_focusable()
    }

    fn clear_focus(&mut self) {
        if self.child.is_visible() {
            self.child.clear_focus();
        }
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        if !self.child.is_visible() {
            return false;
        }
        self.child.focus_nth(n)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        if !self.child.is_visible() {
            return None;
        }
        self.child.on_mouse(event, self.middled_region(region))
    }

    fn clear_hover(&mut self) {
        if self.child.is_visible() {
            self.child.clear_hover();
        }
    }

    // Note: child_count and get_child_mut return child for tree traversal
    fn child_count(&self) -> usize {
        1
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        if index == 0 {
            Some(self.child.as_mut())
        } else {
            None
        }
    }
}
