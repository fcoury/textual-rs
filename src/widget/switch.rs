use crate::{Canvas, Region, Size, Widget};

pub struct Switch {
    pub on: bool,
}

impl Switch {
    pub fn new() -> Self {
        Self { on: false }
    }
}

impl Widget for Switch {
    fn desired_size(&self) -> Size {
        Size {
            width: 8,
            height: 3,
        } // Fixed size for the switch widget
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        // Draw a simple border and the status
        let label = if self.on { "[  ON ]" } else { "[ OFF  ]" };

        // We use the region's x/y which was calculated by the containers
        canvas.put_str(region.x, region.y, "┌──────┐");
        canvas.put_str(region.x, region.y + 1, label);
        canvas.put_str(region.x, region.y + 2, "└──────┘");
    }
}
