//! Vertical Container Margin Tests
//!
//! Tests for margin handling in the Vertical container,
//! specifically verifying that margin_right reduces child width.

use std::cell::RefCell;
use std::rc::Rc;
use tcss::ComputedStyle;
use tcss::types::Scalar;
use textual::containers::vertical::Vertical;
use textual::widget::Widget;
use textual::{Canvas, Region, Size};

// Dummy message type for tests
enum Msg {}

// =============================================================================
// Test Widget: Captures the region it receives during render
// =============================================================================

struct RegionCapture {
    captured_region: Rc<RefCell<Option<Region>>>,
    style: ComputedStyle,
}

impl RegionCapture {
    fn new(captured: Rc<RefCell<Option<Region>>>) -> Self {
        Self {
            captured_region: captured,
            style: ComputedStyle::default(),
        }
    }
}

impl Widget<Msg> for RegionCapture {
    fn desired_size(&self) -> Size {
        Size {
            width: 10,
            height: 3,
        }
    }

    fn render(&self, _canvas: &mut Canvas, region: Region) {
        *self.captured_region.borrow_mut() = Some(region);
    }

    fn get_style(&self) -> ComputedStyle {
        self.style.clone()
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.style = style;
    }
}

// =============================================================================
// Tests for margin_right in Vertical container
// =============================================================================

#[test]
fn test_margin_right_reduces_child_width() {
    // Create a container region with width 50
    let container_region = Region::new(0, 0, 50, 20);

    // Capture the region that gets passed to the child
    let captured = Rc::new(RefCell::new(None::<Region>));

    // Create a child with margin_right = 10
    let mut child = RegionCapture::new(Rc::clone(&captured));
    let mut style = ComputedStyle::default();
    style.margin.right = Scalar::cells(10.0);
    // Give it an explicit width of 100% so it would normally take full width
    style.width = Some(Scalar::percent(100.0));
    child.set_style(style);

    // Create vertical container with this child
    let container = Vertical::new(vec![Box::new(child) as Box<dyn Widget<Msg>>]);

    // Render
    let mut canvas = Canvas::new(50, 20);
    container.render(&mut canvas, container_region);

    // Check the captured region
    let region = captured.borrow();
    let region = region.as_ref().expect("Child should have been rendered");

    // With margin_right = 10 and container width = 50,
    // the child width should be 50 - 10 = 40
    assert_eq!(
        region.width, 40,
        "Child width should be reduced by margin_right (50 - 10 = 40), but got {}",
        region.width
    );
}

#[test]
fn test_margin_left_and_right_both_reduce_child_width() {
    // Create a container region with width 50
    let container_region = Region::new(0, 0, 50, 20);

    // Capture the region that gets passed to the child
    let captured = Rc::new(RefCell::new(None::<Region>));

    // Create a child with margin_left = 5 and margin_right = 10
    let mut child = RegionCapture::new(Rc::clone(&captured));
    let mut style = ComputedStyle::default();
    style.margin.left = Scalar::cells(5.0);
    style.margin.right = Scalar::cells(10.0);
    // Give it an explicit width of 100% so it would normally take full width
    style.width = Some(Scalar::percent(100.0));
    child.set_style(style);

    // Create vertical container with this child
    let container = Vertical::new(vec![Box::new(child) as Box<dyn Widget<Msg>>]);

    // Render
    let mut canvas = Canvas::new(50, 20);
    container.render(&mut canvas, container_region);

    // Check the captured region
    let region = captured.borrow();
    let region = region.as_ref().expect("Child should have been rendered");

    // The x position should account for margin_left
    assert_eq!(
        region.x, 5,
        "Child x should be offset by margin_left (5), but got {}",
        region.x
    );

    // With margin_left = 5, margin_right = 10, and container width = 50,
    // the child width should be 50 - 5 - 10 = 35
    assert_eq!(
        region.width, 35,
        "Child width should be reduced by both margins (50 - 5 - 10 = 35), but got {}",
        region.width
    );
}

#[test]
fn test_desired_size_accounts_for_margins() {
    // Create a child with margins
    let captured = Rc::new(RefCell::new(None::<Region>));
    let mut child = RegionCapture::new(Rc::clone(&captured));
    let mut style = ComputedStyle::default();
    style.margin.left = Scalar::cells(5.0);
    style.margin.right = Scalar::cells(10.0);
    style.margin.top = Scalar::cells(2.0);
    style.margin.bottom = Scalar::cells(3.0);
    child.set_style(style);

    // Create vertical container
    let container = Vertical::new(vec![Box::new(child) as Box<dyn Widget<Msg>>]);

    // Check desired size includes margins
    let size = container.desired_size();

    // Child desired size is (10, 3)
    // With margins: width = 10 + 5 + 10 = 25, height = 3 + 2 + 3 = 8
    assert_eq!(
        size.width, 25,
        "Desired width should include horizontal margins (10 + 5 + 10 = 25), but got {}",
        size.width
    );
    assert_eq!(
        size.height, 8,
        "Desired height should include vertical margins (3 + 2 + 3 = 8), but got {}",
        size.height
    );
}

// =============================================================================
// Tests for CSS margin collapsing in VerticalLayout
// =============================================================================

use textual::canvas::Size as LayoutSize;
use textual::layouts::{Layout, VerticalLayout};

