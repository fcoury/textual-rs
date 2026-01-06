use textual::{App, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct HeightApp;

impl App for HeightApp {
    type Message = Message;

    const CSS: &'static str = include_str!("height.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static("Widget")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = HeightApp;
    app.run()
}
