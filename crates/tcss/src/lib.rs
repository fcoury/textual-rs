//! # TCSS - Textual CSS Parser
//!
//! A Rust implementation of Textual's CSS dialect (TCSS) for terminal user interfaces.
//!
//! TCSS extends standard CSS with terminal-specific features like cell-based units,
//! theme variables, and TUI-focused properties. This crate provides:
//!
//! - **Parsing**: Convert TCSS source text into a structured [`StyleSheet`](parser::StyleSheet)
//! - **Cascade**: Apply CSS specificity rules to compute final styles for widgets
//! - **Types**: Rich type system for colors, spacing, borders, and layout properties
//!
//! ## Quick Start
//!
//! ```rust
//! use tcss::parser::parse_stylesheet;
//!
//! let source = r#"
//!     Button {
//!         color: red;
//!         width: 100%;
//!         margin: 1 2;
//!     }
//!
//!     Button.primary {
//!         background: blue;
//!     }
//! "#;
//!
//! let stylesheet = parse_stylesheet(source).expect("valid TCSS");
//! assert_eq!(stylesheet.rules.len(), 2);
//! ```
//!
//! ## Supported Features
//!
//! ### Selectors
//! - Type selectors: `Button`, `Label`, `Container`
//! - Class selectors: `.primary`, `.active`
//! - ID selectors: `#submit`, `#header`
//! - Universal selector: `*`
//! - Compound selectors: `Button.primary#submit`
//! - Descendant combinator: `Container Button`
//! - Child combinator: `Container > Button`
//!
//! ### Properties
//! - Colors: `color`, `background`
//! - Dimensions: `width`, `height`
//! - Box model: `margin`, `padding`, `border`
//!
//! ### Units
//! - Cells (default): `10`, `20`
//! - Percentage: `50%`, `100%`
//! - Viewport: `50vw`, `100vh`
//! - Fraction: `1fr`, `2fr`
//! - Auto: `auto`
//!
//! ### Pseudo-classes
//! - `:focus` - Widget has keyboard focus
//! - `:hover` - Mouse is over widget
//! - `:active` - Widget is being pressed
//! - `:disabled` - Widget is not interactive
//!
//! ## Not Yet Implemented
//!
//! - CSS comments (`/* */`)
//!
//! ## Modules
//!
//! - [`parser`]: TCSS parsing and stylesheet data structures
//! - [`types`]: Core types for colors, geometry, borders, and layout
//! - [`error`]: Error types for parsing failures

pub mod error;
pub mod parser;
pub mod types;

pub use error::TcssError;
pub use parser::cascade::{WidgetMeta, WidgetStates};
pub use types::ComputedStyle;
