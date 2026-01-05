use textual::{App, Compose, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct DisplayApp;

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

impl App for DisplayApp {
    const CSS: &'static str = include_str!("display.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = DisplayApp;
    app.run()
}
