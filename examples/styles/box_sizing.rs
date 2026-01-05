use textual::{App, Compose, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BoxSizingApp;

impl Compose for BoxSizingApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static("I'm using border-box!", id: "static1")
            Static("I'm using content-box!", id: "static2")
        }
    }
}

impl App for BoxSizingApp {
    const CSS: &'static str = include_str!("box_sizing.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = BoxSizingApp;
    app.run()
}
