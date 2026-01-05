use textual::{App, Placeholder, VerticalScroll, Widget, ui};

#[derive(Clone)]
enum Message {}

struct MaxWidthApp;

impl App for MaxWidthApp {
    type Message = Message;

    const CSS: &'static str = include_str!("max_width.tcss");

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            VerticalScroll {
                Placeholder(id: "p1", label: "max-width: 50h")
                Placeholder(id: "p2", label: "max-width: 999")
                Placeholder(id: "p3", label: "max-width: 50%")
                Placeholder(id: "p4", label: "max-width: 10")
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = MaxWidthApp;
    app.run()
}
