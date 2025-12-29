//! Horizontal Layout Margin Tests
//!
//! Tests for margin handling in the HorizontalLayout,
//! verifying consistency with VerticalLayout.

use tcss::types::{ComputedStyle, Scalar};
use textual::canvas::Region;
use textual::layouts::{HorizontalLayout, Layout};

// =============================================================================
// Tests for margin handling in HorizontalLayout
// =============================================================================

#[test]
fn test_horizontal_layout_margin_left_offsets_x() {
    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 20);

    // Create a child with margin_left = 5
    let mut child_style = ComputedStyle::default();
    child_style.margin.left = Scalar::cells(5.0);
    child_style.width = Some(Scalar::cells(20.0));

    let children = vec![(0, child_style)];
    let placements = layout.arrange(&parent_style, &children, available);

    assert_eq!(placements.len(), 1);
    // x should be offset by margin_left
    assert_eq!(
        placements[0].region.x, 5,
        "Child x should be offset by margin_left (5), got {}",
        placements[0].region.x
    );
}

#[test]
fn test_horizontal_layout_margin_right_advances_x() {
    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 20);

    // Create two children, first with margin_right = 10
    let mut child1_style = ComputedStyle::default();
    child1_style.margin.right = Scalar::cells(10.0);
    child1_style.width = Some(Scalar::cells(20.0));

    let mut child2_style = ComputedStyle::default();
    child2_style.width = Some(Scalar::cells(15.0));

    let children = vec![(0, child1_style), (1, child2_style)];
    let placements = layout.arrange(&parent_style, &children, available);

    assert_eq!(placements.len(), 2);
    // First child at x=0, width=20
    assert_eq!(placements[0].region.x, 0);
    assert_eq!(placements[0].region.width, 20);
    // Second child should be at x = 0 + 20 + 10 (margin_right) = 30
    assert_eq!(
        placements[1].region.x, 30,
        "Second child x should account for first child's margin_right, expected 30, got {}",
        placements[1].region.x
    );
}

#[test]
fn test_horizontal_layout_margin_top_offsets_y() {
    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 20);

    // Create a child with margin_top = 3
    let mut child_style = ComputedStyle::default();
    child_style.margin.top = Scalar::cells(3.0);
    child_style.width = Some(Scalar::cells(20.0));

    let children = vec![(0, child_style)];
    let placements = layout.arrange(&parent_style, &children, available);

    assert_eq!(placements.len(), 1);
    // y should be offset by margin_top
    assert_eq!(
        placements[0].region.y, 3,
        "Child y should be offset by margin_top (3), got {}",
        placements[0].region.y
    );
}

#[test]
fn test_horizontal_layout_vertical_margins_reduce_height() {
    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 20);

    // Create a child with margin_top = 3 and margin_bottom = 5
    let mut child_style = ComputedStyle::default();
    child_style.margin.top = Scalar::cells(3.0);
    child_style.margin.bottom = Scalar::cells(5.0);
    child_style.width = Some(Scalar::cells(20.0));
    // Height defaults to available height (20)

    let children = vec![(0, child_style)];
    let placements = layout.arrange(&parent_style, &children, available);

    assert_eq!(placements.len(), 1);
    // Height should be reduced by vertical margins: 20 - 3 - 5 = 12
    assert_eq!(
        placements[0].region.height, 12,
        "Child height should be reduced by vertical margins (20 - 3 - 5 = 12), got {}",
        placements[0].region.height
    );
}

#[test]
fn test_horizontal_layout_all_margins() {
    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 20);

    // Create a child with all margins
    let mut child_style = ComputedStyle::default();
    child_style.margin.left = Scalar::cells(5.0);
    child_style.margin.right = Scalar::cells(10.0);
    child_style.margin.top = Scalar::cells(2.0);
    child_style.margin.bottom = Scalar::cells(3.0);
    child_style.width = Some(Scalar::cells(20.0));

    let children = vec![(0, child_style)];
    let placements = layout.arrange(&parent_style, &children, available);

    assert_eq!(placements.len(), 1);
    let region = &placements[0].region;

    // x = margin_left = 5
    assert_eq!(region.x, 5, "x should be margin_left (5)");
    // y = margin_top = 2
    assert_eq!(region.y, 2, "y should be margin_top (2)");
    // width = 20 (unchanged)
    assert_eq!(region.width, 20, "width should be 20");
    // height = 20 - 2 - 3 = 15
    assert_eq!(
        region.height, 15,
        "height should be reduced by vertical margins (20 - 2 - 3 = 15)"
    );
}

#[test]
fn test_horizontal_layout_multiple_children_with_margins() {
    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 20);

    // Child 1: margin_left=2, margin_right=3, width=10
    let mut child1 = ComputedStyle::default();
    child1.margin.left = Scalar::cells(2.0);
    child1.margin.right = Scalar::cells(3.0);
    child1.width = Some(Scalar::cells(10.0));

    // Child 2: margin_left=4, margin_right=5, width=15
    let mut child2 = ComputedStyle::default();
    child2.margin.left = Scalar::cells(4.0);
    child2.margin.right = Scalar::cells(5.0);
    child2.width = Some(Scalar::cells(15.0));

    let children = vec![(0, child1), (1, child2)];
    let placements = layout.arrange(&parent_style, &children, available);

    assert_eq!(placements.len(), 2);

    // Child 1: x = 0 + margin_left(2) = 2
    assert_eq!(placements[0].region.x, 2);
    assert_eq!(placements[0].region.width, 10);

    // After child 1: current_x = 2 + 10 + margin_right(3) = 15
    // Child 2: x = 15 + margin_left(4) = 19
    assert_eq!(
        placements[1].region.x, 19,
        "Second child x should be 19, got {}",
        placements[1].region.x
    );
    assert_eq!(placements[1].region.width, 15);
}
