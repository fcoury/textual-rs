//! Label widget for styled text display.
//!
//! Label wraps Static and adds semantic variant styling (success, error, etc.).

use std::marker::PhantomData;

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

    /// Set the border title (displayed in the top border).
    ///
    /// The title supports markup for styling (e.g., `[b]Bold Title[/]`).
    pub fn with_border_title(mut self, title: impl Into<String>) -> Self {
        self.inner = self.inner.with_border_title(title);
        self
    }

    /// Set the border subtitle (displayed in the bottom border).
    ///
    /// The subtitle supports markup for styling (e.g., `[i]Italic Subtitle[/]`).
    pub fn with_border_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.inner = self.inner.with_border_subtitle(subtitle);
        self
    }

    /// Set the border title at runtime.
    pub fn set_border_title(&mut self, title: impl Into<String>) {
        self.inner.set_border_title(title);
    }

    /// Set the border subtitle at runtime.
    pub fn set_border_subtitle(&mut self, subtitle: impl Into<String>) {
        self.inner.set_border_subtitle(subtitle);
    }

    /// Get the border title.
    pub fn border_title(&self) -> Option<&str> {
        self.inner.border_title()
    }

    /// Get the border subtitle.
    pub fn border_subtitle(&self) -> Option<&str> {
        self.inner.border_subtitle()
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

// Manual Widget implementation to provide Label-specific default_css
impl<M: 'static> crate::Widget<M> for Label<M> {
    fn default_css(&self) -> &'static str {
        r#"
Label {
    width: auto;
    height: auto;
}
"#
    }

    fn render(&self, canvas: &mut crate::Canvas, region: crate::Region) {
        self.inner.render(canvas, region)
    }

    fn desired_size(&self) -> crate::Size {
        self.inner.desired_size()
    }

    fn intrinsic_height_for_width(&self, width: u16) -> u16 {
        self.inner.intrinsic_height_for_width(width)
    }

    fn get_meta(&self) -> tcss::WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "Label";
        meta.type_names = vec!["Label", "Static", "Widget", "DOMNode"];
        meta
    }

    fn get_state(&self) -> tcss::WidgetStates {
        self.inner.get_state()
    }

    fn set_style(&mut self, style: tcss::ComputedStyle) {
        self.inner.set_style(style)
    }

    fn get_style(&self) -> tcss::ComputedStyle {
        self.inner.get_style()
    }

    fn set_inline_style(&mut self, style: tcss::StyleOverride) {
        self.inner.set_inline_style(style)
    }

    fn inline_style(&self) -> Option<&tcss::StyleOverride> {
        self.inner.inline_style()
    }

    fn clear_inline_style(&mut self) {
        self.inner.clear_inline_style()
    }

    fn is_dirty(&self) -> bool {
        self.inner.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.inner.mark_dirty()
    }

    fn mark_clean(&mut self) {
        self.inner.mark_clean()
    }

    fn on_event(&mut self, key: crate::KeyCode) -> Option<M> {
        self.inner.on_event(key)
    }

    fn on_mouse(&mut self, event: crate::MouseEvent, region: crate::Region) -> Option<M> {
        self.inner.on_mouse(event, region)
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        self.inner.set_hover(is_hovered)
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        self.inner.set_active(is_active)
    }

    fn clear_hover(&mut self) {
        self.inner.clear_hover()
    }

    fn take_pending_action(&self) -> Option<String> {
        self.inner.take_pending_action()
    }

    fn is_focusable(&self) -> bool {
        self.inner.is_focusable()
    }

    fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.inner.set_visible(visible)
    }

    fn is_loading(&self) -> bool {
        self.inner.is_loading()
    }

    fn set_loading(&mut self, loading: bool) {
        self.inner.set_loading(loading)
    }

    fn is_disabled(&self) -> bool {
        self.inner.is_disabled()
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.inner.set_disabled(disabled)
    }

    fn count_focusable(&self) -> usize {
        self.inner.count_focusable()
    }

    fn clear_focus(&mut self) {
        self.inner.clear_focus()
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.inner.focus_nth(n)
    }

    fn set_focus(&mut self, is_focused: bool) {
        self.inner.set_focus(is_focused)
    }

    fn is_focused(&self) -> bool {
        self.inner.is_focused()
    }

    fn child_count(&self) -> usize {
        self.inner.child_count()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn crate::Widget<M> + '_)> {
        self.inner.get_child_mut(index)
    }

    fn handle_message(&mut self, envelope: &mut crate::MessageEnvelope<M>) -> Option<M> {
        self.inner.handle_message(envelope)
    }

    fn id(&self) -> Option<&str> {
        self.inner.id()
    }

    fn type_name(&self) -> &'static str {
        "Label"
    }

    fn set_border_title(&mut self, title: &str) {
        self.inner.set_border_title(title);
    }

    fn set_border_subtitle(&mut self, subtitle: &str) {
        self.inner.set_border_subtitle(subtitle);
    }

    fn border_title(&self) -> Option<&str> {
        self.inner.border_title()
    }

    fn border_subtitle(&self) -> Option<&str> {
        self.inner.border_subtitle()
    }

    fn on_resize(&mut self, size: crate::Size) {
        self.inner.on_resize(size)
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn crate::Widget<M>)) {
        self.inner.for_each_child(f)
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn add_class(&mut self, class: &str) {
        self.inner.add_class(class);
    }

    fn remove_class(&mut self, class: &str) {
        self.inner.remove_class(class);
    }

    fn has_class(&self, class: &str) -> bool {
        self.inner.has_class(class)
    }

    fn set_classes(&mut self, classes: &str) {
        self.inner.set_classes(classes);
    }

    fn classes(&self) -> Vec<String> {
        self.inner.classes()
    }
}
