//! Snapshot tests for the Label widget.

use textual::testing::TestCanvas;
use textual::{Label, LabelVariant};

// Message type for tests
enum Msg {}

#[test]
fn label_renders_simple_text() {
    let widget: Label<Msg> = Label::new("Hello, World!");
    let mut canvas = TestCanvas::new(20, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn label_renders_multiline() {
    let widget: Label<Msg> = Label::new("Line 1\nLine 2");
    let mut canvas = TestCanvas::new(10, 3);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn label_with_success_variant() {
    let widget: Label<Msg> = Label::new("Success!")
        .with_variant(LabelVariant::Success);
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    // Variant adds class but doesn't change text rendering without CSS
    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn label_with_error_variant() {
    let widget: Label<Msg> = Label::new("Error!")
        .with_variant(LabelVariant::Error);
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn label_with_warning_variant() {
    let widget: Label<Msg> = Label::new("Warning!")
        .with_variant(LabelVariant::Warning);
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn label_with_id() {
    let widget: Label<Msg> = Label::new("Identified")
        .with_id("my-label");
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn label_with_classes() {
    let widget: Label<Msg> = Label::new("Classed")
        .with_classes("highlight bold");
    let mut canvas = TestCanvas::new(10, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn label_disabled() {
    let widget: Label<Msg> = Label::new("Disabled")
        .with_disabled(true);
    let mut canvas = TestCanvas::new(10, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}
