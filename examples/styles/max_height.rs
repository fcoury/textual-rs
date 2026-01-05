use textual::{App, Compose, Horizontal, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MaxHeightApp;

impl Compose for MaxHeightApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Horizontal {
                Placeholder(id: "p1", label: "max-height: 10w")
                Placeholder(id: "p2", label: "max-height: 999")
                Placeholder(id: "p3", label: "max-height: 50%")
                Placeholder(id: "p4", label: "max-height: 10")
            }
        }
    }
}

impl App for MaxHeightApp {
    const CSS: &'static str = include_str!("max_height.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = MaxHeightApp;
    app.run()
}
