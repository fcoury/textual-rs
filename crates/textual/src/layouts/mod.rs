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
pub mod size_resolver;
mod vertical;

pub use grid::GridLayout;
pub use horizontal::HorizontalLayout;
pub use size_resolver::{
    resolve_height, resolve_height_fill, resolve_height_fixed, resolve_height_with_intrinsic,
    resolve_width, resolve_width_fill, resolve_width_fixed, resolve_width_with_intrinsic,
    DEFAULT_FIXED_HEIGHT, DEFAULT_FIXED_WIDTH,
};
pub use vertical::VerticalLayout;

use crate::canvas::{Region, Size};
use tcss::types::{ComputedStyle, Dock, Layout as LayoutKind};

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
    /// * `children` - Vector of (child_index, child_style, desired_size) for visible children
    /// * `available` - The region available for layout
    fn arrange(
        &mut self,
        parent_style: &ComputedStyle,
        children: &[(usize, ComputedStyle, Size)],
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
/// 1. Separates docked widgets from normal layout widgets
/// 2. Positions docked widgets at their edges (top, bottom, left, right)
/// 3. Shrinks available region by the space consumed by docked widgets
/// 4. Creates the appropriate layout instance based on `parent_style.layout`
/// 5. Runs the layout algorithm on remaining widgets
/// 6. Applies post-layout alignment based on `align_horizontal` and `align_vertical`
///
/// # Arguments
/// * `parent_style` - The computed style of the parent container
/// * `children` - Vector of (child_index, child_style, desired_size) for visible children
/// * `available` - The region available for layout
pub fn arrange_children(
    parent_style: &ComputedStyle,
    children: &[(usize, ComputedStyle, Size)],
    available: Region,
) -> Vec<WidgetPlacement> {
    // Separate docked widgets from layout widgets
    let (docked, layout_children): (Vec<_>, Vec<_>) =
        children.iter().partition(|(_, style, _)| style.dock.is_some());

    // Process docked widgets first
    let (mut placements, dock_spacing) = arrange_docked_widgets(&docked, available);

    // Shrink available region for layout widgets
    let content_region = Region {
        x: available.x + dock_spacing.left,
        y: available.y + dock_spacing.top,
        width: available
            .width
            .saturating_sub(dock_spacing.left + dock_spacing.right),
        height: available
            .height
            .saturating_sub(dock_spacing.top + dock_spacing.bottom),
    };

    // Run normal layout on remaining widgets
    let layout_children_vec: Vec<_> = layout_children
        .iter()
        .map(|(idx, style, size)| (*idx, style.clone(), *size))
        .collect();

    let mut layout_placements = match parent_style.layout {
        LayoutKind::Grid => {
            let mut layout = GridLayout::default();
            layout.arrange(parent_style, &layout_children_vec, content_region)
        }
        LayoutKind::Vertical => {
            let mut layout = VerticalLayout;
            layout.arrange(parent_style, &layout_children_vec, content_region)
        }
        LayoutKind::Horizontal => {
            let mut layout = HorizontalLayout;
            layout.arrange(parent_style, &layout_children_vec, content_region)
        }
    };

    // Apply post-layout alignment to layout widgets only
    apply_alignment(&mut layout_placements, parent_style, content_region);

    // Combine docked and layout placements
    placements.extend(layout_placements);

    placements
}

/// Spacing consumed by docked widgets on each edge.
#[derive(Debug, Clone, Copy, Default)]
struct DockSpacing {
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
}

/// Arrange docked widgets at their respective edges.
///
/// Docked widgets are removed from normal layout flow and positioned
/// at the container edges. Returns placements and the spacing consumed.
///
/// Following Textual Python's algorithm:
/// - Widgets are processed in DOM order
/// - Each docked widget is positioned relative to the full container
/// - Space tracking accumulates to reduce available space for content
fn arrange_docked_widgets(
    docked: &[&(usize, ComputedStyle, Size)],
    available: Region,
) -> (Vec<WidgetPlacement>, DockSpacing) {
    let mut placements = Vec::new();
    let mut spacing = DockSpacing::default();

    for (child_index, style, size) in docked.iter().copied() {
        let dock = style.dock.expect("docked widget must have dock property");

        // Convert size to i32 for region calculations
        let widget_width = size.width as i32;
        let widget_height = size.height as i32;

        // Calculate region based on dock direction
        let region = match dock {
            Dock::Top => {
                let region = Region {
                    x: available.x,
                    y: available.y,
                    width: available.width,
                    height: widget_height,
                };
                spacing.top = spacing.top.max(widget_height);
                region
            }
            Dock::Bottom => {
                let region = Region {
                    x: available.x,
                    y: available.y + available.height - widget_height,
                    width: available.width,
                    height: widget_height,
                };
                spacing.bottom = spacing.bottom.max(widget_height);
                region
            }
            Dock::Left => {
                let region = Region {
                    x: available.x,
                    y: available.y,
                    width: widget_width,
                    height: available.height,
                };
                spacing.left = spacing.left.max(widget_width);
                region
            }
            Dock::Right => {
                let region = Region {
                    x: available.x + available.width - widget_width,
                    y: available.y,
                    width: widget_width,
                    height: available.height,
                };
                spacing.right = spacing.right.max(widget_width);
                region
            }
        };

        placements.push(WidgetPlacement {
            child_index: *child_index,
            region,
        });
    }

    (placements, spacing)
}

/// Dispatch with pre_layout hook support.
///
/// Like `arrange_children`, but takes a mutable reference to a trait object
/// that can configure the layout before arrangement.
///
/// # Arguments
/// * `pre_layout` - A callback that receives the layout for configuration
/// * `parent_style` - The computed style of the parent container
/// * `children` - Vector of (child_index, child_style, desired_size) for visible children
/// * `available` - The region available for layout
pub fn arrange_children_with_pre_layout<F>(
    pre_layout: F,
    parent_style: &ComputedStyle,
    children: &[(usize, ComputedStyle, Size)],
    available: Region,
) -> Vec<WidgetPlacement>
where
    F: FnOnce(&mut dyn Layout),
{
    // Separate docked widgets from layout widgets
    let (docked, layout_children): (Vec<_>, Vec<_>) =
        children.iter().partition(|(_, style, _)| style.dock.is_some());

    // Process docked widgets first
    let (mut placements, dock_spacing) = arrange_docked_widgets(&docked, available);

    // Shrink available region for layout widgets
    let content_region = Region {
        x: available.x + dock_spacing.left,
        y: available.y + dock_spacing.top,
        width: available
            .width
            .saturating_sub(dock_spacing.left + dock_spacing.right),
        height: available
            .height
            .saturating_sub(dock_spacing.top + dock_spacing.bottom),
    };

    // Run normal layout on remaining widgets
    let layout_children_vec: Vec<_> = layout_children
        .iter()
        .map(|(idx, style, size)| (*idx, style.clone(), *size))
        .collect();

    let mut layout_placements = match parent_style.layout {
        LayoutKind::Grid => {
            let mut layout = GridLayout::default();
            pre_layout(&mut layout);
            layout.arrange(parent_style, &layout_children_vec, content_region)
        }
        LayoutKind::Vertical => {
            let mut layout = VerticalLayout;
            pre_layout(&mut layout);
            layout.arrange(parent_style, &layout_children_vec, content_region)
        }
        LayoutKind::Horizontal => {
            let mut layout = HorizontalLayout;
            pre_layout(&mut layout);
            layout.arrange(parent_style, &layout_children_vec, content_region)
        }
    };

    // Apply post-layout alignment to layout widgets only
    apply_alignment(&mut layout_placements, parent_style, content_region);

    // Combine docked and layout placements
    placements.extend(layout_placements);

    placements
}

/// Apply alignment to placements based on the parent's align properties.
///
/// This is a POST-LAYOUT operation that translates all placements to achieve
/// the desired horizontal and vertical alignment within the container.
///
/// The algorithm:
/// 1. Calculate the bounding box of all placements
/// 2. Calculate the offset needed to align that bounding box
/// 3. Translate all placements by that offset
fn apply_alignment(
    placements: &mut [WidgetPlacement],
    parent_style: &ComputedStyle,
    available: Region,
) {
    use tcss::types::AlignHorizontal;
    use tcss::types::AlignVertical;

    // Skip if default alignment (left/top)
    if parent_style.align_horizontal == AlignHorizontal::Left
        && parent_style.align_vertical == AlignVertical::Top
    {
        return;
    }

    if placements.is_empty() {
        return;
    }

    // Calculate bounding box of all placements
    let bounds = get_placement_bounds(placements);

    // Calculate alignment offset, clamping to 0 to prevent negative offsets
    // when content is larger than the available space
    let offset_x = match parent_style.align_horizontal {
        AlignHorizontal::Left => 0,
        AlignHorizontal::Center => (available.width - bounds.width).max(0) / 2,
        AlignHorizontal::Right => (available.width - bounds.width).max(0),
    };

    let offset_y = match parent_style.align_vertical {
        AlignVertical::Top => 0,
        AlignVertical::Middle => (available.height - bounds.height).max(0) / 2,
        AlignVertical::Bottom => (available.height - bounds.height).max(0),
    };

    // Translate all placements
    if offset_x != 0 || offset_y != 0 {
        for placement in placements {
            placement.region.x += offset_x;
            placement.region.y += offset_y;
        }
    }
}

/// Calculate the bounding box that contains all placements.
fn get_placement_bounds(placements: &[WidgetPlacement]) -> Region {
    if placements.is_empty() {
        return Region::new(0, 0, 0, 0);
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for p in placements {
        min_x = min_x.min(p.region.x);
        min_y = min_y.min(p.region.y);
        max_x = max_x.max(p.region.x + p.region.width);
        max_y = max_y.max(p.region.y + p.region.height);
    }

    Region {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}
