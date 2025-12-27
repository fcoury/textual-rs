pub mod scrollbar;
pub mod scrollbar_corner;
pub mod switch;

use tcss::{ComputedStyle, WidgetMeta, WidgetStates};

use crate::{
    KeyCode, MouseEvent, Size,
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
            id: None,
            classes: Vec::new(),
            states: self.get_state(),
        }
    }

    // Default style management
    fn set_style(&mut self, _style: ComputedStyle) {}

    fn get_style(&self) -> ComputedStyle {
        ComputedStyle::default()
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
}

/// Allow boxed widgets to be used as widgets.
impl<M> Widget<M> for Box<dyn Widget<M>> {
    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.as_ref().render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.as_ref().desired_size()
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
}

/// Trait for types that can compose a widget tree.
/// The associated `Message` type defines what events the UI can produce.
pub trait Compose {
    type Message;

    fn compose(&self) -> Box<dyn Widget<Self::Message>>;
}
