//! Core types for TCSS styling.
//!
//! This module provides the fundamental types used throughout TCSS:
//!
//! - [`RgbaColor`]: Colors with RGBA components and theme variable support
//! - [`Scalar`], [`Spacing`]: Dimension and box model values
//! - [`Border`], [`BorderEdge`]: Border styling
//! - [`TextStyle`], [`TextAlign`]: Text formatting
//! - [`Display`], [`Layout`], [`Visibility`], [`Overflow`]: Layout control
//! - [`GridStyle`], [`GridPlacement`]: CSS Grid support
//! - [`Theme`]: Color theme definitions
//! - [`ComputedStyle`]: Final computed styles for a widget
//! - [`ScrollbarStyle`]: Scrollbar styling and configuration
//! - [`LinkStyle`]: Link styling (colors and text styles)
//! - [`Hatch`], [`HatchPattern`]: Hatch pattern fills
//!
//! ## Module Organization
//!
//! - [`color`]: RGBA color with parsing and theme variable support
//! - [`geometry`]: Scalars (dimensions) and spacing (margins/padding)
//! - [`border`]: Border kinds, edges, and full border definitions
//! - [`text`]: Text styling and alignment
//! - [`layout`]: Display modes, layout modes, visibility, and overflow
//! - [`grid`]: CSS Grid configuration and child placement
//! - [`hatch`]: Hatch pattern fills
//! - [`link`]: Link styling configuration
//! - [`theme`]: Theme color palettes
//! - [`scrollbar`]: Scrollbar styling, sizes, and visibility

pub mod border;
pub mod color;
pub mod geometry;
pub mod grid;
pub mod hatch;
pub mod keyline;
pub mod layout;
pub mod link;
pub mod scrollbar;
pub mod text;
pub mod theme;

pub use border::{Border, BorderEdge, BorderKind};
pub use color::RgbaColor;
pub use geometry::{Scalar, Spacing, Unit};
pub use grid::{GridPlacement, GridStyle};
pub use hatch::{Hatch, HatchPattern};
pub use keyline::{Keyline, KeylineStyle};
pub use layout::{BoxSizing, Display, Dock, Layout, Overflow, Position, Visibility};
pub use link::LinkStyle;
pub use scrollbar::{ScrollbarGutter, ScrollbarSize, ScrollbarStyle, ScrollbarVisibility};
pub use text::{AlignHorizontal, AlignVertical, TextAlign, TextOverflow, TextStyle, TextWrap};
pub use theme::{ColorSystem, Theme};

