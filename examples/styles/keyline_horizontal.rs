use textual::{App, Compose, Horizontal, Placeholder, Widget, ui};

#[derive(Clone)]
enum Message {}

struct KeylineApp;

impl Compose for KeylineApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Horizontal {
                Placeholder()
                Placeholder()
                Placeholder()
            }
        }
    }
}

impl App for KeylineApp {
    const CSS: &'static str = include_str!("keyline_horizontal.tcss");
}

fn main() -> textual::Result<()> {
    textual::init_logger("height_comparison.log");
    let mut app = KeylineApp;
    app.run()
}
