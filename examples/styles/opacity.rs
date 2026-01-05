use textual::{App, Compose, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct OpacityApp;

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

impl App for OpacityApp {
    const CSS: &'static str = include_str!("opacity.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = OpacityApp;
    app.run()
}
