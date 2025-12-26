use crate::{Canvas, KeyCode, Region, Size, Widget};

pub struct Horizontal<M> {
    pub children: Vec<Box<dyn Widget<M>>>,
}

impl<M> Horizontal<M> {
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self { children }
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

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        for child in &mut self.children {
            if let Some(msg) = child.on_event(key) {
                return Some(msg);
            }
        }
        None
    }
}
