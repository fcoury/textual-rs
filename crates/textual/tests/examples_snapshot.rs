//! Snapshot tests for all example apps.
//!
//! These tests render each example app to a Canvas at 80x24 and compare
//! against stored snapshots. Run `cargo insta test --accept` to update snapshots.

use insta::assert_snapshot;
use textual::style_resolver::resolve_styles;
use textual::testing::{build_combined_css, render_to_canvas};
use textual::tree::WidgetTree;
use textual::widget::screen::Screen;
use textual::{
    Button, Canvas, Compose, Grid, Label, Region, Size, Theme, Widget, parse_stylesheet, ui,
};

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

#[test]
fn snapshot_border_example_ansi() {
    let app = border_example::BorderApp;
    let canvas = render_to_canvas(&app, border_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

#[test]
fn snapshot_border_example_svg() {
    let app = border_example::BorderApp;
    let canvas = render_to_canvas(&app, border_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_svg(Some("Border Example")));
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

#[test]
fn snapshot_background_example_ansi() {
    let app = background_example::BackgroundApp;
    let canvas = render_to_canvas(&app, background_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

#[test]
fn snapshot_background_example_svg() {
    let app = background_example::BackgroundApp;
    let canvas = render_to_canvas(&app, background_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_svg(Some("Background Example")));
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
                .map(|i| {
                    Box::new(Label::new(format!("Item {}", i)).with_classes("cell"))
                        as Box<dyn Widget<Message>>
                })
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
                make_label_container("nine labels", "lbl5", "Title", "Subtitle"),
                make_label_container("and ended up redoing it", "lbl6", "Title", "Subtitle"),
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

#[test]
fn snapshot_border_title_colors_example_ansi() {
    let app = border_title_colors_example::BorderTitleColorsApp;
    let canvas = render_to_canvas(&app, border_title_colors_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

#[test]
fn snapshot_border_title_colors_example_svg() {
    let app = border_title_colors_example::BorderTitleColorsApp;
    let canvas = render_to_canvas(&app, border_title_colors_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_svg(Some("Border Title Colors Example")));
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

// ============================================================================
// Height Comparison Example (tests vw/vh/w/h/% units)
// ============================================================================

mod height_comparison_example {
    use super::*;
    use textual::{Placeholder, Static, VerticalScroll};

    #[derive(Clone)]
    pub enum Message {}

    pub struct HeightComparisonApp;

    impl Compose for HeightComparisonApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                VerticalScroll {
                    Placeholder(id: "cells")
                    Placeholder(id: "percent")
                    Placeholder(id: "w")
                    Placeholder(id: "h")
                    Placeholder(id: "vw")
                    Placeholder(id: "vh")
                    Placeholder(id: "auto")
                    Placeholder(id: "fr1")
                    Placeholder(id: "fr2")
                }
                Static("", id: "ruler")
            }
        }
    }

    pub const CSS: &str = r#"
#cells {
    height: 2;
}
#percent {
    height: 12.5%;
}
#w {
    height: 5w;
}
#h {
    height: 12.5h;
}
#vw {
    height: 6.25vw;
}
#vh {
    height: 12.5vh;
}
#auto {
    height: auto;
}
#fr1 {
    height: 1fr;
}
#fr2 {
    height: 2fr;
}

Screen {
    overflow: hidden;
}

#ruler {
    dock: right;
    width: 1;
    background: $accent;
}
"#;
}

#[test]
fn snapshot_height_comparison_example() {
    textual::reset_placeholder_counter(); // Ensure deterministic colors
    let app = height_comparison_example::HeightComparisonApp;
    // Test at 80x24 viewport
    let canvas = render_to_canvas(&app, height_comparison_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

#[test]
fn snapshot_height_comparison_example_ansi() {
    textual::reset_placeholder_counter(); // Ensure deterministic colors
    let app = height_comparison_example::HeightComparisonApp;
    // Test with ANSI colors to catch Placeholder color regressions
    let canvas = render_to_canvas(&app, height_comparison_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

// ============================================================================
// Layout Example
// ============================================================================

mod layout_example {
    use super::*;
    use textual::Container;

    #[derive(Clone)]
    pub enum Message {}

    pub struct LayoutApp;

    impl Compose for LayoutApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Container(id: "vertical-layout") {
                    Label("Layout")
                    Label("Is")
                    Label("Vertical")
                }
                Container(id: "horizontal-layout") {
                    Label("Layout")
                    Label("Is")
                    Label("Horizontal")
                }
            }
        }
    }

    pub const CSS: &str = r#"
#vertical-layout {
    layout: vertical;
    background: darkmagenta;
    height: auto;
}

#horizontal-layout {
    layout: horizontal;
    background: darkcyan;
    height: auto;
}

