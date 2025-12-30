//! Snapshot tests for all example apps.
//!
//! These tests render each example app to a Canvas at 80x24 and compare
//! against stored snapshots. Run `cargo insta test --accept` to update snapshots.

use insta::assert_snapshot;
use textual::testing::render_to_canvas;
use textual::{Compose, Label, Widget, ui};

// ============================================================================
// Border Example
// ============================================================================

mod border_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BorderApp;

    impl Compose for BorderApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("My border is solid red", id: "label1")
                Label("My border is dashed green", id: "label2")
                Label("My border is tall blue", id: "label3")
            }
        }
    }

    pub const CSS: &str = r#"
#label1 {
    background: red 20%;
    color: red;
    border: solid red;
}

#label2 {
    background: green 20%;
    color: green;
    border: dashed green;
}

#label3 {
    background: blue 20%;
    color: blue;
    border: tall blue;
}

Screen {
    background: white;
}

Screen > Label {
    width: 100%;
    height: 5;
    content-align: center middle;
    color: white;
    margin: 1;
    box-sizing: border-box;
}
"#;
}

#[test]
fn snapshot_border_example() {
    let app = border_example::BorderApp;
    let canvas = render_to_canvas(&app, border_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Align Example
// ============================================================================

mod align_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct AlignApp;

    impl Compose for AlignApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("Vertical alignment with [b]Textual[/]", classes: "box")
                Label("Take note, browsers.", classes: "box")
            }
        }
    }

    pub const CSS: &str = r#"
Screen {
    align: center middle;
}

.box {
    width: 40;
    height: 5;
    margin: 1;
    padding: 1;
    background: green;
    color: white 90%;
    border: heavy white;
}
"#;
}

#[test]
fn snapshot_align_example() {
    let app = align_example::AlignApp;
    let canvas = render_to_canvas(&app, align_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Align All Example
// ============================================================================

mod align_all_example {
    use super::*;
    use textual::Container;

    #[derive(Clone)]
    pub enum Message {}

    pub struct AlignAllApp;

    impl Compose for AlignAllApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Container(id: "left-top")      { Label("left top") }
                Container(id: "center-top")    { Label("center top") }
                Container(id: "right-top")     { Label("right top") }
                Container(id: "left-middle")   { Label("left middle") }
                Container(id: "center-middle") { Label("center middle") }
                Container(id: "right-middle")  { Label("right middle") }
                Container(id: "left-bottom")   { Label("left bottom") }
                Container(id: "center-bottom") { Label("center bottom") }
                Container(id: "right-bottom")  { Label("right bottom") }
            }
        }
    }

    pub const CSS: &str = r#"
#left-top {
}

#center-top {
    align: center top;
}

#right-top {
    align: right top;
}

#left-middle {
    align: left middle;
}

#center-middle {
    align: center middle;
}

#right-middle {
    align: right middle;
}

#left-bottom {
    align: left bottom;
}

#center-bottom {
    align: center bottom;
}

#right-bottom {
    align: right bottom;
}

Screen {
    layout: grid;
    grid-size: 3 3;
    grid-gutter: 1;
}

Container {
    background: $boost;
    border: solid gray;
    height: 100%;
}

Label {
    width: auto;
    height: 1;
    background: $accent;
}
"#;
}

#[test]
fn snapshot_align_all_example() {
    let app = align_all_example::AlignAllApp;
    let canvas = render_to_canvas(&app, align_all_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Background Example
// ============================================================================

mod background_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BackgroundApp;

    impl Compose for BackgroundApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("darkred", id: "label1")
                Label("orange", id: "label2")
                Label("green", id: "label3")
            }
        }
    }

    pub const CSS: &str = r#"
Screen {
    background: white;
}

Screen > Label {
    width: 100%;
    height: 1fr;
    content-align: center middle;
    color: white;
}

#label1 { background: darkred; }
#label2 { background: orange; }
#label3 { background: green; }
"#;
}

