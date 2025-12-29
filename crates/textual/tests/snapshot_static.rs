//! Snapshot tests for the Static widget.

use textual::testing::TestCanvas;
use textual::widget::static_widget::Static;

// Message type for tests
enum Msg {}

#[test]
fn static_renders_simple_text() {
    let widget: Static<Msg> = Static::new("Hello, World!");
    let mut canvas = TestCanvas::new(20, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn static_renders_multiline() {
    let widget: Static<Msg> = Static::new("Line 1\nLine 2\nLine 3");
    let mut canvas = TestCanvas::new(10, 5);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn static_truncates_long_text() {
    let widget: Static<Msg> = Static::new("This is a very long line that should be truncated");
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text());
}

#[test]
fn static_empty_content() {
    let widget: Static<Msg> = Static::new("");
    let mut canvas = TestCanvas::new(10, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text());
}

#[test]
fn static_single_char() {
    let widget: Static<Msg> = Static::new("X");
    let mut canvas = TestCanvas::new(5, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn static_special_chars() {
    let widget: Static<Msg> = Static::new("@#$%^&*()");
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn static_unicode_text() {
    let widget: Static<Msg> = Static::new("Hello 世界!");
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn static_with_tabs() {
    let widget: Static<Msg> = Static::new("Col1\tCol2\tCol3");
    let mut canvas = TestCanvas::new(20, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn static_with_id() {
    let widget: Static<Msg> = Static::new("Identified")
        .with_id("my-static");
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    // ID doesn't affect rendering directly
    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn static_with_classes() {
    let widget: Static<Msg> = Static::new("Classed")
        .with_classes("highlight bold");
    let mut canvas = TestCanvas::new(10, 1);
    let snapshot = canvas.render_widget(&widget);

    // Classes don't affect rendering without CSS
    insta::assert_snapshot!(snapshot.to_text_trimmed());
}
