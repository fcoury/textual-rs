use textual::{App, Compose, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct PositionApp;

impl Compose for PositionApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Absolute", id: "label1")
            Label("Relative", id: "label2")
        }
    }
}

impl App for PositionApp {
    const CSS: &'static str = include_str!("position.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = PositionApp;
    app.run()
}
