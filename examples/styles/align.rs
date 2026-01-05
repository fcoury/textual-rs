use textual::{App, Compose, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct AlignApp;

impl Compose for AlignApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Vertical alignment with [b]Textual[/]", classes: "box")
            Label("Take note, browsers.", classes: "box")
        }
    }
}

impl App for AlignApp {
    const CSS: &'static str = include_str!("align.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = AlignApp;
    app.run()
}
