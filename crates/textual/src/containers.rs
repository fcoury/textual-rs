pub mod container;
pub mod grid;
pub mod horizontal;
pub mod item_grid;
pub mod scrollable;
pub mod vertical;

use crate::KeyCode;
use crate::MouseEvent;
use crate::canvas::{Canvas, Region, Size};
use crate::widget::Widget;

/// Centered horizontally
pub struct Center<M> {
    children: Vec<Box<dyn Widget<M>>>,
    dirty: bool,
}

impl<M> Center<M> {
    /// Creates a new Center container with the given children.
    ///
    /// # Panics
    /// Panics if `children` does not contain exactly one child.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        assert!(
            children.len() == 1,
            "Center requires exactly 1 child, got {}",
            children.len()
        );
        Self {
            children,
            dirty: true, // Start dirty so initial styles are computed
        }
    }

    /// Creates a new Center container with a single child.
    pub fn from_child(child: Box<dyn Widget<M>>) -> Self {
        Self::new(vec![child])
    }

    fn child(&self) -> &dyn Widget<M> {
        self.children[0].as_ref()
    }

    fn child_mut(&mut self) -> &mut dyn Widget<M> {
        self.children[0].as_mut()
    }

    /// Calculate the centered region for the child within the given region.
    fn centered_region(&self, region: Region) -> Region {
        let child_size = self.child().desired_size();
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
        if !self.child().is_visible() {
            return Size { width: 0, height: 0 };
        }
        self.child().desired_size()
    }

    fn on_resize(&mut self, size: Size) {
        self.child_mut().on_resize(size);
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if !self.child().is_visible() {
            return;
        }
        self.child().render(canvas, self.centered_region(region));
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
        if !self.child().is_visible() {
            return None;
        }
        self.child_mut().on_event(key)
    }

    fn count_focusable(&self) -> usize {
        if !self.child().is_visible() {
            return 0;
        }
        self.child().count_focusable()
    }

    fn clear_focus(&mut self) {
        if self.child().is_visible() {
            self.child_mut().clear_focus();
        }
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        if !self.child().is_visible() {
            return false;
        }
        self.child_mut().focus_nth(n)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        if !self.child().is_visible() {
            return None;
        }
        let centered = self.centered_region(region);
        self.child_mut().on_mouse(event, centered)
    }

    fn clear_hover(&mut self) {
        if self.child().is_visible() {
            self.child_mut().clear_hover();
        }
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        self.children.get_mut(index).map(|c| c.as_mut() as &mut dyn Widget<M>)
    }
}

/// Centered vertically
pub struct Middle<M> {
    children: Vec<Box<dyn Widget<M>>>,
    dirty: bool,
}

impl<M> Middle<M> {
    /// Creates a new Middle container with the given children.
    ///
    /// # Panics
    /// Panics if `children` does not contain exactly one child.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        assert!(
            children.len() == 1,
            "Middle requires exactly 1 child, got {}",
            children.len()
        );
        Self {
            children,
            dirty: true, // Start dirty so initial styles are computed
        }
    }

    /// Creates a new Middle container with a single child.
    pub fn from_child(child: Box<dyn Widget<M>>) -> Self {
        Self::new(vec![child])
    }

    fn child(&self) -> &dyn Widget<M> {
        self.children[0].as_ref()
    }

    fn child_mut(&mut self) -> &mut dyn Widget<M> {
        self.children[0].as_mut()
    }

    /// Calculate the vertically centered region for the child within the given region.
    fn middled_region(&self, region: Region) -> Region {
        let child_size = self.child().desired_size();
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
        if !self.child().is_visible() {
            return Size { width: 0, height: 0 };
        }
        self.child().desired_size()
    }

    fn on_resize(&mut self, size: Size) {
        self.child_mut().on_resize(size);
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if !self.child().is_visible() {
            return;
        }
        self.child().render(canvas, self.middled_region(region));
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
        if !self.child().is_visible() {
            return None;
        }
        self.child_mut().on_event(key)
    }

    fn count_focusable(&self) -> usize {
        if !self.child().is_visible() {
            return 0;
        }
        self.child().count_focusable()
    }

    fn clear_focus(&mut self) {
        if self.child().is_visible() {
            self.child_mut().clear_focus();
        }
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        if !self.child().is_visible() {
            return false;
        }
        self.child_mut().focus_nth(n)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        if !self.child().is_visible() {
            return None;
        }
        let middled = self.middled_region(region);
        self.child_mut().on_mouse(event, middled)
    }

    fn clear_hover(&mut self) {
        if self.child().is_visible() {
            self.child_mut().clear_hover();
        }
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        self.children.get_mut(index).map(|c| c.as_mut() as &mut dyn Widget<M>)
    }
}
