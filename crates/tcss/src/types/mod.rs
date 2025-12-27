//! Core types for TCSS styling.
//!
//! This module provides the fundamental types used throughout TCSS:
//!
//! - [`RgbaColor`]: Colors with RGBA components and theme variable support
//! - [`Scalar`], [`Spacing`]: Dimension and box model values
//! - [`Border`], [`BorderEdge`]: Border styling
//! - [`TextStyle`], [`TextAlign`]: Text formatting
//! - [`Display`], [`Visibility`], [`Overflow`]: Layout control
//! - [`Theme`]: Color theme definitions
//! - [`ComputedStyle`]: Final computed styles for a widget
//!
//! ## Module Organization
//!
//! - [`color`]: RGBA color with parsing and theme variable support
//! - [`geometry`]: Scalars (dimensions) and spacing (margins/padding)
//! - [`border`]: Border kinds, edges, and full border definitions
//! - [`text`]: Text styling and alignment
//! - [`layout`]: Display modes, visibility, and overflow
//! - [`theme`]: Theme color palettes

pub mod border;
pub mod color;
pub mod geometry;
pub mod layout;
pub mod text;
pub mod theme;

pub use border::{Border, BorderEdge, BorderKind};
pub use color::RgbaColor;
pub use geometry::{Scalar, Spacing, Unit};
pub use layout::{Display, Overflow, Visibility};
pub use text::{AlignHorizontal, AlignVertical, TextAlign, TextStyle};
pub use theme::Theme;

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

    // Text & Content Alignment
    pub text_align: TextAlign,
    pub text_style: TextStyle,
    pub content_align_horizontal: AlignHorizontal,
    pub content_align_vertical: AlignVertical,

    // Display & Visibility
    pub display: Display,
    pub visibility: Visibility,
    pub opacity: f64,

    // Scroller behavior
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            color: None,
            background: None,
            auto_color: false,
            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            margin: Spacing::default(),
            padding: Spacing::default(),
            border: Border::default(),
            text_align: TextAlign::default(),
            text_style: TextStyle::default(),
            content_align_horizontal: AlignHorizontal::default(),
            content_align_vertical: AlignVertical::default(),
            display: Display::default(),
            visibility: Visibility::default(),
            opacity: 1.0,
            overflow_x: Overflow::default(),
            overflow_y: Overflow::default(),
        }
    }
}