#[test]
fn snapshot_background_example() {
    let app = background_example::BackgroundApp;
    let canvas = render_to_canvas(&app, background_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Grid Example
// ============================================================================

mod grid_example {
    use super::*;
    use textual::containers::grid::Grid;

    #[derive(Clone)]
    pub enum Message {}

    pub struct GridApp;

    impl Compose for GridApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            let labels: Vec<Box<dyn Widget<Message>>> = (1..=12)
                .map(|i| Box::new(Label::new(format!("Item {}", i)).with_classes("cell")) as Box<dyn Widget<Message>>)
                .collect();
            vec![Box::new(Grid::new(labels))]
        }
    }

    pub const CSS: &str = r#"
Grid {
    grid-size: 4 3;
}

.cell {
    width: 100%;
    height: 100%;
    background: green;
    color: white;
    content-align: center middle;
    border: solid white;
}
"#;
}

#[test]
fn snapshot_grid_example() {
    let app = grid_example::GridApp;
    let canvas = render_to_canvas(&app, grid_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Border All Example
// ============================================================================

mod border_all_example {
    use super::*;
    use textual::Grid;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BorderAllApp;

    impl Compose for BorderAllApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Grid {
                    Label("ascii", id: "ascii")
                    Label("blank", id: "blank")
                    Label("dashed", id: "dashed")
                    Label("double", id: "double")
                    Label("heavy", id: "heavy")
                    Label("hidden/none", id: "hidden")
                    Label("hkey", id: "hkey")
                    Label("inner", id: "inner")
                    Label("outer", id: "outer")
                    Label("panel", id: "panel")
                    Label("round", id: "round")
                    Label("solid", id: "solid")
                    Label("tall", id: "tall")
                    Label("thick", id: "thick")
                    Label("vkey", id: "vkey")
                    Label("wide", id: "wide")
                }
            }
        }
    }

    pub const CSS: &str = r#"
#ascii {
    border: ascii $accent;
}

#blank {
    border: blank $accent;
}

#dashed {
    border: dashed $accent;
}

#double {
    border: double $accent;
}

#heavy {
    border: heavy $accent;
}

#hidden {
    border: hidden $accent;
}

#hkey {
    border: hkey $accent;
}

#inner {
    border: inner $accent;
}

#outer {
    border: outer $accent;
}

#panel {
    border: panel $accent;
}

#round {
    border: round $accent;
}

#solid {
    border: solid $accent;
}

#tall {
    border: tall $accent;
}

#thick {
    border: thick $accent;
}

#vkey {
    border: vkey $accent;
}

#wide {
    border: wide $accent;
}

Grid {
    grid-size: 4 4;
    align: center middle;
    grid-gutter: 1 2;
}

Label {
    width: 20;
    height: 3;
    content-align: center middle;
}
"#;
}

#[test]
fn snapshot_border_all_example() {
    let app = border_all_example::BorderAllApp;
    let canvas = render_to_canvas(&app, border_all_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Border Sub Title Align All Example
// ============================================================================

mod border_sub_title_align_all_example {
    use super::*;
    use textual::Container;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BorderSubTitleAlignAllApp;

    fn make_label_container<M: 'static>(
        text: &str,
        id: &str,
        border_title: &str,
        border_subtitle: &str,
    ) -> Box<Container<M>> {
        let label = Label::new(text)
            .with_id(id)
            .with_border_title(border_title)
            .with_border_subtitle(border_subtitle);

        Box::new(Container::new(vec![Box::new(label)]))
    }

    impl Compose for BorderSubTitleAlignAllApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            use textual::Grid;
            vec![Box::new(Grid::new(vec![
                make_label_container(
                    "This is the story of",
                    "lbl1",
                    "[b]Border [i]title[/i][/]",
                    "[u][r]Border[/r] subtitle[/]",
                ),
                make_label_container(
                    "a Python",
                    "lbl2",
                    "[b red]Left, but it's loooooooooooong",
                    "[reverse]Center, but it's loooooooooooong",
                ),
                make_label_container(
                    "developer that",
                    "lbl3",
                    "[b i on purple]Left[/]",
                    "[r u white on black]@@@[/]",
                ),
                make_label_container(
                    "had to fill up",
                    "lbl4",
                    "",
                    "[link='https://textual.textualize.io']Left[/]",
                ),
                make_label_container(
                    "nine labels",
                    "lbl5",
                    "Title",
                    "Subtitle",
                ),
                make_label_container(
                    "and ended up redoing it",
                    "lbl6",
                    "Title",
                    "Subtitle",
                ),
                make_label_container(
                    "because the first try",
                    "lbl7",
                    "Title, but really loooooooooong!",
                    "Subtitle, but really loooooooooong!",
                ),
                make_label_container(
                    "had some labels",
                    "lbl8",
                    "Title, but really loooooooooong!",
                    "Subtitle, but really loooooooooong!",
                ),
                make_label_container(
                    "that were too long.",
                    "lbl9",
                    "Title, but really loooooooooong!",
                    "Subtitle, but really loooooooooong!",
                ),
            ]))]
        }
    }

    pub const CSS: &str = r#"
