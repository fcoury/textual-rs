pub mod switch;

use tcss::{ComputedStyle, parser::cascade::WidgetMeta};

use crate::{
    KeyCode, Size,
    canvas::{Canvas, Region},
};

/// A widget that can render itself and handle events.
/// Generic over `M`, the message type that events produce.
pub trait Widget<M> {
    /// Draw the widget onto the provided canvas within the specified region.
    fn render(&self, canvas: &mut Canvas, region: Region);

    /// Tell the parent container how much space this widget needs.
    fn desired_size(&self) -> Size;

    fn for_each_child(&mut self, _f: &mut dyn FnMut(&mut dyn Widget<M>)) {}

    // Metadata for CSS selectors
    fn get_meta(&self) -> WidgetMeta {
        let full_name = std::any::type_name::<Self>();
        // Strip paths and generics: "textual::widget::switch::Switch<M, F>" -> "Switch"
        let type_name = full_name
            .split('<')
            .next()
            .unwrap_or(full_name)
            .split("::")
            .last()
            .unwrap_or(full_name)
            .to_string();

        let mut classes = Vec::new();
        if self.is_focused() {
            classes.push("focus".to_string());
        }

        WidgetMeta {
            type_name,
            id: None,
            classes,
        }
    }

    // Default style management
    fn set_style(&mut self, _style: ComputedStyle) {}

    fn get_style(&self) -> ComputedStyle {
        ComputedStyle::default()
    }

    fn set_focus(&mut self, _is_focused: bool) {}

    fn is_focused(&self) -> bool {
        false
    }

    /// Handle a key event and optionally return a message.
    fn on_event(&mut self, _key: KeyCode) -> Option<M> {
        None
    }
}

/// Allow boxed widgets to be used as widgets.
impl<M> Widget<M> for Box<dyn Widget<M>> {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.as_ref().render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.as_ref().desired_size()
    }

    fn get_style(&self) -> ComputedStyle {
        self.as_ref().get_style()
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.as_mut().set_style(style);
    }

    fn get_meta(&self) -> WidgetMeta {
        self.as_ref().get_meta()
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.as_mut().on_event(key)
    }
}

/// Trait for types that can compose a widget tree.
/// The associated `Message` type defines what events the UI can produce.
pub trait Compose {
    type Message;

    fn compose(&self) -> Box<dyn Widget<Self::Message>>;
}
