//! Vertical container that arranges children top-to-bottom.
//!
//! This is a thin wrapper around `Container` with `layout: vertical` enforced.

use crate::containers::container::{Container, ContainerLayoutDirection};
use crate::impl_widget_delegation;
use crate::Widget;

/// A container that arranges children vertically (top-to-bottom).
///
/// This is equivalent to a Container with `layout: vertical`, but the
/// layout direction is enforced regardless of CSS settings.
pub struct Vertical<M: 'static> {
    inner: Container<M>,
}

impl<M: 'static> Vertical<M> {
    /// Create a new Vertical container with the given children.
    pub fn new(children: Vec<Box<dyn Widget<M>>>) -> Self {
        Self {
            inner: Container::new(children).with_layout(ContainerLayoutDirection::Vertical),
        }
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner = self.inner.with_id(id);
        self
    }

    /// Set CSS classes.
    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        let classes_str: String = classes.into();
        for class in classes_str.split_whitespace() {
            self.inner.add_class(class);
        }
        self
    }

    /// Set the border title.
    pub fn with_border_title(mut self, title: impl Into<String>) -> Self {
        self.inner = self.inner.with_border_title(title);
        self
    }

    /// Set the border subtitle.
    pub fn with_border_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.inner = self.inner.with_border_subtitle(subtitle);
        self
    }
}

impl_widget_delegation!(Vertical<M> => inner, type_name = "Vertical", default_css = |_| r#"
Vertical {
    width: 1fr;
    height: 1fr;
    layout: vertical;
}
"#);
