use textual::{App, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BoxSizingApp;

impl App for BoxSizingApp {
    type Message = Message;

    const CSS: &'static str = include_str!("box_sizing.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static("I'm using border-box!", id: "static1")
            Static("I'm using content-box!", id: "static2")
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = BoxSizingApp;
    app.run()
}
