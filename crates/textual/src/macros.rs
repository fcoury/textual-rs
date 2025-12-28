//! Macros for widget implementation.
//!
//! This module provides macros to reduce boilerplate when creating widgets
//! that wrap other widgets (composition pattern), and declarative UI building.

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

    // Full form with explicit type_name
    ($ty:ident<$m:ident> => $field:ident, type_name = $name:expr) => {
        impl<$m> $crate::Widget<$m> for $ty<$m> {
            fn render(&self, canvas: &mut $crate::Canvas, region: $crate::Region) {
                self.$field.render(canvas, region)
            }

            fn desired_size(&self) -> $crate::Size {
                self.$field.desired_size()
            }

            fn get_meta(&self) -> ::tcss::WidgetMeta {
                let mut meta = self.$field.get_meta();
                meta.type_name = $name.to_string();
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
                self.$field.default_css()
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
        }
    };
}

/// Declarative macro for building widget trees.
///
/// This macro provides a concise DSL for composing UI layouts without
/// explicit `Box::new()` calls and nested function invocations.
///
/// # Supported Containers
///
/// - `Vertical { ... }` - Vertical layout container
/// - `Horizontal { ... }` - Horizontal layout container
/// - `Middle { ... }` - Vertically centers a single child
/// - `Center { ... }` - Horizontally centers a single child
///
/// # Example
///
/// ```ignore
/// use textual::{ui, Vertical, Horizontal, Switch, Label};
///
/// fn compose(&self) -> Box<dyn Widget<Message>> {
///     ui!(
///         Vertical {
///             Label::new("Header"),
///             Horizontal {
///                 Switch::new(false, |v| Message::Toggle(v)),
///                 Label::new("Enable feature"),
///             },
///         }
///     )
/// }
/// ```
#[macro_export]
macro_rules! ui {
    // === Entry Points for Layouts ===
    (Vertical { $($children:tt)* }) => {
        $crate::ui!(@collect Vertical, [], $($children)*)
    };

    (Horizontal { $($children:tt)* }) => {
        $crate::ui!(@collect Horizontal, [], $($children)*)
    };

    // === Entry Points for Single-child Wrappers ===
    (Middle { $($inner:tt)* }) => {
        Box::new($crate::Middle::new($crate::ui!($($inner)*)))
    };

    (Center { $($inner:tt)* }) => {
        Box::new($crate::Center::new($crate::ui!($($inner)*)))
    };

    // === The Collector (Muncher) ===
    // This part moves items from the "todo" list into the "accumulator" list

    // 1. Process a nested container child
    (@collect $kind:ident, [$($acc:expr),*], $child:ident { $($inner:tt)* }, $($rest:tt)*) => {
        $crate::ui!(@collect $kind, [$($acc,)* $crate::ui!($child { $($inner)* })], $($rest)*)
    };
    (@collect $kind:ident, [$($acc:expr),*], $child:ident { $($inner:tt)* }) => {
        $crate::ui!(@collect $kind, [$($acc,)* $crate::ui!($child { $($inner)* })])
    };

    // 2. Process a leaf widget child (e.g. Switch::new)
    (@collect $kind:ident, [$($acc:expr),*], $leaf:ident :: new ( $($args:tt)* ) $( . $meth:ident ( $($m_args:tt)* ) )* , $($rest:tt)*) => {
        $crate::ui!(@collect $kind, [$($acc,)* $crate::ui!($leaf :: new ( $($args)* ) $( . $meth ( $($m_args)* ) )*)], $($rest)*)
    };
    (@collect $kind:ident, [$($acc:expr),*], $leaf:ident :: new ( $($args:tt)* ) $( . $meth:ident ( $($m_args:tt)* ) )*) => {
        $crate::ui!(@collect $kind, [$($acc,)* $crate::ui!($leaf :: new ( $($args)* ) $( . $meth ( $($m_args)* ) )*)])
    };

    // 3. Finalization: All children are in the accumulator, create the Boxed container
    (@collect $kind:ident, [$($acc:expr),*]) => {
        Box::new($kind::new(vec![$($acc),*]))
    };

    // === Leaf Widget & Fallback ===
    ($leaf:ident :: new ( $($args:expr),* ) $( . $meth:ident ( $($m_args:expr),* ) )*) => {
        Box::new($leaf::new( $($args),* ) $( . $meth ( $($m_args),* ) )*)
    };

    ($e:expr) => {
        $e
    };
}
