//! Header widget for displaying a title and optional subtitle.
//!
//! This is a minimal Rust analogue of Textual's Header widget, focused on
//! rendering the formatted title string with sensible defaults.

use std::marker::PhantomData;

use crate::{Static, Widget, impl_widget_delegation};

/// A header widget that displays a title and optional subtitle.
#[derive(Debug, Clone)]
pub struct Header<M: 'static> {
    inner: Static<M>,
    title: String,
    subtitle: Option<String>,
    tall: bool,
    _phantom: PhantomData<M>,
}

impl<M: 'static> Header<M> {
    /// Create a new Header with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        let content = Self::format_content(&title, None);
        Self {
            inner: Static::new(content),
            title,
            subtitle: None,
            tall: false,
            _phantom: PhantomData,
        }
    }

    /// Set the subtitle shown after the title.
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self.refresh_content();
        self
    }

    /// Set the widget ID for CSS targeting.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner = self.inner.with_id(id);
        self
    }

    /// Set CSS classes (space-separated).
    pub fn with_classes(mut self, classes: impl Into<String>) -> Self {
        self.inner = self.inner.with_classes(classes);
        self
    }

    /// Set whether the widget expands to fill available space.
    pub fn with_expand(mut self, expand: bool) -> Self {
        self.inner = self.inner.with_expand(expand);
        self
    }

    /// Set whether the widget shrinks to fit content.
    pub fn with_shrink(mut self, shrink: bool) -> Self {
        self.inner = self.inner.with_shrink(shrink);
        self
    }

    /// Set whether content should be parsed as markup.
    pub fn with_markup(mut self, markup: bool) -> Self {
        self.inner = self.inner.with_markup(markup);
        self
    }

    /// Set the widget name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.inner = self.inner.with_name(name);
        self
    }

    /// Set the disabled state.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.with_disabled(disabled);
        self
    }

    /// Toggle the tall header style (adds/removes the `-tall` class).
    pub fn with_tall(mut self, tall: bool) -> Self {
        self.set_tall(tall);
        self
    }

    /// Set the title at runtime.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.refresh_content();
    }

    /// Set the subtitle at runtime.
    pub fn set_subtitle(&mut self, subtitle: impl Into<String>) {
        self.subtitle = Some(subtitle.into());
        self.refresh_content();
    }

    /// Clear the subtitle.
    pub fn clear_subtitle(&mut self) {
        self.subtitle = None;
        self.refresh_content();
    }

    /// Get the current title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the current subtitle, if any.
    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    /// Return whether the header is in tall mode.
    pub fn is_tall(&self) -> bool {
        self.tall
    }

    /// Access the inner Static widget.
    pub fn as_static(&self) -> &Static<M> {
        &self.inner
    }

    /// Access the inner Static widget mutably.
    pub fn as_static_mut(&mut self) -> &mut Static<M> {
        &mut self.inner
    }

    fn format_content(title: &str, subtitle: Option<&str>) -> String {
        let subtitle = subtitle.filter(|value| !value.is_empty());
        match subtitle {
            Some(value) => format!("{title} [dim]— {value}[/]"),
            None => title.to_string(),
        }
    }

    fn refresh_content(&mut self) {
        let content = Self::format_content(&self.title, self.subtitle.as_deref());
        self.inner.update(content);
    }

    fn set_tall(&mut self, tall: bool) {
        if self.tall == tall {
            return;
        }
        self.tall = tall;
        if tall {
            Widget::add_class(&mut self.inner, "-tall");
        } else {
            Widget::remove_class(&mut self.inner, "-tall");
        }
    }
}

impl_widget_delegation!(Header<M> => inner, type_name = "Header", default_css = |_| r#"
Header {
    dock: top;
    width: 100%;
    background: $panel;
    color: $foreground;
    height: 1;
    text-wrap: nowrap;
    text-overflow: ellipsis;
    content-align: center middle;
}

Header.-tall {
    height: 3;
}
"#);
