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

/// Declarative macro for building widget trees with Dioxus-style DSL syntax.
///
/// This macro provides a concise, ergonomic DSL for composing UI layouts.
/// It supports positional arguments, named attributes, and nested children.
///
/// # Syntax
///
/// ```ignore
/// // Container with children only
/// Vertical {
///     child1
///     child2
/// }
///
/// // Widget with positional arg(s) only
/// Static("Hello world")
///
/// // Widget with positional arg(s) and named attributes
/// Static("Hello", id: "greeting", classes: "bold")
///
/// // Container with attributes and children
/// Grid(id: "my-grid") {
///     child1
///     child2
/// }
///
/// // Widget with callback
/// Switch(false, |v| Msg::Toggle(v), id: "toggle")
///
/// // Multiple root widgets (auto-wrapped in Vertical)
/// Label("First line")
/// Label("Second line")
/// ```
///
/// # Attribute Mapping
///
/// Named attributes are converted to builder method calls:
/// - `id: "foo"` → `.with_id("foo")`
/// - `classes: "a b"` → `.with_classes("a b")`
/// - `disabled: true` → `.with_disabled(true)`
///
/// # Example
///
/// ```ignore
/// use textual::{ui, Vertical, Horizontal, Switch, Static};
///
/// fn compose(&self) -> Box<dyn Widget<Message>> {
///     ui! {
///         Vertical {
///             Static("Header", id: "header", classes: "bold")
///
///             Horizontal {
///                 Switch(false, |v| Message::Toggle(v), id: "toggle")
///                 Static("Enable feature")
///             }
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! ui {
    // ==========================================================================
    // ROOT FINALIZATION - Handle single vs multiple root widgets
    // (Must come before the catch-all entry point)
    // ==========================================================================

    // Multiple widgets at root - wrap in Vertical
    (@root_finalize [$first:expr, $($rest:expr),+]) => {
        Box::new($crate::Vertical::new(vec![$first, $($rest),+])) as Box<dyn $crate::Widget<_>>
    };

    // Single widget at root - return directly
    (@root_finalize [$single:expr]) => {
        $single
    };

    // Empty - compile error
    (@root_finalize []) => {
        compile_error!("ui! macro requires at least one widget")
    };

    // ==========================================================================
    // ROOT COLLECTION - Collect widgets at root level
    // ==========================================================================

    // Done collecting - finalize
    (@root_collect [$($acc:expr),*]) => {
        $crate::ui!(@root_finalize [$($acc),*])
    };

    // Root: Container with args and children - Widget(args) { children }
    (@root_collect [$($acc:expr),*] $widget:ident ( $($args:tt)* ) { $($inner:tt)* } $($rest:tt)*) => {
        $crate::ui!(@root_collect [$($acc,)* $crate::ui!(@widget $widget ( $($args)* ) { $($inner)* })] $($rest)*)
    };

    // Root: Container with children only - Widget { children }
    (@root_collect [$($acc:expr),*] $widget:ident { $($inner:tt)* } $($rest:tt)*) => {
        $crate::ui!(@root_collect [$($acc,)* $crate::ui!(@widget $widget { $($inner)* })] $($rest)*)
    };

    // Root: Widget with args - Widget(args)
    (@root_collect [$($acc:expr),*] $widget:ident ( $($args:tt)* ) $($rest:tt)*) => {
        $crate::ui!(@root_collect [$($acc,)* $crate::ui!(@widget $widget ( $($args)* ))] $($rest)*)
    };

    // ==========================================================================
    // WIDGET BUILDERS - Build individual widgets (used by root and child collectors)
    // ==========================================================================

    // Widget with args/attrs AND children: Widget(args) { children }
    (@widget $widget:ident ( $($args:tt)* ) { $($children:tt)* }) => {{
        let children: Vec<Box<dyn $crate::Widget<_>>> = $crate::ui!(@collect_children [] $($children)*);
        Box::new($crate::ui!(@parse_args_container $widget, children, [], $($args)*)) as Box<dyn $crate::Widget<_>>
    }};

    // Widget with children only (no parens): Widget { children }
    (@widget $widget:ident { $($children:tt)* }) => {{
        let children: Vec<Box<dyn $crate::Widget<_>>> = $crate::ui!(@collect_children [] $($children)*);
        Box::new($widget::new(children)) as Box<dyn $crate::Widget<_>>
    }};

    // Widget with args/attrs only (no children): Widget(args)
    (@widget $widget:ident ( $($args:tt)* )) => {
        Box::new($crate::ui!(@parse_args_leaf $widget, [], $($args)*)) as Box<dyn $crate::Widget<_>>
    };

    // ==========================================================================
    // ARGUMENT PARSING (for leaf widgets - no children)
    // Builds unboxed widget, applies all .with_*() calls
    // ==========================================================================

    // Named attribute: id: value - with trailing comma
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], id : $val:expr, $($rest:tt)*) => {
        $crate::ui!(@parse_args_leaf $widget, [$($pos),*], $($rest)*).with_id($val)
    };
    // Named attribute: id: value - last attribute
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], id : $val:expr) => {
        $widget::new($($pos),*).with_id($val)
    };

    // Named attribute: classes: value
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], classes : $val:expr, $($rest:tt)*) => {
        $crate::ui!(@parse_args_leaf $widget, [$($pos),*], $($rest)*).with_classes($val)
    };
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], classes : $val:expr) => {
        $widget::new($($pos),*).with_classes($val)
    };

    // Named attribute: disabled: value
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], disabled : $val:expr, $($rest:tt)*) => {
        $crate::ui!(@parse_args_leaf $widget, [$($pos),*], $($rest)*).with_disabled($val)
    };
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], disabled : $val:expr) => {
        $widget::new($($pos),*).with_disabled($val)
    };

    // Named attribute: loading: value
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], loading : $val:expr, $($rest:tt)*) => {
        $crate::ui!(@parse_args_leaf $widget, [$($pos),*], $($rest)*).with_loading($val)
    };
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], loading : $val:expr) => {
        $widget::new($($pos),*).with_loading($val)
    };

    // Named attribute: spinner_frame: value
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], spinner_frame : $val:expr, $($rest:tt)*) => {
        $crate::ui!(@parse_args_leaf $widget, [$($pos),*], $($rest)*).with_spinner_frame($val)
    };
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], spinner_frame : $val:expr) => {
        $widget::new($($pos),*).with_spinner_frame($val)
    };

    // Positional argument (any expression) - with trailing comma
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], $arg:expr, $($rest:tt)*) => {
        $crate::ui!(@parse_args_leaf $widget, [$($pos,)* $arg], $($rest)*)
    };

    // Positional argument - no trailing comma (last positional, no attrs)
    (@parse_args_leaf $widget:ident, [$($pos:expr),*], $arg:expr) => {
        $widget::new($($pos,)* $arg)
    };

    // Empty args - just construct
    (@parse_args_leaf $widget:ident, [$($pos:expr),*],) => {
        $widget::new($($pos),*)
    };

    // ==========================================================================
    // ARGUMENT PARSING WITH CHILDREN (for containers)
    // Builds unboxed widget with children, applies all .with_*() calls
    // ==========================================================================

    // Named attribute: id: value - with trailing comma
    (@parse_args_container $widget:ident, $children:ident, [$($pos:expr),*], id : $val:expr, $($rest:tt)*) => {
        $crate::ui!(@parse_args_container $widget, $children, [$($pos),*], $($rest)*).with_id($val)
    };
    // Named attribute: id: value - last attribute
    (@parse_args_container $widget:ident, $children:ident, [$($pos:expr),*], id : $val:expr) => {
        $widget::new($($pos,)* $children).with_id($val)
    };

    // Named attribute: classes: value
    (@parse_args_container $widget:ident, $children:ident, [$($pos:expr),*], classes : $val:expr, $($rest:tt)*) => {
        $crate::ui!(@parse_args_container $widget, $children, [$($pos),*], $($rest)*).with_classes($val)
    };
    (@parse_args_container $widget:ident, $children:ident, [$($pos:expr),*], classes : $val:expr) => {
        $widget::new($($pos,)* $children).with_classes($val)
    };

    // Positional argument - with trailing comma
    (@parse_args_container $widget:ident, $children:ident, [$($pos:expr),*], $arg:expr, $($rest:tt)*) => {
        $crate::ui!(@parse_args_container $widget, $children, [$($pos,)* $arg], $($rest)*)
    };

    // Positional argument - last one
    (@parse_args_container $widget:ident, $children:ident, [$($pos:expr),*], $arg:expr) => {
        $widget::new($($pos,)* $arg, $children)
    };

    // Empty args - construct with children only
    (@parse_args_container $widget:ident, $children:ident, [$($pos:expr),*],) => {
        $widget::new($($pos,)* $children)
    };

    // ==========================================================================
    // CHILD COLLECTION (no commas between children)
    // ==========================================================================

    // Empty: done collecting
    (@collect_children [$($acc:expr),*]) => {
        vec![$($acc),*]
    };

    // Child: Container with args and children - Widget(args) { children }
    (@collect_children [$($acc:expr),*] $child:ident ( $($args:tt)* ) { $($inner:tt)* } $($rest:tt)*) => {
        $crate::ui!(@collect_children [$($acc,)* $crate::ui!(@widget $child ( $($args)* ) { $($inner)* })] $($rest)*)
    };

    // Child: Container with children only - Widget { children }
    (@collect_children [$($acc:expr),*] $child:ident { $($inner:tt)* } $($rest:tt)*) => {
        $crate::ui!(@collect_children [$($acc,)* $crate::ui!(@widget $child { $($inner)* })] $($rest)*)
    };

    // Child: Widget with args - Widget(args)
    (@collect_children [$($acc:expr),*] $child:ident ( $($args:tt)* ) $($rest:tt)*) => {
        $crate::ui!(@collect_children [$($acc,)* $crate::ui!(@widget $child ( $($args)* ))] $($rest)*)
    };

    // ==========================================================================
    // LEGACY SUPPORT (can be removed after migration)
    // ==========================================================================

    // Legacy: Widget::new(...).with_...(...) syntax
    ($leaf:ident :: new ( $($args:expr),* ) $( . $meth:ident ( $($m_args:expr),* ) )*) => {
        Box::new($leaf::new( $($args),* ) $( . $meth ( $($m_args),* ) )*) as Box<dyn $crate::Widget<_>>
    };

    // ==========================================================================
    // ENTRY POINT - Route all input through root collector
    // (Must be LAST - catches all remaining patterns)
    // ==========================================================================
    ($($tokens:tt)*) => {
        $crate::ui!(@root_collect [] $($tokens)*)
    };
}
