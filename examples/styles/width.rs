use textual::{App, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct WidthApp;

impl App for WidthApp {
    type Message = Message;

    const CSS: &'static str = include_str!("width.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static("Widget")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = WidthApp;
    app.run()
}