Grid {
    grid-size: 3 3;
    align: center middle;
}

Container {
    width: 100%;
    height: 100%;
    align: center middle;
}

#lbl1 {
    border: vkey $secondary;
}

#lbl2 {
    border: round $secondary;
    border-title-align: right;
    border-subtitle-align: right;
}

#lbl3 {
    border: wide $secondary;
    border-title-align: center;
    border-subtitle-align: center;
}

#lbl4 {
    border: ascii $success;
    border-title-align: center;
    border-subtitle-align: left;
}

#lbl5 {
    border: none $success;
    border-title-align: center;
    border-subtitle-align: center;
}

#lbl6 {
    border-top: solid $success;
    border-bottom: solid $success;
}

#lbl7 {
    border-top: solid $error;
    border-bottom: solid $error;
    padding: 1 2;
    border-subtitle-align: left;
}

#lbl8 {
    border-top: solid $error;
    border-bottom: solid $error;
    border-title-align: center;
    border-subtitle-align: center;
}

#lbl9 {
    border-top: solid $error;
    border-bottom: solid $error;
    border-title-align: right;
}
"#;
}

#[test]
fn snapshot_border_sub_title_align_all_example() {
    let app = border_sub_title_align_all_example::BorderSubTitleAlignAllApp;
    let canvas = render_to_canvas(&app, border_sub_title_align_all_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Border Subtitle Align Example
// ============================================================================

mod border_subtitle_align_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BorderSubtitleAlignApp;

    impl Compose for BorderSubtitleAlignApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("My subtitle is on the left.", id: "label1", border_subtitle: "< Left")
                Label("My subtitle is centered", id: "label2", border_subtitle: "Centered!")
                Label("My subtitle is on the right", id: "label3", border_subtitle: "Right >")
            }
        }
    }

    pub const CSS: &str = r#"
#label1 {
    border: solid $secondary;
    border-subtitle-align: left;
}

#label2 {
    border: dashed $secondary;
    border-subtitle-align: center;
}

#label3 {
    border: tall $secondary;
    border-subtitle-align: right;
}

Screen > Label {
    width: 100%;
    height: 5;
    content-align: center middle;
    color: white;
    margin: 1;
    box-sizing: border-box;
}
"#;
}

#[test]
fn snapshot_border_subtitle_align_example() {
    let app = border_subtitle_align_example::BorderSubtitleAlignApp;
    let canvas = render_to_canvas(&app, border_subtitle_align_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Border Title Align Example
// ============================================================================

mod border_title_align_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BorderTitleAlignApp;

    impl Compose for BorderTitleAlignApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("My title is on the left.", id: "label1", border_title: "< Left")
                Label("My title is centered", id: "label2", border_title: "Centered!")
                Label("My title is on the right", id: "label3", border_title: "Right >")
            }
        }
    }

    pub const CSS: &str = r#"
#label1 {
    border: solid $secondary;
    border-title-align: left;
}

#label2 {
    border: dashed $secondary;
    border-title-align: center;
}

#label3 {
    border: tall $secondary;
    border-title-align: right;
}

Screen > Label {
    width: 100%;
    height: 5;
    content-align: center middle;
    color: white;
    margin: 1;
    box-sizing: border-box;
}
"#;
}

