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
//!
//! ## Module Organization
//!
//! - [`color`]: RGBA color with parsing and theme variable support
//! - [`geometry`]: Scalars (dimensions) and spacing (margins/padding)
//! - [`border`]: Border kinds, edges, and full border definitions
//! - [`text`]: Text styling and alignment
//! - [`layout`]: Display modes, layout modes, visibility, and overflow
//! - [`grid`]: CSS Grid configuration and child placement
//! - [`link`]: Link styling configuration
//! - [`theme`]: Theme color palettes
//! - [`scrollbar`]: Scrollbar styling, sizes, and visibility

pub mod border;
pub mod color;
pub mod geometry;
pub mod grid;
pub mod layout;
pub mod link;
pub mod scrollbar;
pub mod text;
pub mod theme;

pub use border::{Border, BorderEdge, BorderKind};
pub use color::RgbaColor;
pub use geometry::{Scalar, Spacing, Unit};
pub use grid::{GridPlacement, GridStyle};
pub use layout::{BoxSizing, Display, Dock, Layout, Overflow, Visibility};
pub use link::LinkStyle;
pub use scrollbar::{ScrollbarGutter, ScrollbarSize, ScrollbarStyle, ScrollbarVisibility};
pub use text::{AlignHorizontal, AlignVertical, TextAlign, TextStyle};
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
    pub text_style: TextStyle,
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

    // Dock position (removes from layout flow)
    pub dock: Option<Dock>,

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
            text_style: TextStyle::default(),
            content_align_horizontal: AlignHorizontal::default(),
            content_align_vertical: AlignVertical::default(),
            align_horizontal: AlignHorizontal::default(),
            align_vertical: AlignVertical::default(),
            display: Display::default(),
            visibility: Visibility::default(),
            opacity: 1.0,
            layout: Layout::default(),
            dock: None,
            grid: GridStyle::default(),
            grid_placement: GridPlacement::default(),
            overflow_x: Overflow::default(),
            overflow_y: Overflow::default(),
            scrollbar: ScrollbarStyle::default(),
            link: LinkStyle::default(),
        }
    }
}
