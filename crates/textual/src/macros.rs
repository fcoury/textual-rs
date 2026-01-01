//! Macros for widget implementation.
//!
//! This module provides macros to reduce boilerplate when creating widgets
//! that wrap other widgets (composition pattern).
//!
//! For declarative UI building, see the `ui!` proc macro re-exported from
//! `textual_macros`.

/// Generates a Widget trait implementation that delegates to an inner widget.
///
/// This macro is useful when creating widgets that wrap another widget and
/// want to delegate most Widget trait methods to the inner widget.
///
/// # Usage
///
/// ```ignore
/// // Basic usage - type_name defaults to the struct name
/// impl_widget_delegation!(Label<M> => inner);
///
/// // With explicit type_name
/// impl_widget_delegation!(MyWidget<M> => base, type_name = "MyWidget");
/// ```
///
/// # Example
///
/// ```ignore
/// use textual::{impl_widget_delegation, Widget, Static};
///
/// pub struct Label<M> {
///     inner: Static<M>,
///     variant: Option<LabelVariant>,
/// }
///
/// impl_widget_delegation!(Label<M> => inner, type_name = "Label");
/// ```
#[macro_export]
macro_rules! impl_widget_delegation {
    // Basic form: type_name defaults to stringify of the type
    ($ty:ident<$m:ident> => $field:ident) => {
        $crate::impl_widget_delegation!($ty<$m> => $field, type_name = stringify!($ty));
    };

    // Form with explicit type_name (delegates default_css to inner)
    ($ty:ident<$m:ident> => $field:ident, type_name = $name:expr) => {
        $crate::impl_widget_delegation!($ty<$m> => $field, type_name = $name, default_css = |s: &Self| s.$field.default_css());
    };

    // Full form with explicit type_name and default_css
    ($ty:ident<$m:ident> => $field:ident, type_name = $name:expr, default_css = $css:expr) => {
        impl<$m> $crate::Widget<$m> for $ty<$m> {
            fn render(&self, canvas: &mut $crate::Canvas, region: $crate::Region) {
                self.$field.render(canvas, region)
            }

            fn desired_size(&self) -> $crate::Size {
                self.$field.desired_size()
            }

            fn get_meta(&self) -> ::tcss::WidgetMeta {
                let mut meta = self.$field.get_meta();
                meta.type_name = $name;
                meta
            }

            fn get_state(&self) -> ::tcss::WidgetStates {
                self.$field.get_state()
            }

            fn set_style(&mut self, style: ::tcss::ComputedStyle) {
                self.$field.set_style(style)
            }

            fn get_style(&self) -> ::tcss::ComputedStyle {
                self.$field.get_style()
            }

            fn default_css(&self) -> &'static str {
                let f: fn(&Self) -> &'static str = $css;
                f(self)
            }

            fn is_dirty(&self) -> bool {
                self.$field.is_dirty()
            }

            fn mark_dirty(&mut self) {
                self.$field.mark_dirty()
            }

            fn mark_clean(&mut self) {
                self.$field.mark_clean()
            }

            fn on_event(&mut self, key: $crate::KeyCode) -> Option<$m> {
                self.$field.on_event(key)
            }

            fn on_mouse(&mut self, event: $crate::MouseEvent, region: $crate::Region) -> Option<$m> {
                self.$field.on_mouse(event, region)
            }

            fn set_hover(&mut self, is_hovered: bool) -> bool {
                self.$field.set_hover(is_hovered)
            }

            fn set_active(&mut self, is_active: bool) -> bool {
                self.$field.set_active(is_active)
            }

            fn clear_hover(&mut self) {
                self.$field.clear_hover()
            }

            fn is_focusable(&self) -> bool {
                self.$field.is_focusable()
            }

            fn is_visible(&self) -> bool {
                self.$field.is_visible()
            }

            fn set_visible(&mut self, visible: bool) {
                self.$field.set_visible(visible)
            }

            fn is_loading(&self) -> bool {
                self.$field.is_loading()
            }

            fn set_loading(&mut self, loading: bool) {
                self.$field.set_loading(loading)
            }

            fn is_disabled(&self) -> bool {
                self.$field.is_disabled()
            }

            fn set_disabled(&mut self, disabled: bool) {
                self.$field.set_disabled(disabled)
            }

            fn count_focusable(&self) -> usize {
                self.$field.count_focusable()
            }

            fn clear_focus(&mut self) {
                self.$field.clear_focus()
            }

            fn focus_nth(&mut self, n: usize) -> bool {
                self.$field.focus_nth(n)
            }

            fn set_focus(&mut self, is_focused: bool) {
                self.$field.set_focus(is_focused)
            }

            fn is_focused(&self) -> bool {
                self.$field.is_focused()
            }

            fn child_count(&self) -> usize {
                self.$field.child_count()
            }

            fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn $crate::Widget<$m> + '_)> {
                self.$field.get_child_mut(index)
            }

            fn handle_message(&mut self, envelope: &mut $crate::MessageEnvelope<$m>) -> Option<$m> {
                self.$field.handle_message(envelope)
            }

            fn id(&self) -> Option<&str> {
                self.$field.id()
            }

            fn type_name(&self) -> &'static str {
                // Return the overridden type name
                // Note: This is a static str, so we use a match on the macro input
                $name
            }

            fn on_resize(&mut self, size: $crate::Size) {
                self.$field.on_resize(size)
            }

            fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn $crate::Widget<$m>)) {
                self.$field.for_each_child(f)
            }

            fn content_height_for_scroll(&self, available_width: u16, available_height: u16) -> u16 {
                self.$field
                    .content_height_for_scroll(available_width, available_height)
            }

            fn content_width_for_scroll(&self, available_width: u16) -> u16 {
                self.$field.content_width_for_scroll(available_width)
            }
        }
    };
}
