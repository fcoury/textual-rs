pub mod switch;

use crate::{
    KeyCode, Message, Size,
    canvas::{Canvas, Region},
};

pub trait Widget {
    /// Draw the widget onto the provided canvas within the specified region.
    /// This is where the 'pixels' (characters) are set.
    fn render(&self, canvas: &mut Canvas, region: Region);

    /// Tell the parent container how much space this widget needs.
    /// For example, a Switch might always return width: 8, height: 3.
    fn desired_size(&self) -> Size;

    fn set_focus(&mut self, _is_focused: bool) {}

    fn is_focused(&self) -> bool {
        false
    }

    // Allow widgets to respond to keys and bubble up a message
    fn on_event(&mut self, _key: KeyCode) -> Option<Message> {
        None
    }
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
