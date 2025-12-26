use crate::{Canvas, KeyCode, Region, Size, Widget};

pub struct Vertical<M> {
    pub children: Vec<Box<dyn Widget<M>>>,
}

impl<M> Vertical<M> {
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self { children }
    }
}

impl<M> Widget<M> for Vertical<M> {
    fn desired_size(&self) -> Size {
        let mut width = 0;
        let mut height = 0;
        for child in &self.children {
            let size = child.desired_size();
            width = width.max(size.width);
            height += size.height;
        }
        Size { width, height }
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let mut current_y = region.y;
        for child in &self.children {
            let size = child.desired_size();
            let child_region = Region {
                x: region.x,
                y: current_y,
                width: region.width,
                height: size.height,
            };
            child.render(canvas, child_region);
            current_y += size.height;
        }
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        // Pass event to children until one handles it
        for child in &mut self.children {
            if let Some(msg) = child.on_event(key) {
                return Some(msg);
            }
        }
        None
    }
}
