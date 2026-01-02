//! Horizontal Layout Margin Tests
//!
//! Tests for margin handling in the HorizontalLayout,
//! verifying consistency with VerticalLayout.

use tcss::types::{ComputedStyle, Scalar};
use textual::canvas::{Region, Size};
use textual::layouts::{HorizontalLayout, Layout, LayoutChild, LayoutNode};

struct DummyNode;

impl LayoutNode for DummyNode {
    fn desired_size(&self) -> Size {
        Size::new(0, 0)
    }

    fn intrinsic_height_for_width(&self, _width: u16) -> u16 {
        0
    }
}

fn make_child<'a>(
    index: usize,
    style: ComputedStyle,
    size: Size,
    node: &'a DummyNode,
) -> LayoutChild<'a> {
    LayoutChild {
        index,
        style,
        desired_size: size,
        node,
    }
}

// =============================================================================
// Tests for margin handling in HorizontalLayout
// =============================================================================

#[test]
fn test_horizontal_layout_margin_left_offsets_x() {
    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 20);
    let dummy = DummyNode;

    // Create a child with margin_left = 5
    let mut child_style = ComputedStyle::default();
    child_style.margin.left = Scalar::cells(5.0);
    child_style.width = Some(Scalar::cells(20.0));

    let children = vec![make_child(0, child_style, Size::new(10, 3), &dummy)];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

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
    let dummy = DummyNode;

    // Create two children, first with margin_right = 10
    let mut child1_style = ComputedStyle::default();
    child1_style.margin.right = Scalar::cells(10.0);
    child1_style.width = Some(Scalar::cells(20.0));

    let mut child2_style = ComputedStyle::default();
    child2_style.width = Some(Scalar::cells(15.0));

    let children = vec![
        make_child(0, child1_style, Size::new(10, 3), &dummy),
        make_child(1, child2_style, Size::new(10, 3), &dummy),
    ];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

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
    let dummy = DummyNode;

    // Create a child with margin_top = 3
    let mut child_style = ComputedStyle::default();
    child_style.margin.top = Scalar::cells(3.0);
    child_style.width = Some(Scalar::cells(20.0));

    let children = vec![make_child(0, child_style, Size::new(10, 3), &dummy)];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

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
    let dummy = DummyNode;

    // Create a child with margin_top = 3 and margin_bottom = 5
    let mut child_style = ComputedStyle::default();
    child_style.margin.top = Scalar::cells(3.0);
    child_style.margin.bottom = Scalar::cells(5.0);
    child_style.width = Some(Scalar::cells(20.0));
    // Height defaults to available height (20)

    let children = vec![make_child(0, child_style, Size::new(10, 3), &dummy)];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

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
    let dummy = DummyNode;

    // Create a child with all margins
    let mut child_style = ComputedStyle::default();
    child_style.margin.left = Scalar::cells(5.0);
    child_style.margin.right = Scalar::cells(10.0);
    child_style.margin.top = Scalar::cells(2.0);
    child_style.margin.bottom = Scalar::cells(3.0);
    child_style.width = Some(Scalar::cells(20.0));

    let children = vec![make_child(0, child_style, Size::new(10, 3), &dummy)];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

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
    let dummy = DummyNode;

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

    let children = vec![
        make_child(0, child1, Size::new(10, 3), &dummy),
        make_child(1, child2, Size::new(10, 3), &dummy),
    ];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

    assert_eq!(placements.len(), 2);

    // Child 1: x = 0 + margin_left(2) = 2
    assert_eq!(placements[0].region.x, 2);
    assert_eq!(placements[0].region.width, 10);

    // After child 1: current_x = 2 + 10 + margin_right(3) = 15
    // With margin collapsing: gap = max(3, 4) = 4
    // Child 2: x = 2 + 10 + max(3, 4) = 16
    assert_eq!(
        placements[1].region.x, 16,
        "Second child x should be 16 (margin collapsing: max(3,4)=4), got {}",
        placements[1].region.x
    );
    assert_eq!(placements[1].region.width, 15);
}

// =============================================================================
// Tests for CSS margin collapsing in HorizontalLayout
// =============================================================================

