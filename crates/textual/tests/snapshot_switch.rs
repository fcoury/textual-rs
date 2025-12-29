//! Snapshot tests for the Switch widget.

use textual::testing::TestCanvas;
use textual::Switch;

// Message type for tests
#[allow(dead_code)]
enum Msg {
    Toggled(bool),
}

#[test]
fn switch_off_state() {
    let widget = Switch::new(false, |v| Msg::Toggled(v));
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn switch_on_state() {
    let widget = Switch::new(true, |v| Msg::Toggled(v));
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn switch_focused_off() {
    let widget = Switch::new(false, |v| Msg::Toggled(v))
        .with_focus(true);
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn switch_focused_on() {
    let widget = Switch::new(true, |v| Msg::Toggled(v))
        .with_focus(true);
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn switch_loading() {
    let widget = Switch::new(false, |v| Msg::Toggled(v))
        .with_loading(true)
        .with_spinner_frame(0);
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}

#[test]
fn switch_disabled() {
    let widget = Switch::new(false, |v| Msg::Toggled(v))
        .with_disabled(true);
    let mut canvas = TestCanvas::new(15, 1);
    let snapshot = canvas.render_widget(&widget);

    insta::assert_snapshot!(snapshot.to_text_trimmed());
}
