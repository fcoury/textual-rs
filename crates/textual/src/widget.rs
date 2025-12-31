pub mod app_widget;
pub mod label;
pub mod placeholder;
pub mod ruler;
pub mod screen;
pub mod scrollbar;
pub mod scrollbar_corner;
pub mod static_widget;
pub mod switch;

use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

use crate::{
    KeyCode, MouseEvent, Size,
    canvas::{Canvas, Region},
    layouts::Layout,
};

/// A widget that can render itself and handle events.
/// Generic over `M`, the message type that events produce.
pub trait Widget<M> {
    /// Draw the widget onto the provided canvas within the specified region.
    fn render(&self, canvas: &mut Canvas, region: Region);

    /// Tell the parent container how much space this widget needs.
    fn desired_size(&self) -> Size;

    /// Returns the actual content height for scroll calculations.
    ///
    /// This is used by ScrollableContainer when desired_size returns u16::MAX
    /// (indicating "fill available space"). The default returns desired_size().height,
    /// but containers with flexible children should calculate actual content height.
    ///
    /// `available_height` is the viewport height that would be available.
    fn content_height_for_scroll(&self, available_height: u16) -> u16 {
        let size = self.desired_size();
        if size.height == u16::MAX {
            available_height
        } else {
            size.height
        }
    }

    fn for_each_child(&mut self, _f: &mut dyn FnMut(&mut dyn Widget<M>)) {}

    /// Called when the terminal or parent container is resized.
    ///
    /// Use this to update responsive state (e.g., breakpoint classes based on width).
    /// Containers should override this to propagate to children.
    /// The default implementation does nothing.
    fn on_resize(&mut self, _size: Size) {}

    /// Returns the widget's current pseudo-class states (focus, hover, active, disabled).
    ///
    /// Override this in widgets that support interactive states.
    fn get_state(&self) -> WidgetStates {
        let mut states = WidgetStates::empty();
        if self.is_focused() {
            states |= WidgetStates::FOCUS;
        }
        if self.is_disabled() {
            states |= WidgetStates::DISABLED;
        }
        states
    }

    /// Returns metadata for CSS selector matching.
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

