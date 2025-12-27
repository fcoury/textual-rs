pub mod horizontal;
pub mod vertical;

use crate::KeyCode;
use crate::canvas::{Canvas, Region, Size};
use crate::widget::Widget;

/// Centered horizontally
pub struct Center<M> {
    child: Box<dyn Widget<M>>,
}

impl<M> Center<M> {
    pub fn new(child: Box<dyn Widget<M>>) -> Self {
        Self { child }
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

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.child.on_event(key)
    }
}

/// Centered vertically
pub struct Middle<M> {
    pub child: Box<dyn Widget<M>>,
}

impl<M> Middle<M> {
    pub fn new(child: Box<dyn Widget<M>>) -> Self {
        Self { child }
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

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.child.on_event(key)
    }
}