Label {
    margin: 1;
    width: 12;
    color: black;
    background: yellowgreen;
}
"#;
}

#[test]
fn snapshot_layout_example() {
    let app = layout_example::LayoutApp;
    let canvas = render_to_canvas(&app, layout_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Color Example
// ============================================================================

mod color_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct ColorApp;

    impl Compose for ColorApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("I'm red!", id: "label1")
                Label("I'm rgb(0, 255, 0)!", id: "label2")
                Label("I'm hsl(240, 100%, 50%)!", id: "label3")
            }
        }
    }

    pub const CSS: &str = r#"
Label {
    height: 1fr;
    content-align: center middle;
    width: 100%;
}

#label1 {
    color: red;
}

#label2 {
    color: rgb(0, 255, 0);
}

#label3 {
    color: hsl(240, 100%, 50%);
}
"#;
}

#[test]
fn snapshot_color_example() {
    let app = color_example::ColorApp;
    let canvas = render_to_canvas(&app, color_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

#[test]
fn snapshot_color_example_ansi() {
    let app = color_example::ColorApp;
    let canvas = render_to_canvas(&app, color_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

#[test]
fn snapshot_color_example_svg() {
    let app = color_example::ColorApp;
    let canvas = render_to_canvas(&app, color_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_svg(Some("Color Example")));
}

// ============================================================================
// Color Auto Example
// ============================================================================

mod color_auto_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct ColorAutoApp;

    impl Compose for ColorAutoApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("The quick brown fox jumps over the lazy dog!", id: "lbl1")
                Label("The quick brown fox jumps over the lazy dog!", id: "lbl2")
                Label("The quick brown fox jumps over the lazy dog!", id: "lbl3")
                Label("The quick brown fox jumps over the lazy dog!", id: "lbl4")
                Label("The quick brown fox jumps over the lazy dog!", id: "lbl5")
            }
        }
    }

    pub const CSS: &str = r#"
Label {
    color: auto 80%;
    content-align: center middle;
    height: 1fr;
    width: 100%;
}

#lbl1 {
    background: red 80%;
}

#lbl2 {
    background: yellow 80%;
}

#lbl3 {
    background: blue 80%;
}

#lbl4 {
    background: pink 80%;
}

#lbl5 {
    background: green 80%;
}
"#;
}

#[test]
fn snapshot_color_auto_example() {
    let app = color_auto_example::ColorAutoApp;
    let canvas = render_to_canvas(&app, color_auto_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Background Tint Example
// ============================================================================

mod background_tint_example {
    use super::*;
    use textual::Vertical;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BackgroundTintApp;

    impl Compose for BackgroundTintApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Vertical(id: "tint1") {
                    Label("0%")
                }
                Vertical(id: "tint2") {
                    Label("25%")
                }
                Vertical(id: "tint3") {
                    Label("50%")
                }
                Vertical(id: "tint4") {
                    Label("75%")
                }
                Vertical(id: "tint5") {
                    Label("100%")
                }
            }
        }
    }

    pub const CSS: &str = r#"
Vertical {
    background: $panel;
    color: auto 90%;
    height: 1fr;
}
#tint1 { background-tint: $foreground 0%; }
#tint2 { background-tint: $foreground 25%; }
#tint3 { background-tint: $foreground 50%; }
#tint4 { background-tint: $foreground 75%; }
#tint5 { background-tint: $foreground 100%; }
"#;
}