#[test]
fn snapshot_border_title_align_example() {
    let app = border_title_align_example::BorderTitleAlignApp;
    let canvas = render_to_canvas(&app, border_title_align_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Border Title Colors Example
// ============================================================================

mod border_title_colors_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BorderTitleColorsApp;

    impl Compose for BorderTitleColorsApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("Hello, World!", id: "label", border_title: "Textual Rocks", border_subtitle: "Textual Rocks")
            }
        }
    }

    pub const CSS: &str = r#"
Screen {
    align: center middle;
}

Label {
    padding: 4 8;
    border: heavy red;

    border-title-color: green;
    border-title-background: white;
    border-title-style: bold;

    border-subtitle-color: magenta;
    border-subtitle-background: yellow;
    border-subtitle-style: italic;
}
"#;
}

#[test]
fn snapshot_border_title_colors_example() {
    let app = border_title_colors_example::BorderTitleColorsApp;
    let canvas = render_to_canvas(&app, border_title_colors_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Display Example
// ============================================================================

mod display_example {
    use super::*;
    use textual::Static;

    #[derive(Clone)]
    pub enum Message {}

    pub struct DisplayApp;

    impl Compose for DisplayApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Static("Widget 1")
                Static("Widget 2", classes: "remove")
                Static("Widget 3")
            }
        }
    }

    pub const CSS: &str = r#"
Screen {
    background: green;
}

Static {
    height: 5;
    background: white;
    color: blue;
    border: heavy blue;
}

Static.remove {
    display: none;
}
"#;
}

#[test]
fn snapshot_display_example() {
    let app = display_example::DisplayApp;
    let canvas = render_to_canvas(&app, display_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Box Sizing Example
// ============================================================================

mod box_sizing_example {
    use super::*;
    use textual::Static;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BoxSizingApp;

    impl Compose for BoxSizingApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Static("I'm using border-box!", id: "static1")
                Static("I'm using content-box!", id: "static2")
            }
        }
    }

    pub const CSS: &str = r#"
#static1 {
    box-sizing: border-box;
}

#static2 {
    box-sizing: content-box;
}

Screen {
    background: white;
    color: black;
}

Static {
    background: blue 20%;
    height: 5;
    margin: 2;
    padding: 1;
    border: wide black;
}
"#;
}

#[test]
fn snapshot_box_sizing_example() {
    let app = box_sizing_example::BoxSizingApp;
    let canvas = render_to_canvas(&app, box_sizing_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Content Align Example
// ============================================================================

mod content_align_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct ContentAlignApp;

    impl Compose for ContentAlignApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("With [i]content-align[/] you can...", id: "box1")
                Label("...[b]Easily align content[/]...", id: "box2")
                Label("...Horizontally [i]and[/] vertically!", id: "box3")
            }
        }
    }

    pub const CSS: &str = r#"
#box1 {
    content-align: left top;
    background: red;
}

#box2 {
    content-align-horizontal: center;
    content-align-vertical: middle;
    background: green;
}

#box3 {
    content-align: right bottom;
    background: blue;
}

Label {
    width: 100%;
    height: 1fr;
    padding: 1;
    color: white;
}
"#;
}

#[test]
fn snapshot_content_align_example() {
    let app = content_align_example::ContentAlignApp;
    let canvas = render_to_canvas(&app, content_align_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Content Align All Example
// ============================================================================

mod content_align_all_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct ContentAlignAllApp;

    impl Compose for ContentAlignAllApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("left top", id: "left-top")
                Label("center top", id: "center-top")
                Label("right top", id: "right-top")
                Label("left middle", id: "left-middle")
                Label("center middle", id: "center-middle")
                Label("right middle", id: "right-middle")
                Label("left bottom", id: "left-bottom")
                Label("center bottom", id: "center-bottom")
                Label("right bottom", id: "right-bottom")
            }
        }
    }

    pub const CSS: &str = r#"
#left-top {
    /* content-align: left top; this is the default implied value. */
}
#center-top {
    content-align: center top;
}
#right-top {
    content-align: right top;
}
#left-middle {
    content-align: left middle;
}
#center-middle {
    content-align: center middle;
}
#right-middle {
    content-align: right middle;
}
#left-bottom {
    content-align: left bottom;
}
#center-bottom {
    content-align: center bottom;
}
#right-bottom {
    content-align: right bottom;
}

