//! Layout control types for widget display behavior.
//!
//! This module provides types that control how widgets participate
//! in layout and how they handle overflow:
//!
//! - [`Display`]: Whether a widget is rendered and takes space
//! - [`Visibility`]: Whether a widget is visible (still takes space)
//! - [`Overflow`]: How content exceeding bounds is handled
//!
//! ## Display vs Visibility
//!
//! - `display: none` removes the widget from layout entirely
//! - `visibility: hidden` hides the widget but preserves its space
//!
//! ## CSS Syntax
//!
//! ```css
//! .hidden { display: none; }
//! .invisible { visibility: hidden; }
//! .scrollable { overflow: auto; }
//! ```

/// Controls whether a widget is rendered and participates in layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    /// Widget is rendered as a block element (default).
    #[default]
    Block,
    /// Widget is not rendered and takes no space.
    None,
}

/// Controls whether a widget is visually shown.
///
/// Unlike `Display::None`, a hidden widget still occupies space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Widget is visible (default).
    #[default]
    Visible,
    /// Widget is invisible but still takes space.
    Hidden,
}

/// Controls how content exceeding container bounds is handled.
///
/// Used with `overflow-x` and `overflow-y` properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    /// Clip content that exceeds bounds (default).
    #[default]
    Hidden,
    /// Show scrollbars only when content overflows.
    Auto,
    /// Always show scrollbars.
    Scroll,
}
