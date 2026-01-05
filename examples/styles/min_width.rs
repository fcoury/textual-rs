use textual::{App, Placeholder, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MinWidthApp;

impl App for MinWidthApp {
    type Message = Message;

    const CSS: &'static str = include_str!("min_width.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            VerticalScroll {
                Placeholder(id: "p1", label: "min-width: 25%")
                Placeholder(id: "p2", label: "min-width: 75%")
                Placeholder(id: "p3", label: "min-width: 100")
                Placeholder(id: "p4", label: "min-width: 400h")
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = MinWidthApp;
    app.run()
}
