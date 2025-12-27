//! Border styling types for terminal UI widgets.
//!
//! This module provides types for defining widget borders:
//!
//! - [`BorderKind`]: The visual style of the border (solid, dashed, etc.)
//! - [`BorderEdge`]: A single edge with style and optional color
//! - [`Border`]: All four edges of a widget's border
//!
//! ## Border Styles
//!
//! TCSS supports various border styles suited for terminal rendering:
//!
//! | Style    | Description                              |
//! |----------|------------------------------------------|
//! | `solid`  | Standard line border (─│┐└)              |
//! | `round`  | Rounded corners (─│╮╯)                   |
//! | `double` | Double-line border (═║╔╝)                |
//! | `heavy`  | Thick/bold border characters             |
//! | `dashed` | Dashed line border                       |
//! | `ascii`  | ASCII-only characters (+---+)            |
//! | `block`  | Block characters for thick appearance    |
//!
//! ## CSS Syntax
//!
//! ```css
//! Button {
//!     border: solid blue;
//!     border-top: double red;
//! }
//! ```

use crate::types::color::RgbaColor;

/// The visual style of a border edge.
///
/// Each variant represents a different way to draw the border
/// using terminal characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderKind {
    /// No border (default).
    #[default]
    None,
    /// ASCII-only border using +, -, and | characters.
    Ascii,
    /// Invisible border that still occupies space.
    Blank,
    /// Block/box-drawing characters for a filled appearance.
    Block,
    /// Double-line border (═║╔╗╚╝).
    Double,
    /// Dashed line border.
    Dashed,
    /// Heavy/bold line border.
    Heavy,
    /// Hidden border (no visual, no space).
    Hidden,
    /// Outer-only border style.
    Outer,
    /// Inner-only border style.
    Inner,
    /// Standard solid line border (─│┌┐└┘).
    Solid,
    /// Rounded corner border (─│╭╮╯╰).
    Round,
    /// Extra-thick border appearance.
    Thick,
}

/// A single border edge with style and optional color.
///
/// Each edge of a border can have its own style and color,
/// allowing for asymmetric border designs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BorderEdge {
    /// The visual style of this edge.
    pub kind: BorderKind,
    /// The color of this edge (inherits if `None`).
    pub color: Option<RgbaColor>,
}

/// Complete border definition for all four sides of a widget.
///
/// # Examples
///
/// ```
/// use tcss::types::{Border, BorderEdge, BorderKind};
///
/// // Same style on all sides
/// let uniform = Border::all(BorderEdge {
///     kind: BorderKind::Solid,
///     color: None,
/// });
///
/// // Check if any border is visible
/// assert!(!uniform.is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Border {
    /// Top edge border.
    pub top: BorderEdge,
    /// Right edge border.
    pub right: BorderEdge,
    /// Bottom edge border.
    pub bottom: BorderEdge,
    /// Left edge border.
    pub left: BorderEdge,
}

impl Border {
    /// Creates a border with the same edge style on all four sides.
    pub fn all(edge: BorderEdge) -> Self {
        Self {
            top: edge.clone(),
            right: edge.clone(),
            bottom: edge.clone(),
            left: edge,
        }
    }

    /// Returns `true` if no border edges are visible.
    pub fn is_none(&self) -> bool {
        self.top.kind == BorderKind::None
            && self.right.kind == BorderKind::None
            && self.bottom.kind == BorderKind::None
            && self.left.kind == BorderKind::None
    }
}
