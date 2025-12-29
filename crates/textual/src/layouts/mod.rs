//! Layout algorithms for arranging child widgets within containers.
//!
//! This module provides the core layout infrastructure, separating layout algorithms
//! from container widgets. Any container can use any layout via the `layout` CSS property.
//!
//! ## Architecture
//!
//! - **Layout trait**: Defines the `arrange()` method that layouts must implement
//! - **GridLayout**: CSS Grid-like layout with spanning and flexible sizing
//! - **VerticalLayout**: Stacks children top-to-bottom
//! - **HorizontalLayout**: Stacks children left-to-right
//!
//! ## Usage
//!
//! Containers call `arrange_children()` which dispatches to the appropriate layout
//! based on the container's `style.layout` CSS property.

mod grid;
mod horizontal;
mod vertical;

pub use grid::GridLayout;
pub use horizontal::HorizontalLayout;
pub use vertical::VerticalLayout;

use crate::canvas::Region;
use tcss::types::{ComputedStyle, Layout as LayoutKind};

/// Result of layout arrangement - maps child indices to their computed regions.
#[derive(Debug, Clone)]
pub struct WidgetPlacement {
    /// Index of the child widget in the children vector
    pub child_index: usize,
    /// Computed region where this child should be rendered
    pub region: Region,
}

/// Layout algorithm trait.
///
/// Layouts compute the regions where each child widget should be rendered,
/// based on the parent's style, children's styles, and available space.
pub trait Layout {
    /// Arrange children within the available region.
    ///
    /// Returns a vector of placements mapping child indices to their computed regions.
    ///
    /// # Arguments
    /// * `parent_style` - The computed style of the parent container
    /// * `children` - Vector of (child_index, child_style) for visible children
    /// * `available` - The region available for layout
    fn arrange(
        &mut self,
        parent_style: &ComputedStyle,
        children: &[(usize, ComputedStyle)],
        available: Region,
    ) -> Vec<WidgetPlacement>;

    /// Downcast to GridLayout for pre_layout configuration.
    ///
    /// Used by ItemGrid-like containers to configure grid-specific properties
    /// at runtime before layout.
    fn as_grid_mut(&mut self) -> Option<&mut GridLayout> {
        None
    }
}

/// Dispatch to the appropriate layout algorithm based on CSS.
///
/// This is the main entry point for containers. It:
/// 1. Creates the appropriate layout instance based on `parent_style.layout`
/// 2. Calls `parent.pre_layout()` to allow runtime configuration
/// 3. Runs the layout algorithm and returns placements
///
/// # Arguments
/// * `parent_style` - The computed style of the parent container
/// * `children` - Vector of (child_index, child_style) for visible children
/// * `available` - The region available for layout
pub fn arrange_children(
    parent_style: &ComputedStyle,
    children: &[(usize, ComputedStyle)],
    available: Region,
) -> Vec<WidgetPlacement> {
    match parent_style.layout {
        LayoutKind::Grid => {
            let mut layout = GridLayout::default();
            layout.arrange(parent_style, children, available)
        }
        LayoutKind::Vertical => {
            let mut layout = VerticalLayout;
            layout.arrange(parent_style, children, available)
        }
        LayoutKind::Horizontal => {
            let mut layout = HorizontalLayout;
            layout.arrange(parent_style, children, available)
        }
    }
}

/// Dispatch with pre_layout hook support.
///
/// Like `arrange_children`, but takes a mutable reference to a trait object
/// that can configure the layout before arrangement.
///
/// # Arguments
/// * `pre_layout` - A callback that receives the layout for configuration
/// * `parent_style` - The computed style of the parent container
/// * `children` - Vector of (child_index, child_style) for visible children
/// * `available` - The region available for layout
pub fn arrange_children_with_pre_layout<F>(
    pre_layout: F,
    parent_style: &ComputedStyle,
    children: &[(usize, ComputedStyle)],
    available: Region,
) -> Vec<WidgetPlacement>
where
    F: FnOnce(&mut dyn Layout),
{
    match parent_style.layout {
        LayoutKind::Grid => {
            let mut layout = GridLayout::default();
            pre_layout(&mut layout);
            layout.arrange(parent_style, children, available)
        }
        LayoutKind::Vertical => {
            let mut layout = VerticalLayout;
            pre_layout(&mut layout);
            layout.arrange(parent_style, children, available)
        }
        LayoutKind::Horizontal => {
            let mut layout = HorizontalLayout;
            pre_layout(&mut layout);
            layout.arrange(parent_style, children, available)
        }
    }
}
