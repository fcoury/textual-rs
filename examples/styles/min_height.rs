use textual::{App, Compose, Horizontal, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MinHeightApp;

impl Compose for MinHeightApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Horizontal {
                Placeholder(id: "p1", label: "min-height: 25%")
                Placeholder(id: "p2", label: "min-height: 75%")
                Placeholder(id: "p3", label: "min-height: 30")
                Placeholder(id: "p4", label: "min-height: 40w")
            }
        }
    }
}

impl App for MinHeightApp {
    const CSS: &'static str = include_str!("min_height.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = MinHeightApp;
    app.run()
}
