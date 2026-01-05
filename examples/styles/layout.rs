use textual::{App, Compose, Container, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct LayoutApp;

impl Compose for LayoutApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Container(id: "vertical-layout") {
                Label("Layout")
                Label("Is")
                Label("Vertical")
            }
            Container(id: "horizontal-layout") {
                Label("Layout")
                Label("Is")
                Label("Horizontal")
            }
        }
    }
}

impl App for LayoutApp {
    const CSS: &'static str = include_str!("layout.tcss");
}

fn main() -> textual::Result<()> {
    textual::init_logger("height_comparison.log");
    let mut app = LayoutApp;
    app.run()
}