#[test]
fn snapshot_background_tint_example() {
    let app = background_tint_example::BackgroundTintApp;
    let canvas = render_to_canvas(&app, background_tint_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Background Transparency Example
// ============================================================================

mod background_transparency_example {
    use super::*;
    use textual::Static;

    #[derive(Clone)]
    pub enum Message {}

    pub struct BackgroundTransparencyApp;

    impl Compose for BackgroundTransparencyApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Static("10%", id: "t10")
                Static("20%", id: "t20")
                Static("30%", id: "t30")
                Static("40%", id: "t40")
                Static("50%", id: "t50")
                Static("60%", id: "t60")
                Static("70%", id: "t70")
                Static("80%", id: "t80")
                Static("90%", id: "t90")
                Static("100%", id: "t100")
            }
        }
    }

    pub const CSS: &str = r#"
#t10 { background: red 10%; }
#t20 { background: red 20%; }
#t30 { background: red 30%; }
#t40 { background: red 40%; }
#t50 { background: red 50%; }
#t60 { background: red 60%; }
#t70 { background: red 70%; }
#t80 { background: red 80%; }
#t90 { background: red 90%; }
#t100 { background: red 100%; }

Screen {
    layout: horizontal;
}

Static {
    height: 100%;
    width: 1fr;
    content-align: center middle;
}
"#;
}

#[test]
fn snapshot_background_transparency_example() {
    let app = background_transparency_example::BackgroundTransparencyApp;
    let canvas = render_to_canvas(&app, background_transparency_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Grid Columns Example
// ============================================================================

mod grid_columns_example {
    use super::*;
    use textual::Grid;

    #[derive(Clone)]
    pub enum Message {}

    pub struct GridColumnsApp;

    impl Compose for GridColumnsApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Grid {
                    Label("1fr")
                    Label("width = 16")
                    Label("2fr")
                    Label("1fr")
                    Label("width = 16")
                    Label("1fr")
                    Label("width = 16")
                    Label("2fr")
                    Label("1fr")
                    Label("width = 16")
                }
            }
        }
    }

    pub const CSS: &str = r#"
Grid {
    grid-size: 5 2;
    grid-columns: 1fr 16 2fr;
}

Label {
    border: round white;
    content-align-horizontal: center;
    width: 100%;
    height: 100%;
}
"#;
}

#[test]
fn snapshot_grid_columns_example() {
    let app = grid_columns_example::GridColumnsApp;
    let canvas = render_to_canvas(&app, grid_columns_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Grid Size Both Example
// ============================================================================

mod grid_size_both_example {
    use super::*;
    use textual::Grid;

    #[derive(Clone)]
    pub enum Message {}

    pub struct GridSizeBothApp;

    impl Compose for GridSizeBothApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Grid {
                    Label("1")
                    Label("2")
                    Label("3")
                    Label("4")
                    Label("5")
                }
            }
        }
    }

    pub const CSS: &str = r#"
Grid {
    grid-size: 2 4;
}

Label {
    border: round white;
    content-align: center middle;
    width: 100%;
    height: 100%;
}
"#;
}

#[test]
fn snapshot_grid_size_both_example() {
    let app = grid_size_both_example::GridSizeBothApp;
    let canvas = render_to_canvas(&app, grid_size_both_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Grid Size Columns Example
// ============================================================================

mod grid_size_columns_example {
    use super::*;
    use textual::Grid;

    #[derive(Clone)]
    pub enum Message {}

    pub struct GridSizeColumnsApp;

    impl Compose for GridSizeColumnsApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Grid {
                    Label("1")
                    Label("2")
                    Label("3")
                    Label("4")
                    Label("5")
                }
            }
        }
    }

    pub const CSS: &str = r#"
Grid {
    grid-size: 2;
}

Label {
    border: round white;
    content-align: center middle;
    width: 100%;
    height: 100%;
}
"#;
}

#[test]
fn snapshot_grid_size_columns_example() {
    let app = grid_size_columns_example::GridSizeColumnsApp;
    let canvas = render_to_canvas(&app, grid_size_columns_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Height Example
// ============================================================================

mod height_example {
    use super::*;
    use textual::Container;

    #[derive(Clone)]
    pub enum Message {}

    pub struct HeightApp;

    impl Compose for HeightApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Container {}
            }
        }
    }

    pub const CSS: &str = r#"
