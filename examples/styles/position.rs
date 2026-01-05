use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct PositionApp;

impl App for PositionApp {
    type Message = Message;

    const CSS: &'static str = include_str!("position.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Absolute", id: "label1")
            Label("Relative", id: "label2")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = PositionApp;
    app.run()
}
