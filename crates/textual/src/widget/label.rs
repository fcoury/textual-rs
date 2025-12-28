//! Label widget for styled text display.
//!
//! Label wraps Static and adds semantic variant styling (success, error, etc.).

use std::marker::PhantomData;

use crate::impl_widget_delegation;
use crate::widget::static_widget::Static;

/// Semantic variants for Label styling.
///
/// Each variant adds a CSS class that can be styled in your stylesheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelVariant {
    /// Default styling (no extra class)
    Default,
    /// Success state (adds "success" class)
    Success,
    /// Error state (adds "error" class)
    Error,
    /// Warning state (adds "warning" class)
    Warning,
    /// Primary emphasis (adds "primary" class)
    Primary,
    /// Secondary emphasis (adds "secondary" class)
    Secondary,
    /// Accent styling (adds "accent" class)
    Accent,
}

impl LabelVariant {
    /// Get the CSS class name for this variant.
    pub fn as_class(&self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Success => "success",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Accent => "accent",
        }
    }
}

/// A styled text label with optional semantic variants.
///
/// Label wraps [`Static`] and adds variant-based styling. The variant adds
/// a CSS class (e.g., "success", "error") that can be styled in CSS.
///
/// # Example
///
/// ```ignore
/// use textual::{Label, LabelVariant};
///
/// // Simple label
/// let greeting: Label<MyMessage> = Label::new("Hello, World!");
///
/// // With semantic variant
/// let error: Label<MyMessage> = Label::new("Something went wrong")
///     .with_variant(LabelVariant::Error);
///
/// // With ID and classes
/// let status: Label<MyMessage> = Label::new("Ready")
///     .with_id("status-bar")
///     .with_classes("bold centered");
/// ```
///
/// # CSS Example
///
/// ```css
/// Label {
///     width: auto;
///     height: auto;
/// }
///
/// Label.success {
///     color: $success;
/// }
///
/// Label.error {
///     color: $error;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Label<M> {
    inner: Static<M>,
    variant: Option<LabelVariant>,
    _phantom: PhantomData<M>,
}

impl<M> Label<M> {
    /// Create a new Label with the given text content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            inner: Static::new(content),
            variant: None,
            _phantom: PhantomData,
        }
    }

    /// Set the semantic variant (success, error, warning, etc.).
    ///
    /// The variant adds a CSS class that can be styled in your stylesheet.
    pub fn with_variant(mut self, variant: LabelVariant) -> Self {
        let class = variant.as_class();
        if !class.is_empty() {
            self.inner.add_class(class);
        }
        self.variant = Some(variant);
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

    /// Update the label content.
    ///
    /// This changes the displayed text and marks the widget as dirty.
    pub fn update(&mut self, content: impl Into<String>) {
        self.inner.update(content);
    }

    /// Get the current variant, if any.
    pub fn variant(&self) -> Option<LabelVariant> {
        self.variant
    }

    /// Access the inner Static widget.
    pub fn as_static(&self) -> &Static<M> {
        &self.inner
    }

    /// Access the inner Static widget mutably.
    pub fn as_static_mut(&mut self) -> &mut Static<M> {
        &mut self.inner
    }
}

// Use the delegation macro for Widget trait implementation
impl_widget_delegation!(Label<M> => inner, type_name = "Label");