Screen > Container {
    background: green;
    height: 50%;
    color: white;
}
"#;
}

#[test]
fn snapshot_height_example() {
    let app = height_example::HeightApp;
    let canvas = render_to_canvas(&app, height_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Keyline Example
// ============================================================================

mod keyline_example {
    use super::*;
    use textual::{Grid, Placeholder};

    #[derive(Clone)]
    pub enum Message {}

    pub struct KeylineApp;

    impl Compose for KeylineApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Grid {
                    Placeholder(id: "foo")
                    Placeholder(id: "bar")
                    Placeholder()
                    Placeholder(classes: "hidden")
                    Placeholder(id: "baz")
                }
            }
        }
    }

    pub const CSS: &str = r#"
Grid {
    grid-size: 3 3;
    grid-gutter: 1;
    padding: 2 3;
    keyline: heavy green;
}
Placeholder {
    height: 1fr;
}
.hidden {
    visibility: hidden;
}
#foo {
    column-span: 2;
}
#bar {
    row-span: 2;
}
#baz {
    column-span: 3;
}
"#;
}

#[test]
fn snapshot_keyline_example() {
    let app = keyline_example::KeylineApp;
    let canvas = render_to_canvas(&app, keyline_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Visibility Containers Example
// ============================================================================

mod visibility_containers_example {
    use super::*;
    use textual::{Horizontal, Placeholder, VerticalScroll};

    #[derive(Clone)]
    pub enum Message {}

    pub struct VisibilityContainersApp;

    impl Compose for VisibilityContainersApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                VerticalScroll {
                    Horizontal(id: "top") {
                        Placeholder()
                        Placeholder()
                        Placeholder()
                    }
                    Horizontal(id: "middle") {
                        Placeholder()
                        Placeholder()
                        Placeholder()
                    }
                    Horizontal(id: "bot") {
                        Placeholder()
                        Placeholder()
                        Placeholder()
                    }
                }
            }
        }
    }

    pub const CSS: &str = r#"
Horizontal {
    padding: 1 2;
    background: white;
    height: 1fr;
}

#top {}

#middle {
    visibility: hidden;
}

#bot {
    visibility: hidden;
}

#bot > Placeholder {
    visibility: visible;
}

Placeholder {
    width: 1fr;
}
"#;
}

#[test]
fn snapshot_visibility_containers_example_svg() {
    let app = visibility_containers_example::VisibilityContainersApp;
    let canvas = render_to_canvas(&app, visibility_containers_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_svg(Some("Visibility Containers Example")));
}

// ============================================================================
// Keyline Horizontal Example
// ============================================================================

mod keyline_horizontal_example {
    use super::*;
    use textual::{Horizontal, Placeholder};

    #[derive(Clone)]
    pub enum Message {}

    pub struct KeylineHorizontalApp;

    impl Compose for KeylineHorizontalApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Horizontal {
                    Placeholder()
                    Placeholder()
                    Placeholder()
                }
            }
        }
    }

    pub const CSS: &str = r#"
Placeholder {
    margin: 1;
    width: 1fr;
}

Horizontal {
    keyline: thin $secondary;
}
"#;
}

#[test]
fn snapshot_keyline_horizontal_example() {
    let app = keyline_horizontal_example::KeylineHorizontalApp;
    let canvas = render_to_canvas(&app, keyline_horizontal_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Link Background Example
// ============================================================================

mod link_background_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct LinkBackgroundApp;

    impl Compose for LinkBackgroundApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("Visit the [link='https://textualize.io']Textualize[/link] website.", id: "lbl1")
                Label("Click [@click=app.bell]here[/] for the bell sound.", id: "lbl2")
                Label("You can also click [@click=app.bell]here[/] for the bell sound.", id: "lbl3")
                Label("[@click=app.quit]Exit this application.[/]", id: "lbl4")
            }
        }
    }

    pub const CSS: &str = r#"
