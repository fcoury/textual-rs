use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BorderTitleAlignApp;

impl App for BorderTitleAlignApp {
    type Message = Message;

    const CSS: &'static str = include_str!("border_title_align.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("My title is on the left.", id:"label1", border_title: "< Left")
            Label("My title is centered", id:"label2", border_title: "Centered!")
            Label("My title is on the right", id:"label3", border_title: "Right >")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = BorderTitleAlignApp;
    app.run()
}
