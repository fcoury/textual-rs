use textual::{App, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct ColorApp;

impl App for ColorApp {
    type Message = Message;

    const CSS: &'static str = include_str!("color.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("I'm red!", id: "label1")
            Label("I'm rgb(0, 255, 0)!", id: "label2")
            Label("I'm hsl(240, 100%, 50%)!", id: "label3")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = ColorApp;
    app.run()
}