#[test]
fn test_horizontal_margin_collapsing_between_siblings() {
    // CSS margin collapsing: Adjacent horizontal margins collapse to max(margin_right, margin_left)
    // Child 1: margin_right = 3
    // Child 2: margin_left = 5
    // Expected gap = max(3, 5) = 5 (NOT 3 + 5 = 8)

    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 100);
    let dummy = DummyNode;

    // Child 1 with margin_right = 3, width = 10
    let mut child1_style = ComputedStyle::default();
    child1_style.width = Some(Scalar::cells(10.0));
    child1_style.margin.right = Scalar::cells(3.0);

    // Child 2 with margin_left = 5, width = 10
    let mut child2_style = ComputedStyle::default();
    child2_style.width = Some(Scalar::cells(10.0));
    child2_style.margin.left = Scalar::cells(5.0);

    let children = vec![
        make_child(0, child1_style, Size::new(10, 3), &dummy),
        make_child(1, child2_style, Size::new(10, 3), &dummy),
    ];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

    assert_eq!(placements.len(), 2);

    // Child 1 at x=0, width=10
    assert_eq!(placements[0].region.x, 0, "First child should start at x=0");
    assert_eq!(
        placements[0].region.width, 10,
        "First child width should be 10"
    );

    // Child 2 should start at x = 0 + 10 + max(3, 5) = 15
    // NOT x = 0 + 10 + 3 + 5 = 18 (additive)
    assert_eq!(
        placements[1].region.x,
        15,
        "Second child x should be 15 (margin collapsing: max(3,5)=5), but got {}. Gap is {} instead of expected 5.",
        placements[1].region.x,
        placements[1].region.x - 10
    );
}

#[test]
fn test_horizontal_margin_collapsing_equal_margins() {
    // When margins are equal, collapsed margin = max(m, m) = m
    // Child 1: margin_right = 4
    // Child 2: margin_left = 4
    // Expected gap = max(4, 4) = 4 (NOT 4 + 4 = 8)

    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 100);
    let dummy = DummyNode;

    let mut child1_style = ComputedStyle::default();
    child1_style.width = Some(Scalar::cells(10.0));
    child1_style.margin.right = Scalar::cells(4.0);

    let mut child2_style = ComputedStyle::default();
    child2_style.width = Some(Scalar::cells(10.0));
    child2_style.margin.left = Scalar::cells(4.0);

    let children = vec![
        make_child(0, child1_style, Size::new(10, 3), &dummy),
        make_child(1, child2_style, Size::new(10, 3), &dummy),
    ];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

    // Child 2 at x = 0 + 10 + max(4,4) = 14
    assert_eq!(
        placements[1].region.x,
        14,
        "Second child x should be 14 (collapsed margin=4), but got {}. Gap is {} instead of expected 4.",
        placements[1].region.x,
        placements[1].region.x - 10
    );
}

#[test]
fn test_horizontal_margin_collapsing_first_child_keeps_left_margin() {
    // First child's left margin should NOT be collapsed (no previous sibling)

    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 100);
    let dummy = DummyNode;

    let mut child_style = ComputedStyle::default();
    child_style.width = Some(Scalar::cells(10.0));
    child_style.margin.left = Scalar::cells(5.0);

    let children = vec![make_child(0, child_style, Size::new(10, 3), &dummy)];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

    // First child should start at x = 0 + 5 = 5
    assert_eq!(
        placements[0].region.x, 5,
        "First child should preserve its full left margin, got x={}",
        placements[0].region.x
    );
}

#[test]
fn test_horizontal_margin_collapsing_three_children() {
    // Three children: test collapsing between 1-2 and 2-3
    // Child 1: margin_right = 2
    // Child 2: margin_left = 6, margin_right = 4
    // Child 3: margin_left = 3

    let mut layout = HorizontalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 100);
    let dummy = DummyNode;

    let mut child1_style = ComputedStyle::default();
    child1_style.width = Some(Scalar::cells(10.0));
    child1_style.margin.right = Scalar::cells(2.0);

    let mut child2_style = ComputedStyle::default();
    child2_style.width = Some(Scalar::cells(10.0));
    child2_style.margin.left = Scalar::cells(6.0);
    child2_style.margin.right = Scalar::cells(4.0);

    let mut child3_style = ComputedStyle::default();
    child3_style.width = Some(Scalar::cells(10.0));
    child3_style.margin.left = Scalar::cells(3.0);

    let children = vec![
        make_child(0, child1_style, Size::new(10, 3), &dummy),
        make_child(1, child2_style, Size::new(10, 3), &dummy),
        make_child(2, child3_style, Size::new(10, 3), &dummy),
    ];
    let placements = layout.arrange(&parent_style, &children, available, available.into());

    assert_eq!(placements.len(), 3);

    // Child 1: x=0, width=10
    assert_eq!(placements[0].region.x, 0);

    // Child 2: x = 0 + 10 + max(2, 6) = 16
    assert_eq!(
        placements[1].region.x, 16,
        "Child 2 x should be 16 (gap=max(2,6)=6), got {}",
        placements[1].region.x
    );

    // Child 3: x = 16 + 10 + max(4, 3) = 30
    assert_eq!(
        placements[2].region.x, 30,
        "Child 3 x should be 30 (gap=max(4,3)=4), got {}",
        placements[2].region.x
    );
}
