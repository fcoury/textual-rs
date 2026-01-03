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

pub use grid::{GridLayout, GridTrackInfo};
pub use horizontal::HorizontalLayout;
pub use size_resolver::{
    DEFAULT_FIXED_HEIGHT, DEFAULT_FIXED_WIDTH, resolve_height, resolve_height_fill,
    resolve_height_fixed, resolve_height_with_intrinsic, resolve_width, resolve_width_fill,
    resolve_width_fixed, resolve_width_with_intrinsic,
};
pub use vertical::VerticalLayout;

use crate::canvas::{Region, Size};
use crate::widget::Widget;
use tcss::types::{ComputedStyle, Dock, Layout as LayoutKind, Position};

/// Layout-specific view of a widget for intrinsic measurements.
pub trait LayoutNode {
    fn desired_size(&self) -> Size;
    fn intrinsic_height_for_width(&self, width: u16) -> u16;
}

impl<M> LayoutNode for dyn Widget<M> {
    fn desired_size(&self) -> Size {
        Widget::desired_size(self)
    }

    fn intrinsic_height_for_width(&self, width: u16) -> u16 {
        Widget::intrinsic_height_for_width(self, width)
    }
}

impl<M> LayoutNode for Box<dyn Widget<M>> {
    fn desired_size(&self) -> Size {
        self.as_ref().desired_size()
    }

    fn intrinsic_height_for_width(&self, width: u16) -> u16 {
        self.as_ref().intrinsic_height_for_width(width)
    }
}

/// Child metadata for layout algorithms.
pub struct LayoutChild<'a> {
    pub index: usize,
    pub style: ComputedStyle,
    pub desired_size: Size,
    pub node: &'a dyn LayoutNode,
}

/// Result of layout arrangement - maps child indices to their computed regions.
#[derive(Debug, Clone)]
pub struct WidgetPlacement {
    /// Index of the child widget in the children vector
    pub child_index: usize,
    /// Computed region where this child should be rendered
    pub region: Region,
}

/// Viewport dimensions for CSS vw/vh unit resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: i32,
    pub height: i32,
}

