use crate::{Region, Size, Widget};

struct Center<W: Widget> {
    child: W,
}

impl<W: Widget> Widget for Center<W> {
    fn desired_size(&self) -> Size {
        // Center usually wants to take up all available horizontal space
        self.child.desired_size()
    }

    fn render(&self, region: Region) {
        let child_size = self.child.desired_size();

        // Calculate the horizontal offset to center the child
        let x_offset = (region.width.saturating_sub(child_size.width)) / 2;

        let child_region = Region {
            x: region.x + x_offset,
            y: region.y,
            width: child_size.width,
            height: child_size.height,
        };

        self.child.render(child_region);
    }
}