#lbl1, #lbl2 {
    link-background: red;
}

#lbl3 {
    link-background: hsl(60,100%,50%) 50%;
}

#lbl4 {
    link-background: $accent;
}
"#;
}

#[test]
fn snapshot_link_background_example() {
    let app = link_background_example::LinkBackgroundApp;
    let canvas = render_to_canvas(&app, link_background_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

#[test]
fn snapshot_link_background_example_ansi() {
    let app = link_background_example::LinkBackgroundApp;
    let canvas = render_to_canvas(&app, link_background_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

#[test]
fn snapshot_link_background_example_svg() {
    let app = link_background_example::LinkBackgroundApp;
    let canvas = render_to_canvas(&app, link_background_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_svg(Some("Link Background Example")));
}

// ============================================================================
// Test Grid Align Example
// ============================================================================

mod test_grid_align_example {
    use super::*;
    use textual::{Container, Grid};

    #[derive(Clone)]
    pub enum Message {}

    pub struct TestGridAlignApp;

    impl Compose for TestGridAlignApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Grid {
                    Container { Label("1") }
                    Container { Label("2") }
                    Container { Label("3") }
                    Container { Label("4") }
                    Container { Label("5") }
                    Container { Label("6") }
                    Container { Label("7") }
                    Container { Label("8") }
                    Container { Label("9") }
                }
            }
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
    border: solid white;
}
"#;
}

#[test]
fn snapshot_test_grid_align_example() {
    let app = test_grid_align_example::TestGridAlignApp;
    let canvas = render_to_canvas(&app, test_grid_align_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

// ============================================================================
// Link Style Example
// ============================================================================

mod link_style_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct LinkStyleApp;

    impl Compose for LinkStyleApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("Visit the [link='https://textualize.io']Textualize[/link] website.", id: "lbl1")
                Label("Click [@click=app.bell]here[/] for the bell sound.", id: "lbl2")
                Label("You can also click [@click=app.bell]here[/] for the bell sound.", id: "lbl3")
                Label("[@click=app.quit]Exit this application.[/]", id: "lbl4")
            }
        }
    }

    pub const CSS: &str = r#"
#lbl1, #lbl2 {
    link-style: bold italic;
}

#lbl3 {
    link-style: reverse strike;
}

#lbl4 {
    link-style: bold;
}
"#;
}

#[test]
fn snapshot_link_style_example() {
    let app = link_style_example::LinkStyleApp;
    let canvas = render_to_canvas(&app, link_style_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

#[test]
fn snapshot_link_style_example_ansi() {
    let app = link_style_example::LinkStyleApp;
    let canvas = render_to_canvas(&app, link_style_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

#[test]
fn snapshot_link_style_example_svg() {
    let app = link_style_example::LinkStyleApp;
    let canvas = render_to_canvas(&app, link_style_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_svg(Some("Link Style Example")));
}

// ============================================================================
// Link Color Hover Example
// ============================================================================

mod link_color_hover_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct LinkColorHoverApp;

    impl Compose for LinkColorHoverApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("Visit the [link='https://textualize.io']Textualize[/link] website.", id: "lbl1")
                Label("Click [@click=app.bell]here[/] for the bell sound.", id: "lbl2")
                Label("You can also click [@click=app.bell]here[/] for the bell sound.", id: "lbl3")
                Label("[@click=app.quit]Exit this application.[/]", id: "lbl4")
            }
        }
    }

    pub const CSS: &str = r#"
#lbl1, #lbl2 {
    link-color-hover: red;
}

#lbl3 {
    link-color-hover: hsl(60,100%,50%) 50%;
}

#lbl4 {
    link-color-hover: black;
}
"#;
}