impl From<Region> for Viewport {
    fn from(region: Region) -> Self {
        Self {
            width: region.width,
            height: region.height,
        }
    }
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
    /// * `viewport` - The viewport dimensions for vw/vh unit resolution
    fn arrange(
        &mut self,
        parent_style: &ComputedStyle,
        children: &[LayoutChild],
        available: Region,
        viewport: Viewport,
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
    children: &[LayoutChild],
    available: Region,
) -> Vec<WidgetPlacement> {
    // Use available region as viewport (fallback for containers that don't know viewport)
    arrange_children_with_viewport(parent_style, children, available, available.into())
}

/// Dispatch to the appropriate layout algorithm with explicit viewport dimensions.
///
/// This variant allows specifying the viewport dimensions separately from the available
/// region. This is important for correct `vw`/`vh` unit resolution when the available
/// region differs from the viewport (e.g., due to docked widgets on the default layer).
///
/// # Arguments
/// * `parent_style` - The computed style of the parent container
/// * `children` - Vector of (child_index, child_style, desired_size) for visible children
/// * `available` - The region available for layout
/// * `viewport` - The viewport dimensions for vw/vh unit resolution
pub fn arrange_children_with_viewport(
    parent_style: &ComputedStyle,
    children: &[LayoutChild],
    available: Region,
    viewport: Viewport,
) -> Vec<WidgetPlacement> {
    // Separate docked widgets from layout widgets
    let (docked, non_docked): (Vec<_>, Vec<_>) = children
        .iter()
        .partition(|child| child.style.dock.is_some());

    // Absolute-positioned widgets are removed from normal flow
    let (absolute_children, layout_children): (Vec<&LayoutChild>, Vec<&LayoutChild>) = non_docked
        .into_iter()
        .partition(|child| child.style.position == Position::Absolute);

    // Process docked widgets first
    let docked_vec: Vec<LayoutChild> = docked
        .iter()
        .map(|child| LayoutChild {
            index: child.index,
            style: child.style.clone(),
            desired_size: child.desired_size,
            node: child.node,
        })
        .collect();
    let (mut placements, dock_spacing) = arrange_docked_widgets(&docked_vec, available);

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
        .map(|child| LayoutChild {
            index: child.index,
            style: child.style.clone(),
            desired_size: child.desired_size,
            node: child.node,
        })
        .collect();

    let absolute_children_vec: Vec<_> = absolute_children
        .iter()
        .map(|child| LayoutChild {
            index: child.index,
            style: child.style.clone(),
            desired_size: child.desired_size,
            node: child.node,
        })
        .collect();

    let mut layout_placements = match parent_style.layout {
        LayoutKind::Grid => {
            let mut layout = GridLayout::default();
            layout.arrange(parent_style, &layout_children_vec, content_region, viewport)
        }
        LayoutKind::Vertical => {
            let mut layout = VerticalLayout;
            layout.arrange(parent_style, &layout_children_vec, content_region, viewport)
        }
        LayoutKind::Horizontal => {
            let mut layout = HorizontalLayout;
            layout.arrange(parent_style, &layout_children_vec, content_region, viewport)
        }
    };

    // Apply post-layout alignment to layout widgets only
    apply_alignment(
        &mut layout_placements,
        &layout_children_vec,
        parent_style,
        content_region,
    );

    // Apply CSS offset to all placements (both docked and layout)
    apply_offset(&mut placements, children, viewport);
    apply_offset(&mut layout_placements, children, viewport);

    // Absolute placements: position at the content origin, then apply offset
    let mut absolute_placements: Vec<WidgetPlacement> = absolute_children_vec
        .iter()
        .map(|child| resolve_absolute_placement(child, content_region, viewport))
        .collect();
    apply_offset(&mut absolute_placements, children, viewport);

    // Combine docked and layout placements
    placements.extend(layout_placements);
    placements.extend(absolute_placements);

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
    docked: &[LayoutChild],
    available: Region,
) -> (Vec<WidgetPlacement>, DockSpacing) {
    let mut placements = Vec::new();
    let mut spacing = DockSpacing::default();

    for child in docked.iter() {
        let dock = child
            .style
            .dock
            .expect("docked widget must have dock property");

        // Convert size to i32 for region calculations
        let widget_width = child.desired_size.width as i32;
        let widget_height = child.desired_size.height as i32;

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
            child_index: child.index,
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
    children: &[LayoutChild],
    available: Region,
) -> Vec<WidgetPlacement>
where
    F: FnOnce(&mut dyn Layout),
{
    // Use available region as viewport (fallback)
    let viewport: Viewport = available.into();

    // Separate docked widgets from layout widgets
    let (docked, layout_children): (Vec<_>, Vec<_>) = children
        .iter()
        .partition(|child| child.style.dock.is_some());

    // Process docked widgets first
    let docked_vec: Vec<LayoutChild> = docked
        .iter()
        .map(|child| LayoutChild {
            index: child.index,
            style: child.style.clone(),
            desired_size: child.desired_size,
            node: child.node,
        })
        .collect();
    let (mut placements, dock_spacing) = arrange_docked_widgets(&docked_vec, available);

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
        .map(|child| LayoutChild {
            index: child.index,
            style: child.style.clone(),
            desired_size: child.desired_size,
            node: child.node,
        })
        .collect();

    let mut layout_placements = match parent_style.layout {
        LayoutKind::Grid => {
            let mut layout = GridLayout::default();
            pre_layout(&mut layout);
            layout.arrange(parent_style, &layout_children_vec, content_region, viewport)
        }
        LayoutKind::Vertical => {
            let mut layout = VerticalLayout;
            pre_layout(&mut layout);
            layout.arrange(parent_style, &layout_children_vec, content_region, viewport)
        }
        LayoutKind::Horizontal => {
            let mut layout = HorizontalLayout;
            pre_layout(&mut layout);
            layout.arrange(parent_style, &layout_children_vec, content_region, viewport)
        }
    };

    // Apply post-layout alignment to layout widgets only
    apply_alignment(
        &mut layout_placements,
        &layout_children_vec,
        parent_style,
        content_region,
    );

    // Apply CSS offset to all placements (both docked and layout)
    apply_offset(&mut placements, children, viewport);
    apply_offset(&mut layout_placements, children, viewport);

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
pub(crate) fn apply_alignment(
    placements: &mut [WidgetPlacement],
    children: &[LayoutChild],
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

    // Calculate bounding box of all placements, including margins
    let bounds = get_placement_bounds_with_margins(placements, children);

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
fn get_placement_bounds_with_margins(
    placements: &[WidgetPlacement],
    children: &[LayoutChild],
) -> Region {
    if placements.is_empty() {
        return Region::new(0, 0, 0, 0);
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for p in placements {
        let (margin_left, margin_right, margin_top, margin_bottom) =
            if let Some(child) = children.iter().find(|child| child.index == p.child_index) {
                let style = &child.style;
                (
                    style.margin.left.value as i32,
                    style.margin.right.value as i32,
                    style.margin.top.value as i32,
                    style.margin.bottom.value as i32,
                )
            } else {
                (0, 0, 0, 0)
            };

        min_x = min_x.min(p.region.x - margin_left);
        min_y = min_y.min(p.region.y - margin_top);
        max_x = max_x.max(p.region.x + p.region.width + margin_right);
        max_y = max_y.max(p.region.y + p.region.height + margin_bottom);
    }

    Region {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

/// Apply CSS offset to placements.
///
/// Offset is a POST-LAYOUT operation that visually shifts widgets from their
/// calculated positions. It does not affect sibling layout - only the visual
/// position of the offset widget itself.
///
/// # Arguments
/// * `placements` - The widget placements to modify
/// * `children` - The original children array to find styles
/// * `viewport` - Viewport for resolving viewport-relative units
fn apply_offset(placements: &mut [WidgetPlacement], children: &[LayoutChild], viewport: Viewport) {
    use tcss::types::geometry::Unit;

    for placement in placements {
        // Find the style for this child
        if let Some(child) = children
            .iter()
            .find(|child| child.index == placement.child_index)
        {
            // Resolve offset_x
            let offset_x = if let Some(scalar) = &child.style.offset_x {
                match scalar.unit {
                    Unit::Cells => scalar.value as i32,
                    Unit::Percent => {
                        ((scalar.value / 100.0) * placement.region.width as f64) as i32
                    }
                    Unit::ViewWidth => ((scalar.value / 100.0) * viewport.width as f64) as i32,
                    Unit::ViewHeight => ((scalar.value / 100.0) * viewport.height as f64) as i32,
                    Unit::Width => ((scalar.value / 100.0) * placement.region.width as f64) as i32,
                    Unit::Height => {
                        ((scalar.value / 100.0) * placement.region.height as f64) as i32
                    }
                    _ => scalar.value as i32,
                }
            } else {
                0
            };

            // Resolve offset_y
            let offset_y = if let Some(scalar) = &child.style.offset_y {
                match scalar.unit {
                    Unit::Cells => scalar.value as i32,
                    Unit::Percent => {
                        ((scalar.value / 100.0) * placement.region.height as f64) as i32
                    }
                    Unit::ViewWidth => ((scalar.value / 100.0) * viewport.width as f64) as i32,
                    Unit::ViewHeight => ((scalar.value / 100.0) * viewport.height as f64) as i32,
                    Unit::Width => ((scalar.value / 100.0) * placement.region.width as f64) as i32,
                    Unit::Height => {
                        ((scalar.value / 100.0) * placement.region.height as f64) as i32
                    }
                    _ => scalar.value as i32,
                }
            } else {
                0
            };

            // Apply offsets
            if offset_x != 0 || offset_y != 0 {
                placement.region.x += offset_x;
                placement.region.y += offset_y;
            }
        }
    }
}

fn resolve_absolute_placement(
    child: &LayoutChild,
    available: Region,
    viewport: Viewport,
) -> WidgetPlacement {
    use tcss::types::geometry::Unit;

    let style = &child.style;
    let desired = child.desired_size;

    let intrinsic_width = if desired.width == u16::MAX {
        available.width.clamp(0, u16::MAX as i32) as u16
    } else {
        desired.width
    };

    let width = if let Some(w) = &style.width {
        let raw = match w.unit {
            Unit::Cells => w.value as i32,
            Unit::Percent => ((w.value / 100.0) * available.width as f64).round() as i32,
            Unit::Width => ((w.value / 100.0) * available.width as f64).round() as i32,
            Unit::Height => ((w.value / 100.0) * available.height as f64).round() as i32,
            Unit::ViewWidth => ((w.value / 100.0) * viewport.width as f64).round() as i32,
            Unit::ViewHeight => ((w.value / 100.0) * viewport.height as f64).round() as i32,
            Unit::Fraction => available.width,
            Unit::Auto => intrinsic_width as i32,
        };
        size_resolver::apply_box_sizing_width(raw, style)
    } else {
        intrinsic_width as i32
    };

    let width = if let Some(max_w) = &style.max_width {
        let max_width_value = match max_w.unit {
            Unit::Cells => max_w.value as i32,
            Unit::Percent => ((max_w.value / 100.0) * available.width as f64) as i32,
            Unit::Width => ((max_w.value / 100.0) * available.width as f64) as i32,
            Unit::Height => ((max_w.value / 100.0) * available.height as f64) as i32,
            Unit::ViewWidth => ((max_w.value / 100.0) * viewport.width as f64) as i32,
            Unit::ViewHeight => ((max_w.value / 100.0) * viewport.height as f64) as i32,
            _ => max_w.value as i32,
        };
        width.min(max_width_value)
    } else {
        width
    };

    let width = if let Some(min_w) = &style.min_width {
        let min_width_value = match min_w.unit {
            Unit::Cells => min_w.value as i32,
            Unit::Percent => ((min_w.value / 100.0) * available.width as f64) as i32,
            Unit::Width => ((min_w.value / 100.0) * available.width as f64) as i32,
            Unit::Height => ((min_w.value / 100.0) * available.height as f64) as i32,
            Unit::ViewWidth => ((min_w.value / 100.0) * viewport.width as f64) as i32,
            Unit::ViewHeight => ((min_w.value / 100.0) * viewport.height as f64) as i32,
            _ => min_w.value as i32,
        };
        width.max(min_width_value)
    } else {
        width
    };

    let width_u16 = width.clamp(0, u16::MAX as i32) as u16;

    let height = if let Some(h) = &style.height {
        let raw = match h.unit {
            Unit::Cells => h.value as i32,
            Unit::Percent => ((h.value / 100.0) * available.height as f64).round() as i32,
            Unit::Width => ((h.value / 100.0) * available.width as f64).round() as i32,
            Unit::Height => ((h.value / 100.0) * available.height as f64).round() as i32,
            Unit::ViewWidth => ((h.value / 100.0) * viewport.width as f64).round() as i32,
            Unit::ViewHeight => ((h.value / 100.0) * viewport.height as f64).round() as i32,
            Unit::Fraction => available.height,
            Unit::Auto => child.node.intrinsic_height_for_width(width_u16) as i32,
        };
        size_resolver::apply_box_sizing_height(raw, style)
    } else {
        child.node.intrinsic_height_for_width(width_u16) as i32
    };

    let height = if let Some(max_h) = &style.max_height {
        let max_height_value = match max_h.unit {
            Unit::Cells => max_h.value,
            Unit::Percent => (max_h.value / 100.0) * available.height as f64,
            Unit::Width => (max_h.value / 100.0) * available.width as f64,
            Unit::Height => (max_h.value / 100.0) * available.height as f64,
            Unit::ViewWidth => (max_h.value / 100.0) * viewport.width as f64,
            Unit::ViewHeight => (max_h.value / 100.0) * viewport.height as f64,
            _ => max_h.value,
        };
        height.min(max_height_value as i32)
    } else {
        height
    };

    let height = if let Some(min_h) = &style.min_height {
        let min_height_value = match min_h.unit {
            Unit::Cells => min_h.value,
            Unit::Percent => (min_h.value / 100.0) * available.height as f64,
            Unit::Width => (min_h.value / 100.0) * available.width as f64,
            Unit::Height => (min_h.value / 100.0) * available.height as f64,
            Unit::ViewWidth => (min_h.value / 100.0) * viewport.width as f64,
            Unit::ViewHeight => (min_h.value / 100.0) * viewport.height as f64,
            _ => min_h.value,
        };
        height.max(min_height_value as i32)
    } else {
        height
    };

    WidgetPlacement {
        child_index: child.index,
        region: Region {
            x: available.x,
            y: available.y,
            width: width.max(0),
            height: height.max(0),
        },
    }
}

/// Layer-aware arrangement of children.
///
/// This is the primary entry point for child arrangement. It implements
/// Textual Python's layer system where:
/// 1. Each layer gets the FULL available region (layers don't compete for space)
/// 2. Lower layer indices render first (bottom), higher indices on top
/// 3. Within each layer, docked widgets are processed, then layout widgets
///
/// When no layers are defined and no children have layer assignments, this
/// falls back to the standard arrangement for efficiency.
///
/// # Arguments
/// * `parent_style` - The computed style of the parent container (contains `layers` definition)
/// * `children` - Vector of (child_index, child_style, desired_size) for visible children
/// * `available` - The region available for layout
/// * `viewport` - The viewport dimensions for vw/vh unit resolution
pub fn arrange_children_with_layers(
    parent_style: &ComputedStyle,
    children: &[LayoutChild],
    available: Region,
    viewport: Viewport,
) -> Vec<WidgetPlacement> {
    // Fast path: if no layers defined and no children have layer assignments,
    // use the standard arrangement to avoid overhead
    let needs_layers =
        parent_style.layers.is_some() || children.iter().any(|child| child.style.layer.is_some());

    if !needs_layers {
        return arrange_children_with_viewport(parent_style, children, available, viewport);
    }

    // Get the layer order from parent style, defaulting to ["default"]
    let layer_order: Vec<String> = parent_style
        .layers
        .clone()
        .unwrap_or_else(|| vec!["default".to_string()]);

    // Build a map from layer name to its widgets
    let layers = build_layers(children, &layer_order);

    // Process each layer in order - each layer gets the FULL available region
    let mut all_placements = Vec::new();

    for (_layer_name, layer_children) in layers {
        if layer_children.is_empty() {
            continue;
        }

        // Convert references to owned data for the layout functions
        let children_vec: Vec<LayoutChild> = layer_children
            .iter()
            .map(|child| LayoutChild {
                index: child.index,
                style: child.style.clone(),
                desired_size: child.desired_size,
                node: child.node,
            })
            .collect();

        // Each layer gets the FULL available region - this is the key behavior!
        // Layers don't compete for space; they overlay each other.
        let layer_placements =
            arrange_children_with_viewport(parent_style, &children_vec, available, viewport);

        all_placements.extend(layer_placements);
    }

    all_placements
}

/// Separate widgets by layer.
///
/// Returns a vector of (layer_name, widgets) tuples in layer order.
/// Widgets without an explicit layer are assigned to "default".
///
/// Layer order follows Python Textual's behavior:
/// - Widgets on undefined layers (like "default" if not in layer_order) render FIRST (bottom)
/// - Explicitly defined layers render in order (later = on top)
fn build_layers<'a>(
    children: &'a [LayoutChild],
    layer_order: &[String],
) -> Vec<(String, Vec<&'a LayoutChild<'a>>)> {
    use std::collections::HashMap;

    // Group children by their layer
    let mut layer_map: HashMap<String, Vec<&LayoutChild>> = HashMap::new();

    for child in children {
        let layer_name = child
            .style
            .layer
            .clone()
            .unwrap_or_else(|| "default".to_string());
        layer_map.entry(layer_name).or_default().push(child);
    }

    let mut result = Vec::new();

    // First: add widgets on layers NOT in layer_order (they render at the bottom)
    // This matches Python where undefined layers default to index 0 (bottom-most)
    for (name, widgets) in layer_map.iter() {
        if !layer_order.contains(name) {
            result.push((name.clone(), widgets.clone()));
        }
    }

    // Then: add explicitly defined layers in order (later = on top)
    for name in layer_order {
        if let Some(widgets) = layer_map.get(name) {
            result.push((name.clone(), widgets.clone()));
        }
    }

    result
}