        WidgetMeta {
            type_name,
            id: self.id().map(|s| s.to_string()),
            classes: Vec::new(),
            states: self.get_state(),
        }
    }

    // Default style management

    /// Returns the default CSS for this widget type.
    ///
    /// Override this method to provide built-in styles for your widget.
    /// Default CSS has lower precedence than app-level CSS, allowing users
    /// to override widget defaults in their `App::CSS`.
    ///
    /// The base Widget implementation provides sensible defaults for all widgets,
    /// including scrollbar theming. Override this in subclasses to add widget-specific
    /// styles while preserving the base defaults.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn default_css(&self) -> &'static str {
    ///     r#"
    ///     Label {
    ///         width: auto;
    ///         height: auto;
    ///     }
    ///     Label.error {
    ///         color: red;
    ///     }
    ///     "#
    /// }
    /// ```
    fn default_css(&self) -> &'static str {
        ""
    }

    /// Returns the base widget CSS that applies to all widgets.
    ///
    /// This is collected once during style resolution and provides defaults
    /// for scrollbar theming and other universal properties. Individual widget
    /// `default_css()` implementations add to (not replace) these base styles.
    ///
    /// Uses the universal selector `*` to match all widget types.
    fn base_widget_css() -> &'static str
    where
        Self: Sized,
    {
        r#"
        * {
            scrollbar-background: $scrollbar-background;
            scrollbar-background-hover: $scrollbar-background-hover;
            scrollbar-background-active: $scrollbar-background-active;
            scrollbar-color: $scrollbar;
            scrollbar-color-hover: $scrollbar-hover;
            scrollbar-color-active: $scrollbar-active;
            scrollbar-corner-color: $scrollbar-corner-color;
            scrollbar-size-vertical: 2;
            scrollbar-size-horizontal: 1;
            link-background: $link-background;
            link-background-hover: $link-background-hover;
            link-color: $link-color;
            link-color-hover: $link-color-hover;
            link-style: $link-style;
            link-style-hover: $link-style-hover;
            background: transparent;
        }
        "#
    }

    fn set_style(&mut self, _style: ComputedStyle) {}

    fn get_style(&self) -> ComputedStyle {
        ComputedStyle::default()
    }

    /// Called before layout arrangement to allow runtime configuration.
    ///
    /// Override this method to configure layout properties at runtime.
    /// This is called by containers before `Layout::arrange()` to allow
    /// widgets like ItemGrid to configure properties like `min_column_width`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn pre_layout(&mut self, layout: &mut dyn Layout) {
    ///     if let Some(grid) = layout.as_grid_mut() {
    ///         grid.min_column_width = self.min_column_width;
    ///         grid.stretch_height = self.stretch_height;
    ///     }
    /// }
    /// ```
    fn pre_layout(&mut self, _layout: &mut dyn Layout) {
        // Default: no-op
    }

    // Focus management
    fn set_focus(&mut self, _is_focused: bool) {}

    fn is_focused(&self) -> bool {
        false
    }

    /// Returns true if this widget's style needs to be recomputed.
    ///
    /// Widgets should return true when their state has changed in a way
    /// that might affect styling (e.g., focus, hover, active states).
    fn is_dirty(&self) -> bool {
        false
    }

    /// Marks this widget as needing style recomputation.
    ///
    /// Call this when the widget's state changes in a way that might
    /// affect its styling.
    fn mark_dirty(&mut self) {}

    /// Marks this widget as having up-to-date styles.
    ///
    /// Called by the style resolver after recomputing the widget's style.
    fn mark_clean(&mut self) {}

    /// Handle a key event and optionally return a message.
    fn on_event(&mut self, _key: KeyCode) -> Option<M> {
        None
    }

    /// Handle a mouse event within the widget's region.
    ///
    /// The `region` parameter describes where this widget was rendered,
    /// allowing hit-testing without storing bounds on the widget.
    fn on_mouse(&mut self, _event: MouseEvent, _region: Region) -> Option<M> {
        None
    }

    /// Sets the hover state on this widget and clears hover from all other widgets.
    ///
    /// Returns true if this widget's hover state changed.
    fn set_hover(&mut self, _is_hovered: bool) -> bool {
        false
    }

    /// Sets the active (pressed) state on this widget.
    ///
    /// Returns true if this widget's active state changed.
    fn set_active(&mut self, _is_active: bool) -> bool {
        false
    }

    /// Clears hover state from this widget and all children.
    fn clear_hover(&mut self) {}

    /// Returns true if this widget can receive focus.
    ///
    /// Default implementation returns false. Widgets should also return
    /// false if invisible or disabled.
    fn is_focusable(&self) -> bool {
        false
    }

    // =========================================================================
    // Reactive Attributes (Visibility, Loading, Disabled)
    // =========================================================================

    /// Returns false if this widget should be excluded from layout, rendering, and events.
    ///
    /// Invisible widgets don't occupy space and cannot receive focus.
    fn is_visible(&self) -> bool {
        true
    }

    /// Returns true if this widget should participate in layout, rendering, and events.
    ///
    /// A widget participates if:
    /// 1. `is_visible()` returns true (runtime visibility)
    /// 2. CSS `display` is not `none` (style-driven visibility)
    ///
    /// Use this in container loops instead of checking both conditions separately.
    /// This centralizes the visibility logic and makes it easy to extend with
    /// future conditions.
    fn participates_in_layout(&self) -> bool {
        use tcss::types::Display;
        self.is_visible() && self.get_style().display != Display::None
    }

    /// Set the visibility of this widget.
    fn set_visible(&mut self, _visible: bool) {}

    /// Returns true if this widget is in a loading state.
    ///
    /// Loading widgets render a loading indicator instead of normal content.
    fn is_loading(&self) -> bool {
        false
    }

    /// Set the loading state of this widget.
    fn set_loading(&mut self, _loading: bool) {}

    /// Returns true if this widget is disabled (visible but non-interactive).
    ///
    /// Disabled widgets are rendered in a muted style and cannot receive input.
    fn is_disabled(&self) -> bool {
        false
    }

    /// Set the disabled state of this widget.
    fn set_disabled(&mut self, _disabled: bool) {}

    /// Counts the total number of focusable widgets in this subtree.
    fn count_focusable(&self) -> usize {
        // Base implementation for leaf widgets
        // Containers override this to sum their children's counts
        if self.is_focusable() { 1 } else { 0 }
    }

    /// Clears focus from this widget and all children.
    fn clear_focus(&mut self) {
        self.set_focus(false);
    }

    /// Sets focus on the nth focusable widget (0-indexed).
    /// Returns true if focus was set, false if index was out of range.
    fn focus_nth(&mut self, n: usize) -> bool {
        if self.is_focusable() {
            if n == 0 {
                self.set_focus(true);
                return true;
            }
        }
        false
    }

    /// Returns the number of direct children this widget has.
    ///
    /// Used by WidgetTree for O(d) focus-targeted dispatch.
    fn child_count(&self) -> usize {
        0
    }

    /// Returns a mutable reference to the child at the given index.
    ///
    /// Used by WidgetTree for O(d) focus-targeted dispatch.
    fn get_child_mut(&mut self, _index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        None
    }

    /// Handle a message bubbling up from a descendant widget.
    ///
    /// Return `Some(M)` to transform the message, or `None` to pass it through unchanged.
    /// Call `envelope.stop()` to prevent further bubbling.
    fn handle_message(&mut self, _envelope: &mut crate::MessageEnvelope<M>) -> Option<M> {
        None
    }

    /// Returns the widget's optional ID for message tracking.
    ///
    /// Set via `widget.with_id("my-widget")`. Used to identify which widget
    /// produced a message in `MessageEnvelope.sender_id`.
    fn id(&self) -> Option<&str> {
        None
    }

    /// Returns the widget's type name (e.g., "Switch", "Button").
    ///
    /// Used for `MessageEnvelope.sender_type` to identify what kind of widget
    /// produced a message.
    fn type_name(&self) -> &'static str {
        // Extract simple type name from full path
        let full = std::any::type_name::<Self>();
        full.split('<')
            .next()
            .unwrap_or(full)
            .split("::")
            .last()
            .unwrap_or(full)
    }

    // =========================================================================
    // Border Title/Subtitle Methods (for Query API)
    // =========================================================================

    /// Set the border title (displayed in the top border).
    ///
    /// Default implementation does nothing. Override in widgets that support
    /// border titles (e.g., Static, Label).
    fn set_border_title(&mut self, _title: &str) {}

    /// Set the border subtitle (displayed in the bottom border).
    ///
    /// Default implementation does nothing. Override in widgets that support
    /// border subtitles (e.g., Static, Label).
    fn set_border_subtitle(&mut self, _subtitle: &str) {}

    /// Get the border title, if any.
    fn border_title(&self) -> Option<&str> {
        None
    }

    /// Get the border subtitle, if any.
    fn border_subtitle(&self) -> Option<&str> {
        None
    }

    // =========================================================================
    // Downcasting support for typed queries
    // =========================================================================

    /// Returns a reference to `self` as `&dyn Any` for downcasting.
    ///
    /// This enables typed widget queries like `query_one_as::<Label>()`.
    /// The default implementation returns `None`, meaning the widget
    /// doesn't support downcasting. Override this in concrete widget types.
    ///
    /// # Example
    /// ```ignore
    /// fn as_any(&self) -> Option<&dyn std::any::Any> {
    ///     Some(self)
    /// }
    /// ```
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }

    /// Returns a mutable reference to `self` as `&mut dyn Any` for downcasting.
    ///
    /// This enables typed widget queries like `query_one_as::<Label>()`.
    /// The default implementation returns `None`, meaning the widget
    /// doesn't support downcasting. Override this in concrete widget types.
    ///
    /// # Example
    /// ```ignore
    /// fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
    ///     Some(self)
    /// }
    /// ```
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }

    // =========================================================================
    // CSS Class Management (Python Textual DOMNode parity)
    // =========================================================================

    /// Add a CSS class to this widget.
    ///
    /// Does nothing by default. Override in widgets that support CSS classes.
    fn add_class(&mut self, _class: &str) {}

    /// Remove a CSS class from this widget.
    ///
    /// Does nothing by default. Override in widgets that support CSS classes.
    fn remove_class(&mut self, _class: &str) {}

    /// Toggle a CSS class on this widget.
    ///
    /// If the class is present, it's removed. If absent, it's added.
    fn toggle_class(&mut self, class: &str) {
        if self.has_class(class) {
            self.remove_class(class);
        } else {
            self.add_class(class);
        }
    }

    /// Check if this widget has a CSS class.
    ///
    /// Returns false by default. Override in widgets that support CSS classes.
    fn has_class(&self, _class: &str) -> bool {
        false
    }

    /// Conditionally add or remove a class based on a condition.
    ///
    /// If `add` is true, adds the class; otherwise removes it.
    fn set_class(&mut self, add: bool, class: &str) {
        if add {
            self.add_class(class);
        } else {
            self.remove_class(class);
        }
    }

    /// Replace all CSS classes with the given space-separated string.
    ///
    /// Does nothing by default. Override in widgets that support CSS classes.
    fn set_classes(&mut self, _classes: &str) {}

    /// Get all CSS classes on this widget.
    ///
    /// Returns an empty vector by default. Override in widgets that support CSS classes.
    fn classes(&self) -> Vec<String> {
        Vec::new()
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

    fn content_height_for_scroll(&self, available_height: u16) -> u16 {
        self.as_ref().content_height_for_scroll(available_height)
    }

    fn get_state(&self) -> WidgetStates {
        self.as_ref().get_state()
    }

    fn get_style(&self) -> ComputedStyle {
        self.as_ref().get_style()
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.as_mut().set_style(style);
    }

    fn default_css(&self) -> &'static str {
        self.as_ref().default_css()
    }

    fn get_meta(&self) -> WidgetMeta {
        self.as_ref().get_meta()
    }

    fn is_dirty(&self) -> bool {
        self.as_ref().is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.as_mut().mark_dirty();
    }

    fn mark_clean(&mut self) {
        self.as_mut().mark_clean();
    }

    fn pre_layout(&mut self, layout: &mut dyn Layout) {
        self.as_mut().pre_layout(layout);
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        self.as_mut().on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.as_mut().on_mouse(event, region)
    }

    fn set_hover(&mut self, is_hovered: bool) -> bool {
        self.as_mut().set_hover(is_hovered)
    }

    fn set_active(&mut self, is_active: bool) -> bool {
        self.as_mut().set_active(is_active)
    }

    fn clear_hover(&mut self) {
        self.as_mut().clear_hover();
    }

    fn is_focusable(&self) -> bool {
        self.as_ref().is_focusable()
    }

    fn is_visible(&self) -> bool {
        self.as_ref().is_visible()
    }

    fn participates_in_layout(&self) -> bool {
        self.as_ref().participates_in_layout()
    }

    fn set_visible(&mut self, visible: bool) {
        self.as_mut().set_visible(visible);
    }

    fn is_loading(&self) -> bool {
        self.as_ref().is_loading()
    }

    fn set_loading(&mut self, loading: bool) {
        self.as_mut().set_loading(loading);
    }

    fn is_disabled(&self) -> bool {
        self.as_ref().is_disabled()
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.as_mut().set_disabled(disabled);
    }

    fn count_focusable(&self) -> usize {
        self.as_ref().count_focusable()
    }

    fn clear_focus(&mut self) {
        self.as_mut().clear_focus();
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.as_mut().focus_nth(n)
    }

    fn child_count(&self) -> usize {
        self.as_ref().child_count()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        self.as_mut().get_child_mut(index)
    }

    fn handle_message(&mut self, envelope: &mut crate::MessageEnvelope<M>) -> Option<M> {
        self.as_mut().handle_message(envelope)
    }

    fn id(&self) -> Option<&str> {
        self.as_ref().id()
    }

    fn type_name(&self) -> &'static str {
        // Box<dyn Widget> should delegate to inner widget's type_name
        // but we can't call it through trait object, so return generic name
        "Widget"
    }

    fn on_resize(&mut self, size: Size) {
        self.as_mut().on_resize(size);
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        self.as_mut().for_each_child(f);
    }

    fn set_border_title(&mut self, title: &str) {
        self.as_mut().set_border_title(title);
    }

    fn set_border_subtitle(&mut self, subtitle: &str) {
        self.as_mut().set_border_subtitle(subtitle);
    }

    fn border_title(&self) -> Option<&str> {
        self.as_ref().border_title()
    }

    fn border_subtitle(&self) -> Option<&str> {
        self.as_ref().border_subtitle()
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        self.as_ref().as_any()
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.as_mut().as_any_mut()
    }

    fn add_class(&mut self, class: &str) {
        self.as_mut().add_class(class);
    }

    fn remove_class(&mut self, class: &str) {
        self.as_mut().remove_class(class);
    }

    fn toggle_class(&mut self, class: &str) {
        self.as_mut().toggle_class(class);
    }

    fn has_class(&self, class: &str) -> bool {
        self.as_ref().has_class(class)
    }

    fn set_class(&mut self, add: bool, class: &str) {
        self.as_mut().set_class(add, class);
    }

    fn set_classes(&mut self, classes: &str) {
        self.as_mut().set_classes(classes);
    }

    fn classes(&self) -> Vec<String> {
        self.as_ref().classes()
    }
}

/// Trait for types that can compose a widget tree.
/// The associated `Message` type defines what events the UI can produce.
pub trait Compose {
    type Message;

    /// Returns a vector of widgets that make up this composition.
    ///
    /// The returned widgets become children of the implicit `Screen` container.
    /// Screen applies CSS layout properties (grid, horizontal, vertical) to arrange them.
    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>>;
}
