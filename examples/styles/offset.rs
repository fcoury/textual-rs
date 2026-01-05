use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct OffsetApp;

impl App for OffsetApp {
    type Message = Message;

    const CSS: &'static str = include_str!("offset.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Paul (offset 8 2)", classes: "paul")
            Label("Duncan (offset 4 10)", classes: "duncan")
            Label("Chani (offset 0 -3)", classes: "chani")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = OffsetApp;
    app.run()
}
