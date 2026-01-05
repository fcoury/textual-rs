use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct AlignApp;

impl App for AlignApp {
    type Message = Message;

    const CSS: &'static str = include_str!("align.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Vertical alignment with [b]Textual[/]", classes: "box")
            Label("Take note, browsers.", classes: "box")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = AlignApp;
    app.run()
}