Screen {
    layout: grid;
    grid-size: 3 3;
    grid-gutter: 1;
}

Label {
    width: 100%;
    height: 100%;
    background: $primary;
}
"#;
}

#[test]
fn snapshot_content_align_all_example() {
    let app = content_align_all_example::ContentAlignAllApp;
    let canvas = render_to_canvas(&app, content_align_all_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Column Span Example
// ============================================================================

mod column_span_example {
    use super::*;
    use textual::{Grid, Placeholder};

    #[derive(Clone)]
    pub enum Message {}

    pub struct ColumnSpanApp;

    impl Compose for ColumnSpanApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Grid {
                    Placeholder(id: "p1")
                    Placeholder(id: "p2")
                    Placeholder(id: "p3")
                    Placeholder(id: "p4")
                    Placeholder(id: "p5")
                    Placeholder(id: "p6")
                    Placeholder(id: "p7")
                }
            }
        }
    }

    pub const CSS: &str = r#"
#p1 {
    column-span: 4;
}
#p2 {
    column-span: 3;
}
#p3 {
    column-span: 1;
}
#p4 {
    column-span: 2;
}
#p5 {
    column-span: 2;
}
#p6 {
    /* Default value is 1. */
}
#p7 {
    column-span: 3;
}

Grid {
    grid-size: 4 4;
    grid-gutter: 1 2;
}

Placeholder {
    height: 100%;
}
"#;
}

#[test]
fn snapshot_column_span_example() {
    let app = column_span_example::ColumnSpanApp;
    let canvas = render_to_canvas(&app, column_span_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Dock All Example
// ============================================================================

mod dock_all_example {
    use super::*;
    use textual::Container;

    #[derive(Clone)]
    pub enum Message {}

    pub struct DockAllApp;

    impl Compose for DockAllApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Container(id: "big_container") {
                    Container(id: "left") { Label("left") }
                    Container(id: "top") { Label("top") }
                    Container(id: "right") { Label("right") }
                    Container(id: "bottom") { Label("bottom") }
                }
            }
        }
    }

    pub const CSS: &str = r#"
#left {
    dock: left;
    height: 100%;
    width: auto;
    align-vertical: middle;
}
#top {
    dock: top;
    height: auto;
    width: 100%;
    align-horizontal: center;
}
#right {
    dock: right;
    height: 100%;
    width: auto;
    align-vertical: middle;
}
#bottom {
    dock: bottom;
    height: auto;
    width: 100%;
    align-horizontal: center;
}

Screen {
    align: center middle;
}

#big_container {
    width: 75%;
    height: 75%;
    border: round white;
}
"#;
}

#[test]
fn snapshot_dock_all_example() {
    let app = dock_all_example::DockAllApp;
    let canvas = render_to_canvas(&app, dock_all_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Hatch Example
// ============================================================================

mod hatch_example {
    use super::*;
    use textual::{Horizontal, Static, Vertical, widget};

    const HATCHES: &[&str] = &["cross", "horizontal", "custom", "left", "right"];

    #[derive(Clone)]
    pub enum Message {}

    pub struct HatchApp;

    impl Compose for HatchApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            let hatch_widgets: Vec<Box<dyn Widget<Message>>> = HATCHES
                .iter()
                .map(|&hatch| {
                    widget! {
                        Vertical {
                            Static("", classes: format!("hatch {}", hatch), border_title: hatch)
                        }
                    }
                })
                .collect();

            vec![Box::new(Horizontal::new(hatch_widgets))]
        }
    }

    pub const CSS: &str = r#"
.hatch {
    width: 1fr;
    height: 1fr;
    border: solid $secondary;

    &.cross {
        hatch: cross $success;
    }
    &.horizontal {
        hatch: horizontal $success 80%;
    }
    &.custom {
        hatch: "T" $success 60%;
    }
    &.left {
        hatch: left $success 40%;
    }
    &.right {
        hatch: right $success 20%;
    }
}
"#;
}

#[test]
fn snapshot_hatch_example() {
    let app = hatch_example::HatchApp;
    let canvas = render_to_canvas(&app, hatch_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}
