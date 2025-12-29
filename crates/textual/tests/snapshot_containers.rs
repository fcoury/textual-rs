//! Snapshot tests for container widgets.

use textual::testing::TestCanvas;
use textual::widget::static_widget::Static;
use textual::{Center, Horizontal, Middle, Vertical, Widget};

// Message type for tests
#[allow(dead_code)]
enum Msg {}

#[test]
fn vertical_stacks_children() {
    let children: Vec<Box<dyn Widget<Msg>>> = vec![
        Box::new(Static::new("One")),
        Box::new(Static::new("Two")),
        Box::new(Static::new("Three")),
    ];
    let container = Vertical::new(children);
    let mut canvas = TestCanvas::new(10, 5);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn vertical_empty() {
    let children: Vec<Box<dyn Widget<Msg>>> = vec![];
    let container = Vertical::new(children);
    let mut canvas = TestCanvas::new(10, 3);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn vertical_single_child() {
    let children: Vec<Box<dyn Widget<Msg>>> = vec![Box::new(Static::new("Single"))];
    let container = Vertical::new(children);
    let mut canvas = TestCanvas::new(10, 3);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn horizontal_lays_out_children() {
    let children: Vec<Box<dyn Widget<Msg>>> = vec![
        Box::new(Static::new("A")),
        Box::new(Static::new("B")),
        Box::new(Static::new("C")),
    ];
    let container = Horizontal::new(children);
    let mut canvas = TestCanvas::new(10, 1);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn horizontal_empty() {
    let children: Vec<Box<dyn Widget<Msg>>> = vec![];
    let container = Horizontal::new(children);
    let mut canvas = TestCanvas::new(10, 1);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn horizontal_single_child() {
    let children: Vec<Box<dyn Widget<Msg>>> = vec![Box::new(Static::new("Only"))];
    let container = Horizontal::new(children);
    let mut canvas = TestCanvas::new(10, 1);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn center_centers_horizontally() {
    let child: Box<dyn Widget<Msg>> = Box::new(Static::new("Hi"));
    let container = Center::from_child(child);
    let mut canvas = TestCanvas::new(10, 1);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text());
}

#[test]
fn center_long_content() {
    let child: Box<dyn Widget<Msg>> = Box::new(Static::new("LongText"));
    let container = Center::from_child(child);
    let mut canvas = TestCanvas::new(10, 1);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text());
}

#[test]
fn middle_centers_vertically() {
    let child: Box<dyn Widget<Msg>> = Box::new(Static::new("Hi"));
    let container = Middle::from_child(child);
    let mut canvas = TestCanvas::new(5, 5);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn middle_single_line() {
    let child: Box<dyn Widget<Msg>> = Box::new(Static::new("Text"));
    let container = Middle::from_child(child);
    let mut canvas = TestCanvas::new(6, 3);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn nested_vertical_in_horizontal() {
    let vertical: Box<dyn Widget<Msg>> = Box::new(Vertical::new(vec![
        Box::new(Static::new("A")),
        Box::new(Static::new("B")),
    ]));
    let children: Vec<Box<dyn Widget<Msg>>> = vec![
        vertical,
        Box::new(Static::new("C")),
    ];
    let container = Horizontal::new(children);
    let mut canvas = TestCanvas::new(10, 3);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn center_in_middle() {
    let child: Box<dyn Widget<Msg>> = Box::new(Static::new("X"));
    let centered = Center::from_child(child);
    let container = Middle::from_child(Box::new(centered));
    let mut canvas = TestCanvas::new(7, 5);
    let snapshot = canvas.render_widget(&container);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}
