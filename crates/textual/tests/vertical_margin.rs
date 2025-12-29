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
