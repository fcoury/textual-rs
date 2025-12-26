use crate::{Canvas, KeyCode, Message, Region, Size, Widget};

pub struct Vertical {
    pub children: Vec<Box<dyn Widget>>,
}

impl Vertical {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Self {
        Self { children }
    }
}

impl Widget for Vertical {
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

    fn on_event(&mut self, key: KeyCode) -> Option<Message> {
        // For now, we pass the event to ALL children.
        // (We will fix this with Focus in the next step!)
        for child in &mut self.children {
            if let Some(msg) = child.on_event(key) {
                return Some(msg);
            }
        }
        None
    }
}
