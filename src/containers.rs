pub mod vertical;

use crate::canvas::{Canvas, Region, Size};
use crate::widget::Widget;
use crate::{KeyCode, Message};

/// Centered horizontally
pub struct Center {
    child: Box<dyn Widget>,
}

impl Center {
    pub fn new(child: Box<dyn Widget + 'static>) -> Self {
        Self { child }
    }
}

impl Widget for Center {
    fn desired_size(&self) -> Size {
        // Center takes the child's height but wants to be flexible in width.
        // For simplicity, we return the child's size requirements.
        self.child.desired_size()
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let child_size = self.child.desired_size();

        // Calculate x-offset to center the child within the assigned region
        let x_offset = region.width.saturating_sub(child_size.width) / 2;

        let centered_region = Region {
            x: region.x + x_offset,
            y: region.y,
            width: child_size.width,
            height: region.height,
        };

        self.child.render(canvas, centered_region);
    }

    fn on_event(&mut self, key: KeyCode) -> Option<Message> {
        // Pass the event down to the child
        self.child.on_event(key)
    }
}

/// Centered vertically
pub struct Middle {
    pub child: Box<dyn Widget>,
}

impl Middle {
    pub fn new(child: Box<dyn Widget>) -> Self {
        Self { child }
    }
}

impl Widget for Middle {
    fn desired_size(&self) -> Size {
        // Middle takes the child's width but wants to be flexible in height.
        self.child.desired_size()
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        let child_size = self.child.desired_size();

        // Calculate y-offset to middle the child within the assigned region
        let y_offset = region.height.saturating_sub(child_size.height) / 2;

        let middled_region = Region {
            x: region.x,
            y: region.y + y_offset,
            width: region.width,
            height: child_size.height,
        };

        self.child.render(canvas, middled_region);
    }

    fn on_event(&mut self, key: KeyCode) -> Option<Message> {
        // Pass the event down to the child
        self.child.on_event(key)
    }
}