#[test]
fn snapshot_link_color_hover_example() {
    let app = link_color_hover_example::LinkColorHoverApp;
    let canvas = render_to_canvas(&app, link_color_hover_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

#[test]
fn snapshot_link_color_hover_example_ansi() {
    let app = link_color_hover_example::LinkColorHoverApp;
    let canvas = render_to_canvas(&app, link_color_hover_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

#[test]
fn snapshot_link_color_hover_example_svg() {
    let app = link_color_hover_example::LinkColorHoverApp;
    let canvas = render_to_canvas(&app, link_color_hover_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_svg(Some("Link Color Hover Example")));
}

// ============================================================================
// Opacity Example
// ============================================================================

mod opacity_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct OpacityApp;

    impl Compose for OpacityApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("opacity: 0%", id: "zero-opacity")
                Label("opacity: 25%", id: "quarter-opacity")
                Label("opacity: 50%", id: "half-opacity")
                Label("opacity: 75%", id: "three-quarter-opacity")
                Label("opacity: 100%", id: "full-opacity")
            }
        }
    }

    pub const CSS: &str = r#"
#zero-opacity {
    opacity: 0%;
}

#quarter-opacity {
    opacity: 25%;
}

#half-opacity {
    opacity: 50%;
}

#three-quarter-opacity {
    opacity: 75%;
}

#full-opacity {
    opacity: 100%;
}

Screen {
    background: black;
}

Label {
    width: 100%;
    height: 1fr;
    border: outer dodgerblue;
    background: lightseagreen;
    content-align: center middle;
    text-style: bold;
}
"#;
}

#[test]
fn snapshot_opacity_example() {
    let app = opacity_example::OpacityApp;
    let canvas = render_to_canvas(&app, opacity_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

#[test]
fn snapshot_opacity_example_ansi() {
    let app = opacity_example::OpacityApp;
    let canvas = render_to_canvas(&app, opacity_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

#[test]
fn debug_opacity_border_color() {
    use std::collections::VecDeque;
    use textual::style_resolver::resolve_styles;
    use textual::testing::build_combined_css;
    use textual::tree::WidgetTree;
    use textual::widget::screen::Screen;

    let themes = tcss::types::Theme::standard_themes();
    let theme = themes
        .get("textual-dark")
        .cloned()
        .unwrap_or_else(|| tcss::types::Theme::new("default", true));

    // Build widget tree
    let root = Box::new(Screen::new(opacity_example::OpacityApp.compose()));
    let mut tree = WidgetTree::new(root);

    // Initialize Screen
    tree.root_mut().on_resize(textual::Size::new(80, 24));

    // Build combined CSS
    let combined_css = build_combined_css(tree.root_mut(), opacity_example::CSS);
    eprintln!("=== COMBINED CSS ===\n{}", combined_css);

    let stylesheet = tcss::parser::parse_stylesheet(&combined_css).expect("CSS parsing failed");

    // Resolve styles
    let mut ancestors = VecDeque::new();
    resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);

    // Check the first Label's style
    let first_label_style = {
        let mut found_style = None;
        tree.root_mut().for_each_child(&mut |child| {
            if child.type_name() == "Label" && found_style.is_none() {
                let style = child.get_style();
                found_style = Some(style);
            }
        });
        found_style.unwrap_or_default()
    };

    eprintln!("=== FIRST LABEL STYLE ===");
    eprintln!("  border.top.kind: {:?}", first_label_style.border.top.kind);
    eprintln!(
        "  border.top.color: {:?}",
        first_label_style.border.top.color
    );
    eprintln!("  background: {:?}", first_label_style.background);
    eprintln!(
        "  inherited_background: {:?}",
        first_label_style.inherited_background
    );
    eprintln!("  opacity: {}", first_label_style.opacity);

    // Now render and check canvas
    let canvas = render_to_canvas(&opacity_example::OpacityApp, opacity_example::CSS, 80, 24);

    // Check the first cell on row 0 - should be the border corner (▛)
    let first_cell = canvas.cell_at(0);
    eprintln!("=== CANVAS OUTPUT ===");
    eprintln!(
        "Cell at (0,0): symbol='{}', fg={:?}, bg={:?}",
        first_cell.symbol, first_cell.fg, first_cell.bg
    );

    // At 0% opacity, the border should blend to match the inherited background (black)
    // This is the correct behavior - borders fade with the widget
    if let Some(fg) = &first_cell.fg {
        match fg {
            crossterm::style::Color::Rgb { r, g, b } => {
                eprintln!("  -> foreground RGB: r={}, g={}, b={}", r, g, b);
                // At 0% opacity, dodgerblue should blend to black (inherited background)
                assert_eq!(
                    *r, 0,
                    "Expected red=0 (blended to black at 0% opacity), got {}",
                    r
                );
                assert_eq!(
                    *g, 0,
                    "Expected green=0 (blended to black at 0% opacity), got {}",
                    g
                );
                assert_eq!(
                    *b, 0,
                    "Expected blue=0 (blended to black at 0% opacity), got {}",
                    b
                );
            }
            _ => panic!("Expected RGB color, got {:?}", fg),
        }
    } else {
        panic!("Expected foreground color to be set for border");
    }
}

// ============================================================================
// Outline All Example
// ============================================================================

mod outline_all_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct OutlineAllApp;

    impl Compose for OutlineAllApp {
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
                    Label("none", id: "none")
                    Label("outer", id: "outer")
                    Label("round", id: "round")
                    Label("solid", id: "solid")
                    Label("tall", id: "tall")
                    Label("vkey", id: "vkey")
                    Label("wide", id: "wide")
                }
            }
        }
    }

    pub const CSS: &str = r#"