/// The final computed style for a widget after cascade resolution.
///
/// This struct contains all resolved style properties that can be
/// applied to a widget. Values are `Option` where the property may
/// be inherited or use a default.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    // Colors
    pub color: Option<RgbaColor>,
    pub background: Option<RgbaColor>,
    pub auto_color: bool,
    /// Tint color overlay applied to all colors (both fg and bg).
    /// Uses alpha for blend strength (e.g., `tint: magenta 40%` → 0.4 alpha).
    pub tint: Option<RgbaColor>,
    /// Tint color overlay applied only to background.
    /// Uses alpha for blend strength (e.g., `background-tint: white 50%` → 0.5 alpha).
    pub background_tint: Option<RgbaColor>,
    /// Inherited effective background from parent (for auto color resolution).
    /// This is used when the widget is transparent to resolve auto colors
    /// against the parent's background.
    pub inherited_background: Option<RgbaColor>,

    // Layout Dimensions
    pub width: Option<Scalar>,
    pub height: Option<Scalar>,
    pub min_width: Option<Scalar>,
    pub max_width: Option<Scalar>,
    pub min_height: Option<Scalar>,
    pub max_height: Option<Scalar>,

    // Box Model
    pub margin: Spacing,
    pub padding: Spacing,
    pub border: Border,
    pub box_sizing: BoxSizing,

    // Border title/subtitle styling
    pub border_title_align: AlignHorizontal,
    pub border_subtitle_align: AlignHorizontal,
    pub border_title_color: Option<RgbaColor>,
    pub border_subtitle_color: Option<RgbaColor>,
    pub border_title_background: Option<RgbaColor>,
    pub border_subtitle_background: Option<RgbaColor>,
    pub border_title_style: TextStyle,
    pub border_subtitle_style: TextStyle,

    // Text & Content Alignment
    pub text_align: TextAlign,
    pub text_opacity: f64,
    pub text_style: TextStyle,
    pub text_overflow: TextOverflow,
    pub text_wrap: TextWrap,
    pub content_align_horizontal: AlignHorizontal,
    pub content_align_vertical: AlignVertical,

    // Container Alignment (positions children within container)
    pub align_horizontal: AlignHorizontal,
    pub align_vertical: AlignVertical,

    // Display & Visibility
    pub display: Display,
    pub visibility: Visibility,
    pub opacity: f64,

    // Layout mode
    pub layout: Layout,

    // Positioning (relative/absolute)
    pub position: Position,

    // Dock position (removes from layout flow)
    pub dock: Option<Dock>,

    // Layer support (controls rendering order and overlay behavior)
    /// Available layer names for this container's children.
    /// Layers are rendered in order: lower indices first (bottom), higher indices on top.
    /// Example: `layers: below above;` defines "below" as index 0, "above" as index 1.
    pub layers: Option<Vec<String>>,
    /// The layer this widget is assigned to.
    /// Must match a name from the nearest ancestor's `layers` definition.
    /// Default is "default" if not specified.
    pub layer: Option<String>,

    // Grid layout
    pub grid: GridStyle,
    pub grid_placement: GridPlacement,

    // Scroller behavior
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,

    // Scrollbar styling
    pub scrollbar: ScrollbarStyle,

    // Link styling
    pub link: LinkStyle,

    // Hatch pattern fill
    pub hatch: Option<Hatch>,

    // Keyline (box-drawing borders around widgets)
    pub keyline: Keyline,

    // Offset (visual position adjustment after layout)
    /// Horizontal offset from calculated position. Positive moves right.
    pub offset_x: Option<Scalar>,
    /// Vertical offset from calculated position. Positive moves down.
    pub offset_y: Option<Scalar>,

    // Outline (non-layout-affecting border overlay)
    /// Outline renders ON TOP of content, unlike border which affects layout.
    /// Uses same Border struct but renders as final overlay pass.
    pub outline: Border,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            color: None,
            background: None,
            auto_color: false,
            tint: None,
            background_tint: None,
            inherited_background: None,
            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            margin: Spacing::default(),
            padding: Spacing::default(),
            border: Border::default(),
            box_sizing: BoxSizing::default(),
            border_title_align: AlignHorizontal::Left,
            border_subtitle_align: AlignHorizontal::Right,
            border_title_color: None,
            border_subtitle_color: None,
            border_title_background: None,
            border_subtitle_background: None,
            border_title_style: TextStyle::default(),
            border_subtitle_style: TextStyle::default(),
            text_align: TextAlign::default(),
            text_opacity: 1.0,
            text_style: TextStyle::default(),
            text_overflow: TextOverflow::default(),
            text_wrap: TextWrap::default(),
            content_align_horizontal: AlignHorizontal::default(),
            content_align_vertical: AlignVertical::default(),
            align_horizontal: AlignHorizontal::default(),
            align_vertical: AlignVertical::default(),
            display: Display::default(),
            visibility: Visibility::default(),
            opacity: 1.0,
            layout: Layout::default(),
            position: Position::default(),
            dock: None,
            layers: None,
            layer: None,
            grid: GridStyle::default(),
            grid_placement: GridPlacement::default(),
            overflow_x: Overflow::default(),
            overflow_y: Overflow::default(),
            scrollbar: ScrollbarStyle::default(),
            link: LinkStyle::default(),
            hatch: None,
            keyline: Keyline::default(),
            offset_x: None,
            offset_y: None,
            outline: Border::default(),
        }
    }
}

impl ComputedStyle {
    /// Get the effective background color with opacity, alpha compositing, and background-tint applied.
    /// Falls back to inherited background from parent if this widget has no background.
    ///
    /// This method applies opacity SQUARED to match Python Textual's behavior where
    /// `_apply_opacity` post-processes ALL segments (blending colors toward base_background).
    /// Two sequential blends at factor `f` equals one blend at `f²`.
    pub fn effective_background(&self) -> Option<RgbaColor> {
        // First compute the base background (with alpha compositing and tint)
        let base_bg = match (&self.background, &self.inherited_background) {
            (Some(bg), Some(inherited)) if bg.a < 1.0 => {
                // Composite semi-transparent background over inherited
                let composited = bg.blend_over(inherited);
                // Then apply tint if present
                match &self.background_tint {
                    Some(tint) => Some(composited.tint(tint)),
                    None => Some(composited),
                }
            }
            (Some(bg), _) => {
                // Opaque background or no inherited - just apply tint
                match &self.background_tint {
                    Some(tint) => Some(bg.tint(tint)),
                    None => Some(bg.clone()),
                }
            }
            (None, Some(inherited)) => {
                // No background specified, inherit from parent
                Some(inherited.clone())
            }
            (None, None) => None,
        };

        // Apply opacity SQUARED by blending toward inherited_background
        // This matches Python Textual's behavior where _apply_opacity post-processes ALL segments
        let effective_opacity = self.opacity * self.opacity;
        match (&base_bg, &self.inherited_background) {
            (Some(bg), Some(inherited)) if self.opacity < 1.0 => {
                Some(bg.blend_toward(inherited, effective_opacity))
            }
            _ => base_bg,
        }
    }
}
