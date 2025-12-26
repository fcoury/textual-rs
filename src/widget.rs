pub mod switch;

use crate::{
    Size,
    canvas::{Canvas, Region},
};

pub trait Widget {
    /// Draw the widget onto the provided canvas within the specified region.
    /// This is where the 'pixels' (characters) are set.
    fn render(&self, canvas: &mut Canvas, region: Region);

    /// Tell the parent container how much space this widget needs.
    /// For example, a Switch might always return width: 8, height: 3.
    fn desired_size(&self) -> Size;
}

/// Helper to allow us to store boxed widgets in containers like Center/Middle
impl Widget for Box<dyn Widget> {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.as_ref().render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.as_ref().desired_size()
    }
}

pub trait Compose {
    fn compose(&self) -> Box<dyn Widget>;
}