#ascii {
    outline: ascii $accent;
}

#blank {
    outline: blank $accent;
}

#dashed {
    outline: dashed $accent;
}

#double {
    outline: double $accent;
}

#heavy {
    outline: heavy $accent;
}

#hidden {
    outline: hidden $accent;
}

#hkey {
    outline: hkey $accent;
}

#inner {
    outline: inner $accent;
}

#none {
    outline: none $accent;
}

#outer {
    outline: outer $accent;
}

#round {
    outline: round $accent;
}

#solid {
    outline: solid $accent;
}

#tall {
    outline: tall $accent;
}

#vkey {
    outline: vkey $accent;
}

#wide {
    outline: wide $accent;
}

Grid {
    grid-size: 3 5;
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
fn snapshot_outline_all_example() {
    let app = outline_all_example::OutlineAllApp;
    let canvas = render_to_canvas(&app, outline_all_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_snapshot());
}

#[test]
fn snapshot_outline_all_example_ansi() {
    let app = outline_all_example::OutlineAllApp;
    let canvas = render_to_canvas(&app, outline_all_example::CSS, 80, 24);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

// ============================================================================
// Question01 Example (Button focus + border shading)
// ============================================================================

mod question01_example {
    use super::*;

    #[derive(Clone)]
    pub enum Message {}

    pub struct QuestionApp;

    impl Compose for QuestionApp {
        type Message = Message;

        fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
            ui! {
                Label("Do you love Textual?")
                Button("Yes", id: "yes", variant: "primary")
                Button("No", id: "no", variant: "error")
            }
        }
    }

    pub const CSS: &str = "";
}

fn render_question_canvas(focus_index: usize) -> Canvas {
    let app = question01_example::QuestionApp;
    let themes = Theme::standard_themes();
    let theme = themes
        .get("textual-dark")
        .cloned()
        .unwrap_or_else(|| Theme::new("default", true));

    let root = Box::new(Screen::new(app.compose()));
    let mut tree = WidgetTree::new(root);
    let size = Size::new(40, 10);
    tree.root_mut().on_resize(size);
    tree.set_focus_index(focus_index);

    let combined_css = build_combined_css(tree.root_mut(), question01_example::CSS);
    let stylesheet = parse_stylesheet(&combined_css).expect("CSS parsing failed");
    let mut ancestors = std::collections::VecDeque::new();
    resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);

    let mut canvas = Canvas::new(size.width, size.height);
    let region = Region::from_u16(0, 0, size.width, size.height);
    tree.root().render(&mut canvas, region);
    canvas
}

#[test]
fn snapshot_question01_focus_yes_ansi() {
    let canvas = render_question_canvas(0);
    assert_snapshot!(canvas.to_ansi_snapshot());
}

#[test]
fn snapshot_question01_focus_no_ansi() {
    let canvas = render_question_canvas(1);
    assert_snapshot!(canvas.to_ansi_snapshot());
}
