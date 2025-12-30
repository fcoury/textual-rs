//! Layout control types for widget display behavior.
//!
//! This module provides types that control how widgets participate
//! in layout and how they handle overflow:
//!
//! - [`Display`]: Whether a widget is rendered and takes space
//! - [`Layout`]: How children are arranged (vertical, horizontal, grid)
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
//! .grid-container { layout: grid; }
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

/// Controls how children are arranged within a container.
///
/// Used with the `layout` CSS property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    /// Stack children vertically (default).
    #[default]
    Vertical,
    /// Stack children horizontally.
    Horizontal,
    /// Arrange children in a CSS Grid.
    Grid,
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

/// Controls how width and height are calculated.
///
/// - `content-box`: Width/height apply to content only; padding and border are added outside
/// - `border-box`: Width/height include content, padding, and border
///
/// ## CSS Syntax
///
/// ```css
/// .component { box-sizing: border-box; }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxSizing {
    /// Width/height is content only; padding/border added outside.
    ContentBox,
    /// Width/height includes content, padding, and border (default, matches Python Textual).
    #[default]
    BorderBox,
}
