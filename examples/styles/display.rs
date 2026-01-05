use textual::{App, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct DisplayApp;

impl App for DisplayApp {
    type Message = Message;

    const CSS: &'static str = include_str!("display.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static("Widget 1")
            Static("Widget 2", classes: "remove")
            Static("Widget 3")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = DisplayApp;
    app.run()
}