#[test]
fn test_vertical_margin_collapsing_between_siblings() {
    // CSS margin collapsing: Adjacent vertical margins collapse to max(margin_bottom, margin_top)
    // Child 1: margin_bottom = 3
    // Child 2: margin_top = 5
    // Expected gap = max(3, 5) = 5 (NOT 3 + 5 = 8)

    let mut layout = VerticalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 100);

    // Child 1 with margin_bottom = 3, height = 10
    let mut child1_style = ComputedStyle::default();
    child1_style.height = Some(Scalar::cells(10.0));
    child1_style.margin.bottom = Scalar::cells(3.0);

    // Child 2 with margin_top = 5, height = 10
    let mut child2_style = ComputedStyle::default();
    child2_style.height = Some(Scalar::cells(10.0));
    child2_style.margin.top = Scalar::cells(5.0);

    let children = vec![(0, child1_style, LayoutSize::new(10, 3)), (1, child2_style, LayoutSize::new(10, 3))];
    let placements = layout.arrange(&parent_style, &children, available);

    assert_eq!(placements.len(), 2);

    // Child 1 at y=0, height=10
    assert_eq!(placements[0].region.y, 0, "First child should start at y=0");
    assert_eq!(placements[0].region.height, 10, "First child height should be 10");

    // Child 2 should start at y = 0 + 10 + max(3, 5) = 15
    // NOT y = 0 + 10 + 3 + 5 = 18 (additive)
    assert_eq!(
        placements[1].region.y, 15,
        "Second child y should be 15 (margin collapsing: max(3,5)=5), but got {}. Gap is {} instead of expected 5.",
        placements[1].region.y,
        placements[1].region.y - 10
    );
}

#[test]
fn test_vertical_margin_collapsing_equal_margins() {
    // When margins are equal, collapsed margin = max(m, m) = m
    // Child 1: margin_bottom = 4
    // Child 2: margin_top = 4
    // Expected gap = max(4, 4) = 4 (NOT 4 + 4 = 8)

    let mut layout = VerticalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 100);

    let mut child1_style = ComputedStyle::default();
    child1_style.height = Some(Scalar::cells(10.0));
    child1_style.margin.bottom = Scalar::cells(4.0);

    let mut child2_style = ComputedStyle::default();
    child2_style.height = Some(Scalar::cells(10.0));
    child2_style.margin.top = Scalar::cells(4.0);

    let children = vec![(0, child1_style, LayoutSize::new(10, 3)), (1, child2_style, LayoutSize::new(10, 3))];
    let placements = layout.arrange(&parent_style, &children, available);

    // Child 2 at y = 0 + 10 + max(4,4) = 14
    assert_eq!(
        placements[1].region.y, 14,
        "Second child y should be 14 (collapsed margin=4), but got {}. Gap is {} instead of expected 4.",
        placements[1].region.y,
        placements[1].region.y - 10
    );
}

#[test]
fn test_vertical_margin_collapsing_first_child_keeps_top_margin() {
    // First child's top margin should NOT be collapsed (no previous sibling)

    let mut layout = VerticalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 100);

    let mut child_style = ComputedStyle::default();
    child_style.height = Some(Scalar::cells(10.0));
    child_style.margin.top = Scalar::cells(5.0);

    let children = vec![(0, child_style, LayoutSize::new(10, 3))];
    let placements = layout.arrange(&parent_style, &children, available);

    // First child should start at y = 0 + 5 = 5
    assert_eq!(
        placements[0].region.y, 5,
        "First child should preserve its full top margin, got y={}",
        placements[0].region.y
    );
}

#[test]
fn test_vertical_margin_collapsing_three_children() {
    // Three children: test collapsing between 1-2 and 2-3
    // Child 1: margin_bottom = 2
    // Child 2: margin_top = 6, margin_bottom = 4
    // Child 3: margin_top = 3

    let mut layout = VerticalLayout;
    let parent_style = ComputedStyle::default();
    let available = Region::new(0, 0, 100, 100);

    let mut child1_style = ComputedStyle::default();
    child1_style.height = Some(Scalar::cells(10.0));
    child1_style.margin.bottom = Scalar::cells(2.0);

    let mut child2_style = ComputedStyle::default();
    child2_style.height = Some(Scalar::cells(10.0));
    child2_style.margin.top = Scalar::cells(6.0);
    child2_style.margin.bottom = Scalar::cells(4.0);

    let mut child3_style = ComputedStyle::default();
    child3_style.height = Some(Scalar::cells(10.0));
    child3_style.margin.top = Scalar::cells(3.0);

    let children = vec![(0, child1_style, LayoutSize::new(10, 3)), (1, child2_style, LayoutSize::new(10, 3)), (2, child3_style, LayoutSize::new(10, 3))];
    let placements = layout.arrange(&parent_style, &children, available);

    assert_eq!(placements.len(), 3);

    // Child 1: y=0, height=10
    assert_eq!(placements[0].region.y, 0);

    // Child 2: y = 0 + 10 + max(2, 6) = 16
    assert_eq!(
        placements[1].region.y, 16,
        "Child 2 y should be 16 (gap=max(2,6)=6), got {}",
        placements[1].region.y
    );

    // Child 3: y = 16 + 10 + max(4, 3) = 30
    assert_eq!(
        placements[2].region.y, 30,
        "Child 3 y should be 30 (gap=max(4,3)=4), got {}",
        placements[2].region.y
    );
}
